#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

pub use internal::std_string as std_string;
pub use internal::pxrInternal_v0_22__pxrReserved___VtValue as pxr_VtValue;

pub use internal::std_string_ctor as std_string_ctor;

pub use internal::std_string_from_char_ptr as std_string_from_char_ptr;

pub use internal::std_string_dtor as std_string_dtor;

pub use internal::std_string_copy_ctor as std_string_copy_ctor;

pub use internal::std_string_move_ctor as std_string_move_ctor;

pub use internal::std_string_c_str as std_string_c_str;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_ctor as pxr_VtValue_ctor;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_copy_ctor as pxr_VtValue_copy_ctor;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_move_ctor as pxr_VtValue_move_ctor;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_dtor as pxr_VtValue_dtor;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetArraySize as pxr_VtValue_GetArraySize;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetTypeName as pxr_VtValue_GetTypeName;


mod internal {

use std::os::raw::*;


#[repr(C)]
pub struct std_string {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct pxrInternal_v0_22__pxrReserved___VtValue {
    _unused: [u8; 0],
}
pub type pxr_VtValue = pxrInternal_v0_22__pxrReserved___VtValue;

extern "C" {

pub fn std_string_ctor(result: *mut *mut std_string);

pub fn std_string_from_char_ptr(result: *mut *mut std_string, char_ptr: *const c_char);

pub fn std_string_dtor(this_: *mut std_string);

pub fn std_string_copy_ctor(result: *mut *mut std_string, other: *const std_string);

pub fn std_string_move_ctor(result: *mut *mut std_string, other: *mut std_string);

pub fn std_string_c_str(this_: *const std_string, result: *mut *const c_char);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_ctor(result: *mut *mut pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_copy_ctor(result: *mut *mut pxr_VtValue, other: *const pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_move_ctor(result: *mut *mut pxr_VtValue, other: *mut pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_dtor(this_: *mut pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetArraySize(this_: *const pxr_VtValue, result: *mut size_t);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetTypeName(this_: *const pxr_VtValue, result: *mut std_string);

} // extern C
} // mod internal
