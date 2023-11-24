mod transport;
mod models;

use allo_isolate::{
    ffi::{DartCObject, DartPort},
    IntoDart, Isolate,
};
use lazy_static::lazy_static;
use std::{ffi::c_void, intrinsics::transmute, io, os::raw::c_char};
use tokio::runtime::{Builder, Runtime};

use models::{HandleError, ToStringFromPtr};

lazy_static! {
    static ref RUNTIME: io::Result<Runtime> = Builder::new_multi_thread()
        .enable_all()
        .thread_name("ledger_ffi")
        .build();
}

#[macro_export]
macro_rules! runtime {
    () => {
        RUNTIME.as_ref().unwrap()
    };
}

#[no_mangle]
pub unsafe extern "C" fn ll_store_dart_post_cobject(ptr: *mut c_void) {
    let ptr = transmute::<
        *mut c_void,
        unsafe extern "C" fn(port_id: DartPort, message: *mut DartCObject) -> bool,
    >(ptr);

    allo_isolate::store_dart_post_cobject(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn ll_cstring_to_void_ptr(ptr: *mut c_char) -> *mut c_void {
    ptr.to_string_from_ptr().to_ptr_from_address::<c_void>()
}

#[no_mangle]
pub unsafe extern "C" fn ll_free_cstring(ptr: *mut c_char) {
    ptr.to_string_from_ptr();
}

pub trait ToPtrAddress {
    fn to_ptr_address(self) -> String;
}

impl<T> ToPtrAddress for *mut T {
    fn to_ptr_address(self) -> String {
        (self as usize).to_string()
    }
}

pub trait ToPtrFromAddress {
    fn to_ptr_from_address<T>(self) -> *mut T;
}

impl ToPtrFromAddress for String {
    fn to_ptr_from_address<T>(self) -> *mut T {
        self.parse::<usize>().unwrap() as *mut T
    }
}
trait PostWithResult {
    fn post_with_result(&self, data: impl IntoDart) -> Result<(), String>;
}

impl PostWithResult for Isolate {
    fn post_with_result(&self, data: impl IntoDart) -> Result<(), String> {
        match self.post(data) {
            true => Ok(()),
            false => Err("Message was not posted successfully").handle_error(),
        }
    }
}
