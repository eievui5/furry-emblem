use stata::{convert_dir_as_mod, Resource};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use fe_data::*;

fn main() {
    // This is a macro rather than a function because the return type changes
    // depending on which invocation recieves it.
    macro_rules! parse {
        () => {
            // By calling `.unwrap_or(None)`, we ignore parse errors.
            // This is useful for mixing multiple types of files in the same directory.
            // If you would prefer to panic instead, call `.unwrap()` and wrap the result in `Some()`
            |s| toml::from_str(s).unwrap_or(None)
        };
    }

    // Build scripts are supposed to use the `OUT_DIR` environment variables to determine where their
    // resources should be placed.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut o = File::create(out_dir.join("res.rs")).unwrap();

    write!(o, "use core::num::*;").unwrap();
    write!(o, "{}", Stats::static_struct().to_string()).unwrap();
    write!(o, "{}", ItemType::static_struct().to_string()).unwrap();
    write!(o, "{}", WeaponItem::static_struct().to_string()).unwrap();

    convert_dir_as_mod::<Class>(&mut o, "../example-game/classes/", parse!()).unwrap();
    convert_dir_as_mod::<Item>(&mut o, "../example-game/items/", parse!()).unwrap();
}
