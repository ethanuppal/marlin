// Taken from
// https://gitlab.com/spade-lang/swim/-/blob/208d9da45d9feb3243d83b74b2d446c616e23b86/src/spade.rs
// SPDX-License-Identifier: EUPL-1.2

use camino::{Utf8Path, Utf8PathBuf};
use snafu::{whatever, ResultExt, Whatever};

#[derive(Debug)]
pub struct Namespace {
    pub namespace: String,
    pub base_namespace: String,
}

impl Namespace {
    pub fn new_lib(name: &str) -> Self {
        Self {
            namespace: name.into(),
            base_namespace: name.into(),
        }
    }
}

#[derive(Debug)]
pub struct SpadeFile {
    pub namespace: Namespace,
    pub path: Utf8PathBuf,
}

//impl SpadeFile {
//    fn as_compiler_arg(&self) -> String {
//        let Self {
//            namespace:
//                Namespace {
//                    namespace,
//                    base_namespace,
//                },
//            path,
//        } = self;
//        format!("{base_namespace},{namespace},{}", make_relative(path))
//    }
//}

pub fn spade_files_in_dir(
    namespace: Namespace,
    dir: impl AsRef<Utf8Path>,
) -> Result<Vec<SpadeFile>, Whatever> {
    let dir = dir.as_ref();
    let paths = dir
        .read_dir_utf8()
        .with_whatever_context(|_| format!("Failed to read files in {}", dir))?
        .map(|entry| {
            entry.whatever_context::<_, Whatever>("Failed to read dir entry")
        })
        .collect::<Result<Vec<_>, _>>()
        .with_whatever_context(|_| format!("While reading files in {}", dir))?;

    let mut has_main = false;

    let mut result = vec![];
    for path_utf8 in paths.iter() {
        let path = path_utf8.path();
        let file_stem = if let Some(stem) = path.file_stem() {
            stem
        } else {
            continue;
        };

        let new_namespace = if file_stem == "main" {
            has_main = true;
            namespace.namespace.clone()
        } else {
            format!("{}::{}", namespace.namespace, file_stem)
        };
        let new_namespace = Namespace {
            namespace: new_namespace,
            base_namespace: namespace.base_namespace.clone(),
        };

        if path.is_dir() {
            if file_stem == "main" {
                whatever!(
                    "{} is a reserved folder name.\nFound it in {path}",
                    file_stem,
                );
            }
            let mut files = spade_files_in_dir(new_namespace, path)?;
            result.append(&mut files);
        } else if path.extension() == Some("spade") {
            result.push(SpadeFile {
                namespace: new_namespace,
                path: path.into(),
            })
        }
    }

    if !has_main {
        whatever!(
            "{dir} is missing a main.spade\nHelp: Create a main.spade file in there as the entry point of the module",
        );
    }

    Ok(result)
}
