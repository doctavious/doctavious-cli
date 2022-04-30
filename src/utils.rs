// use crate::templates::TemplateExtension;
// use crate::templates::TEMPLATE_EXTENSIONS;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::doctavious_error::{
    DoctaviousError, EnumError, Result as DoctavousResult,
};
use crate::file_structure::FileStructure;
use crate::markup_format::MarkupFormat;
use crate::output::{get_output, print_output, Output};
use crate::MARKUP_FORMAT_EXTENSIONS;
use serde::ser::SerializeSeq;
use serde::Serializer;
use std::fmt::{Debug, Display, Formatter};
use unidecode::unidecode;
use walkdir::WalkDir;

pub(crate) fn parse_enum<A: Copy>(
    env: &'static HashMap<&'static str, A>,
    src: &str,
) -> Result<A, EnumError> {
    match env.get(src) {
        Some(p) => Ok(*p),
        None => {
            let supported: Vec<&&str> = env.keys().collect();
            Err(EnumError {
                message: format!(
                    "Unsupported value: \"{}\". Supported values: {:?}",
                    src, supported
                ),
            })
        }
    }
}

pub(crate) fn ensure_path(path: &PathBuf) -> DoctavousResult<()> {
    if path.exists() {
        println!("File already exists at: {}", path.to_string_lossy());
        print!("Overwrite? (y/N): ");
        io::stdout().flush()?;
        let mut decision = String::new();
        io::stdin().read_line(&mut decision)?;
        return if decision.trim().eq_ignore_ascii_case("Y") {
            Ok(())
        } else {
            Err(DoctaviousError::NoConfirmation(
                format!(
                    "Unable to write config file to: {}",
                    path.to_string_lossy()
                )
                .into(),
            ))
        };
    } else {
        let parent_dir = path.parent();
        if parent_dir.is_some() {
            fs::create_dir_all(parent_dir.unwrap())?;
        }
        Ok(())
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
    extension: MarkupFormat,
    file_structure: FileStructure,
) -> PathBuf {
    return match file_structure {
        FileStructure::Flat => {
            let slug = slugify(&title);
            let file_name = format!("{}-{}", reserved_number, slug);
            Path::new(dir).join(file_name).with_extension(extension.to_string())
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
) -> DoctavousResult<i32> {
    return if let Some(i) = number {
        if is_number_reserved(dir, i, file_structure) {
            // TODO: the prompt to overwrite be here?
            // TODO: return custom error NumberAlreadyReservedErr(number has already been reserved);
            eprintln!("{} has already been reserved", i);
            return Err(DoctaviousError::ReservedNumberError(i));
        }
        Ok(i)
    } else {
        Ok(get_next_number(dir, file_structure))
    };
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
    return MARKUP_FORMAT_EXTENSIONS
        .contains_key(&path.extension().unwrap().to_str().unwrap());
}

pub(crate) fn get_next_number(dir: &str, file_structure: FileStructure) -> i32 {
    return if let Some(max) =
        get_allocated_numbers(dir, file_structure).iter().max()
    {
        max + 1
    } else {
        1
    };

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

struct List<A>(Vec<A>);

impl<A> Debug for List<A>
where
    A: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for value in self.0.iter() {
            writeln!(f, "{:?}", value)?;
        }

        Ok(())
    }
}

impl<A> Display for List<A>
where
    A: Display,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for value in self.0.iter() {
            writeln!(f, "{}", value)?;
        }

        Ok(())
    }
}

impl<A> serde::ser::Serialize for List<A>
where
    A: serde::ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for elem in self.0.iter() {
            seq.serialize_element(elem)?;
        }

        seq.end()
    }
}

pub(crate) fn list(dir: &str, opt_output: Option<Output>) {
    match fs::metadata(&dir) {
        Ok(_) => {
            let files = get_files(dir);
            print_output(get_output(opt_output), List(files)).unwrap();
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("the {} directory should exist", dir)
            }
            _ => eprintln!("Error occurred: {:?}", e),
        },
    }
}

pub(crate) fn get_files(dir: &str) -> Vec<String> {
    let mut f: Vec<_> = WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|f| is_valid_file(&f.path()))
        .map(|f| String::from(strip_current_dir(&f.path()).to_str().unwrap()))
        .collect();

    f.sort();
    return f;
}

/// Remove the `./` prefix from a path.
fn strip_current_dir(path: &Path) -> &Path {
    return path.strip_prefix(".").unwrap_or(path);
}
