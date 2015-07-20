extern crate c_ares_sys;
extern crate libc;

use std::ffi::CStr;
use std::mem;
use std::ptr;
use std::slice;
use std::str;

use types::{
    AresError,
    hostent,
};
use utils::ares_error;

/// The result of a successful lookup for a CNAME record.
#[allow(raw_pointer_derive)]
#[derive(Debug)]
pub struct CNameResult<'a> {
    /// The canonical name record.
    pub cname: &'a str,

    /// The underlying hostent into which the cname string points.
    hostent: *mut hostent,
}

impl<'a> CNameResult<'a> {
    /// Obtain a CNameResult from the response to a CNAME lookup.
    pub fn parse_from(data: &[u8]) -> Result<CNameResult, AresError> {
        let mut hostent: *mut hostent = ptr::null_mut();
        let parse_status = unsafe {
            c_ares_sys::ares_parse_a_reply(
                data.as_ptr(),
                data.len() as libc::c_int,
                &mut hostent as *mut *mut _ as *mut *mut c_ares_sys::Struct_hostent,
                ptr::null_mut(),
                ptr::null_mut())
        };
        if parse_status != c_ares_sys::ARES_SUCCESS {
            Err(ares_error(parse_status))
        } else {
            let result = unsafe { CNameResult::new(hostent) };
            Ok(result)
        }
    }

    unsafe fn new(hostent: *mut hostent) -> CNameResult<'a> {
        let c_str = CStr::from_ptr((*hostent).h_name);
        let slice = str::from_utf8_unchecked(c_str.to_bytes());
        CNameResult {
            cname: slice,
            hostent: hostent,
        }
    }
}

impl<'a> Drop for CNameResult<'a> {
    fn drop(&mut self) {
        unsafe {
            c_ares_sys::ares_free_hostent(
                self.hostent as *mut c_ares_sys::Struct_hostent);
        }
    }
}

pub unsafe extern "C" fn query_cname_callback<F>(
    arg: *mut libc::c_void,
    status: libc::c_int,
    _timeouts: libc::c_int,
    abuf: *mut libc::c_uchar,
    alen: libc::c_int)
    where F: FnOnce(Result<CNameResult, AresError>) + 'static {
    let result = if status != c_ares_sys::ARES_SUCCESS {
        Err(ares_error(status))
    } else {
        let data = slice::from_raw_parts(abuf, alen as usize);
        CNameResult::parse_from(data)
    };
    let handler: Box<F> = mem::transmute(arg);
    handler(result);
}