use core::slice;

use super::frame::LTCFrame;
use super::LTCTVStandard;
use super::SMPTETimecode;
use super::SampleType;
use crate::error::LTCEncoderError;
use crate::raw;

#[derive(Debug)]
pub struct LTCEncoder {
    inner_unsafe_ptr: *mut raw::LTCEncoder,
}

impl Drop for LTCEncoder {
    fn drop(&mut self) {
        unsafe {
            raw::ltc_encoder_free(self.inner_unsafe_ptr);
        }
    }
}

impl<'a> LTCEncoder {
    pub fn try_new(
        sample_rate: f64,
        fps: f64,
        standard: LTCTVStandard,
        flags: crate::consts::LtcBgFlags,
    ) -> Result<Self, LTCEncoderError> {
        // Safety: the C function does not modify memory, it only allocates memory. Drop is implemented for LTCEncoder
        let encoder =
            unsafe { raw::ltc_encoder_create(sample_rate, fps, standard.to_raw(), flags as i32) };
        if encoder.is_null() {
            Err(LTCEncoderError::CreateError)
        } else {
            Ok(LTCEncoder {
                inner_unsafe_ptr: encoder,
            })
        }
    }

    // TODO: this might be incorrect
    pub fn set_timecode(&mut self, timecode: &SMPTETimecode) {
        // Safety: We own self, the function is assumed to only read the timecode and write to self
        unsafe {
            raw::ltc_encoder_set_timecode(self.inner_unsafe_ptr, timecode.inner_unsafe_ptr);
        }
    }

    pub fn get_timecode(&self) -> SMPTETimecode {
        let mut timecode = SMPTETimecode::default();
        // We own timecode, the function is assumed to only read from self and write to timecode
        unsafe {
            #[allow(clippy::needless_borrow)] // for clarity
            raw::ltc_encoder_get_timecode(self.inner_unsafe_ptr, (&mut timecode).inner_unsafe_ptr);
        }
        timecode
    }

    pub fn set_user_bits(&mut self, data: u32) {
        // SAFETY: We own self
        unsafe {
            raw::ltc_encoder_set_user_bits(self.inner_unsafe_ptr, data as libc::c_ulong);
        }
    }

    pub fn inc_timecode(&mut self) -> bool {
        // SAFETY: We own self
        unsafe { raw::ltc_encoder_inc_timecode(self.inner_unsafe_ptr) != 0 }
    }

    pub fn dec_timecode(&mut self) -> bool {
        // SAFETY: We own self
        unsafe { raw::ltc_encoder_dec_timecode(self.inner_unsafe_ptr) != 0 }
    }

    pub fn set_frame(&mut self, frame: &LTCFrame) {
        // SAFETY: We own self, the function is assumed to only read the frame and write to self
        unsafe { raw::ltc_encoder_set_frame(self.inner_unsafe_ptr, frame.inner_unsafe_ptr) }
    }

    pub fn get_frame(&self) -> LTCFrame {
        let mut frame = LTCFrame::new();
        // SAFETY: We own frame. The function is assumed to only read from self and write to frame
        unsafe {
            #[allow(clippy::needless_borrow)] // for clarity
            raw::ltc_encoder_get_frame(self.inner_unsafe_ptr, (&mut frame).inner_unsafe_ptr);
        }
        frame
    }

    pub fn copy_buffer_inplace(&self, buf: &mut [SampleType]) -> i32 {
        unsafe { raw::ltc_encoder_copy_buffer(self.inner_unsafe_ptr, buf.as_mut_ptr()) }
    }

    pub fn copy_buffer(&self) -> (Vec<u8>, usize) {
        let mut buf = vec![0; self.get_buffersize()];
        let size = unsafe { raw::ltc_encoder_copy_buffer(self.inner_unsafe_ptr, buf.as_mut_ptr()) };
        (buf, size as usize)
    }

    // TODO: Possible leak? does ptr ever get deallocated - maybe when the encoder is deallocated?
    pub fn get_buf_ref(&'a self, flush: bool) -> (&'a [SampleType], usize) {
        // SAFETY: The buffer (pointed at by ptr) outlives the function as it has the same
        // lifetime as self
        let mut ptr = std::ptr::null_mut();
        // SAFETY: Self is assumed to only be read - for the buffersize
        let size = unsafe {
            raw::ltc_encoder_get_bufferptr(
                self.inner_unsafe_ptr,
                &mut ptr,
                if flush { 1 } else { 0 },
            )
        };

        (
            unsafe { slice::from_raw_parts(ptr, size as usize) },
            size as usize,
        )
    }

    // TODO: Possible leak? does ptr ever get deallocated - maybe when the encoder is deallocated?
    pub fn get_buf_ref_mut(&'a mut self, flush: bool) -> (&'a mut [SampleType], usize) {
        // SAFETY: The buffer (pointed at by ptr) outlives the function as it has the same
        // lifetime as self
        let mut ptr = std::ptr::null_mut();
        // SAFETY: Self is assumed to only be read - for the buffersize
        let size = unsafe {
            raw::ltc_encoder_get_bufferptr(
                self.inner_unsafe_ptr,
                &mut ptr,
                if flush { 1 } else { 0 },
            )
        };
        (
            unsafe { slice::from_raw_parts_mut(ptr, size as usize) },
            size as usize,
        )
    }

    pub fn buffer_flush(&mut self) {
        unsafe {
            raw::ltc_encoder_buffer_flush(self.inner_unsafe_ptr);
        }
    }

    pub fn get_buffersize(&self) -> usize {
        // SAFETY: The function is assumed to only read self
        unsafe { raw::ltc_encoder_get_buffersize(self.inner_unsafe_ptr) }
    }

    pub fn reinit(
        &mut self,
        sample_rate: f64,
        fps: f64,
        standard: LTCTVStandard,
        flags: crate::consts::LtcBgFlags,
    ) -> Result<(), LTCEncoderError> {
        let result = unsafe {
            raw::ltc_encoder_reinit(
                self.inner_unsafe_ptr,
                sample_rate,
                fps,
                standard.to_raw(),
                flags as i32,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::ReinitError)
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            raw::ltc_encoder_reset(self.inner_unsafe_ptr);
        }
    }

    pub fn set_bufsize(&mut self, sample_rate: f64, fps: f64) -> Result<(), LTCEncoderError> {
        let result =
            unsafe { raw::ltc_encoder_set_buffersize(self.inner_unsafe_ptr, sample_rate, fps) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::BufferSizeError)
        }
    }

    pub fn get_volume(&self) -> f64 {
        unsafe { raw::ltc_encoder_get_volume(self.inner_unsafe_ptr) }
    }

    pub fn set_volume(&mut self, dbfs: f64) -> Result<(), LTCEncoderError> {
        let result = unsafe { raw::ltc_encoder_set_volume(self.inner_unsafe_ptr, dbfs) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::VolumeError)
        }
    }

    pub fn get_filter(&self) -> f64 {
        unsafe { raw::ltc_encoder_get_filter(self.inner_unsafe_ptr) }
    }

    pub fn set_filter(&mut self, rise_time: f64) {
        unsafe {
            raw::ltc_encoder_set_filter(self.inner_unsafe_ptr, rise_time);
        }
    }

    pub fn encode_byte(&mut self, byte: i32, speed: f64) -> Result<(), LTCEncoderError> {
        let result = unsafe { raw::ltc_encoder_encode_byte(self.inner_unsafe_ptr, byte, speed) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::EncodeError)
        }
    }

    pub fn end_encode(&mut self) -> Result<(), LTCEncoderError> {
        let result = unsafe { raw::ltc_encoder_end_encode(self.inner_unsafe_ptr) };
        if result == 0 {
            Ok(())
        } else {
            Err(LTCEncoderError::EncodeError)
        }
    }

    pub fn encode_frame(&mut self) {
        unsafe {
            raw::ltc_encoder_encode_frame(self.inner_unsafe_ptr);
        }
    }

    pub fn encode_reversed_frame(&mut self) {
        unsafe {
            raw::ltc_encoder_encode_reversed_frame(self.inner_unsafe_ptr);
        }
    }
}
