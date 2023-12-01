use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::num::ParseIntError;

pub type Column = String;
pub type Columns = Vec<Column>;
pub type Value = String;
pub type Record = Vec<DataType>;

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum DataType {
    String(String),
    Decimal(i64, u8),
    Int(i64),
}

impl DataType {
    pub fn from_string(s: &String) -> DataType {
        let int_result: Result<i64, _> = s.parse();
        match int_result {
            Ok(i) => return DataType::Int(i),
            _ => (),
        }

        let float_result: Result<f64, _> = s.parse();
        match float_result {
            Ok(_f) => {
                let parts: Vec<&str> = s.split(".").collect();
                let decimal_points = parts[1].len();
                let value = String::from(parts[0]) + parts[1];
                let num_result: Result<i64, ParseIntError> = value.parse();
                return match num_result {
                    Ok(num) => DataType::Decimal(num, decimal_points as u8),
                    _ => DataType::String(String::from(s))
                }
            },
            _ => DataType::String(String::from(s)),
        }
    }

    fn cmp(x: &DataType, y: &DataType) -> Ordering {
        let t = (x, y);

        return match t {
            (DataType::String(s1), DataType::String(s2)) => s1.cmp(s2),
            (DataType::String(_), _) => Ordering::Less,
            (_, DataType::String(_)) => Ordering::Greater,
            (DataType::Int(i1), DataType::Int(i2)) => i1.cmp(i2),
            (DataType::Decimal(num1, p1), DataType::Decimal(num2, p2)) => {
                Self::precision_mul(*num1, *p2).cmp(&Self::precision_mul(*num2, *p1))
            },
            (DataType::Decimal(num1, p), DataType::Int(num2)) => {
                num1.cmp(&Self::precision_mul(*num2, *p))
            },
            (DataType::Int(num1), DataType::Decimal(num2, p)) => {
                Self::precision_mul(*num1, *p).cmp(num2)
            },
        };
    }

    fn add(d1: i64, d2: i64, p1: u8, p2: u8) -> DataType {
        if p1 == 0 && p2 == 0 {
            return DataType::Int(d1 + d2);
        }
    
        if p1 == p2 {
            return DataType::Decimal(d1 + d2, p1);
        } else if p1 > p2 {
            let diff = p1 - p2;
            return DataType::Decimal(d1 + Self::precision_mul(d2, diff), p1);
        } else {
            let diff = p2 - p1;
            return DataType::Decimal(Self::precision_mul(d1, diff) + d2, p2);
        }
    }
    
    fn sum(acc: DataType, next: DataType) -> DataType {
        let num1;
        let p1;
        let num2;
        let p2;
        match acc {
            DataType::Int(n) => {
                num1 = n;
                p1 = 0;
            },
            DataType::Decimal(n, p) => {
                num1 = n;
                p1 = p;
            },
            DataType::String(_) => {
                num1 = 0;
                p1 = 0;
            }
        }
        match next {
            DataType::Int(n) => {
                num2 = n;
                p2 = 0;
            },
            DataType::Decimal(n, p) => {
                num2 = n;
                p2 = p;
            },
            DataType::String(_) => {
                num2 = 0;
                p2 = 0;
            }
        }

        Self::add(num1, num2, p1, p2)
    }
    
    fn precision_mul(num: i64, prec: u8) -> i64 {
        if prec == 0 {
            return num;
        }
        let mut res = num;
        for _i in 0..prec {
            res = res * 10;
        }
        res
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DataType::Decimal(v, p) => {
                if *p == 0 {
                    write!(f, "{}", v)
                } else {
                    let dec_pos = *p as usize;
                    let mut v_str = v.to_string();
                    while v_str.len() <= dec_pos {
                        v_str.insert(0, '0');
                    }
                    v_str.insert(v_str.len() - dec_pos, '.');
                    v_str.fmt(f)
                }
            },
            DataType::Int(n) => n.fmt(f),
            DataType::String(s) => s.fmt(f),
        }
    }
}



#[derive(Eq, Hash, PartialEq, Debug, Clone)]
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
    pub fn select(&mut self, selection: &Selection) -> &DataContext {
        self.selection.push(selection.clone());
        self.update_selected_records();
        self
    }

    pub fn deselect(&mut self, selection: &Selection) -> &DataContext {
        for (i, select) in self.selection.iter().enumerate() {
            if select == selection {
                self.selection.remove(i);
                break;
            }
        }
        self.update_selected_records();
        self
    }

    fn update_selected_records(&mut self) {
        let possible_records = self.table.get_possible(&self.selection);
        self.selected_records = possible_records;
    }

    pub fn count(&self) -> usize {
        let vals = &self.selected_records;
        vals.iter().count()
    }

    pub fn sum(&self, col: Column) -> Option<DataType> {
        self.table.get_col_index(&col).map(|i| {
            self.selected_records
                .iter()
                .map(|x| &x[i])
                .filter(|x| match x {
                    DataType::Int(_) => true,
                    DataType::Decimal(_, _) => true,
                    _ => false,
                })
                .cloned()
                .reduce(DataType::sum).unwrap()
        })
    }

    pub fn max(&self, col: Column) -> Option<DataType> {
        self.table.get_col_index(&col).and_then(|i| {
            self.selected_records
                .iter()
                .map(|x| &x[i])
                .filter(|x| match x {
                    DataType::Int(_) => true,
                    DataType::Decimal(_, _) => true,
                    _ => false,
                })
                .cloned()
                .max_by(DataType::cmp)
                // .reduce(DataType::max)
        })
    }

    pub fn min(&self, col: Column) -> Option<DataType> {
        self.table.get_col_index(&col).and_then(|i| {
            self.selected_records
                .iter()
                .map(|x| &x[i])
                .filter(|x| match x {
                    DataType::Int(_) => true,
                    DataType::Decimal(_, _) => true,
                    _ => false,
                })
                .cloned()
                .min_by(DataType::cmp)
                // .reduce(DataType::min)
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
        let mut ctx = DataContext {
            table: self,
            selection: vec![],
            selected_records: vec![],
        };
        ctx.update_selected_records();
        ctx
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
                value: field.to_string(),
                // match field {
                //     DataType::String(d) => String::from(d),
                //     DataType::Int(i) => i.to_string(),
                //     DataType::Decimal(f, p) => {
                //         self.floats.insert(String::from(f), f.parse().unwrap_or(0.0));
                //         String::from(f)
                //     }
                // },
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
