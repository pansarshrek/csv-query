use std::collections::HashMap;

#[derive(Debug)]
struct Person {
    name: String,
    country: String,
    sex: String,
    // age: i32,
    job: String,
}

impl Person {
    fn new(name: &str, country: &str, sex: &str, age: i32, job: &str) -> Person {
        Person {
            name: String::from(name),
            // age: age,
            country: String::from(country),
            job: String::from(job),
            sex: String::from(sex),
        }
    }
}

#[derive(Debug)]
struct Catalog {
    persons: Vec<Person>,
    index: HashMap<String, Vec<usize>>,
}

impl Catalog {
    fn new() -> Catalog {
        Catalog {
            persons: Vec::new(),
            index: HashMap::new()
        }
    }
    fn insert(&mut self, p: Person) {
        let v = self.index.entry(p.name.clone()).or_insert(Vec::new());
        v.push(self.persons.len());
        let v = self.index.entry(p.country.clone()).or_insert(Vec::new());
        v.push(self.persons.len());
        let v = self.index.entry(p.sex.clone()).or_insert(Vec::new());
        v.push(self.persons.len());
        let v = self.index.entry(p.job.clone()).or_insert(Vec::new());
        v.push(self.persons.len());

        self.persons.push(p);
    }
}

fn main() {
    let mut catalog = Catalog::new();
    catalog.insert(Person::new("Ni", "SWE", "M", 34, "Engineer"));
    catalog.insert(Person::new("Si", "CN", "F", 34, "Engineer"));
    catalog.insert(Person::new("Ai", "SWE", "M", 1, "Baby"));
    catalog.insert(Person::new("Gu", "SWE", "F", 2, "Baby"));

    println!("Hello catalog: {:#?}", catalog);
}
