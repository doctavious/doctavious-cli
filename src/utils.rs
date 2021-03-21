use crate::templates::TemplateExtension;
use crate::templates::TEMPLATE_EXTENSIONS;
use std::collections::HashMap;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::file_structure::FileStructure;
use unidecode::unidecode;
use walkdir::WalkDir;

pub(crate) fn parse_enum<A: Copy>(
    env: &'static HashMap<&'static str, A>,
    src: &str,
) -> Result<A, String> {
    match env.get(src) {
        Some(p) => Ok(*p),
        None => {
            let supported: Vec<&&str> = env.keys().collect();
            Err(format!(
                "Unsupported value: \"{}\". Supported values: {:?}",
                src, supported
            ))
        }
    }
}

pub(crate) fn ensure_path(
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        println!("File already exists at: {}", path.to_string_lossy());
        print!("Overwrite? (y/N): ");
        io::stdout().flush()?;
        let mut decision = String::new();
        io::stdin().read_line(&mut decision)?;
        if decision.trim().eq_ignore_ascii_case("Y") {
            return Ok(());
        } else {
            return Err(format!(
                "Unable to write config file to: {}",
                path.to_string_lossy()
            )
            .into());
        }
    } else {
        let parent_dir = path.parent();
        match parent_dir {
            Some(path) => {
                fs::create_dir_all(path)?;
                Ok(())
            }
            None => Err(format!(
                "Unable to write file to: {}",
                path.to_string_lossy()
            )
            .into()),
        }
    }
}

pub(crate) fn format_number(number: i32) -> String {
    return format!("{:0>4}", number);
}

// TODO: is there a more concise way to do this?
pub(crate) fn build_path(
    dir: &str,
    title: &str,
    reserved_number: &str,
    extension: TemplateExtension,
    file_structure: FileStructure,
) -> PathBuf {
    return match file_structure {
        FileStructure::Flat => {
            let slug = slugify(&title);
            let file_name = format!("{}-{}", reserved_number, slug);
            Path::new(dir)
                .join(file_name)
                .with_extension(extension.to_string())
        }

        FileStructure::Nested => Path::new(dir)
            .join(&reserved_number)
            .join("README.")
            .with_extension(extension.to_string()),
    };
}

pub(crate) fn reserve_number(
    dir: &str,
    number: Option<i32>,
    file_structure: FileStructure,
) -> Result<i32, Box<dyn std::error::Error>> {
    if let Some(i) = number {
        if is_number_reserved(dir, i, file_structure) {
            // TODO: the prompt to overwrite be here?
            // TODO: return custom error NumberAlreadyReservedErr(number has already been reserved);
            eprintln!("{} has already been reserved", i);
            return Err(format!("{} has already been reserved", i).into());
        }
        return Ok(i);
    } else {
        return Ok(get_next_number(dir, file_structure));
    }
}

pub(crate) fn is_number_reserved(
    dir: &str,
    number: i32,
    file_structure: FileStructure,
) -> bool {
    return get_allocated_numbers(dir, file_structure).contains(&number);

    // TODO: revisit iterator
    // return get_allocated_numbers(dir)
    //     .find(|n| n == &number)
    //     .is_some();
}

pub(crate) fn get_allocated_numbers(
    dir: &str,
    file_structure: FileStructure,
) -> Vec<i32> {
    match file_structure {
        FileStructure::Flat => get_allocated_numbers_via_flat_files(dir),
        FileStructure::Nested => get_allocated_numbers_via_nested(dir),
    }
}

// TODO: do we want a ReservedNumber type?
// TODO: would be nice to do this via an Iterator but having trouble with empty
// expected struct `std::iter::Map`, found struct `std::iter::Empty`
// using vec for now
pub(crate) fn get_allocated_numbers_via_nested(dir: &str) -> Vec<i32> {
    match fs::read_dir(dir) {
        Ok(files) => {
            return files
                .filter_map(Result::ok)
                .filter_map(|e| {
                    // TODO: is there a better way to do this?
                    if e.file_type().is_ok() && e.file_type().unwrap().is_dir()
                    {
                        return Some(
                            e.file_name()
                                .to_string_lossy()
                                .parse::<i32>()
                                .unwrap(),
                        );
                    } else {
                        None
                    }
                })
                .collect();
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // return std::iter::empty();
            return Vec::new();
        }
        Err(e) => {
            panic!("Error reading directory {}. Error: {}", dir, e);
        }
    }
}

// TODO: would be nice to do this via an Iterator but having trouble with empty
// expected struct `std::iter::Map`, found struct `std::iter::Empty`
// using vec for now
pub(crate) fn get_allocated_numbers_via_flat_files(dir: &str) -> Vec<i32> {
    //impl Iterator<Item = i32> {

    let mut allocated_numbers = Vec::new();
    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_valid_file(&e.path()))
    {
        // The only way I can get this to pass the borrow checker is first mapping
        // to file_name and then doing the rest.
        // I'm probably doing this wrong and should review later
        let file_name = entry.file_name();
        let ss = file_name.to_str().unwrap();
        let first_space_index = ss.find("-").expect("didnt find a hyphen");
        let num: String = ss.chars().take(first_space_index).collect();
        allocated_numbers.push(num.parse::<i32>().unwrap());
    }

    return allocated_numbers;
}

///
pub(crate) fn is_valid_file(path: &Path) -> bool {
    return TEMPLATE_EXTENSIONS
        .contains_key(&path.extension().unwrap().to_str().unwrap());
}

pub(crate) fn get_next_number(
    dir: &str,
    file_structure: FileStructure,
) -> i32 {
    return if let Some(max) = get_allocated_numbers(dir, file_structure).iter().max()
    {
        max + 1
    } else {
        1
    }

    // TODO: revisit iterator
    // return get_allocated_numbers(dir)
    //     .max()
    //     .unwrap() + 1;
}

pub(crate) fn slugify(string: &str) -> String {
    let separator_char = '-';
    let separator = separator_char.to_string();

    let string: String = unidecode(string.into())
        .to_lowercase()
        .trim_matches(separator_char)
        .replace(' ', &separator);

    let mut slug = Vec::with_capacity(string.len());
    let mut is_sep = true;

    for x in string.chars() {
        match x {
            'a'..='z' | '0'..='9' => {
                is_sep = false;
                slug.push(x as u8);
            }
            _ => {
                if !is_sep {
                    is_sep = true;
                    slug.push(separator_char as u8);
                }
            }
        }
    }

    if slug.last() == Some(&(separator_char as u8)) {
        slug.pop();
    }

    let s = String::from_utf8(slug).unwrap();
    s.trim_end_matches(separator_char).to_string();
    s
}
