use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::ParseIntError;
use std::vec;

pub type Column = String;
pub type Columns = Vec<Column>;
pub type Value = String;
pub type Record = Vec<DataType>;

#[derive(Eq, Hash, PartialEq, Debug, Clone, Ord, PartialOrd)]
pub enum DataType {
    String(String),
    Decimal(i64, u8),
    Int(i64),
}

impl DataType {
    pub fn from_string(s: &str) -> DataType {
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
                    _ => DataType::String(String::from(s)),
                };
            }
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
            }
            (DataType::Decimal(num1, p), DataType::Int(num2)) => {
                num1.cmp(&Self::precision_mul(*num2, *p))
            }
            (DataType::Int(num1), DataType::Decimal(num2, p)) => {
                Self::precision_mul(*num1, *p).cmp(num2)
            }
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
            }
            DataType::Decimal(n, p) => {
                num1 = n;
                p1 = p;
            }
            DataType::String(_) => {
                num1 = 0;
                p1 = 0;
            }
        }
        match next {
            DataType::Int(n) => {
                num2 = n;
                p2 = 0;
            }
            DataType::Decimal(n, p) => {
                num2 = n;
                p2 = p;
            }
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
            }
            DataType::Int(n) => n.fmt(f),
            DataType::String(s) => s.fmt(f),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Selection {
    pub column: Column,
    pub value: Value,
}

pub type DataContextCallback = fn(&DataContext) -> ();

pub struct DataContext<'a> {
    table: &'a Table,
    selection: Vec<Selection>,

    selected_records: Vec<&'a Record>,
    callbacks: Vec<DataContextCallback>,
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

    pub fn observe(&mut self, cb: DataContextCallback) {
        self.callbacks.push(cb);
    }

    fn notify_observers(&self) {
        for cb in &self.callbacks {
            cb(self);
        }
    }

    fn update_selected_records(&mut self) {
        let possible_records = self.table.get_possible(&self.selection);
        self.selected_records = possible_records;
        self.notify_observers();
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
                .reduce(DataType::sum)
                .unwrap_or(DataType::Int(0))
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

/// represents a data table loaded from
/// an external source, like a CSV file
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
            callbacks: vec![],
        };
        ctx.update_selected_records();
        ctx
    }

    fn index_value(&mut self, v: IndexValue) {
        let v = self.index.entry(v).or_insert(Vec::new());
        v.push(self.records.len());
    }

    pub fn get_columns(&self) -> &Columns {
        &self.columns
    }

    pub fn get_col_index(&self, col: &str) -> Option<usize> {
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
        for Selection { column, value } in selection {
            let rows = self
                .index
                .get(&IndexValue {
                    column: String::from(column),
                    value: String::from(value),
                })
                .unwrap_or(&empty)
                .clone();
            for row in rows {
                bts.insert(row);
            }
        }

        let mut ids: Vec<usize> = vec![];
        for id in bts {
            ids.push(id);
        }

        return self.get_rows_by_id(&ids);
    }
}

pub struct Model {
    tables: Vec<Table>,
    columns: HashMap<Column, Vec<usize>>,
}

impl Model {
    /// creates a new model
    pub fn new() -> Model {
        Model {
            tables: vec![],
            columns: HashMap::new(),
        }
    }

    /// adds a table to the model
    pub fn add_table(&mut self, table: Table) {
        for col in &table.columns {
            let table_indexes = self.columns.entry(String::from(col)).or_insert(Vec::new());
            table_indexes.push(self.tables.len());
        }
        self.tables.push(table);
    }

    /// get a table by name from the model
    pub fn get_table(&self, table_name: &str) -> Option<&Table> {
        let table = self.tables.iter().find(|t| t.name == table_name);
        table
    }

    fn get_tables_and_col_indices(&self, col: &str) -> Vec<(&Table, usize)> {
        let tables_and_col_indices: Vec<(&Table, usize)> = self
            .tables
            .iter()
            .map(|t| (t, t.get_col_index(col)))
            .filter(|(t, index)| *index != None)
            .map(|(t, index)| (t, index.unwrap()))
            .collect();
        tables_and_col_indices
    }

    /// get the unique values for a given column
    pub fn get_all_values(&self, col: &str) -> Vec<&DataType> {
        let tables_and_col_indices = self.get_tables_and_col_indices(col);

        let mut bset: BTreeSet<&DataType> = std::collections::btree_set::BTreeSet::new();

        for (t, index) in tables_and_col_indices {
            for r in &t.records {
                bset.insert(&r[index]);
            }
        }

        let vec: Vec<&DataType> = bset.into_iter().collect();

        vec
    }

    /// starts a new data context for the model
    pub fn new_data_context(&self) -> ModelContext {
        ModelContext::new(self)
    }

    /// get values and their associative state for a given column
    /// depending on current selections
    pub fn get_values(&self, col: &str) -> Vec<()> {
        vec![]
    }

    // pub fn new_context(&self) -> DataContext {
    //     let mut ctx = DataContext {
    //         table: self,
    //         selection: vec![],
    //         selected_records: vec![],
    //         callbacks: vec![],
    //     };
    //     ctx.update_selected_records();
    //     ctx
    // }
}

pub struct ModelContext<'a> {
    model: &'a Model,
    selection: Vec<Selection>,
}

impl ModelContext<'_> {
    pub fn new<'a>(model: &'a Model) -> ModelContext {
        ModelContext {
            model: model,
            selection: vec![],
        }
    }
    pub fn select(&mut self, select: &Selection) -> &ModelContext {
        self.selection.push(select.clone());
        self
    }

    pub fn deselect(&mut self, select: &Selection) -> &ModelContext {
        self.selection
            .iter()
            .position(|p| p == select)
            .map(|i| self.selection.remove(i));

        self
    }

    pub fn get_selected(&self, col: &str) -> Vec<&DataType> {
        todo!()
    }

    pub fn get_possible(&self, col: &str) -> Vec<&DataType> {
        // country
        let target_tables = self.model.get_tables_and_col_indices(col);

        let mut by_col = HashMap::new(); // item=phone
        for s in &self.selection {
            let values = by_col.entry(s.column.as_str()).or_insert(Vec::new());
            values.push(s.value.as_str());
        }

        let mut virtual_selection: Vec<Selection> = self.selection.clone();

        // for each column with selected values
        // find tables
        // for each table find records matching selection
        // add all selected values to virtual selection
        for (selected_column, selected_values) in by_col {
            let ts = self.model.get_tables_and_col_indices(selected_column);
            for (table, col_index) in &ts {
                let possible_records = table.get_possible(&self.selection);
                for record in &possible_records {
                    for (index, column) in table.get_columns().iter().enumerate() {
                        virtual_selection.push(Selection {
                            column: String::from(column),
                            value: record[index].to_string(),
                        });
                    }
                }
            }
        }

        // Get values from tables with column
        let mut values: BTreeSet<&DataType> = BTreeSet::new();
        for (table, col_index) in &target_tables {
            let records = table.get_possible(&virtual_selection);
            for record in &records {
                values.insert(record.get(*col_index).unwrap());
            }
        }

        values.into_iter().collect()
    }

    pub fn get_excluded(&self, col: &str) -> Vec<&DataType> {
        todo!()
    }
}

pub struct BetterSelection<'a> {
    pub selected_values: HashMap<&'a str, Vec<&'a str>>
}

impl BetterSelection<'_> {
    pub fn new<'a>(vals: Vec<(&'a str, Vec<&'a str>)>) -> BetterSelection<'a> {
        let mut bs = BetterSelection {
            selected_values: HashMap::new()
        };

        for (k, v) in vals {
            bs.selected_values.insert(k, v);
        }

        return bs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_selection() {
        let mut bs = BetterSelection{ selected_values: HashMap::new() };

        bs = BetterSelection::new(vec![("name", vec!["ni", "ai"]), ("country", vec!["swe"])]);
    }

    fn copy_strings(strs: &Vec<&str>) -> Vec<String> {
        strs.into_iter().map(|&s| String::from(s)).collect()
    }

    fn data_types(strs: &Vec<&str>) -> Vec<DataType> {
        strs.into_iter().map(|s| DataType::from_string(s)).collect()
    }

    fn table(name: &str, data: Vec<Vec<&str>>) -> Table {
        let table_header = data.get(0).expect("table header must exist");
        let mut t = Table::new(name, copy_strings(table_header));
        for row in data.iter().skip(1) {
            t.insert(data_types(row));
        }
        t
    }

    fn model(tables: Vec<Table>) -> Model {
        let mut m = Model::new();
        for t in tables {
            m.add_table(t);
        }
        m
    }

    fn fixture_model() -> Model {
        let data1 = vec![
            vec!["name", "country"],
            vec!["ni", "swe"],
            vec!["ai", "swe"],
            vec!["ni", "swe"],
            vec!["qe", "cn"],
            vec!["usa", "usa"],
        ];
        let t1 = table("t1", data1);

        let data2 = vec![
            vec!["name", "item"],
            vec!["ni", "phone"],
            vec!["ni", "keys"],
            vec!["ai", "toy"],
            vec!["qe", "sandwich"],
        ];
        let t2 = table("t2", data2);

        let data3 = vec![
            vec!["item", "price"],
            vec!["phone", "10"],
            vec!["sandwich", "1.5"],
            vec!["toy", "2"],
        ];
        let t3 = table("t3", data3);

        let m = model(vec![t1, t2, t3]);
        m
    }

    #[test]
    fn model_get_values_1() {
        let data = vec![vec!["name"], vec!["ni"], vec!["ai"], vec!["ni"]];

        let t1 = table("t1", data);

        let m = model(vec![t1]);

        let values = m.get_all_values("name");

        let data_t: Vec<DataType> = data_types(&vec!["ai", "ni"]);
        let as_ref: Vec<&DataType> = data_t.iter().collect();

        assert_eq!(values, as_ref);
    }

    #[test]
    fn model_get_values_2() {
        // Arrange
        let data1 = vec![vec!["name"], vec!["ni"], vec!["ai"], vec!["ni"]];
        let t1 = table("t1", data1);

        let data2 = vec![
            vec!["name", "item"],
            vec!["ni", "phone"],
            vec!["ni", "keys"],
            vec!["ai", "toy"],
            vec!["qe", "sandwich"],
        ];
        let t2 = table("t2", data2);

        let m = model(vec![t1, t2]);

        // Act
        let get_names = m.get_all_values("name");
        let get_items = m.get_all_values("item");

        let name_values: Vec<DataType> = data_types(&vec!["ai", "ni", "qe"]);
        let name_val_refs: Vec<&DataType> = name_values.iter().collect();

        // Assert
        assert_eq!(get_names, name_val_refs);

        let item_values: Vec<DataType> = data_types(&vec!["keys", "phone", "sandwich", "toy"]);
        let item_refs: Vec<&DataType> = item_values.iter().collect();

        assert_eq!(get_items, item_refs);
    }

    #[test]
    fn model_get_values_3() {
        let model = fixture_model();

        let mut ctx = model.new_data_context();
        ctx.select(&Selection { column: String::from("item"), value: String::from("phone") });
        ctx.select(&Selection { column: String::from("item"), value: String::from("sandwich") });

        let possible_countries = ctx.get_possible("country");

        assert_eq!(possible_countries, vec![&DataType::from_string("cn"), &DataType::from_string("swe")]);

        let possible_names = ctx.get_possible("name");

        assert_eq!(possible_names, vec![&DataType::from_string("ni"), &DataType::from_string("qe")]);

        let possible_prices = ctx.get_possible("price");
        
        assert_eq!(possible_prices, vec![&DataType::from_string("1.5"), &DataType::from_string("10")])
    }
}
