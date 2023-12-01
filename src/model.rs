use std::collections::BTreeSet;
use std::collections::HashMap;

pub type Column = String;
pub type Columns = Vec<Column>;
pub type Value = String;
pub type Record = Vec<DataType>;

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum DataType {
    String(String),
    // Float(f64),
    Int(i32),
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct Selection {
    pub column: Column,
    pub value: Vec<Value>,
}

pub struct DataContext<'a> {
    table: &'a Table,
    selection: Vec<Selection>,

    selected_records: Vec<&'a Record>,
}

impl DataContext<'_> {
    pub fn select(&mut self, selection: Selection) -> &DataContext {
        self.selection.push(selection);
        let possible_records = self.table.get_possible(&self.selection);
        self.selected_records = possible_records;
        self
    }

    pub fn count(&self) -> usize {
        let vals = &self.selected_records;
        vals.iter().count()
    }

    pub fn sum(&self, col: Column) -> Option<i32> {
        self.table.get_col_index(&col).map(|i| {
            self.selected_records
                .iter()
                .map(|x| &x[i])
                .filter(|x| match x {
                    DataType::Int(_) => true,
                    _ => false,
                })
                .map(|s| match s {
                    DataType::Int(num) => num,
                    _ => &0,
                })
                .sum()
        })
    }

    pub fn max(&self, col: Column) -> Option<i32> {
        self.table.get_col_index(&col).map(|i| {
            self.selected_records
                .iter()
                .map(|x| &x[i])
                .filter(|x| match x {
                    DataType::Int(_) => true,
                    _ => false,
                })
                .map(|s| match s {
                    DataType::Int(num) => num,
                    _ => &0,
                })
                .max()
                .unwrap_or(&0)
                .clone()
        })
    }

    pub fn min(&self, col: Column) -> Option<i32> {
        self.table.get_col_index(&col).map(|i| {
            self.selected_records
                .iter()
                .map(|x| &x[i])
                .filter(|x| match x {
                    DataType::Int(_) => true,
                    _ => false,
                })
                .map(|s| match s {
                    DataType::Int(num) => num,
                    _ => &0,
                })
                .min()
                .unwrap_or(&0)
                .clone()
        })
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
struct IndexValue {
    column: Column,
    value: Value,
}

pub struct Table {
    pub name: String,
    pub columns: Columns,
    pub records: Vec<Record>,

    index: HashMap<IndexValue, Vec<usize>>,
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

    pub fn new_context(&self) -> DataContext {
        DataContext {
            table: self,
            selection: vec![],
            selected_records: vec![],
        }
    }

    fn index_value(&mut self, v: IndexValue) {
        let v = self.index.entry(v).or_insert(Vec::new());
        v.push(self.records.len());
    }

    pub fn get_col_index(&self, col: &Column) -> Option<usize> {
        for (i, column) in self.columns.iter().enumerate() {
            if column == col {
                return Some(i);
            }
        }
        return None;
    }

    pub fn insert(&mut self, record: Record) {
        for (i, field) in record.iter().enumerate() {
            let column_name = self.columns.get(i).expect("column should exist");
            self.index_value(IndexValue {
                column: String::from(column_name),
                value: match field {
                    DataType::String(d) => String::from(d),
                    DataType::Int(i) => i.to_string(),
                },
            })
        }

        self.records.push(record);
    }

    pub fn get_rows_by_id(&self, row_ids: &Vec<usize>) -> Vec<&Record> {
        let persons = row_ids.iter().filter_map(|id| self.records.get(*id));
        return persons.collect();
    }

    pub fn get_possible(&self, selection: &Vec<Selection>) -> Vec<&Record> {
        if selection.len() == 0 {
            return self.records.iter().collect();
        }

        let mut bts = BTreeSet::new();
        let empty = Vec::new();
        for s in selection {
            for v in &s.value {
                let rows = self
                    .index
                    .get(&IndexValue {
                        column: String::from(&s.column),
                        value: String::from(v),
                    })
                    .unwrap_or(&empty)
                    .clone();
                for row in rows {
                    bts.insert(row);
                }
            }
        }

        let mut ids: Vec<usize> = vec![];
        for id in bts {
            ids.push(id);
        }

        return self.get_rows_by_id(&ids);
    }
}

// pub struct Model {
//     pub tables: Vec<Table>,
// }
