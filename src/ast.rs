#[cfg(test)]
mod test {
use super::*;

    #[test]
    fn add_1_to_string() {
        let e1 = Expression::IntConst(5);
        let e2 = Expression::Variable(String::from("age"));
        let b1 = Box::new(e1);
        let b2 = Box::new(e2);
    
        let added = Expression::Add(b1, b2);
    
        assert_eq!(added.to_string(), "5 + age");
    }
    
    #[test]
    fn add_2_to_string() {
        let age = Expression::Variable(String::from("age"));
        let sum_age = Expression::Sum(Box::new(age));
        let one = Expression::IntConst(1);
        let add_one = Expression::Add(Box::new(sum_age), Box::new(one));
    
        assert_eq!(add_one.to_string(), "sum(age) + 1");
    }
    
    #[test]
    fn sum_1_to_string() {
        let e1 = Expression::IntConst(5);
        let e2 = Expression::Variable(String::from("age"));
        let b1 = Box::new(e1);
        let b2 = Box::new(e2);
    
        let added = Expression::Add(b1, b2);
    
        let sum = Expression::Sum(Box::new(added));
    
        assert_eq!(sum.to_string(), "sum(5 + age)");
    }

    #[test]
    fn tokenize_1() {
        let mut tokenizer = ExpressionTokenizer::new(String::from("5"));

        assert_eq!(tokenizer.next(), Some(String::from("5")));

        let mut tokenizer = ExpressionTokenizer::new(String::from("1+2"));

        assert_eq!(tokenizer.next(), Some(String::from("1")));
        assert_eq!(tokenizer.next(), Some(String::from("+")));
        assert_eq!(tokenizer.next(), Some(String::from("2")));

        let mut tokenizer = ExpressionTokenizer::new(String::from("1 + 2"));

        assert_eq!(tokenizer.next(), Some(String::from("1")));
        assert_eq!(tokenizer.next(), Some(String::from("+")));
        assert_eq!(tokenizer.next(), Some(String::from("2")));
    }

    #[test]
    fn tokenize_2() {
        let mut tokenizer = ExpressionTokenizer::new(String::from("sum(age + 1) + 1"));

        let expected = vec!["sum", "(", "age", "+", "1", ")", "+", "1"];

        for expected_token in expected {
            assert_eq!(tokenizer.next(), Some(String::from(expected_token)));
        }

        
        assert_eq!(tokenizer.next(), None);
    }
}



impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Expression::Add(a, b) => write!(f, "{} + {}", a.as_ref().to_string(), b.as_ref().to_string()),
            Expression::IntConst(i) => write!(f, "{}", i.to_string()),
            Expression::Variable(s) => write!(f, "{}", s),
            Expression::Sum(e) => write!(f, "sum({})", e.as_ref().to_string()),
            Expression::Count => write!(f, "count()"),
            Expression::Values(s) => write!(f, "values({})", s),
        }
    }
}

struct ExpressionTokenizer {
    s: String,
    index: usize,
}

impl ExpressionTokenizer {
    fn new(s: String) -> ExpressionTokenizer {
        ExpressionTokenizer { s: s, index: 0 }
    }
    fn is_token_separator(c: char) -> bool {
        let tokens = ['(', ')', '+'];
        tokens.contains(&c)
    }
}

impl std::iter::Iterator for ExpressionTokenizer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.s.len() {
            return None;
        }
        
        let ws: String = self.s.chars().skip(self.index).take_while(|c| *c == ' ').collect();
        self.index = self.index + ws.len();

        let curr_char: Vec<char> = self.s.chars().skip(self.index).take(1).collect();
        if ExpressionTokenizer::is_token_separator(curr_char[0]) {
            self.index = self.index + 1;
            return Some(String::from(curr_char[0]));
        }

        let token: String = self.s.
            chars()
            .skip(self.index)
            .take_while(|c| !ExpressionTokenizer::is_token_separator(*c))
            .collect();

        self.index = self.index + token.len();

        Some(String::from(token.trim()))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseExpressionError {}

impl Expression {
    // pub fn from_string(s: &str) -> Result<Expression, ParseExpressionError> {
    //     let tkn = ExpressionTokenizer::new(String::from(s));
    //     let token_it = tkn.into_iter();
    //     {
    //         match token {
    //             ""
    //         }
    //     }


    //     Ok(Expression::IntConst(0))
    // }

    // fn from_iter(iter: &mut ExpressionTokenizer) -> Result<Expression, ParseExpressionError> {
    //     let tok = iter.next();
    //     let as_deref = tok.as_deref();
    //     match as_deref {
    //         Some("sum") => {
    //             let tok = iter.next();
    //             if tok.as_deref() != Some("(") {
    //                 return Err(ParseExpressionError {  });
    //             }
    //             let expr = Expression::from_iter(iter)?;

    //             Ok(Expression::Sum(Box::new(expr)))
    //         },
    //         Some("(") => {
    //             Expression::from_iter(iter)
    //         },
    //         Some(")") => {

    //         }
    //         Some(s) if s.parse::<i64>() == Ok => {
    //             let num: i64 = s.parse();
    //             Expression::IntConst(())
    //         }
    //     }
    // }
}

pub struct Value {
    num: i64
}

pub struct BinOp {
    e1: Box<Expression>,
    e2: Box<Expression>,
}

pub type ColReference = String;

pub enum Expression {
    // StrConst(String),
    IntConst(i64),
    // DecConst(i64, u8),
    Variable(ColReference),
    Add(Box<Expression>, Box<Expression>),
    // Minus(BinOp<'a>),
    Sum(Box<Expression>),
    Count,
    Values(ColReference),
}

fn build() {

}

pub struct SelectParam {
    pub field: String,
    pub value: Vec<String>,
}

pub enum Command {
    Select(SelectParam),
    Deselect(SelectParam),
    Watch(Expression),
    Unwatch(String),
}

// select field=value

// 5 <- Value
// age <- Variable
// age + 5 <- BinaryOperation