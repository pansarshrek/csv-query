mod catalog;

use catalog::{Person, Catalog, Column, Value};

fn main() {
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
    // println!(
    //     "Get by nothing: {:#?}",
    //     catalog.get_possible_rows(&vec![])
    // );
}
