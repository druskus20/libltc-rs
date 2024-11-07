// lib.rs
mod error;
mod raw;

use std::slice;

pub use error::{LTCDecoderError, LTCEncoderError};

#[derive(Debug)]
pub struct LTCEncoder {
    inner: *mut raw::LTCEncoder,
}

#[derive(Debug)]
pub struct LTCDecoder {
    inner: *mut raw::LTCDecoder,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SMPTETimecode {
    pub hours: i32,
    pub mins: i32,
    pub secs: i32,
    pub frame: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LTCFrame {
    pub ltc: [u8; 10],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LTCFrameExt {
    pub ltc: LTCFrame,
    pub off_start: i64,
    pub off_end: i64,
    pub reverse: i32,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum LTCTVStandard {
    LTCTV_525_60,  // 30fps
    LTCTV_625_50,  // 25fps
    LTCTV_1125_60, // 30fps
    LTCTV_FILM_24, // 24fps
}

// Frame-related functions
impl LTCFrame {
    pub fn new() -> Self {
        let mut frame = LTCFrame { ltc: [0; 10] };
        unsafe {
            raw::ltc_frame_reset(&mut frame as *mut _);
        }
        frame
    }

    pub fn to_timecode(&self, flags: i32) -> SMPTETimecode {
        let mut timecode = SMPTETimecode {
            hours: 0,
            mins: 0,
            secs: 0,
            frame: 0,
        };
        unsafe {
            raw::ltc_frame_to_time(&mut timecode as *mut _, self as *const _, flags);
        }
        timecode
    }

    pub fn from_timecode(timecode: &SMPTETimecode, standard: LTCTVStandard, flags: i32) -> Self {
        let mut frame = Self::new();
        unsafe {
            raw::ltc_time_to_frame(
                &mut frame as *mut _,
                timecode as *const _,
                standard.to_raw(),
                flags,
            );
        }
        frame
    }

    pub fn increment(&mut self, fps: i32, standard: LTCTVStandard, flags: i32) -> bool {
        unsafe { raw::ltc_frame_increment(self as *mut _, fps, standard.to_raw(), flags) != 0 }
    }

    pub fn decrement(&mut self, fps: i32, standard: LTCTVStandard, flags: i32) -> bool {
        unsafe { raw::ltc_frame_decrement(self as *mut _, fps, standard.to_raw(), flags) != 0 }
    }

    pub fn set_parity(&mut self, standard: LTCTVStandard) {
        unsafe {
            raw::ltc_frame_set_parity(self as *mut _, standard.to_raw());
        }
    }

    pub fn parse_bcg_flags(&mut self, standard: LTCTVStandard) -> i32 {
        unsafe { raw::ltc_frame_parse_bcg_flags(self as *mut _, standard.to_raw()) }
    }

    pub fn get_user_bits(&self) -> u32 {
        unsafe { raw::ltc_frame_get_user_bits(self as *const _) as u32 }
    }
    pub fn ltc_frame_alignment(samples_per_frame: f64, standard: LTCTVStandard) -> i64 {
        unsafe { raw::ltc_frame_alignment(samples_per_frame, standard.to_raw()) }
    }
}

impl Default for LTCFrame {
    fn default() -> Self {
        Self::new()
    }
}

// LTCDecoder implementation
impl LTCDecoder {
    pub fn try_new(apv: i32, queue_size: i32) -> Result<Self, LTCDecoderError> {
        let decoder = unsafe { raw::ltc_decoder_create(apv, queue_size) };
        if decoder.is_null() {
            Err(LTCDecoderError::CreateError)
        } else {
            Ok(LTCDecoder { inner: decoder })
        }
    }

    pub fn write(&mut self, buf: &[i32], posinfo: i64) {
        unsafe {
            raw::ltc_decoder_write(self.inner, buf.as_ptr(), buf.len() as libc::size_t, posinfo);
        }
    }

    pub fn write_double(&mut self, buf: &[f64], posinfo: i64) {
        unsafe {
            raw::ltc_decoder_write_double(
                self.inner,
                buf.as_ptr(),
                buf.len() as libc::size_t,
                posinfo,
            );
        }
    }

    pub fn write_float(&mut self, buf: &[f32], posinfo: i64) {
        unsafe {
            raw::ltc_decoder_write_float(
                self.inner,
                buf.as_ptr(),
                buf.len() as libc::size_t,
                posinfo,
            );
        }
    }

    pub fn write_s16(&mut self, buf: &[i16], posinfo: i64) {
        unsafe {
            raw::ltc_decoder_write_s16(
                self.inner,
                buf.as_ptr(),
                buf.len() as libc::size_t,
                posinfo,
            );
        }
    }

    pub fn write_u16(&mut self, buf: &[u16], posinfo: i64) {
        unsafe {
            raw::ltc_decoder_write_u16(
                self.inner,
                buf.as_ptr(),
                buf.len() as libc::size_t,
                posinfo,
            );
        }
    }

    pub fn read(&mut self) -> Option<LTCFrameExt> {
        let mut frame = LTCFrameExt {
            ltc: LTCFrame::new(),
            off_start: 0,
            off_end: 0,
            reverse: 0,
        };
        let result = unsafe { raw::ltc_decoder_read(self.inner, &mut frame as *mut _) };
        if result == 0 {
            None
        } else {
            Some(frame)
        }
    }

    pub fn queue_flush(&mut self) {
        unsafe {
            raw::ltc_decoder_queue_flush(self.inner);
        }
    }

    pub fn queue_length(&self) -> i32 {
        unsafe { raw::ltc_decoder_queue_length(self.inner) }
    }
}

// LTCEncoder implementation
impl LTCEncoder {
    pub fn try_new(
        sample_rate: f64,
        fps: f64,
        standard: LTCTVStandard,
        flags: i32,
    ) -> Result<Self, LTCEncoderError> {
        let raw_standard = standard.to_raw();
        let encoder = unsafe { raw::ltc_encoder_create(sample_rate, fps, raw_standard, flags) };
        if encoder.is_null() {
            Err(LTCEncoderError::CreateError)
        } else {
            Ok(LTCEncoder { inner: encoder })
        }
    }

    pub fn set_timecode(&mut self, timecode: &SMPTETimecode) {
        unsafe {
            raw::ltc_encoder_set_timecode(self.inner, timecode as *const _);
        }
    }

    pub fn get_timecode(&self) -> SMPTETimecode {
        let mut timecode = SMPTETimecode {
            hours: 0,
            mins: 0,
            secs: 0,
            frame: 0,
        };
        unsafe {
            raw::ltc_encoder_get_timecode(self.inner, &mut timecode as *mut _);
        }
        timecode
    }

    pub fn set_user_bits(&mut self, data: u32) {
        unsafe {
            raw::ltc_encoder_set_user_bits(self.inner, data as libc::c_ulong);
        }
    }

    pub fn inc_timecode(&mut self) -> bool {
        unsafe { raw::ltc_encoder_inc_timecode(self.inner) != 0 }
    }

    pub fn dec_timecode(&mut self) -> bool {
        unsafe { raw::ltc_encoder_dec_timecode(self.inner) != 0 }
    }

    pub fn set_frame(&mut self, frame: &LTCFrame) {
        unsafe {
            raw::ltc_encoder_set_frame(self.inner, frame as *const _);
        }
    }

    pub fn get_frame(&self) -> LTCFrame {
        let mut frame = LTCFrame::new();
        unsafe {
            raw::ltc_encoder_get_frame(self.inner, &mut frame as *mut _);
        }
        frame
    }

    pub fn get_buffer(&self, buf: &mut [i32]) -> i32 {
        unsafe { raw::ltc_encoder_get_buffer(self.inner, buf.as_mut_ptr()) }
    }

    pub fn get_bufptr(&self, flush: bool) -> (&[i32], i32) {
        let mut size: i32 = 0;
        let ptr = unsafe {
            raw::ltc_encoder_get_bufptr(self.inner, &mut size as *mut _, if flush { 1 } else { 0 })
        };
        let slice = unsafe { slice::from_raw_parts(ptr, size as usize) };
        (slice, size)
    }

    pub fn buffer_flush(&mut self) {
        unsafe {
            raw::ltc_encoder_buffer_flush(self.inner);
        }
    }

    pub fn get_buffersize(&self) -> usize {
        unsafe { raw::ltc_encoder_get_buffersize(self.inner) as usize }
    }

    pub fn reinit(
        &mut self,
        sample_rate: f64,
        fps: f64,
        standard: LTCTVStandard,
        flags: i32,
    ) -> Result<(), LTCEncoderError> {
        let result = unsafe {
            raw::ltc_encoder_reinit(self.inner, sample_rate, fps, standard.to_raw(), flags)
        };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::ReinitError)
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            raw::ltc_encoder_reset(self.inner);
        }
    }

    pub fn set_bufsize(&mut self, sample_rate: f64, fps: f64) -> Result<(), LTCEncoderError> {
        let result = unsafe { raw::ltc_encoder_set_bufsize(self.inner, sample_rate, fps) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::BufferSizeError)
        }
    }

    pub fn get_volume(&self) -> f64 {
        unsafe { raw::ltc_encoder_get_volume(self.inner) }
    }

    pub fn set_volume(&mut self, dbfs: f64) -> Result<(), LTCEncoderError> {
        let result = unsafe { raw::ltc_encoder_set_volume(self.inner, dbfs) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::VolumeError)
        }
    }

    pub fn get_filter(&self) -> f64 {
        unsafe { raw::ltc_encoder_get_filter(self.inner) }
    }

    pub fn set_filter(&mut self, rise_time: f64) {
        unsafe {
            raw::ltc_encoder_set_filter(self.inner, rise_time);
        }
    }

    pub fn encode_byte(&mut self, byte: i32, speed: f64) -> Result<(), LTCEncoderError> {
        let result = unsafe { raw::ltc_encoder_encode_byte(self.inner, byte, speed) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::EncodeError)
        }
    }

    pub fn end_encode(&mut self) -> Result<(), LTCEncoderError> {
        let result = unsafe { raw::ltc_encoder_end_encode(self.inner) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::EncodeError)
        }
    }

    pub fn encode_frame(&mut self) {
        unsafe {
            raw::ltc_encoder_encode_frame(self.inner);
        }
    }

    pub fn encode_reversed_frame(&mut self) {
        unsafe {
            raw::ltc_encoder_encode_reversed_frame(self.inner);
        }
    }
}

impl LTCTVStandard {
    pub(crate) fn to_raw(self) -> raw::LTC_TV_STANDARD {
        match self {
            LTCTVStandard::LTCTV_525_60 => raw::LTC_TV_STANDARD::LTC_TV_525_60,
            LTCTVStandard::LTCTV_625_50 => raw::LTC_TV_STANDARD::LTC_TV_625_50,
            LTCTVStandard::LTCTV_1125_60 => raw::LTC_TV_STANDARD::LTC_TV_1125_60,
            LTCTVStandard::LTCTV_FILM_24 => raw::LTC_TV_STANDARD::LTC_TV_FILM_24,
        }
    }
}

impl Drop for LTCEncoder {
    fn drop(&mut self) {
        unsafe {
            raw::ltc_encoder_free(self.inner);
        }
    }
}

impl Drop for LTCDecoder {
    fn drop(&mut self) {
        unsafe {
            raw::ltc_decoder_free(self.inner);
        }
    }
}