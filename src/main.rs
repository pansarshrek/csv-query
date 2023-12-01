use std::collections::HashMap;

#[derive(Debug)]
struct Person {
    name: String,
    country: String,
    sex: String,
    // age: i32,
    job: String,
}

#[derive(Eq, PartialEq, Hash, Debug)]
enum Column {
    Name,
    Country,
    Sex,
    Job,
}

#[derive(Eq, Hash, PartialEq, Debug)]
struct Value {
    column: Column,
    value: String,
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
    index: HashMap<Value, Vec<usize>>,
}

impl Catalog {
    fn new() -> Catalog {
        Catalog {
            persons: Vec::new(),
            index: HashMap::new(),
        }
    }

    fn index_value(&mut self, v: Value) {
        let v = self.index.entry(v).or_insert(Vec::new());
        v.push(self.persons.len());
    }

    fn insert(&mut self, p: Person) {
        self.index_value(Value {
            value: p.name.clone(),
            column: Column::Name,
        });
        self.index_value(Value {
            value: p.country.clone(),
            column: Column::Country,
        });
        self.index_value(Value {
            value: p.sex.clone(),
            column: Column::Sex,
        });
        self.index_value(Value {
            value: p.job.clone(),
            column: Column::Job,
        });
        self.persons.push(p);
    }

    fn get_persons_by_row_id(&self, row_ids: &Vec<usize>) -> Vec<&Person> {
        let persons = row_ids.iter().filter_map(|id| self.persons.get(*id));
        return persons.collect();
    }



    fn get_possible_rows(&self, selection: &Vec<Value>) -> Vec<&Person> {
        fn intersect(acc: Vec<usize>, r: Vec<usize>) -> Vec<usize> {
            acc.iter().filter(|e| r.contains(e)).cloned().collect()
        }

        let empty = Vec::new();
        let iter = selection.iter();

        let recs = iter.map(|x| self.index.get(x).unwrap_or(&empty).clone());

        let reduced = recs.reduce(intersect);

        match reduced {
            None => return self.persons.iter().collect(),
            Some(r) => return self.get_persons_by_row_id(&r),
        }
    }
}

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
