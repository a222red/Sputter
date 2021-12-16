mod funcdef;
pub mod output;

use crate::{
    tokenizer::{
        Token,
        Buffer,
        get_tok
    },
    object::{
        Type,
        Object
    },
    call::{
        Arg,
        CallInfo,
        Func,
        call_function
    }
};

use std::{
    collections::HashMap,
    error::Error,
    process::exit,
    fs::read,
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

pub fn eat_expr<'a>(buf: &'a mut Buffer) -> Result<(), Box<dyn Error>> {
    let mut lparens = 0usize;
    let mut lbrackets = 0usize;

    let mut tok = get_tok(buf)?;
    match tok {
        Token::LParen => lparens += 1,
        Token::RParen => lparens -= 1,
        Token::LBracket => lbrackets += 1,
        Token::RBracket => lbrackets -= 1,
        _ => ()
    }

    while lparens > 0 || lbrackets > 0 {
        tok = get_tok(buf)?;
        match tok {
            Token::LParen => lparens += 1,
            Token::RParen => lparens -= 1,
            Token::LBracket => lbrackets += 1,
            Token::RBracket => lbrackets -= 1,
            _ => ()
        }
    }

    return Ok(());
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
        Token::Empty => Object::None,
        _ => {output::error(buf, format!("Expected expression, got `{:?}`", tok))?; Object::None}
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
        Token::Def => funcdef::parse_def_expr(buf, names)?,
        Token::If => parse_if_expr(buf, names, call_stack, scope_stack)?,
        Token::Lambda => funcdef::parse_lambda_expr(buf)?,
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
            _ => {output::error(buf, format!("Invalid operator: `{}`", o))?; Op::Add}
        })?,
        Token::Use => {
            let tok = get_tok(buf)?;

            let slice = {
                if let Token::Str(filename) = tok {read(filename.replace("~", &std::env::var("SPUTTER_INCLUDE")?))?}
                else {output::error(buf, format!("Expected string, got {:?}", tok))?; Vec::new()}
            };
            
            let tok = get_tok(buf)?;

            match tok {
                Token::RParen => (),
                _ => output::error(buf, format!("Expected `)`, got `{:?}`", tok))?
            };

            buf.splice(&slice);

            return Ok(Object::None);
        },
        _ => {output::error(buf, format!("Expected `(`, name or operator, got `{:?}`", tok))?; Object::None}
    };

    let tok = get_tok(buf)?;

    match tok {
        Token::RParen => (),
        _ => output::error(buf, format!("Expected `)`, got `{:?}`", tok))?
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
    let mut param_names = Vec::<String>::new();
    for p in &params {param_names.push(p.name.clone());}

    if param_names.contains(&name) {
        n = args[
            params.iter().position(|r| *r.name == name).unwrap()
        ].val.clone();
    }
    else {
        if names.contains_key(&name) == false {output::error(buf, format!("Undefined name: `{}`", name))?}
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
    let mut param_names = Vec::<String>::new();
    for p in &params {param_names.push(p.name.clone());}

    if param_names.contains(&name) {
        n = args[
            params.iter().position(|r| *r.name == name).unwrap()
        ].val.clone();
    }
    else {
        if names.contains_key(&name) == false {output::error(buf, format!("Undefined name: `{}`", name))?}
        n = names[&name].clone();
    }

    return Ok(match n {
        Object::Function(func) => parse_call_expr(buf, names, call_stack, scope_stack, &func)?,
        _ => n
    });
}

fn parse_if_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>) -> Result<Object, Box<dyn Error>> {
    let mut res = Object::None;

    let mut tok = get_tok(buf)?;

    let t = match_expr(buf, names, call_stack, scope_stack, tok)?;
    let cond = match t {
        Object::Bool(b) => b,
        _ => {output::error(buf, format!("Conditional expression must have type `bool`, not `{:?}`", t))?; false}
    };
    
    if !cond {eat_expr(buf)?}
    else {
        tok = get_tok(buf)?;
        res = match_expr(buf, names, call_stack, scope_stack, tok)?;
    }

    tok = get_tok(buf)?;
    match tok {
        Token::Else => (),
        _ => output::error(buf, format!("Expected `else` after `if` expression, got `{:?}`", tok))?
    }

    if cond {eat_expr(buf)?}
    else {
        tok = get_tok(buf)?;
        res = match_expr(buf, names, call_stack, scope_stack, tok)?;
    }

    return Ok(res);
}

fn parse_let_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>) -> Result<Object, Box<dyn Error>> {
    let mut tok = get_tok(buf)?;
    match tok {
        Token::LParen => (),
        _ => output::error(buf, format!("Expected `(`, got `{:?}`", tok))?
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
            _ => {output::error(buf, format!("Expected name, got `{:?}`", tok))?; String::new()}
        };
        tok = get_tok(buf)?;
        let val = match_expr(buf, names, call_stack, scope_stack, tok)?;
        
        if names.contains_key(&name) {output::error(buf, format!("Name `{}` already exists", name))?}
        scope_stack[len - 1].push(name.clone());
        names.insert(name, val);

        tok = get_tok(buf)?;
        match tok {
            Token::RParen => (),
            _ => output::error(buf, format!("Expected `)`, got `{:?}`", tok))?
        }
    }

    match tok {
        Token::RParen => (),
        _ => output::error(buf, format!("Expected `)`, got `{:?}`", tok))?
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
    let mut args = Vec::<Arg>::new();

    for i in 0..func.params.len() {
        let tok = get_tok(buf)?;
        if let Token::RParen = tok {
            output::error(buf, format!(
                "Function `{}` takes {} arguments, got {}",
                func.name,
                func.params.len(),
                i
            ))?
        }

        let val = match_expr(buf, names, call_stack, scope_stack, tok)?;
        let arg = Arg {
            val: val.clone(),
            arg_type: match val {
                Object::Function(_) => Type::Function,
                Object::Int(_) => Type::Int,
                Object::Bool(_) => Type::Bool,
                Object::Str(_) => Type::Str,
                Object::List(_) => Type::List,
                Object::None => Type::None
            }
        };
        
        if arg.arg_type == func.params[i].arg_type || func.params[i].arg_type == Type::Any {
            args.push(arg);
        }
        else {output::error(buf, format!(
            "Parameter `{}` of function `{}` expects type `{:?}`, got type `{:?}`",
            func.params[i].name,
            func.name,
            func.params[i].arg_type,
            arg.arg_type
        ))?}
    }
    
    return Ok(call_function(buf, names, call_stack, scope_stack, func, &args)?);
}

fn parse_op_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, op: Op) -> Result<Object, Box<dyn Error>> {
    let mut tok: Token;
    
    Ok(match op {
        Op::Add => {
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            match lhs_obj {
                Object::Int(i) => {
                    tok = get_tok(buf)?;

                    let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
                    let rhs = match rhs_obj {
                        Object::Int(i) => i,
                        _ => {output::error(buf, format!("Expected int, got `{:?}`", rhs_obj))?; 0}
                    };

                    Object::Int(i + rhs)
                },
                Object::Str(s) => {
                    tok = get_tok(buf)?;

                    let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
                    let rhs = match rhs_obj {
                        Object::Str(s) => s,
                        _ => {output::error(buf, format!("Expected int, got `{:?}`", rhs_obj))?; String::new()}
                    };
                    let mut out = s;
                    out.push_str(&rhs);
                    
                    Object::Str(out)
                }
                _ => {output::error(buf, format!("Expected int, got `{:?}`", lhs_obj))?; Object::None}
            }
        },
        Op::Sub => Object::Int({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", rhs_obj))?; 0}
            };

            lhs - rhs
        }),
        Op::Mul => Object::Int({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", rhs_obj))?; 0}
            };

            lhs * rhs
        }),
        Op::Div => Object::Int({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", rhs_obj))?; 0}
            };

            if rhs == 0 {output::error(buf, "Cannot divide by 0".to_owned())?}
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
                _ => {output::error(buf, format!("Expected int, got `{:?}`", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", rhs_obj))?; 0}
            };

            lhs < rhs
        }),
        Op::Gt => Object::Bool({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", lhs_obj))?; 0}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", rhs_obj))?; 0}
            };

            lhs > rhs
        }),
    })
}
