use std::os::raw::{c_char, c_uchar, c_int};

#[allow(non_camel_case_types)]
pub type stbi_uc = c_uchar;

extern "C" {
    pub fn stbi_load(filename: *const c_char, x: *mut c_int, y: *mut c_int, comp: *mut c_int, req_comp: c_int) -> *mut stbi_uc;
    pub fn stbi_image_free(retval_from_stbi_load: *mut stbi_uc);
    pub fn stbi_failure_reason() -> *const c_char;
}