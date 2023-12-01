mod model;

use std::env;
use std::fs;
use std::io;
use std::io::BufRead;

#[derive(Debug)]
struct ReadColumnError {
    message: String,
}

fn read_csv_line(
    reader: &mut io::BufReader<fs::File>,
    delim: &str,
) -> Result<Vec<String>, ReadColumnError> {
    let mut buf = String::new();
    let some = reader
        .read_line(&mut buf)
        .expect("read columns should work");
    if some == 0 {
        Result::Err(ReadColumnError {
            message: String::from("file empty"),
        })
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

#[derive(Debug)]
struct Commands {
    infile: String,
    select: Vec<model::Selection>,
}

impl Commands {
    fn from_args() -> Commands {
        let mut commands = Commands::new();
        let args: Vec<String> = env::args().skip(1).collect();
        match commands.parse(&args) {
            Err(err) => panic!("failed to parse command line args: {}", err),
            _ => (),
        }
        commands
    }

    fn new() -> Commands {
        Commands {
            infile: String::new(),
            select: Vec::new(),
        }
    }

    fn parse(&mut self, args: &Vec<String>) -> Result<(), &str> {
        let mut it = args.iter();

        loop {
            let n = it.next();
            match n {
                None => break,
                Some(s) if s == "--in" => {
                    self.infile = match it.next() {
                        None => break,
                        Some(s) => String::from(s),
                    };
                }
                Some(s) if s == "--select" => {
                    let stmt = match it.next() {
                        None => break,
                        Some(s) => String::from(s),
                    };
                    if stmt.contains("=") {
                        let parts: Vec<&str> = stmt.split("=").collect();
                        let sel = model::Selection {
                            column: String::from(parts[0]),
                            value: parts[1].split(",").map(String::from).collect(),
                        };
                        self.select.push(sel);
                    }
                }
                Some(s) => {
                    println!("Unrecognized argument {s}");
                }
            }
        }

        if self.infile == "" {
            return Err("ERROR: no in file specified");
        }
        return Ok(());
    }
}



fn main() {
    let commands = Commands::from_args();
    println!("commands: {:#?}", &commands);
    let f = match fs::File::open(&commands.infile) {
        Ok(f) => f,
        Err(e) => {
            panic!("failed to open file {}: {}", &commands.infile, e);
        }
    };
    let mut r = io::BufReader::new(f);
    let delim = ",";

    let start = std::time::Instant::now();
    println!("Loading files...");

    let columns = read_csv_line(&mut r, delim).expect("to have columns");
    let num_columns = columns.len();
    let mut t = model::Table::new(&commands.infile, columns);

    let mut buf = String::new();
    loop {
        match r.read_line(&mut buf) {
            Ok(0) => break,
            Err(e) => {
                println!("failed to read line {}", e);
                break;
            }
            _ => (),
        }
        let fields = parse_csv_line(&buf, delim, Some(num_columns));
        let dts: Vec<model::DataType> = fields
            .iter()
            .map(model::DataType::from_string)
            .collect();
        t.insert(dts);
        buf.clear();
    }

    println!(
        "Files loaded. Time elapsed: {} ms",
        start.elapsed().as_millis()
    );

    let start_fetch = std::time::Instant::now();
    println!("Start timer...");

    let mut ctx = t.new_context();
    println!("count before select: {}", ctx.count());
    for s in &commands.select {
        ctx.select(s);
    }
    println!("count after select: {}", ctx.count());

    for s in &commands.select {
        ctx.deselect(s);
    }

    println!("sum age: {}", ctx.sum(String::from("age")).unwrap_or(model::DataType::Int(0)));

    println!("max age: {}", ctx.max(String::from("age")).unwrap_or(model::DataType::Int(0)));

    println!("min age: {}", ctx.min(String::from("age")).unwrap_or(model::DataType::Int(0)));

    println!("sum duration: {}", ctx.sum(String::from("duration")).unwrap_or(model::DataType::Int(0)));
    println!("max duration: {}", ctx.max(String::from("duration")).unwrap_or(model::DataType::Int(0)));
    println!("min duration: {}", ctx.min(String::from("duration")).unwrap_or(model::DataType::Int(0)));

    println!("Querying finished. Time elapsed: {} ms", start_fetch.elapsed().as_millis());

    // let records = t.get_possible(&commands.select);
    // let count = t.count(&commands.select);
    // println!("Got {} records. Time elapsed: {} ms", records.len(), start_fetch.elapsed().as_millis());
}
