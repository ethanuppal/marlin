// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

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

//#[doc(inline)]
//#[cfg_attr(docsrs, doc(cfg(feature = "very")))]
//#[cfg(feature = "veryl")]
//pub use marlin_veryl as veryl;
