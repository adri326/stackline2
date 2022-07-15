use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::fmt::Write;
use syn::{Item, ItemImpl, Type};

// This script reads the contents of any rust file in the `tiles/` directory,
// and gathers any type that implements `Tile`. These types are then put into
// the `AnyTile` enum and written to `$OUT_DIR/anytile.rs`.
//
// Any file with a type implementing `Tile` in tiles/ will be imported privately and its type will be re-exported.
//
// Known limitations:
// - only impls in the format "impl Tile for X" are accepted (X must not contain any "::")

// TODO: generate a kind of Reflection API for AnyTile
// - reading and writing can now be done through serde

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("anytile.rs");

    let mut names: Vec<String> = Vec::new();
    let mut files: Vec<(PathBuf, Vec<String>)> = Vec::new();

    // Read and parse the contents of every .rs file in tiles/

    println!("cargo:rerun-if-changed=tiles/");

    for src_path in list_files("tiles/") {
        if let Some("rs") = src_path.extension().and_then(|x| x.to_str()) {
            let contents = fs::read_to_string(src_path.clone())
                .unwrap_or_else(|err| panic!("Couldn't read {:?}: {}", src_path, err));
            let mut local_names: Vec<String> = Vec::new();

            // TODO: don't throw an error when a parsing error occured;
            // Instead, include the file so that rustc can give a helpful error
            let syntax = syn::parse_file(&contents)
                .unwrap_or_else(|err| panic!("Unable to parse file {:?}: {}", src_path, err));

            for item in syntax.items.iter() {
                #[allow(clippy::single_match)] // I'd like to keep the match for future-proofing, for instance if I need to match for a macro call
                match item {
                    Item::Impl(item) => {
                        if let Some(name) = parse_impl_tile(item) {
                            local_names.push(name);
                        }
                    }
                    _ => {}
                }
            }

            if !local_names.is_empty() {
                let canonical = fs::canonicalize(src_path.clone())
                    .unwrap_or_else(|err| panic!("Couldn't canonicalize {}: {}", src_path.display(), err));
                for name in local_names.iter() {
                    names.push(name.clone());
                }
                files.push((canonical, local_names));
            }
        }
    }

    // == Generate code ==

    let res = generate_code(files, names);

    fs::write(dest_path.clone(), &res)
        .unwrap_or_else(|err| panic!("Couldn't write to {:?}: {}", dest_path, err));
}

/// Helper function to recognize `impl Tile for XYZ`
fn parse_impl_tile(item: &ItemImpl) -> Option<String> {
    let (_, trait_, _) = item.trait_.as_ref()?;
    let ident = trait_.get_ident()?;

    if *ident == "Tile" {
        if let Type::Path(path) = &*item.self_ty {
            let name = path.path.get_ident().map(|i| i.to_string())?;
            return Some(name);
        }
    }

    None
}

// TODO: recursively list files in `path` (right now only the top-level files are listed).
// The method can return a `Vec` instead if necessary.
fn list_files(path: impl AsRef<Path>) -> impl Iterator<Item = PathBuf> {
    let iter = fs::read_dir(path.as_ref())
        .unwrap_or_else(|err| panic!("Error while reading {}: {}", path.as_ref().display(), err));

    iter.filter_map(|entry| match entry {
        Ok(entry) => Some(entry.path()),
        Err(_) => None,
    })
}

fn generate_code(files: Vec<(PathBuf, Vec<String>)>, names: Vec<String>) -> String {
    let mut res = String::new();

    // TODO: use a HashMap to prevent duplicate module names
    for (file, names) in files {
        let module_name = file
            .as_path()
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_else(|| panic!(
                "Couldn't extract valid UTF-8 filename from path {:?}",
                file
            ));
        let path = file.as_path().to_str().expect("Invalid UTF-8 path");

        writeln!(res, "#[path = \"{}\"]\nmod {};", path, module_name).unwrap();
        write!(res, "pub use {}::{{", module_name).unwrap();
        for name in names {
            write!(res, "{}, ", name).unwrap();
        }
        res += "};\n\n";
    }

    res += &fs::read_to_string("src/tile/anytile.doc.rs")
        .expect("Couldn't read src/tile/anytile.doc.rs");
    res += "#[derive(Clone, Debug, Serialize, Deserialize)]\n";
    res += "#[enum_dispatch]\n";
    res += "pub enum AnyTile {\n";

    for name in names.iter() {
        writeln!(res, "    {0}({0}),", name).unwrap();
    }
    res += "}\n";

    res += "\n";

    res += "impl AnyTile {\n";
    res += "    pub fn new(name: &str) -> Option<Self> {\n";
    res += "        match name {\n";

    for name in names.iter() {
        writeln!(res, "            \"{0}\" => Some(Self::{0}(<{0} as Default>::default())),", name).unwrap();
    }

    res += "            _ => None\n";
    res += "        }\n    }\n}\n";

    for name in names {
        // impl<T: Tile> TryInto<&T> for &AnyTile
        writeln!(res,
            concat!(
                "impl<'a> TryInto<&'a {0}> for &'a AnyTile {{\n",
                "    type Error = ();\n",
                "    fn try_into(self) -> Result<&'a {0}, Self::Error> {{\n",
                "        match self {{\n",
                "            AnyTile::{0}(tile) => Ok(tile),\n",
                "            _ => Err(()),\n",
                "        }}\n",
                "    }}\n",
                "}}",
            ),
            name
        ).unwrap();

        // impl<T: Tile> TryInto<&mut T> for &mut AnyTile
        writeln!(res,
            concat!(
                "impl<'a> TryInto<&'a mut {0}> for &'a mut AnyTile {{\n",
                "    type Error = ();\n",
                "    fn try_into(self) -> Result<&'a mut {0}, Self::Error> {{\n",
                "        match self {{\n",
                "            AnyTile::{0}(tile) => Ok(tile),\n",
                "            _ => Err(()),\n",
                "        }}\n",
                "    }}\n",
                "}}",
            ),
            name
        ).unwrap();
    }

    res
}
