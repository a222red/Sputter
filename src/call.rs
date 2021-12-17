use crate::{
    tokenizer::{
        Buffer,
        get_tok
    },
    object::{
        Type,
        Object
    },
    parser::{
        match_expr,
        output
    }
};

use std::{
    error::Error,
    collections::HashMap,
    convert::TryInto,
    io::{
        stdin,
        stdout,
        Write
    },
    process::exit
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Param {
    pub name: String,
    pub arg_type: Type
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Arg {
    pub val: Object,
    pub arg_type: Type
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Func {
    pub name: String,
    pub addr: usize,
    pub params: Vec<Param>
}

pub struct CallInfo {
    pub old_addr: usize,
    pub params: Vec<Param>,
    pub args: Vec<Arg>
}

pub fn call_function<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, func: &Func, args: &Vec<Arg>) -> Result<Object, Box<dyn Error>> {
    Ok(match func.name.as_str() {
        "print" => {
            let mut out = String::new();

            output::repr_object(&args[0].val, &mut out);
            print!("{}", out);
            stdout().flush()?;

            Object::None
        },
        "println" => {
            let mut out = String::new();

            output::repr_object(&args[0].val, &mut out);
            println!("{}", out);

            Object::None
        },
        "readln" => {
            let mut s = String::new();
            stdin().read_line(&mut s)?;

            Object::Str(s)
        }
        "format" => {
            let mut out = String::new();

            if let Object::Str(s) = &args[0].val {
                for c in s.chars() {
                    if c == '%' {
                        let mut buf = String::new();

                        output::repr_object(&args[1].val, &mut buf);
                        out.push_str(buf.as_str());
                    }
                    else {out.push(c);}
                }
            }
            else {output::error(buf, format!("Expected string, got `{:?}`", args[0]))?}
            
            Object::Str(out)
        },
        "exit" => {
            exit(match args[0].val {
                Object::Int(i) => i.try_into().unwrap(),
                _ => {output::error(buf, format!("Expected int, got `{:?}`", args[0]))?; 0}
            })
        },
        "get" => {
            let ls = match &args[0].val {
                Object::List(ls) => ls.clone(),
                _ => {output::error(buf, format!("Expected list, got `{:?}`", args[0]))?; vec![]}
            };
            let idx = match args[1].val {
                Object::Int(i) => {
                    if i < 0 {
                        ls.len() - (0 - i) as usize
                    }
                    else {i as usize}
                },
                _ => {output::error(buf, format!("Expected int, got `{:?}`", args[1]))?; 0}
            };

            if idx >= ls.len() {
                output::error(buf, format!("Index is {} but the length of {:?} is {}", idx, ls, ls.len()))?;
            }
            ls[idx].clone()
        },
        "len" => {
            match &args[0].val {
                Object::List(ls) => Object::Int(ls.len().try_into()?),
                Object::Str(s) => Object::Int(s.len().try_into()?),
                _ => {output::error(buf, format!("Expected list or string, got `{:?}`", args[0]))?; Object::None}
            }
        },
        "range" => {
            let start = match args[0].val {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", args[0]))?; 0}
            };
            let end = match args[1].val {
                Object::Int(i) => i,
                _ => {output::error(buf, format!("Expected int, got `{:?}`", args[1]))?; 0}
            };

            let mut res = Vec::<Object>::new();
            for i in start..end {res.push(Object::Int(i));}

            Object::List(res)
        },
        _ => {
            call_stack.push(CallInfo {
                old_addr: buf.index,
                params: func.params.clone(),
                args: args.clone()
            });
            
            let mut temp_names = names.clone();
            for scope in scope_stack.clone() {
                for item in scope {
                    temp_names.remove(&item);
                }
            }
        
            buf.index = func.addr;
            let tok = get_tok(buf)?;
            let res = match_expr(buf, &mut temp_names, call_stack, scope_stack, tok)?;
            
            buf.index = call_stack.pop().unwrap().old_addr;
            res
        }
    })
}
