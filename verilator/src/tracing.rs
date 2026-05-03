// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{marker::PhantomData, path::Path};

#[doc(hidden)]
pub mod __private {
    use std::{ffi, marker::PhantomData};

    use super::Trace;

    pub(crate) struct TraceImpl {
        pub(crate) handle: *mut ffi::c_void,
        pub(crate) dump: extern "C" fn(*mut ffi::c_void, u64),
        pub(crate) open_next: extern "C" fn(*mut ffi::c_void, bool),
        pub(crate) flush: extern "C" fn(*mut ffi::c_void),
        close_and_delete: extern "C" fn(*mut ffi::c_void),
    }

    impl Drop for TraceImpl {
        fn drop(&mut self) {
            (self.close_and_delete)(self.handle);
        }
    }
    #[derive(Clone, Copy)]
    pub struct TraceApi {
        pub open_trace: extern "C" fn(
            *mut ffi::c_void,
            *const ffi::c_char,
        ) -> *mut ffi::c_void,
        pub dump: extern "C" fn(*mut ffi::c_void, u64),
        pub open_next: extern "C" fn(*mut ffi::c_void, bool),
        pub flush: extern "C" fn(*mut ffi::c_void),
        pub close_and_delete: extern "C" fn(*mut ffi::c_void),
    }

    pub fn new_trace<'ctx>(
        handle: *mut ffi::c_void,
        dump: extern "C" fn(*mut ffi::c_void, u64),
        open_next: extern "C" fn(*mut ffi::c_void, bool),
        flush: extern "C" fn(*mut ffi::c_void),
        close_and_delete: extern "C" fn(*mut ffi::c_void),
    ) -> Trace<'ctx> {
        Trace {
            inner: Some(TraceImpl {
                handle,
                dump,
                open_next,
                flush,
                close_and_delete,
            }),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn new_trace_useless<'ctx>() -> Trace<'ctx> {
        Trace {
            inner: None,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Waveform {
    Vcd,
    Fst,
}

/// A waveform trace.
///
/// From Verilator's website:
/// > The thread used to perform certain global operations, such as saving and
/// > tracing, must be done by a “main thread”.
pub struct Trace<'ctx> {
    inner: Option<__private::TraceImpl>,
    _marker: PhantomData<&'ctx ()>,
}

impl Trace<'_> {
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

    /// Documentation taken from the Verilator header file for VCD traces:
    ///
    /// > Continue a VCD dump by rotating to a new file name
    /// > The header is only in the first file created, this allows
    /// > "cat" to be used to combine the header plus any number of data files.
    pub fn open_next(&mut self, increment_filename: bool) {
        if let Some(inner) = &self.inner {
            (inner.open_next)(inner.handle, increment_filename);
        }
    }

    /// Documentation taken from the Verilator header file:
    ///
    /// > Flush dump
    pub fn flush(&mut self) {
        if let Some(inner) = &self.inner {
            (inner.flush)(inner.handle);
        }
    }

    /// The file is automatically closed when dropped, but it may be useful to
    /// call this manually.
    pub fn close(self) {
        drop(self.inner);
    }
}

pub trait OpenTrace<'ctx> {
    fn open_trace(&mut self, path: impl AsRef<Path>) -> Trace<'ctx>;
}
