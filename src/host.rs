extern crate c_ares_sys;
extern crate libc;

use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem;
use std::net::{
    Ipv4Addr,
    Ipv6Addr,
};
use std::ptr;
use std::str;

use error::AresError;
use types::{
    AddressFamily,
    hostent,
    IpAddr,
};
use utils::{
    address_family,
    ares_error,
};

/// The result of a successful host lookup.
pub struct HostResults<'a> {
    hostent: &'a hostent,
}

/// An alias, as retrieved from a host lookup.
pub struct HostAliasResult<'a> {
    h_alias: *const libc::c_char,
    phantom: PhantomData<&'a hostent>,
}

/// An address, as retrieved from a host lookup.
pub struct HostAddressResult<'a> {
    family: AddressFamily,
    h_addr: *const libc::c_char,
    phantom: PhantomData<&'a hostent>,
}

impl<'a> HostResults<'a> {
    fn new(hostent: &'a hostent) -> HostResults {
        HostResults {
            hostent: hostent,
        }
    }

    /// Returns the hostname from this `HostResults`.
    pub fn hostname(&self) -> &str {
        unsafe {
            let c_str = CStr::from_ptr((*self.hostent).h_name);
            str::from_utf8_unchecked(c_str.to_bytes())
        }
    }

    /// Returns an iterator over the `HostAddressResult` values in this
    /// `HostResults`.
    pub fn addresses(&self) -> HostAddressResultsIterator {
        match address_family(self.hostent.h_addrtype) {
            Some(family) => HostAddressResultsIterator {
                family: family,
                next: self.hostent.h_addr_list as *const *const _,
                phantom: PhantomData,
            },
            None => HostAddressResultsIterator {
                family: AddressFamily::INET,
                next: ptr::null_mut(),
                phantom: PhantomData,
            }
        }
    }

    /// Returns an iterator over the `HostAliasResult` values in this
    /// `HostResults`.
    pub fn aliases(&self) -> HostAliasResultsIterator {
        HostAliasResultsIterator {
            next: self.hostent.h_aliases as *const *const _,
            phantom: PhantomData,
        }
    }
}

pub struct HostAddressResultsIterator<'a> {
    family: AddressFamily,
    next: *const *const libc::c_char,
    phantom: PhantomData<&'a hostent>,
}

impl<'a> Iterator for HostAddressResultsIterator<'a> {
    type Item = HostAddressResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let h_addr = unsafe { *self.next };
        if h_addr.is_null() {
            None
        } else {
            self.next = unsafe { self.next.offset(1) };
            let addr_result = HostAddressResult {
                family: self.family,
                h_addr: h_addr,
                phantom: PhantomData,
            };
            Some(addr_result)
        }
    }
}

pub struct HostAliasResultsIterator<'a> {
    next: *const *const libc::c_char,
    phantom: PhantomData<&'a hostent>,
}

impl<'a> Iterator for HostAliasResultsIterator<'a> {
    type Item = HostAliasResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let h_alias = unsafe { *self.next };
        if h_alias.is_null() {
            None
        } else {
            self.next = unsafe { self.next.offset(1) };
            let alias_result = HostAliasResult {
                h_alias: h_alias,
                phantom: PhantomData,
            };
            Some(alias_result)
        }
    }
}

unsafe impl<'a> Send for HostResults<'a> { }
unsafe impl<'a> Sync for HostResults<'a> { }
unsafe impl<'a> Send for HostAliasResult<'a> { }
unsafe impl<'a> Sync for HostAliasResult<'a> { }
unsafe impl<'a> Send for HostAliasResultsIterator<'a> { }
unsafe impl<'a> Sync for HostAliasResultsIterator<'a> { }
unsafe impl<'a> Send for HostAddressResult<'a> { }
unsafe impl<'a> Sync for HostAddressResult<'a> { }
unsafe impl<'a> Send for HostAddressResultsIterator<'a> { }
unsafe impl<'a> Sync for HostAddressResultsIterator<'a> { }

impl<'a> HostAddressResult<'a> {
    /// Returns the IP address in this `HostResult`.
    pub fn ip_address(&self) -> IpAddr {
        match self.family {
            AddressFamily::INET => {
                let ipv4 = self.ipv4_address();
                IpAddr::V4(ipv4)
            },
            AddressFamily::INET6 => {
                let ipv6 = self.ipv6_addr();
                IpAddr::V6(ipv6)
            },
        }
    }

    fn ipv4_address(&self) -> Ipv4Addr {
        let h_addr = self.h_addr;
        unsafe {
            Ipv4Addr::new(
                *h_addr as u8,
                *h_addr.offset(1) as u8,
                *h_addr.offset(2) as u8,
                *h_addr.offset(3) as u8)
        }
    }

    fn ipv6_addr(&self) -> Ipv6Addr {
        let h_addr = self.h_addr;
        unsafe {
            Ipv6Addr::new(
                ((*h_addr as u16) << 8) + *h_addr.offset(1) as u16,
                ((*h_addr.offset(2) as u16) << 8) + *h_addr.offset(3) as u16,
                ((*h_addr.offset(4) as u16) << 8) + *h_addr.offset(5) as u16,
                ((*h_addr.offset(6) as u16) << 8) + *h_addr.offset(7) as u16,
                ((*h_addr.offset(8) as u16) << 8) + *h_addr.offset(9) as u16,
                ((*h_addr.offset(10) as u16) << 8) + *h_addr.offset(11) as u16,
                ((*h_addr.offset(12) as u16) << 8) + *h_addr.offset(13) as u16,
                ((*h_addr.offset(14) as u16) << 8) + *h_addr.offset(15) as u16)
        }
    }
}

impl<'a> HostAliasResult<'a> {
    /// Returns the alias in this `HostAliasResult`.
    pub fn alias(&self) -> &str {
        unsafe {
            let c_str = CStr::from_ptr(self.h_alias);
            str::from_utf8_unchecked(c_str.to_bytes())
        }
    }
}

pub unsafe extern "C" fn get_host_callback<F>(
    arg: *mut libc::c_void,
    status: libc::c_int,
    _timeouts: libc::c_int,
    hostent: *mut c_ares_sys::Struct_hostent)
    where F: FnOnce(Result<HostResults, AresError>) + 'static {
    let handler: Box<F> = mem::transmute(arg);
    let result = if status != c_ares_sys::ARES_SUCCESS {
        Err(ares_error(status))
    } else {
        let hostent_ref = &*(hostent as *mut hostent);
        let host_results = HostResults::new(hostent_ref);
        Ok(host_results)
    };
    handler(result);
}
