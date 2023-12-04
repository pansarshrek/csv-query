#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decimal_num_1_to_string() {
        let e1 = Expression::DecConst(123, 2);
        assert_eq!(e1.to_string(), "1.23");

        let e2 = Expression::DecConst(99, 0);
        assert_eq!(e2.to_string(), "99");

        let e3 = Expression::DecConst(70000, 1);
        assert_eq!(e3.to_string(), "7000.0");
    }

    #[test]
    fn add_1_to_string() {
        let e1 = Expression::IntConst(5);
        let e2 = Expression::Variable(String::from("age"));
        let b1 = Box::new(e1);
        let b2 = Box::new(e2);

        let added = Expression::Add(b1, b2);

        assert_eq!(added.to_string(), "add(5, age)");
    }

    #[test]
    fn add_2_to_string() {
        let age = Expression::Variable(String::from("age"));
        let sum_age = Expression::Sum(Box::new(age));
        let one = Expression::IntConst(1);
        let add_one = Expression::Add(Box::new(sum_age), Box::new(one));

        assert_eq!(add_one.to_string(), "add(sum(age), 1)");
    }

    #[test]
    fn sum_1_to_string() {
        let e1 = Expression::IntConst(5);
        let e2 = Expression::Variable(String::from("age"));
        let b1 = Box::new(e1);
        let b2 = Box::new(e2);

        let added = Expression::Add(b1, b2);

        let sum = Expression::Sum(Box::new(added));

        assert_eq!(sum.to_string(), "sum(add(5, age))");
    }

    #[test]
    fn tokenize_1() {
        let mut tokenizer = ExpressionTokenizer::new(String::from("5"));

        assert_eq!(tokenizer.next(), Some(String::from("5")));

        let mut tokenizer = ExpressionTokenizer::new(String::from("add(1,2)"));

        assert_eq!(tokenizer.next(), Some(String::from("add")));
        assert_eq!(tokenizer.next(), Some(String::from("(")));
        assert_eq!(tokenizer.next(), Some(String::from("1")));
        assert_eq!(tokenizer.next(), Some(String::from(",")));
        assert_eq!(tokenizer.next(), Some(String::from("2")));
        assert_eq!(tokenizer.next(), Some(String::from(")")));

        let mut tokenizer = ExpressionTokenizer::new(String::from("1 + 2"));

        assert_eq!(tokenizer.next(), Some(String::from("1")));
        assert_eq!(tokenizer.next(), Some(String::from("+")));
        assert_eq!(tokenizer.next(), Some(String::from("2")));
    }

    #[test]
    fn tokenize_2() {
        let mut tokenizer = ExpressionTokenizer::new(String::from("add(sum(add(age, 1), 1)"));

        let expected = vec![
            "add", "(", "sum", "(", "add", "(", "age", ",", "1", ")", ",", "1", ")",
        ];

        for expected_token in expected {
            assert_eq!(tokenizer.next(), Some(String::from(expected_token)));
        }

        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn parse_expression_1() {
        let expr_string = "1";
        let expr = Expression::from_string(expr_string);

        assert_eq!(expr, Ok(Expression::IntConst(1)));
    }

    #[test]
    fn parse_expression_2() {
        let expr_string = "add(1,1)";
        let expr = Expression::from_string(expr_string);

        let expect_expr = Expression::Add(
            Box::new(Expression::IntConst(1)),
            Box::new(Expression::IntConst(1)),
        );
        assert_eq!(expr, Ok(expect_expr));
    }

    #[test]
    fn parse_expression_3() {
        let expr_string = "add(var,1)";
        let expr = Expression::from_string(expr_string);

        let expect_expr = Expression::Add(
            Box::new(Expression::Variable(String::from("var"))),
            Box::new(Expression::IntConst(1)),
        );
        assert_eq!(expr, Ok(expect_expr));
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Expression::Add(a, b) => write!(
                f,
                "add({}, {})",
                a.as_ref().to_string(),
                b.as_ref().to_string()
            ),
            Expression::IntConst(i) => write!(f, "{}", i.to_string()),
            Expression::Variable(s) => write!(f, "{}", s),
            Expression::Sum(e) => write!(f, "sum({})", e.as_ref().to_string()),
            Expression::Count => write!(f, "count()"),
            Expression::Values(s) => write!(f, "values({})", s),
            Expression::DecConst(v, p) => {
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
        }
    }
}

struct ExpressionTokenizer {
    s: String,
    index: usize,
}

impl ExpressionTokenizer {
    fn new(s: String) -> ExpressionTokenizer {
        ExpressionTokenizer { s, index: 0 }
    }
    fn is_token_separator(c: char) -> bool {
        let tokens = ['(', ')', '+', ','];
        tokens.contains(&c)
    }
}

impl std::iter::Iterator for ExpressionTokenizer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.s.len() {
            return None;
        }

        let ws: String = self
            .s
            .chars()
            .skip(self.index)
            .take_while(|c| *c == ' ')
            .collect();
        self.index = self.index + ws.len();

        let curr_char: Vec<char> = self.s.chars().skip(self.index).take(1).collect();
        if ExpressionTokenizer::is_token_separator(curr_char[0]) {
            self.index = self.index + 1;
            return Some(String::from(curr_char[0]));
        }

        let token: String = self
            .s
            .chars()
            .skip(self.index)
            .take_while(|c| !ExpressionTokenizer::is_token_separator(*c))
            .collect();

        self.index = self.index + token.len();

        Some(String::from(token.trim()))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseExpressionError<'a> {
    pub message: &'a str,
}

impl Expression {
    pub fn from_string<'a>(s: &str) -> Result<Expression, ParseExpressionError<'a>> {
        let mut tkn = ExpressionTokenizer::new(String::from(s));

        let parsed_expr = Expression::from_iter(&mut tkn)?;

        let tok = tkn.next();
        if tok.as_deref() != None {
            return Err(ParseExpressionError {
                message: "Expected end of input",
            });
        }

        Ok(parsed_expr)
    }

    fn from_iter<'a>(
        iter: &mut ExpressionTokenizer,
    ) -> Result<Expression, ParseExpressionError<'a>> {
        let tok = iter.next();
        let as_deref = tok.as_deref();
        let parsed_expression = match as_deref {
            Some("sum") => {
                let tok = iter.next();
                if tok.as_deref() != Some("(") {
                    return Err(ParseExpressionError {
                        message: "Expected '('",
                    });
                }
                let expr = Expression::from_iter(iter)?;
                let tok = iter.next();
                if tok.as_deref() != Some(")") {
                    return Err(ParseExpressionError {
                        message: "Expected ')'",
                    });
                }

                Expression::Sum(Box::new(expr))
            }
            Some("add") => {
                let tok = iter.next();
                if tok.as_deref() != Some("(") {
                    return Err(ParseExpressionError {
                        message: "Expected '('",
                    });
                }
                let expr1 = Expression::from_iter(iter)?;
                let tok = iter.next();
                if tok.as_deref() != Some(",") {
                    return Err(ParseExpressionError {
                        message: "Expected ','",
                    });
                }
                let expr2 = Expression::from_iter(iter)?;
                let tok = iter.next();
                if tok.as_deref() != Some(")") {
                    return Err(ParseExpressionError {
                        message: "Expected ')'",
                    });
                }

                Expression::Add(Box::new(expr1), Box::new(expr2))
            }
            Some("count") => {
                let tok = iter.next();
                if tok.as_deref() != Some("(") {
                    return Err(ParseExpressionError {
                        message: "Expected '('",
                    });
                }

                let tok = iter.next();
                if tok.as_deref() != Some(")") {
                    return Err(ParseExpressionError {
                        message: "Expected ')'",
                    });
                }

                Expression::Count
            }
            Some("values") => {
                let tok = iter.next();
                if tok.as_deref() != Some("(") {
                    return Err(ParseExpressionError {
                        message: "Expected '('",
                    });
                }

                let e = Expression::from_iter(iter)?;

                let tok = iter.next();
                if tok.as_deref() != Some(")") {
                    return Err(ParseExpressionError {
                        message: "Expected ')'",
                    });
                }

                match e {
                    Expression::Variable(var) => Expression::Values(var),
                    _ => {
                        return Err(ParseExpressionError {
                            message: "Expected variable expression",
                        })
                    }
                }
            }
            Some(s) => {
                if s.len() == 0 {
                    return Err(ParseExpressionError {
                        message: "Expected non-empty string",
                    });
                }

                let int_result: Result<i64, _> = s.parse();
                match int_result {
                    Ok(i) => return Ok(Expression::IntConst(i)),
                    _ => (),
                }

                let float_result: Result<f64, _> = s.parse();
                match float_result {
                    Ok(_f) => {
                        let parts: Vec<&str> = s.split(".").collect();
                        let decimal_points = parts[1].len();
                        let value = String::from(parts[0]) + parts[1];
                        let num_result: Result<i64, _> = value.parse();
                        match num_result {
                            Ok(num) => return Ok(Expression::DecConst(num, decimal_points as u8)),
                            _ => (),
                        }
                    }
                    _ => (),
                }

                if s.len() == 1 {
                    let c = s.chars().next().unwrap();
                    if !c.is_alphanumeric() {
                        return Err(ParseExpressionError {
                            message: "Expected alphanumeric char",
                        });
                    }
                }

                Expression::Variable(String::from(s))
            }
            None => {
                return Err(ParseExpressionError {
                    message: "Unexpected end of input",
                });
            }
        };

        return Ok(parsed_expression);
    }
}

pub type ColReference = String;

#[derive(Eq, PartialEq, Debug)]
pub enum Expression {
    // StrConst(String),
    IntConst(i64),
    DecConst(i64, u8),
    Variable(ColReference),
    Add(Box<Expression>, Box<Expression>),
    // Minus(BinOp<'a>),
    Sum(Box<Expression>),
    Count,
    Values(ColReference),
}

// fn build() {

// }

// pub struct SelectParam {
//     pub field: String,
//     pub value: Vec<String>,
// }

// pub enum Command {
//     Select(SelectParam),
//     Deselect(SelectParam),
//     Watch(Expression),
//     Unwatch(String),
// }

// pub enum Query {
//     Tabular(TabularQuery),
//     Aggregate(AggregateQuery),
//     Possible(PossibleQuery)
// }

// pub enum Func {
//     Sum,
//     Count,
//     Max,
//     Min,
// }

// pub struct AggregateQuery {
//     column: String,
//     by: Vec<String>,
//     func: Func,
// }

// pub struct TabularQuery {
//     columns: Vec<String>,
// }

// pub struct PossibleQuery {
//     column: String
// }

// select field=value

// 5 <- Value
// age <- Variable
// age + 5 <- BinaryOperation
