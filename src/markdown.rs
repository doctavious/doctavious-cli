


// md_string = " | "
// for header in headers:
// md_string += header+" |"
//
// md_string += "\n |"
// for i in range(len(headers)):
// md_string += "--- | "
//
// md_string += "\n"
// for row in list_of_rows:
// md_string += " | "
// for header in headers:
// md_string += row[header]+" | "
// md_string += "\n"

use std::io;
use std::path::PathBuf;
use csv::ReaderBuilder;

// https://docs.rs/csv/1.1.6/csv/cookbook/index.html

// TODO: CSV to markdown table
// TODO: support asciidoc
fn csv_to_table(path: PathBuf) -> String {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path).unwrap();

    let mut md_string = String::from(" | ");
    for header in rdr.headers().unwrap() {
        md_string.push_str(header);
        md_string.push_str(" |")
    }

    md_string.push_str("\n |");
    for i in 1..rdr.headers().unwrap().len() {
        md_string.push_str("--- | ");
    }

    md_string.push_str("\n");

    for result in rdr.records() {
        let row = result.unwrap();
        for record in row.iter() {
            md_string.push_str(format!(" | {} | ", record).as_str());
        }
        md_string.push_str("\n")
    }

    md_string
}



