// Copyright (C) 2026 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::path::Path;

pub trait TraceWrite<'ctx> {
    fn dump(&mut self, timestamp: u64);
    fn flush(&mut self);
    fn close(self);
}

pub trait OpenTrace<'ctx> {
    type Trace: TraceWrite<'ctx>;

    fn open_trace(&mut self, path: impl AsRef<Path>) -> Self::Trace;
}
