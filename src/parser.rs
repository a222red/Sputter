use crate::tokenizer::*;

use std::{
    collections::HashMap,
    error::Error,
    process::exit,
    convert::TryInto,
    fs::read
};

enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Func {
    pub name: String,
    pub addr: usize,
    pub params: Vec<String>
}

pub struct CallInfo {
    old_addr: usize,
    params: Vec<String>,
    args: Vec<Object>
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Object {
    Function(Func),
    Int(i32),
    Bool(bool),
    Str(String),
    List(Vec<Object>),
    None
}

fn error(buf: &Buffer, msg: String) -> Result<(), Box<dyn Error>> {
    let line: String;
    let mut line_start = 0usize;
    let mut line_end = buf.len;
    let mut line_num = 1;
    let mut carat = String::new();
    
    for i in 0..buf.len {
        if buf.bytes[i] == b'\n' {
            line_start = i + 1;
            line_num += 1;

            if i >= buf.index {
                line_end = i - 1;
            };
        }
    }

    line = String::from_utf8(buf.bytes[line_start..line_end].to_vec())?;
    for i in 0..line.len() {
        if i == buf.index - line_start - 1 {
            carat.push('^');
            break;
        }
        carat.push(' ');
    }

    println!(
        "\u{001b}[31mError at line {}\u{001b}[0m: {}\n{}\n\u{001b}[31m{}\u{001b}[0m",
        line_num,
        msg,
        line,
        carat
    );

    exit(1);
}

pub fn match_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, tok: Token) -> Result<Object, Box<dyn Error>> {
    Ok(match tok {
        Token::Name(n) => parse_single_name_expr(buf, names, call_stack, n)?,
        Token::Num(n) => Object::Int(n.parse()?),
        Token::Str(s) => Object::Str(s),
        Token::True => Object::Bool(true),
        Token::False => Object::Bool(false),
        Token::None => Object::None,
        Token::LParen => parse_paren_expr(buf, names, call_stack, scope_stack)?,
        Token::LBracket => parse_list_expr(buf, names, call_stack, scope_stack)?,
        _ => {error(buf, format!("Expected expression, got token {:?}", tok))?; Object::None}
    })
}

fn parse_paren_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>) -> Result<Object, Box<dyn Error>> {
    let tok = get_tok(buf)?;

    let res = match tok {
        Token::Name(n) => parse_name_expr(buf, names, call_stack, scope_stack, n)?,
        Token::Num(n) => Object::Int(n.parse()?),
        Token::Str(s) => Object::Str(s),
        Token::True => Object::Bool(true),
        Token::False => Object::Bool(false),
        Token::None => Object::None,
        Token::Def => parse_def_expr(buf, names)?,
        Token::If => parse_if_expr(buf, names, call_stack, scope_stack)?,
        Token::Lambda => parse_lambda_expr(buf)?,
        Token::Let => parse_let_expr(buf, names, call_stack, scope_stack)?,
        Token::LParen => {
            let temp = parse_paren_expr(buf, names, call_stack, scope_stack)?;
            match temp {
                Object::Function(f) => parse_call_expr(buf, names, call_stack, scope_stack, &f)?,
                _ => temp
            }
        },
        Token::LBracket => parse_list_expr(buf, names, call_stack, scope_stack)?,
        Token::Op(o) => parse_op_expr(buf, names, call_stack, scope_stack, match o.as_str() {
            "+" => Op::Add,
            "-" => Op::Sub,
            "*" => Op::Mul,
            "/" => Op::Div,
            "=" => Op::Eq,
            "<" => Op::Lt,
            ">" => Op::Gt,
            _ => {error(buf, format!("Invalid operator: {}", o))?; Op::Add}
        })?,
        Token::Use => {
            let tok = get_tok(buf)?;

            let slice = {
                if let Token::Str(filename) = tok {
                    read(filename.as_str())?
                }
                else {error(buf, format!("Expected Str, got {:?}", tok))?; Vec::new()}
            };
            
            let tok = get_tok(buf)?;

            match tok {
                Token::RParen => (),
                _ => error(buf, format!("Expected token RParen, got token {:?}", tok))?
            };

            buf.splice(&slice);

            return Ok(Object::None);
        },
        _ => {error(buf, format!("Expected token LParen, Name or Op, got token {:?}", tok))?; Object::None}
    };

    let tok = get_tok(buf)?;

    match tok {
        Token::RParen => (),
        _ => error(buf, format!("Expected token RParen, got token {:?}", tok))?
    };

    return Ok(res);
}

fn parse_list_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>) -> Result<Object, Box<dyn Error>> {
    let mut ls = Vec::<Object>::new();

    loop {
        let tok = get_tok(buf)?;
        ls.push(match tok {
            Token::RBracket => {break;},
            _ => match_expr(buf, names, call_stack, scope_stack, tok)?
        });
    }

    return Ok(Object::List(ls));
}

fn parse_single_name_expr<'a>(buf: &Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, name: String) -> Result<Object, Box<dyn Error>> {
    let params = match call_stack.len() > 0 {
        true => call_stack[call_stack.len() - 1].params.clone(),
        false => Vec::new()
    };
    let args = match call_stack.len() > 0 {
        true => call_stack[call_stack.len() - 1].args.clone(),
        false => Vec::new()
    };

    let n: Object;
    if params.contains(&name) {
        n = args[
            params.iter().position(|r| *r == name).unwrap()
        ].clone();
    }
    else {
        if names.contains_key(&name) == false {error(buf, format!("Undefined name: `{}`", name))?}
        n = names[&name].clone();
    }

    return Ok(n);
}

fn parse_name_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, name: String) -> Result<Object, Box<dyn Error>> {
    let params = match call_stack.len() > 0 {
        true => call_stack[call_stack.len() - 1].params.clone(),
        false => Vec::new()
    };
    let args = match call_stack.len() > 0 {
        true => call_stack[call_stack.len() - 1].args.clone(),
        false => Vec::new()
    };

    let n: Object;
    if params.contains(&name) {
        n = args[
            params.iter().position(|r| *r == name).unwrap()
        ].clone();
    }
    else {
        if names.contains_key(&name) == false {error(buf, format!("Undefined name: `{}`", name))?}
        n = names[&name].clone();
    }

    return Ok(match n {
        Object::Function(func) => parse_call_expr(buf, names, call_stack, scope_stack, &func)?,
        _ => n
    });
}

fn parse_def_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>) -> Result<Object, Box<dyn Error>> {
    let mut tok = get_tok(buf)?;
    match tok {
        Token::LParen => (),
        _ => error(buf, format!("Expected LParen, got {:?}", tok))?
    }
    
    tok = get_tok(buf)?;
    let name = match tok {
        Token::Name(s) => s,
        _ => {error(buf, format!("Expected Name, got {:?}", tok))?; String::new()}
    };

    let mut params = Vec::<String>::new();
    loop {
        tok = get_tok(buf)?;
        params.push(match tok {
            Token::Name(s) => s,
            _ => break
        });
    }

    match tok {
        Token::RParen => (),
        _ => error(buf, format!("Expected RParen, got {:?}", tok))?
    }

    let addr = buf.index;

    let mut lparens: usize = 0;

    tok = get_tok(buf)?;
    match tok {
        Token::LParen => lparens += 1,
        Token::RParen => lparens -= 1,
        _ => ()
    }

    while lparens > 0 {
        tok = get_tok(buf)?;
        match tok {
            Token::LParen => lparens += 1,
            Token::RParen => lparens -= 1,
            _ => ()
        }
    }

    names.insert(name.clone(), Object::Function(Func {
        addr,
        name: name.clone(),
        params
    }));

    return Ok(Object::None);
}

fn parse_if_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>) -> Result<Object, Box<dyn Error>> {
    let mut res = Object::None;

    let mut tok = get_tok(buf)?;

    let t = match_expr(buf, names, call_stack, scope_stack, tok)?;
    let cond = match t {
        Object::Bool(b) => b,
        _ => {error(buf, format!("Conditional expression must have type Bool, not {:?}", t))?; false}
    };
    
    if !cond {
        let mut lparens: usize = 0;

        tok = get_tok(buf)?;
        match tok {
            Token::LParen => lparens += 1,
            Token::RParen => lparens -= 1,
            _ => ()
        }

        while lparens > 0 {
            tok = get_tok(buf)?;
            match tok {
                Token::LParen => lparens += 1,
                Token::RParen => lparens -= 1,
                _ => ()
            }
        }
    }
    else {
        tok = get_tok(buf)?;
        res = match_expr(buf, names, call_stack, scope_stack, tok)?;
    }

    tok = get_tok(buf)?;
    match tok {
        Token::Else => (),
        _ => error(buf, format!("Expected Else after If expression, got {:?}", tok))?
    }

    if cond {
        let mut lparens: usize = 0;

        tok = get_tok(buf)?;
        match tok {
            Token::LParen => lparens += 1,
            Token::RParen => lparens -= 1,
            _ => ()
        }

        while lparens > 0 {
            tok = get_tok(buf)?;
            match tok {
                Token::LParen => lparens += 1,
                Token::RParen => lparens -= 1,
                _ => ()
            }
        }
    }
    else {
        tok = get_tok(buf)?;
        res = match_expr(buf, names, call_stack, scope_stack, tok)?;
    }

    return Ok(res);
}

fn parse_lambda_expr<'a>(buf: &'a mut Buffer) -> Result<Object, Box<dyn Error>> {
    let mut tok = get_tok(buf)?;
    match tok {
        Token::LParen => (),
        _ => error(buf, format!("Expected LParen, got {:?}", tok))?
    }

    let mut params: Vec<String> = Vec::new();
    loop {
        tok = get_tok(buf)?;
        params.push(match tok {
            Token::Name(s) => s,
            _ => break
        });
    }

    match tok {
        Token::RParen => (),
        _ => error(buf, format!("Expected RParen, got {:?}", tok))?
    }

    let addr = buf.index;

    let mut lparens: usize = 0;

    tok = get_tok(buf)?;
    match tok {
        Token::LParen => lparens += 1,
        Token::RParen => lparens -= 1,
        _ => ()
    }

    while lparens > 0 {
        tok = get_tok(buf)?;
        match tok {
            Token::LParen => lparens += 1,
            Token::RParen => lparens -= 1,
            _ => ()
        }
    }

    return Ok(Object::Function(Func {
        addr,
        name: "lambda".to_owned(),
        params
    }));
}

fn parse_let_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>) -> Result<Object, Box<dyn Error>> {
    let mut tok = get_tok(buf)?;
    match tok {
        Token::LParen => (),
        _ => error(buf, format!("Expected LParen, got {:?}", tok))?
    }

    scope_stack.push(vec![]);
    let len = scope_stack.len();

    loop {
        tok = get_tok(buf)?;
        match tok {
            Token::LParen => (),
            _ => break
        }

        tok = get_tok(buf)?;
        let name = match tok {
            Token::Name(s) => s,
            _ => {error(buf, format!("Expected Name, got {:?}", tok))?; String::new()}
        };
        tok = get_tok(buf)?;
        let val = match_expr(buf, names, call_stack, scope_stack, tok)?;
        
        if names.contains_key(&name) {error(buf, format!("Name `{}` already exists", name))?}
        scope_stack[len - 1].push(name.clone());
        names.insert(name, val);

        tok = get_tok(buf)?;
        match tok {
            Token::RParen => (),
            _ => error(buf, format!("Expected RParen, got {:?}", tok))?
        }
    }

    match tok {
        Token::RParen => (),
        _ => error(buf, format!("Expected RParen, got {:?}", tok))?
    }

    tok = get_tok(buf)?;
    let res = match_expr(buf, names, call_stack, scope_stack, tok)?;

    for name in &scope_stack[len - 1] {
        names.remove(name);
    }
    scope_stack.pop().unwrap();

    return Ok(res);
}

fn parse_call_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, func: &Func) -> Result<Object, Box<dyn Error>> {
    let mut args: Vec<Object> = Vec::new();

    for i in 0..func.params.len() {
        let tok = get_tok(buf)?;
        if let Token::RParen = tok {
            error(buf, format!(
                "Function `{}` takes {} arguments, got {}",
                func.name,
                func.params.len(),
                i
            ))?
        }

        let arg = match_expr(buf, names, call_stack, scope_stack, tok)?;

        args.push(arg);
    }
    
    let res = match func.name.as_str() {
        "print" => {
            let mut out = String::new();

            print_object_rec(&args[0], &mut out);
            print!("{}", out);

            Object::None
        },
        "println" => {
            let mut out = String::new();

            print_object_rec(&args[0], &mut out);
            println!("{}", out);

            Object::None
        },
        "format" => {
            let mut out = String::new();

            if let Object::Str(s) = &args[0] {
                for c in s.chars() {
                    if c == '%' {
                        let mut buf = String::new();

                        print_object_rec(&args[1], &mut buf);
                        out.push_str(buf.as_str());
                    }
                    else {out.push(c);}
                }
            }
            else {error(buf, format!("Expected Str, got {:?}", args[0]))?}
            
            Object::Str(out)
        },
        "exit" => {
            exit(match args[0] {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", args[0]))?; 0}
            })
        },
        "get" => {
            match &args[0] {
                Object::List(ls) => ls[match args[1] {
                    Object::Int(i) => {
                        if i < 0 {
                            ls.len() - (0 - i) as usize
                        }
                        else {i as usize}
                    },
                    _ => {error(buf, format!("Expected Int, got {:?}", args[1]))?; 0}
                }].clone(),
                _ => {error(buf, format!("Expected List, got {:?}", args[0]))?; Object::None}
            }
        },
        "len" => {
            match &args[0] {
                Object::List(ls) => Object::Int(ls.len().try_into()?),
                _ => {error(buf, format!("Expected List, got {:?}", args[0]))?; Object::None}
            }
        },
        "range" => {
            let start = match args[0] {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", args[0]))?; 0}
            };
            let end = match args[1] {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", args[1]))?; 0}
            };

            let mut res = Vec::<Object>::new();

            for i in start..end {
                res.push(Object::Int(i));
            }

            Object::List(res)
        },
        _ => {
            call_stack.push(CallInfo {
                old_addr: buf.index,
                params: func.params.clone(),
                args
            });
            call_function(buf, names, call_stack, scope_stack, func.addr)?
        }
    };

    return Ok(res);
}

fn print_object_rec(obj: &Object, buf: &mut String) {
    match obj {
        Object::Int(i) => buf.push_str(i.to_string().as_str()),
        Object::Str(s) => buf.push_str(s.as_str()),
        Object::Bool(b) => buf.push_str(format!("{}", b).as_str()),
        Object::Function(f) => buf.push_str(format!("{:?}", f).as_str()),
        Object::List(l) => {
            buf.push_str("[");
            for item in l {
                print_object_rec(item, buf);
                buf.push_str(" ");
            }
            buf.pop();
            buf.push_str("]");
        },
        Object::None => buf.push_str("none")
    }
}

fn call_function<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, addr: usize) -> Result<Object, Box<dyn Error>> {
    let mut temp_names = names.clone();
    for scope in scope_stack.clone() {
        for item in scope {
            temp_names.remove(&item);
        }
    }

    buf.index = addr;
    let tok = get_tok(buf)?;
    let res = match_expr(buf, &mut temp_names, call_stack, scope_stack, tok)?;
    
    buf.index = call_stack.pop().unwrap().old_addr;
    return Ok(res);
}

fn parse_op_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, op: Op) -> Result<Object, Box<dyn Error>> {
    let mut tok: Token;
    
    Ok(match op {
        Op::Add => Object::Int({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", rhs_obj))?; 0}
            };

            lhs + rhs
        }),
        Op::Sub => Object::Int({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", rhs_obj))?; 0}
            };

            lhs - rhs
        }),
        Op::Mul => Object::Int({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", rhs_obj))?; 0}
            };

            lhs * rhs
        }),
        Op::Div => Object::Int({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", rhs_obj))?; 0}
            };

            if rhs == 0 {error(buf, "Cannot divide by 0".to_owned())?}
            lhs / rhs
        }),
        Op::Eq => Object::Bool({
            tok = get_tok(buf)?;
            let lhs = match_expr(buf, names, call_stack, scope_stack, tok)?;
            tok = get_tok(buf)?;
            let rhs = match_expr(buf, names, call_stack, scope_stack, tok)?;

            lhs == rhs
        }),
        Op::Lt => Object::Bool({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", rhs_obj))?; 0}
            };

            lhs < rhs
        }),
        Op::Gt => Object::Bool({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {error(buf, format!("Expected Int, got {:?}", rhs_obj))?; 0}
            };

            lhs > rhs
        }),
    })
}
