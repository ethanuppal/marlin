// Copyright (C) 2025 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    fs,
    io::Write,
    mem::ManuallyDrop,
    os::fd::FromRawFd,
    sync::{LazyLock, Mutex},
};

use snafu::{ResultExt, Whatever};

// TODO: make cross-platform
static STDERR: LazyLock<Mutex<ManuallyDrop<fs::File>>> = LazyLock::new(|| {
    Mutex::new(ManuallyDrop::new(unsafe { fs::File::from_raw_fd(2) }))
});

#[doc(hidden)]
pub fn eprintln_nocapture_impl(contents: &str) -> Result<(), Whatever> {
    let mut stderr = STDERR.lock().expect("poisoned");
    stderr
        .write_all(contents.as_bytes())
        .whatever_context("Failed to write to non-captured stderr")?;
    stderr
        .write_all(b"\n")
        .whatever_context("Failed to write to non-captured stderr")?;
    Ok(())
}

#[doc(hidden)]
#[macro_export]
macro_rules! eprintln_nocapture {
    ($($contents:tt)*) => {{
        $crate::nocapture::eprintln_nocapture_impl(&format!($($contents)*))
    }};
}
