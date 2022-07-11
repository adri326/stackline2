use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Item, Type, ItemImpl};

// This script reads the contents of any rust file in the `tiles/` directory,
// and gathers any type that implements `Tile`. These types are then put into
// the `AnyTile` enum and written to `$OUT_DIR/anytile.rs`.
//
// Any file with a type implementing `Tile` in tiles/ will be imported privately and its type will be re-exported.
//
// Known limitations:
// - only impls in the format "impl Tile for X" are accepted (X must not contain any "::")

// TODO: generate a kind of Reflection API for AnyTile

fn parse_impl_tile(item: &ItemImpl) -> Option<String> {
    let (_, trait_, _) = item.trait_.as_ref()?;
    let ident = trait_.get_ident()?;

    if ident.to_string() == "Tile" {
        if let Type::Path(path) = &*item.self_ty {
            let name = path.path.get_ident().map(|i| i.to_string())?;
            return Some(name);
        }
    }

    None
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("anytile.rs");

    let mut names: Vec<String> = Vec::new();
    let mut files: Vec<(PathBuf, Vec<String>)> = Vec::new();

    // Read and parse the contents of every .rs file in tiles/

    for entry in fs::read_dir("tiles/").expect("Error while reading tiles/") {
        let entry = entry.expect("Error while reading tiles/");
        let src_path = entry.path();
        if let Some("rs") = src_path.extension().and_then(|x| x.to_str()) {
            let contents = fs::read_to_string(src_path.clone())
                .expect(&format!("Couldn't read {:?}", src_path));
            let mut local_names: Vec<String> = Vec::new();

            let syntax = syn::parse_file(&contents)
                .expect(&format!("Unable to parse file {:?}", src_path));

            for item in syntax.items.iter() {
                match item {
                    Item::Impl(item) => {
                        if let Some(name) = parse_impl_tile(item) {
                            local_names.push(name);
                        }
                    }
                    _ => {}
                }
            }

            if local_names.len() > 0 {
                let canonical = fs::canonicalize(src_path.clone())
                    .expect(&format!("Couldn't canonicalize {:?}", src_path));
                for name in local_names.iter() {
                    names.push(name.clone());
                }
                files.push((canonical, local_names));
            }
        }
    }

    // Generate code

    let mut res = String::from("use enum_dispatch::enum_dispatch;\n\n");

    for file in files {
        let mod_name = file.0.as_path().file_stem().map(|x| x.to_str()).flatten().expect(&format!("Couldn't extract valid UTF-8 filename from path {:?}", file));
        let path = file.0.as_path().to_str().expect("Invalid UTF-8 path");

        res += &format!("#[path = \"{}\"]\nmod {};\n", path, mod_name);
        res += &format!("pub use {}::{{", mod_name);
        for name in file.1 {
            res += &format!("{}, ", name);
        }
        res += "};\n\n";
    }

    res += &fs::read_to_string("src/tile/anytile.doc.rs").expect("Couldn't read src/tile/anytile.doc.rs");
    res += "#[derive(Clone, Debug, Serialize, Deserialize)]\n";
    res += "#[enum_dispatch]\n";
    res += "pub enum AnyTile {\n";

    for name in names.iter() {
        res += &format!("    {0}({0}),\n", name);
    }
    res += "}\n";

    // impl<T: Tile> TryInto<&T> for &AnyTile
    res += "\n";

    for name in names {
        res += &format!(
            concat!(
                "impl<'a> TryInto<&'a {0}> for &'a AnyTile {{\n",
                "    type Error = ();\n",
                "    fn try_into(self) -> Result<&'a {0}, Self::Error> {{\n",
                "        match self {{\n",
                "            AnyTile::{0}(tile) => Ok(tile),\n",
                "            _ => Err(()),\n",
                "        }}\n",
                "    }}\n",
                "}}\n",
            ),
            name
        );

        res += &format!(
            concat!(
                "impl<'a> TryInto<&'a mut {0}> for &'a mut AnyTile {{\n",
                "    type Error = ();\n",
                "    fn try_into(self) -> Result<&'a mut {0}, Self::Error> {{\n",
                "        match self {{\n",
                "            AnyTile::{0}(tile) => Ok(tile),\n",
                "            _ => Err(()),\n",
                "        }}\n",
                "    }}\n",
                "}}\n",
            ),
            name
        );
    }

    fs::write(dest_path.clone(), &res).expect(&format!("Couldn't write to {:?}", dest_path));
}
