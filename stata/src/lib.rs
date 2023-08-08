#![feature(path_file_prefix)]

pub use proc_macro2;
pub use quote::quote;
pub use stata_macros::Resource;

mod to_static;

pub use proc_macro2::TokenStream;
pub use to_static::ToStatic;

/// A type that can be converted to a static structure.
/// You should implement this using `#[derive(Resource)]`.
pub trait Resource: ToStatic {
    fn static_struct() -> TokenStream;
}

/// Converts a directory of data files into resources and writes them to `o`.
///
/// Note that if a file cannot be parsed, it will simply be ignored.
/// This means you can put multiple types of data in the same directory,
/// so long as they don't have the same data format.
///
/// The deserializer is user supplied, so you can use any format you'd like; even multiple in the same project.
///
/// # Panics
///
/// Panics if the directory or its contents could not be found, opened, or written to.
/// Note that failing to parse a file is *not an error*, and will be silently ignored.
/// If you want to panic upon a parse error, manually panic within your `parse` function.
#[cfg(feature = "serde")]
pub fn convert_dir<T: Resource + for<'de> serde::Deserialize<'de>>(
    mut o: impl std::io::Write,
    path: impl AsRef<std::path::Path>,
    parse: impl Fn(&str) -> Option<T>,
) -> std::io::Result<()> {
    use convert_case::{Case, Casing};

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            
            let resource = parse(&text).unwrap();
            let name = path
                .file_prefix()
                .unwrap()
                .to_string_lossy()
                .to_case(Case::UpperSnake);
            let ty = T::static_type().to_string();
            let value = &resource.static_value().to_string();
            write!(
                o,
                "#[allow(non_upper_case_globals, dead_code)] pub const {name}: {ty} = {value};",
            )?;
        }
    }

    Ok(())
}

/// Converts a directory, wrapping it in a module based on the directory's name.
///
/// Also generates a definition of the struct
/// – note that this may cause issues if you use this function for the same type in multiple directories.
///
/// For example, `res/characters` would become `mod characters {}`
///
/// See `[[convert_dir]]` and `[[Resource::generate_struct]]` for more information.
#[cfg(feature = "serde")]
pub fn convert_dir_as_mod<T: Resource + for<'de> serde::Deserialize<'de>>(
    mut o: impl std::io::Write,
    path: impl AsRef<std::path::Path>,
    parse: impl Fn(&str) -> Option<T>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    write!(
        o,
        "{} pub mod {} {{ use super::*;",
        T::static_struct(),
        path.file_stem().unwrap().to_string_lossy(),
    )?;
    convert_dir(&mut o, path, parse).unwrap();
    write!(o, "}}")?;

    Ok(())
}
