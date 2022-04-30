use crate::utils::get_files;
use csv::{ReaderBuilder, Writer};
use gray_matter::engine::YAML;
use gray_matter::Matter;
use std::fs;
use std::path::PathBuf;

pub(crate) fn generate_csv() {
    // let mut rdr = ReaderBuilder::new()
    //     .delimiter(b';')
    //     .from_reader(data.as_bytes());
}

pub(crate) fn write_as_csv(dir: &str, fields: Vec<String>, path: PathBuf) {
    let matter = Matter::<YAML>::new();
    let mut wtr = Writer::from_writer(vec![]);
    wtr.write_record(&fields);
    let files = get_files(dir);
    for file in files {
        let contents = fs::read_to_string(file).unwrap();
        let result = matter.parse(contents.as_str());
        if let Some(frontmatter) = result.data {
            let fm_map = frontmatter.as_hashmap().unwrap();
            for field in &fields {
                let value = if let Some(v) = fm_map.get(field.as_str()) {
                    v.as_string().unwrap()
                } else {
                    String::from("")
                };
                wtr.write_field(value);
            }
        }
    }

    let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    fs::write(path, data).unwrap();
}
