// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::cell::Cell;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::MutexGuard;
#[doc(inline)]
pub use marlin_verilator as verilator;

#[doc(inline)]
#[cfg_attr(docsrs, doc(cfg(feature = "verilog")))]
#[cfg(feature = "verilog")]
pub use marlin_verilog as verilog;

#[doc(inline)]
#[cfg_attr(docsrs, doc(cfg(feature = "spade")))]
#[cfg(feature = "spade")]
pub use marlin_spade as spade;

#[doc(inline)]
#[cfg_attr(docsrs, doc(cfg(feature = "veryl")))]
#[cfg(feature = "veryl")]
pub use marlin_veryl as veryl;

/// A reference to the verilator model
#[derive(Clone)]
pub struct ModelRef<'ctx> {
    /// # Safety
    ///
    /// The Rust binding to the model will not outlive the dynamic library context (with lifetime `'ctx`) and is dropped when this struct is.
    #[doc(hidden)]
    pub ptr: *mut c_void,
    #[doc(hidden)]
    _marker: PhantomData<&'ctx ()>,
    #[doc(hidden)]
    _unsend_unsync: PhantomData<(Cell<()>, MutexGuard<'static, ()>)>
}

impl<'ctx> ModelRef<'ctx> {
    pub unsafe fn new(ptr: *mut c_void) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
            _unsend_unsync: PhantomData,
        }
    }
}
