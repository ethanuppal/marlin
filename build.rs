// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::fs;

use ignore::Walk;
use snafu::{whatever, ResultExt, Whatever};

struct LicenseSpec {
    root: &'static str,
    license_name: &'static str,
    header: &'static str,
}

const MPL_LICENSE_HEADER: &str = "// Copyright (C) 2024 Ethan Uppal.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.";

const GPL_LICENSE_HEADER: &str = "// Copyright (C) 2024 Ethan Uppal.
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// this program.  If not, see <https://www.gnu.org/licenses/>.";

const LICENSE_SPECS: [LicenseSpec; 3] = [
    LicenseSpec {
        root: "src",
        license_name: "MPL",
        header: MPL_LICENSE_HEADER,
    },
    LicenseSpec {
        root: "language-support",
        license_name: "MPL",
        header: MPL_LICENSE_HEADER,
    },
    LicenseSpec {
        root: "examples",
        license_name: "GPL",
        header: GPL_LICENSE_HEADER,
    },
];

#[snafu::report]
fn main() -> Result<(), Whatever> {
    println!("cargo:rerun-if-changed=..");

    for license_spec in LICENSE_SPECS {
        for entry in Walk::new(license_spec.root) {
            let entry = entry.whatever_context(format!(
                "Failed to read contents of {}",
                license_spec.root
            ))?;

            if entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension == "rs")
                .unwrap_or(false)
            {
                let file_contents = fs::read_to_string(entry.path())
                    .whatever_context(format!(
                        "Failed to read Rust file {}",
                        entry.path().to_string_lossy()
                    ))?;
                if !file_contents.trim_start().starts_with(license_spec.header)
                {
                    whatever!(
                        "{} is missing {} license header",
                        entry.path().to_string_lossy(),
                        license_spec.license_name
                    );
                }
            }
        }
    }

    Ok(())
}
