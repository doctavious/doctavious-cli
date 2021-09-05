use std::path::{Path, PathBuf};
use std::{env, fs};

use walkdir::WalkDir;

// TODO: move into src/main?
const TEMPLATES_DIR: &str = "templates";
fn main() {
    let out_dir_path = env::var("OUT_DIR").unwrap();
    let target = Path::new(&out_dir_path).join("../../..");
    if fs::metadata(&target).is_err() {
        println!(" mkdir: {:?}", target);
        fs::create_dir_all(&target).unwrap();
    }

    for entry in WalkDir::new(&TEMPLATES_DIR).into_iter().filter_map(Result::ok)
    {
        let dest_path = target.join(entry.path());
        if entry.file_type().is_dir() {
            if fs::metadata(&dest_path).is_err() {
                println!(" mkdir: {:?}", dest_path);
                fs::create_dir_all(&dest_path).unwrap();
            }
        } else {
            println!("  copy: {:?} -> {:?}", &entry.path(), &dest_path);
            fs::copy(&entry.path(), &dest_path).unwrap();
        }
    }
}
