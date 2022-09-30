#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

pub use internal::_std_type_info as _std_type_info;
pub use internal::std_string as std_string;
pub use internal::pxrInternal_v0_22__pxrReserved___VtValue as pxr_VtValue;

pub use internal::_std_type_info_op_eq as _std_type_info_op_eq;

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

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_IsArrayValued as pxr_VtValue_IsArrayValued;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetArraySize as pxr_VtValue_GetArraySize;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetTypeid as pxr_VtValue_GetTypeid;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetElementTypeid as pxr_VtValue_GetElementTypeid;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetTypeName as pxr_VtValue_GetTypeName;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_CastToTypeOf as pxr_VtValue_CastToTypeOf;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_CastToTypeOf_1 as pxr_VtValue_CastToTypeOf_1;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_CanCastToTypeid as pxr_VtValue_CanCastToTypeid;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_IsEmpty as pxr_VtValue_IsEmpty;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_CanHash as pxr_VtValue_CanHash;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetHash as pxr_VtValue_GetHash;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_op_eq as pxr_VtValue_op_eq;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_VtValue_float as pxr_VtValue_VtValue_float;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_IsHolding_float as pxr_VtValue_IsHolding_float;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_Remove_float as pxr_VtValue_Remove_float;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_Get_float as pxr_VtValue_Get_float;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_GetWithDefault_float as pxr_VtValue_GetWithDefault_float;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_Cast_float as pxr_VtValue_Cast_float;

pub use internal::pxrInternal_v0_22__pxrReserved___VtValue_Cast_float_1 as pxr_VtValue_Cast_float_1;


mod internal {

use std::os::raw::*;

#[repr(C)]
pub struct _std_type_info {
    pub __name: *const c_char,
}

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

pub fn _std_type_info_op_eq(this_: *const _std_type_info, result: *mut c_bool, __arg: *const _std_type_info);

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

pub fn pxrInternal_v0_22__pxrReserved___VtValue_IsArrayValued(this_: *const pxr_VtValue, result: *mut c_bool);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetArraySize(this_: *const pxr_VtValue, result: *mut size_t);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetTypeid(this_: *const pxr_VtValue, result: *mut *const _std_type_info);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetElementTypeid(this_: *const pxr_VtValue, result: *mut *const _std_type_info);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetTypeName(this_: *const pxr_VtValue, result: *mut std_string);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_CastToTypeOf(result: *mut pxr_VtValue, val: *const pxr_VtValue, other: *const pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_CastToTypeOf_1(this_: *mut pxr_VtValue, result: *mut *mut pxr_VtValue, other: *const pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_CanCastToTypeid(this_: *const pxr_VtValue, result: *mut c_bool, type_: *const _std_type_info);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_IsEmpty(this_: *const pxr_VtValue, result: *mut c_bool);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_CanHash(this_: *const pxr_VtValue, result: *mut c_bool);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetHash(this_: *const pxr_VtValue, result: *mut size_t);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_op_eq(this_: *const pxr_VtValue, result: *mut c_bool, rhs: *const pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_VtValue_float(result: *mut *mut pxr_VtValue, obj: *const c_float);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_IsHolding_float(this_: *const pxr_VtValue, result: *mut c_bool);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_Remove_float(this_: *mut pxr_VtValue, result: *mut c_float);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_Get_float(this_: *const pxr_VtValue, result: *mut *const c_float);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_GetWithDefault_float(this_: *const pxr_VtValue, result: *mut c_float, def: *const c_float);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_Cast_float(result: *mut pxr_VtValue, val: *const pxr_VtValue);

pub fn pxrInternal_v0_22__pxrReserved___VtValue_Cast_float_1(this_: *mut pxr_VtValue, result: *mut *mut pxr_VtValue);

} // extern C
} // mod internal
