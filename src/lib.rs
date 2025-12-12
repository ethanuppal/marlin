// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::cell::Cell;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
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

pub struct OutputPort<'ctx, T> {
    model: ModelRef<'ctx>,
    /// # Safety
    ///
    /// The constructor's safety construct ensures that the getter matches the model
    raw_get: unsafe extern "C" fn(*mut c_void) -> T,
    /// Name of the pin for debug output
    name: &'static str,
}

impl<'ctx, T> OutputPort<'ctx, T> {
    /// # Safety
    ///
    /// The `raw_get` function must correspond to the passed model
    pub unsafe fn new(model: ModelRef<'ctx>, raw_get: unsafe extern "C" fn(*mut c_void) -> T, name: &'static str) -> Self {
        Self {
            model,
            raw_get,
            name,
        }
    }

    pub fn get(&self) -> T {
        unsafe {
            (self.raw_get)(self.model.ptr)
        }
    }
}

impl<T: Debug> Debug for OutputPort<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputPort")
            .field("name", &self.name)
            .field("value", &self.get())
            .finish()
    }
}

pub struct InputPort<'ctx, T> {
    model: ModelRef<'ctx>,
    /// # Safety
    ///
    /// The constructor's safety construct ensures that the getter matches the model
    raw_set: unsafe extern "C" fn(*mut c_void, T),
    /// Name of the pin for debug output
    name: &'static str,
}

impl<'ctx, T> InputPort<'ctx, T> {
    /// # Safety
    ///
    /// The `raw_get` function must correspond to the passed model
    pub unsafe fn new(model: ModelRef<'ctx>, raw_set: unsafe extern "C" fn(*mut c_void, T), name: &'static str) -> Self {
        Self {
            model,
            raw_set,
            name,
        }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            (self.raw_set)(self.model.ptr, value);
        }
    }
}

impl<T> Debug for InputPort<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputPort")
            .field("name", &self.name)
            .finish()
    }
}

pub struct Evaluator<'ctx> {
    model: ModelRef<'ctx>,
    /// # Safety
    ///
    /// The constructor's safety construct ensures that the eval matches the model
    eval_fn: unsafe extern "C" fn(*mut c_void),
}

impl<'ctx> Evaluator<'ctx> {
    /// # Safety
    ///
    /// The `raw_get` function must correspond to the passed model
    pub unsafe fn new(model: ModelRef<'ctx>, eval_fn: unsafe extern "C" fn(*mut c_void)) -> Self {
        Self {
            model,
            eval_fn,
        }
    }
}

impl<'ctx> Evaluator<'ctx> {
    pub fn eval(&mut self) {
        unsafe {
            (self.eval_fn)(self.model.ptr)
        }
    }
}