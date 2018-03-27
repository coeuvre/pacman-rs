use std::os::raw::{c_char, c_int, c_uchar};

extern "C" {
    pub fn stbi_load_from_memory(
        buffer: *const c_uchar,
        len: c_int,
        x: *mut c_int,
        y: *mut c_int,
        channels_in_file: *mut c_int,
        desired_channels: c_int,
    ) -> *mut c_uchar;

    pub fn stbi_image_free(retval_from_stbi_load: *mut c_uchar);

    pub fn stbi_failure_reason() -> *const c_char;
}
