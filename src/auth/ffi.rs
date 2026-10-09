//! Minimal ABI declarations from Security/Authorization.h. Keeping the four
//! functions used here avoids the std dependency of security-framework-sys.
#![allow(non_snake_case, non_upper_case_globals)]
use core::ffi::{c_char, c_void};

pub type AuthorizationRef = *mut c_void;
pub const errAuthorizationSuccess: i32 = 0;
pub const kAuthorizationFlagDefaults: u32 = 0;
pub const kAuthorizationFlagInteractionAllowed: u32 = 1;
pub const kAuthorizationFlagExtendRights: u32 = 2;

#[repr(C)]
pub struct AuthorizationItem {
    pub name: *const c_char,
    pub valueLength: usize,
    pub value: *mut c_void,
    pub flags: u32,
}

#[repr(C)]
pub struct AuthorizationRights {
    pub count: u32,
    pub items: *mut AuthorizationItem,
}
pub type AuthorizationEnvironment = AuthorizationRights;

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    pub fn AuthorizationCreate(
        rights: *const AuthorizationRights,
        environment: *const AuthorizationEnvironment,
        flags: u32,
        authorization: *mut AuthorizationRef,
    ) -> i32;
    pub fn AuthorizationCopyRights(
        authorization: AuthorizationRef,
        rights: *const AuthorizationRights,
        environment: *const AuthorizationEnvironment,
        flags: u32,
        authorized_rights: *mut *mut AuthorizationRights,
    ) -> i32;
    pub fn AuthorizationFreeItemSet(rights: *mut AuthorizationRights) -> i32;
    pub fn AuthorizationFree(authorization: AuthorizationRef, flags: u32) -> i32;
}
