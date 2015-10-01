use std::result;
use num;

pub type Result<T> = result::Result<T, String>;

#[derive(Debug,Clone)]
pub enum Op {
    OpAdd,
    OpSub,
    OpDiv,
    OpMul,
    OpExp,
    OpUnNeg
}

impl Op {
    fn from_str(op: &str) -> Option<Op> {
        match op {
            "+"  => Some(Op::OpAdd),
            "-"  => Some(Op::OpSub),
            "/"  => Some(Op::OpDiv),
            "*"  => Some(Op::OpMul),
            "**" => Some(Op::OpExp),
            _    => None,
        }
    }

    fn precedence(&self) -> u8 {
        match *self {
            Op::OpAdd   => 0,
            Op::OpSub   => 0,
            Op::OpDiv   => 1,
            Op::OpMul   => 1,
            Op::OpExp   => 2,
            Op::OpUnNeg => 3,
        }
    }

    fn apply_binary(&self, a: f64, b: f64) -> Result<f64> {
        match *self {
            Op::OpAdd   => Ok(a + b),
            Op::OpSub   => Ok(a - b),
            Op::OpDiv   => Ok(a / b),
            Op::OpMul   => Ok(a * b),
            Op::OpExp   => Ok(num::pow(a, b as usize)),
            Op::OpUnNeg => Err("Not a binary operation".to_string()),
        }
    }
}


fn is_operator_char(c: &char) -> bool {
    match *c {
        '+' | '-' | '/' | '*' => true,
                            _ => false
    }
}
           

#[derive(Debug,Clone)]
pub enum Tok {
    TokNum(f64),
    TokOp(Op),
    TokVar(String),
    TokRParen,
    TokLParen
}

#[derive(Debug)]
pub enum Expr {
    ExprNum(f64),
    ExprOp(Op),
    ExprBinary(Op, Box<Expr>, Box<Expr>),
    ExprUnary(Op, Box<Expr>),
}


pub fn get_number<'a>(stream: &'a [char]) -> Option<(Tok, &'a [char])> {
    let mut i = 0;
    let n = stream.len();
    while i < n && stream[i].is_whitespace() {
        i += 1;
    }
    let mut found = false;
    let mut number = 0f64;
    while i < n && stream[i].is_digit(10) {
        found = true;
        let d = stream[i].to_digit(10).expect("Invalid digit") as f64;
        number = number * 10f64 + d;
        i += 1
    }
    if found { Some((Tok::TokNum(number), &stream[i..n])) } else { None }
}

pub fn get_operator<'a>(stream: &'a [char]) -> Option<Result<(Tok, &'a [char])>> {
    let mut i = 0;
    let n = stream.len();
    while i < n && stream[i].is_whitespace() {
        i += 1;
    }
    let mut opstr = String::new();
    while i < n && is_operator_char(&stream[i]) {
        opstr.push(stream[i]);
        i += 1;
    }
    if !opstr.is_empty() {
        Some(Op::from_str(&opstr)
                .map(|v| (Tok::TokOp(v), &stream[i..n]))
                .ok_or(format!("Invalid operator sequence {:?}", opstr)))
    } else {
        None
    }
}

pub fn get_paren<'a>(stream: &'a [char]) -> Option<(Tok, &'a [char])> {
    let stream = skip_whitespace(stream);
    let n = stream.len();
    if n > 0 {
        match stream[0] {
            '(' => Some(Tok::TokLParen),
            ')' => Some(Tok::TokRParen),
            _   => None
        }
    } else {
        None
    }.map(|x| (x, &stream[1..n]))
}

pub fn get_var<'a>(stream: &'a [char]) -> Option<(Tok, &'a [char])> {
    let stream = skip_whitespace(stream);
    let n = stream.len();
    let mut var = String::new();
    let mut i = 0;
    while i < n && (stream[i].is_alphabetic() || stream[i] == '_') {
        var.push(stream[i]);
        i += 1;
    }
    if !var.is_empty() {
        Some((Tok::TokVar(var), &stream[i..n]))
    } else {
        None
    }
}

pub fn skip_whitespace<'a>(stream: &'a [char]) -> &'a [char] {
    let mut i = 0;
    while i < stream.len() && stream[i].is_whitespace() {
        i += 1;
    }
    &stream[i..stream.len()]
}

pub fn tok(s: &str) -> Result<Vec<Tok>> {
    let mut ret = Vec::new();
    let mut t: &[char] = &s.chars().collect::<Vec<_>>();
    while t.len() != 0 {
        t = skip_whitespace(t);
        let mut found = false;
        if let Some((tok, u)) = get_number(t) {
            ret.push(tok);
            t = u;
            found = true;
        }
        if let Some(r) = get_operator(t) {
            let (tok, u) = try!(r);
            ret.push(tok);
            t = u;
            found = true;
        }
        if let Some((tok, u)) = get_paren(t) {
            ret.push(tok);
            t = u;
            found = true;
        }
        if let Some((tok, u)) = get_var(t) {
            ret.push(tok);
            t = u;
            found = true;
        }
        if !found {
            return Err(format!("Stuck tokenizing: {:?}", t));
        }
    }
    Ok(ret)
}


// TODO: this is ugly; most likely can be written more idiomatically.
pub fn postfix(e: &str) -> Result<Vec<Tok>> {
    let mut tokens = try!(tok(e));
    let mut post: Vec<Tok> = Vec::new();
    let mut stack: Vec<Tok> = Vec::new();
    stack.push(Tok::TokLParen);
    tokens.push(Tok::TokRParen);
    
    for token in &tokens {
        match *token {
            Tok::TokNum(n) => post.push(token.clone()),
            Tok::TokOp(ref op) => {
                while !stack.is_empty() {
                    if stack.last().map_or(false, |t| -> bool {
                        if let Tok::TokOp(ref pp) = *t {
                            pp.precedence() > op.precedence()
                        } else {
                            false
                        }
                    }) { post.push(stack.pop().unwrap()); }
                    else { break; }
                }
                stack.push(token.clone());
            },
            Tok::TokLParen => {
                stack.push(token.clone());
            },
            Tok::TokRParen => {
                loop {
                    let top = stack.pop();
                    if top.is_none() {
                        return Err("Syntax error".to_string());
                    }
                    if let Some(Tok::TokLParen) = top {
                        break;
                    }
                    post.push(top.unwrap());
                }

            },
            _ => {}
        }
    }
    Ok(post)
}


pub fn eval(s: &str) -> Result<f64> {
    let post = try!(postfix(s));
    let mut stack = Vec::new();
    for token in &post {
        match *token {
            Tok::TokNum(n) => stack.push(n),
            Tok::TokOp(ref op) => {
                let b = try!(stack.pop().ok_or("Premature stack end".to_string()));
                let a = try!(stack.pop().ok_or("Premature stack end".to_string()));
                let r = try!(op.apply_binary(a, b));
                stack.push(r);
            }
            _ => {}
        }
    }
    stack.pop().ok_or("No result".to_string())
}


#[cfg(tests)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_tokenize() {
        let expr = "1 + 2 - 5 + (7 +8)";
        let toks = tokenize(expr);
        let expected = vec![Tok::TokNum(1f64),
                            Tok::TokOp(Op::OpAdd),
                            Tok::TokNum(2f64),
                            Tok::TokOp(Op::OpSub),
                            Tok::TokNum(5f64),
                            Tok::TokOp(Op::OpAdd),
                            Tok::TokLParen,
                            Tok::TokNum(7f64),
                            Tok::TokOp(Op::OpAdd),
                            Tok::TokNum(8f64),
                            Tok::TokRParen];

        assert_eq!(toks, expected);
    }


}
