mod tokenizer;
mod parser;

use crate::{
    tokenizer::{
        Buffer,
        Type,
        get_tok
    },
    parser::{
        Object,
        Func,
        CallInfo,
        Param,
        match_expr
    }
};

use std::{
    collections::HashMap,
    error::Error,
    thread::Builder,
    fs::File,
    io::Read,
    path::Path,
    io::{
        stdin,
        stdout,
        Write
    }
};

/// Define builtin functions with Sputter prototype syntax
macro_rules! gen_builtin {
    ($names:ident { $(($name:ident $($params:ident: $types:ident)*))*}) => {
        $($names.insert(
            stringify!($name).to_owned(),
            Object::Function(Func {
                name: stringify!($name).to_owned(),
                addr: 0,
                params: vec![$(Param {
                    name: stringify!($params).to_owned(),
                    arg_type: Type::$types
                }),*]
            })
        ));*
    };
}

const STACK_SIZE: usize = 32 * 1024 * 1024;

fn main() -> Result<(), Box<dyn Error>> {
    let child = Builder::new().name("sputter".to_owned()).stack_size(STACK_SIZE).spawn(|| {
        let mut names = HashMap::<String, Object>::new();
        let mut call_stack = Vec::<CallInfo>::new();
        let mut scope_stack = Vec::<Vec<String>>::new();

        gen_builtin!(names {
            (print content: Any)
            (println content: Any)
            (format format_str: Str object: Any)
            (exit code: Int)
            (get ls: Any idx: Int)
            (len ls: Any)
            (range start: Int end: Int)
        });

        let mut args = std::env::args();
        if args.len() == 2 {
            let mut buf = Buffer::new(read(args.nth(1).unwrap().as_str()).unwrap()).unwrap();

            while buf.index < buf.len {
                let tok = get_tok(&mut buf).unwrap();
                match_expr(&mut buf, &mut names, &mut call_stack, &mut scope_stack, tok).unwrap();
            }
        }
        else if args.len() == 1 {
            let stdin = stdin();
            let mut buf = Buffer::new_empty();

            loop {
                stdout().write(b">>> ").unwrap();
                stdout().flush().unwrap();

                buf.add_line(&stdin).unwrap();

                let mut res = Object::None;
                while buf.index < buf.len {
                    let tok = get_tok(&mut buf).unwrap();
                    res = match_expr(&mut buf, &mut names, &mut call_stack, &mut scope_stack, tok).unwrap();
                }

                println!("\u{001b}[36m=> {:?}\u{001b}[0m", res);
            }
        }
    })?;

    child.join().unwrap();

    return Ok(());
}

fn read(filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let path = Path::new(filename);
    
    let mut file = File::open(&path)?;
  
    let mut v = Vec::<u8>::new();
    file.read_to_end(&mut v)?;
    
    return Ok(v);
}
