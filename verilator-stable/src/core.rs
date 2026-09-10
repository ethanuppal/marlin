// Copyright (C) 2026 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

use crate::types;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortDirection::Input => "input",
            PortDirection::Output => "output",
            PortDirection::Inout => "inout",
        }
        .fmt(f)
    }
}

/// Computes the width upper bound for a wide port with the given the given
/// `word_count` of the [`types::WData`] array Verilator generates.
///
/// See also: [`compute_edata_word_count_from_width_not_msb`]
pub const fn compute_approx_width_from_edata_word_count(
    word_count: usize,
) -> usize {
    word_count * (types::EData::BITS as usize)
}
