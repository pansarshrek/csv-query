
type Table<'a> = Vec<Vec<&'a str>>;

#[derive(Debug)]
pub struct JoinError {}


fn join<'a, T1, T2>(t1: &'a Vec<T1>, t2: &'a Vec<T2>, predicate: impl Fn(&T1, &T2) -> bool) -> Vec<(&'a T1, &'a T2)> {
    let mut result: Vec<(&T1, &T2)> = vec![];

    for t1_row in t1.iter() {
        for t2_row in t2.iter() {
            if predicate(t1_row, t2_row) {
                result.push((t1_row, t2_row));
            }
        }
    }

    return result;
}

pub fn join_tables<'a>(t1: &'a Table, t2: &'a Table) -> Result<Table<'a>, JoinError> {
    let h1 = t1.get(0).expect("t1 must have a header");
    let h2 = t2.get(0).expect("t2 must have a header");

    let mut all_cols = h1.clone();
    let mut join_col_res = None;
    for col in h2 {
        if !all_cols.contains(col) {
            all_cols.push(col);
        } else {
            if join_col_res != None {
                return Err(JoinError {  });
            }
            join_col_res = Some(*col);
            println!("join by {}", col);
        }
    }

    if join_col_res == None {
        return Err(JoinError{});
    }

    let join_col = join_col_res.unwrap();

    let h1_index = h1.iter().position(|col| *col == join_col).expect("join_col should exist in t1");
    let h2_index = h2.iter().position(|col| *col == join_col).expect("join_col should exist in t2");

    let t1_data: Vec<&Vec<&str>> = t1.iter().skip(1).collect();
    let t2_data: Vec<&Vec<&str>> = t2.iter().skip(1).collect();

    let result = join(&t1_data, &t2_data, |r1, r2| r1[h1_index] == r2[h2_index]);

    let mut table_result: Vec<Vec<&str>> = result.iter().map(|(r1, r2)| {
        let mut new_row = vec![];
        for &cell in r1.iter() {
            new_row.push(cell)
        }
        for (col_index, &cell) in r2.iter().enumerate() {
            if col_index == h2_index {
                continue;
            }
            new_row.push(cell);
        }
        new_row
    }).collect();

    let mut new_header = vec![];
    for &cell in h1.iter() {
        new_header.push(cell)
    }
    for (col_index, &cell) in h2.iter().enumerate() {
        if col_index == h2_index {
            continue;
        }
        new_header.push(cell);
    }

    table_result.insert(0, new_header);

    Ok(table_result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn join_tables_test() {
        let table1 = vec![
            vec!["name", "age", "country"],
            vec!["ni", "35", "swe"],
            vec!["si", "34", "cn"],
            vec!["te", "35", "swe"],
            vec!["la", "25", "usa"],
        ];

        let table2 = vec![
            vec!["name", "item", "quantity"],
            vec!["ni", "book", "5"],
            vec!["ni", "phone", "1"],
            vec!["si", "book", "10"],
            vec!["si", "phone", "1"],
            vec!["si", "computer", "1"],
            vec!["te", "phone", "1"],
            vec!["la", "computer", "1"],
            vec!["la", "book", "3"],
        ];

        let table3 = join_tables(&table1, &table2).unwrap();

        println!("{:#?}", table3);

        assert_eq!(table3.iter().len(), 9);
    }
}