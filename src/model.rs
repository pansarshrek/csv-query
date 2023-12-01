use std::collections::HashMap;

pub type Column = String;
pub type Columns = Vec<Column>;
pub type Value = String;
pub type Record = Vec<Value>;

#[derive(Eq, Hash, PartialEq)]
pub struct Selection {
    pub column: Column,
    pub value: Value,
}

pub struct Table {
    pub name: String,
    pub columns: Columns,
    pub records: Vec<Record>,

    pub index: HashMap<Selection, Vec<usize>>,
}

impl Table {
    pub fn new(name: &str, columns: Columns) -> Table {
        Table {
            name: String::from(name),
            columns: columns,
            index: HashMap::new(),
            records: Vec::new(),
        }
    }

    pub fn index_value(&mut self, v: Selection) {
        let v = self.index.entry(v).or_insert(Vec::new());
        v.push(self.records.len());
    }

    pub fn insert(&mut self, record: Record) {
        for (i, field) in record.iter().enumerate() {
            let column_name = self.columns.get(i).expect("column should exist");
            self.index_value(Selection { column: String::from(column_name), value: String::from(field) })
        }

        self.records.push(record);
    }

    pub fn get_rows_by_id(&self, row_ids: &Vec<usize>) -> Vec<&Record> {
        let persons = row_ids.iter().filter_map(|id| self.records.get(*id));
        return persons.collect();
    }



    pub fn get_possible_rows(&self, selection: &Vec<Selection>) -> Vec<&Record> {
        fn intersect(acc: Vec<usize>, r: Vec<usize>) -> Vec<usize> {
            acc.iter().filter(|e| r.contains(e)).cloned().collect()
        }

        let empty = Vec::new();
        let iter = selection.iter();

        let recs = iter.map(|x| self.index.get(x).unwrap_or(&empty).clone());

        let reduced = recs.reduce(intersect);

        match reduced {
            None => return self.records.iter().collect(),
            Some(r) => return self.get_rows_by_id(&r),
        }
    }
}

pub struct Model {
    pub tables: Vec<Table>,
}


