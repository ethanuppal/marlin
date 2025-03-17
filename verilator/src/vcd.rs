// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

use __private::VcdImpl;

#[doc(hidden)]
pub mod __private {
    use super::Vcd;

    pub(crate) struct VcdImpl {
        pub(crate) handle: *mut libc::c_void,
        pub(crate) dump: extern "C" fn(*mut libc::c_void, u64),
        pub(crate) close_and_delete: extern "C" fn(*mut libc::c_void),
    }

    impl Drop for VcdImpl {
        fn drop(&mut self) {
            (self.close_and_delete)(self.handle);
        }
    }

    pub fn new_vcd_useless<'top>() -> Vcd<'top> {
        Vcd {
            inner: None,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Clone, Copy)]
pub struct VcdApi {
    pub open_trace: extern "C" fn(
        *mut libc::c_void,
        *const libc::c_char,
    ) -> *mut libc::c_void,
    pub dump: extern "C" fn(*mut libc::c_void, u64),
    pub close_and_delete: extern "C" fn(*mut libc::c_void),
}

/// A VCD dump.
pub struct Vcd<'top> {
    inner: Option<__private::VcdImpl>,
    _marker: PhantomData<&'top ()>,
}

impl Vcd<'_> {
    pub fn from(
        input: (
            *mut libc::c_void,
            extern "C" fn(*mut libc::c_void, u64),
            extern "C" fn(*mut libc::c_void),
        ),
    ) -> Self {
        Self {
            inner: Some(VcdImpl {
                handle: input.0,
                dump: input.1,
                close_and_delete: input.2,
            }),
            _marker: PhantomData,
        }
    }

    pub fn new_useless() -> Self {
        Self {
            inner: None,
            _marker: PhantomData,
        }
    }

    /// Documentation taken from the Verilator header file:
    ///
    /// > Write one cycle of dump data
    /// > Call with the current context's time just after eval'ed,
    /// > e.g. `->dump(contextp->time())`.
    pub fn dump(&mut self, timestamp: u64) {
        if let Some(inner) = &self.inner {
            (inner.dump)(inner.handle, timestamp);
        }
    }

    /// The VCD is automatically closed when dropped, but it may be useful to
    /// call this manually.
    pub fn close(self) {}
}
