#![allow(dead_code,
non_camel_case_types,
non_upper_case_globals,
non_snake_case)]

extern crate libc;

use std::ptr;

#[link(name = "resample")]
extern "C" {
    pub fn resample_open(highQuality: ::std::os::raw::c_int, minFactor: f64,
                         maxFactor: f64) -> *mut ::std::os::raw::c_void;
    pub fn resample_dup(handle: *const ::std::os::raw::c_void)
                        -> *mut ::std::os::raw::c_void;
    pub fn resample_get_filter_width(handle: *const ::std::os::raw::c_void)
                                     -> ::std::os::raw::c_int;
    pub fn resample_process(handle: *mut ::std::os::raw::c_void, factor: f64,
                            inBuffer: *mut f32,
                            inBufferLen: ::std::os::raw::c_int,
                            lastFlag: ::std::os::raw::c_int,
                            inBufferUsed: *mut ::std::os::raw::c_int,
                            outBuffer: *mut f32,
                            outBufferLen: ::std::os::raw::c_int)
                            -> ::std::os::raw::c_int;
    pub fn resample_close(handle: *mut ::std::os::raw::c_void);
}


pub struct Resample {
    handler: *mut ::std::os::raw::c_void
}

impl Drop for Resample {
    fn drop(&mut self) {
        unsafe {
            resample_close(self.handler)
        }
        self.handler = ptr::null_mut();
    }
}

impl Clone for Resample {
    fn clone(&self) -> Resample {
        let handler = unsafe {
            resample_dup(self.handler)
        };
        Resample { handler: handler }
    }
}

impl Resample {
    pub fn new(high_quality: bool, min_factor: f64, max_factor: f64) -> Option<Resample> {
        let high = match high_quality {
            true => 1,
            false => 0,
        };
        let resample_id = unsafe {
            resample_open(high, min_factor, max_factor)
        };
        if resample_id.is_null() {
            None
        } else {
            Some(Resample { handler: resample_id })
        }
    }

    pub fn get_filter_width(&self) -> i32 {
        unsafe { resample_get_filter_width(self.handler) }
    }

    pub fn process(&self, factor: f64, buf_in: &mut [f32], buf_out: &mut [f32], is_last: bool) ->
    Option<(i32, i32)> {
        let mut in_buffer_used = 0;
        let last = match is_last {
            true => 1,
            false => 0,
        };
        let out_samples_count = unsafe {
            resample_process(self.handler,
                             factor,
                             buf_in.as_mut_ptr() as *mut f32,
                             buf_in.len() as i32,
                             last,
                             &mut in_buffer_used,
                             buf_out.as_mut_ptr() as *mut f32,
                             buf_out.len() as i32)
        };
        if out_samples_count == -1 {
            None
        } else {
            Some((in_buffer_used, out_samples_count))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Resample;

    #[test]
    fn it_works() {
        let resample = Resample::new(true, 1.0, 1.2).unwrap();
        resample.get_filter_width();
        let resample_cloned = resample.clone();
        let buf_in: &mut [f32; 100] = &mut [0.0; 100];
        let buf_out: &mut [f32; 200] = &mut [0.0; 200];
        let (i, o) = resample_cloned.process(1.1, &mut buf_in[0..100], &mut buf_out[0..200],
                                             false).unwrap();
        assert_eq!(i, 100);
        assert_eq!(o, 80);
    }
}
