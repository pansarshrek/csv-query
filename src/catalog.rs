use std::collections::HashMap;

#[derive(Debug)]
pub struct Person {
    pub name: String,
    pub country: String,
    pub sex: String,
    // age: i32,
    pub job: String,
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum Column {
    Name,
    Country,
    Sex,
    Job,
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct Value {
    pub column: Column,
    pub value: String,
}

impl Person {
    pub fn new(name: &str, country: &str, sex: &str, age: i32, job: &str) -> Person {
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
pub struct Catalog {
    persons: Vec<Person>,
    index: HashMap<Value, Vec<usize>>,
}

impl Catalog {
    pub fn new() -> Catalog {
        Catalog {
            persons: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn index_value(&mut self, v: Value) {
        let v = self.index.entry(v).or_insert(Vec::new());
        v.push(self.persons.len());
    }

    pub fn insert(&mut self, p: Person) {
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

    pub fn get_persons_by_row_id(&self, row_ids: &Vec<usize>) -> Vec<&Person> {
        let persons = row_ids.iter().filter_map(|id| self.persons.get(*id));
        return persons.collect();
    }



    pub fn get_possible_rows(&self, selection: &Vec<Value>) -> Vec<&Person> {
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