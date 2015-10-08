use std::result;
use num;

pub type Result<T> = result::Result<T, String>;

#[derive(Debug,Clone)]
pub enum Op {
    Add,
    Sub,
    Div,
    Mul,
    Exp,
    UnNeg
}

impl Op {
    fn from_str(op: &str) -> Option<Op> {
        match op {
            "+"  => Some(Op::Add),
            "-"  => Some(Op::Sub),
            "/"  => Some(Op::Div),
            "*"  => Some(Op::Mul),
            "**" => Some(Op::Exp),
            _    => None,
        }
    }

    fn precedence(&self) -> u8 {
        match *self {
            Op::Add   => 0,
            Op::Sub   => 0,
            Op::Div   => 1,
            Op::Mul   => 1,
            Op::Exp   => 2,
            Op::UnNeg => 3,
        }
    }

    fn apply_binary(&self, a: f64, b: f64) -> Result<f64> {
        match *self {
            Op::Add   => Ok(a + b),
            Op::Sub   => Ok(a - b),
            Op::Div   => Ok(a / b),
            Op::Mul   => Ok(a * b),
            Op::Exp   => Ok(num::pow(a, b as usize)),
            Op::UnNeg => Err("Not a binary operation".to_string()),
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
    Num(f64),
    Op(Op),
    Var(String),
    RParen,
    LParen
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
    if found { Some((Tok::Num(number), &stream[i..n])) } else { None }
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
                .map(|v| (Tok::Op(v), &stream[i..n]))
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
            '(' => Some(Tok::LParen),
            ')' => Some(Tok::RParen),
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
        Some((Tok::Var(var), &stream[i..n]))
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
    stack.push(Tok::LParen);
    tokens.push(Tok::RParen);
    
    for token in &tokens {
        match *token {
            Tok::Num(n) => post.push(token.clone()),
            Tok::Op(ref op) => {
                while !stack.is_empty() {
                    if stack.last().map_or(false, |t| -> bool {
                        if let Tok::Op(ref pp) = *t {
                            pp.precedence() > op.precedence()
                        } else {
                            false
                        }
                    }) { post.push(stack.pop().unwrap()); }
                    else { break; }
                }
                stack.push(token.clone());
            },
            Tok::LParen => {
                stack.push(token.clone());
            },
            Tok::RParen => {
                loop {
                    let top = stack.pop();
                    if top.is_none() {
                        return Err("Syntax error".to_string());
                    }
                    if let Some(Tok::LParen) = top {
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
            Tok::Num(n) => stack.push(n),
            Tok::Op(ref op) => {
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
        let expected = vec![Tok::Num(1f64),
                            Tok::Op(Op::Add),
                            Tok::Num(2f64),
                            Tok::Op(Op::Sub),
                            Tok::Num(5f64),
                            Tok::Op(Op::Add),
                            Tok::LParen,
                            Tok::Num(7f64),
                            Tok::Op(Op::Add),
                            Tok::Num(8f64),
                            Tok::RParen];

        assert_eq!(toks, expected);
    }


}
