use super::LTCTVStandard;
use super::SMPTETimecode;
use crate::consts;
use crate::consts::LtcBgFlags;
use crate::consts::SampleType;
use crate::error::TimecodeError;
use crate::raw;
use crate::TimecodeWasWrapped;

#[derive(Debug)]
pub struct LTCFrame {
    pub(super) inner_unsafe_ptr: *mut raw::LTCFrame,
}

impl LTCFrame {
    pub fn dfbit(&self) -> u32 {
        unsafe { *self.inner_unsafe_ptr }.dfbit()
    }
}

#[derive(Debug)]
pub struct LTCFrameExt {
    pub(super) inner_unsafe_ptr: *mut raw::LTCFrameExt,
}

impl LTCFrameExt {
    // SAFETY: this is safe because we own the pointer
    pub fn ltc(self) -> LTCFrame {
        LTCFrame {
            inner_unsafe_ptr: unsafe { &mut (*self.inner_unsafe_ptr).ltc },
        }
    }
    pub fn off_start(&self) -> i64 {
        unsafe { *self.inner_unsafe_ptr }.off_start
    }
    pub fn set_off_start(&self, off_start: i64) {
        unsafe {
            (*self.inner_unsafe_ptr).off_start = off_start;
        }
    }
    pub fn off_end(&self) -> i64 {
        unsafe { *self.inner_unsafe_ptr }.off_end
    }
    pub fn set_off_end(&self, off_end: i64) {
        unsafe {
            (*self.inner_unsafe_ptr).off_end = off_end;
        }
    }
    pub fn reverse(&self) -> bool {
        unsafe { *self.inner_unsafe_ptr }.reverse != 0
    }
    pub fn set_reverse(&self, reverse: bool) {
        unsafe {
            (*self.inner_unsafe_ptr).reverse = if reverse { 1 } else { 0 };
        }
    }
    pub fn biphase_tics(&self) -> [f32; 80usize] {
        unsafe { *self.inner_unsafe_ptr }.biphase_tics
    }
    pub fn set_biphase_tics(&self, biphase_tics: [f32; 80usize]) {
        unsafe {
            (*self.inner_unsafe_ptr).biphase_tics = biphase_tics;
        }
    }
    pub fn sample_min(&self) -> SampleType {
        unsafe { *self.inner_unsafe_ptr }.sample_min
    }
    pub fn set_sample_min(&self, sample_min: SampleType) {
        unsafe {
            (*self.inner_unsafe_ptr).sample_min = sample_min;
        }
    }
    pub fn sample_max(&self) -> SampleType {
        unsafe { *self.inner_unsafe_ptr }.sample_max
    }
    pub fn set_sample_max(&self, sample_max: SampleType) {
        unsafe {
            (*self.inner_unsafe_ptr).sample_max = sample_max;
        }
    }
    pub fn volume(&self) -> f64 {
        unsafe { *self.inner_unsafe_ptr }.volume
    }
    pub fn set_volume(&self, volume: f64) {
        unsafe {
            (*self.inner_unsafe_ptr).volume = volume;
        }
    }
}

// SAFETY: We are allocating the pointer as a Box so it outlives the function
// Drop is implemented for LTCFrame
impl Default for LTCFrame {
    fn default() -> Self {
        let inner = Box::new(raw::LTCFrame::default());
        LTCFrame {
            inner_unsafe_ptr: Box::into_raw(inner),
        }
    }
}

impl Drop for LTCFrame {
    fn drop(&mut self) {
        dbg!("Dropping LTCFrame");
        if !self.inner_unsafe_ptr.is_null() {
            // SAFETY: the pointer is assumed to not be null
            unsafe {
                let _ = Box::from_raw(self.inner_unsafe_ptr);
            }
        }
    }
}

// SAFETY: We are allocating the pointer as a Box so it outlives the function
// Drop is implemented for LTCFrameExt
impl Default for LTCFrameExt {
    fn default() -> Self {
        let inner = Box::new(raw::LTCFrameExt::default());
        LTCFrameExt {
            inner_unsafe_ptr: Box::into_raw(inner),
        }
    }
}

impl Drop for LTCFrameExt {
    fn drop(&mut self) {
        dbg!("Dropping LTCFrameExt");
        if !self.inner_unsafe_ptr.is_null() {
            // SAFETY: the pointer is assumed to not be null
            unsafe {
                let _ = Box::from_raw(self.inner_unsafe_ptr);
            }
        }
    }
}

impl LTCFrame {
    pub fn new() -> Self {
        // SAFETY: The pointer will outlive the function because it is allocated in a Box
        let inner = Box::new(raw::LTCFrame::default());
        let mut frame = LTCFrame {
            inner_unsafe_ptr: Box::into_raw(inner),
        };

        // SAFETY: frame is created above and is not null
        unsafe {
            #[allow(clippy::needless_borrow)] // for clarity
            raw::ltc_frame_reset((&mut frame).inner_unsafe_ptr);
        }
        frame
    }

    pub fn to_timecode(&self, flags: consts::LtcBgFlags) -> SMPTETimecode {
        let mut timecode = SMPTETimecode::default();

        // SAFETY: We own timecode. The function is assumed to only read the frame.
        unsafe {
            #[allow(clippy::needless_borrow)] // for clarity
            raw::ltc_frame_to_time(
                (&mut timecode).inner_unsafe_ptr,
                self.inner_unsafe_ptr,
                flags.into(),
            );
        }

        timecode
    }

    pub fn from_timecode(
        timecode: &SMPTETimecode,
        standard: LTCTVStandard,
        flags: consts::LtcBgFlags,
    ) -> Self {
        let mut frame = Self::new();

        // SAFETY: We own frame. The function is assumed to only read the timecode.
        unsafe {
            #[allow(clippy::needless_borrow)] // for clarity
            raw::ltc_time_to_frame(
                (&mut frame).inner_unsafe_ptr,
                timecode.inner_unsafe_ptr,
                standard.to_raw(),
                flags.into(),
            );
        }

        frame
    }

    pub fn from_timecode_inplace(
        &mut self,
        timecode: &SMPTETimecode,
        standard: LTCTVStandard,
        flags: consts::LtcBgFlags,
    ) {
        // SAFETY: We own frame. The function is assumed to only read the timecode.
        unsafe {
            #[allow(clippy::needless_borrow)] // for clarity
            raw::ltc_time_to_frame(
                self.inner_unsafe_ptr,
                timecode.inner_unsafe_ptr,
                standard.to_raw(),
                flags.into(),
            );
        }
    }

    pub fn increment(
        &mut self,
        fps: i32,
        standard: LTCTVStandard,
        flags: LtcBgFlags,
    ) -> Result<TimecodeWasWrapped, TimecodeError> {
        // SAFETY: We own self
        let timecode_was_wrapped = unsafe {
            raw::ltc_frame_increment(self.inner_unsafe_ptr, fps, standard.to_raw(), flags.into())
        };
        match timecode_was_wrapped {
            0 => Ok(TimecodeWasWrapped::No),
            1 => Ok(TimecodeWasWrapped::Yes),
            _ => Err(TimecodeError::InvalidReturn),
        }
    }

    pub fn decrement(
        &mut self,
        fps: i32,
        standard: LTCTVStandard,
        flags: LtcBgFlags,
    ) -> Result<TimecodeWasWrapped, TimecodeError> {
        // SAFETY: We own self
        let timecode_was_wrapped = unsafe {
            raw::ltc_frame_decrement(self.inner_unsafe_ptr, fps, standard.to_raw(), flags.into())
        };
        match timecode_was_wrapped {
            0 => Ok(TimecodeWasWrapped::No),
            1 => Ok(TimecodeWasWrapped::Yes),
            _ => Err(TimecodeError::InvalidReturn),
        }
    }

    pub fn set_parity(&mut self, standard: LTCTVStandard) {
        // SAFETY: We own self
        unsafe {
            raw::ltc_frame_set_parity(self.inner_unsafe_ptr, standard.to_raw());
        }
    }

    pub fn parse_bcg_flags(&self, standard: LTCTVStandard) -> LtcBgFlags {
        // SAFETY: The function is assumed to only read self (the frame)
        unsafe { raw::ltc_frame_parse_bcg_flags(self.inner_unsafe_ptr, standard.to_raw()) }.into()
    }

    pub fn get_user_bits(&self) -> u32 {
        // SAFETY: The function is assumed to only read self (the frame)
        unsafe { raw::ltc_frame_get_user_bits(self.inner_unsafe_ptr) as u32 }
    }
}

pub fn calc_frame_alignment(samples_per_frame: f64, standard: LTCTVStandard) -> i64 {
    // SAFETY: The function is assumed to be pure
    unsafe { raw::ltc_frame_alignment(samples_per_frame, standard.to_raw()) }
}
