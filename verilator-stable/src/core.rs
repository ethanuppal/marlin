// Copyright (C) 2026 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{cmp, fmt};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VerilatorVersion {
    pub major: usize,
    pub minor: usize,
}

impl fmt::Display for VerilatorVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:03}", self.major, self.minor)
    }
}

impl cmp::PartialOrd for VerilatorVersion {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for VerilatorVersion {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
    }
}
