// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::marker::PhantomData;

#[doc(hidden)]
pub mod __private {
    use std::{ffi, marker::PhantomData};

    use super::Vcd;

    pub(crate) struct VcdImpl {
        pub(crate) handle: *mut ffi::c_void,
        pub(crate) dump: extern "C" fn(*mut ffi::c_void, u64),
        close_and_delete: extern "C" fn(*mut ffi::c_void),
    }

    impl Drop for VcdImpl {
        fn drop(&mut self) {
            (self.close_and_delete)(self.handle);
        }
    }
    #[derive(Clone, Copy)]
    pub struct VcdApi {
        pub open_trace: extern "C" fn(
            *mut ffi::c_void,
            *const ffi::c_char,
        ) -> *mut ffi::c_void,
        pub dump: extern "C" fn(*mut ffi::c_void, u64),
        pub close_and_delete: extern "C" fn(*mut ffi::c_void),
    }

    pub fn new_vcd<'ctx>(
        handle: *mut ffi::c_void,
        dump: extern "C" fn(*mut ffi::c_void, u64),
        close_and_delete: extern "C" fn(*mut ffi::c_void),
    ) -> Vcd<'ctx> {
        Vcd {
            inner: Some(VcdImpl {
                handle,
                dump,
                close_and_delete,
            }),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn new_vcd_useless<'ctx>() -> Vcd<'ctx> {
        Vcd {
            inner: None,
            _marker: PhantomData,
        }
    }
}

/// A VCD dump.
pub struct Vcd<'ctx> {
    inner: Option<__private::VcdImpl>,
    _marker: PhantomData<&'ctx ()>,
}

impl Vcd<'_> {
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
