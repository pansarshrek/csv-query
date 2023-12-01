mod model;

mod catalog;

use catalog::{Person, Catalog, Column, Value};

use core::num;
use std::error::Error;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Read;

fn example1() {
    let mut catalog = Catalog::new();
    catalog.insert(Person::new("Ni", "SWE", "M", 34, "Engineer"));
    catalog.insert(Person::new("Si", "CN", "F", 34, "Engineer"));
    catalog.insert(Person::new("Ai", "SWE", "M", 1, "Baby"));
    catalog.insert(Person::new("Gu", "SWE", "F", 2, "Baby"));

    println!("Hello catalog: {:#?}", catalog);

    let select = vec![
        Value {
            column: Column::Country,
            value: String::from("SWE"),
        },
        Value {
            column: Column::Sex,
            value: String::from("M"),
        },
    ];
    println!(
        "Get by {:#?}: {:#?}",
        &select,
        catalog.get_possible_rows(&select)
    );
}

#[derive(Debug)]
struct ReadColumnError {
    message: String
}

fn read_csv_line(reader: &mut io::BufReader<fs::File>, delim: &str) -> Result<Vec<String>, ReadColumnError> {
    let mut buf = String::new();
    let some = reader.read_line(&mut buf).expect("read columns should work");
    if some == 0 {
        Result::Err(ReadColumnError { message: String::from("file empty") })
    } else {
        let cols: Vec<String> = parse_csv_line(&buf, delim, None);
        Result::Ok(cols)
    }
}

fn parse_csv_line(line: &str, delim: &str, max_cols: Option<usize>) -> Vec<String> {
    let x = line.split(delim).map(|s| s.trim_end());
    match max_cols {
        None => x.map(String::from).collect(),
        Some(max) => x.take(max).map(String::from).collect(),
    }
}

fn main() {
    let f = fs::File::open("data.csv").expect("data.csv should exist and be readable");
    let mut r = io::BufReader::new(f);
    let delim = ",";
    
    let columns = read_csv_line(&mut r, delim).expect("to have columns");
    let num_columns = columns.len();
    let mut t = model::Table::new("d", columns);
    
    let mut buf = String::new();
    loop {
        match r.read_line(&mut buf) {
            Ok(0) => break,
            Err(e) => {
                println!("failed to read line {}", e);
                break;
            },
            _ => (),
        }
        let fields = parse_csv_line(&buf, delim , Some(num_columns));
        t.insert(fields);
        buf.clear();
    }

    let select = vec![
        model::Selection{ column: String::from("country"), value: String::from("swe") },
        model::Selection{ column: String::from("name"), value: String::from("nicklas")}];
    let records = t.get_possible_rows(&select);
    println!("Got records: {:#?}", records);
}
