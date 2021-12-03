mod tokenizer;
mod parser;

use crate::{
    tokenizer::Buffer,
    parser::{
        Object,
        Func,
        CallInfo,
        parse_expr
    }
};

use std::{
    collections::HashMap,
    error::Error,
    thread,
    fs::File,
    io::Read,
    path::Path,
    io::{
        stdin,
        stdout,
        Write
    },
};

/// Define builtin functions with Sputter prototype syntax
macro_rules! gen_builtin {
    ($names:ident, $(($name:ident $($params:ident)*))*) => {
        $($names.insert(
            stringify!($name).to_owned(),
            Object::Function(Func {
                name: stringify!($name).to_owned(),
                addr: 0,
                params: vec![$(stringify!($params).to_owned()),*]
            })
        ));*
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    let child = thread::Builder::new().stack_size(32 * 1024 * 1024).spawn(|| {
        let mut names = HashMap::<String, Object>::new();
        let mut call_stack: Vec<CallInfo> = Vec::new();
        let mut scope_stack: Vec<Vec<String>> = Vec::new();

        gen_builtin!(names,
            (print content)
            (println content)
            (format string object)
            (exit code)
            (get ls idx)
            (len ls)
        );

        let mut args = std::env::args();
        if args.len() == 2 {
            let mut buf = Buffer::new(read(args.nth(1).unwrap().as_str()).unwrap()).unwrap();

            while buf.index < buf.len {
                parse_expr(&mut buf, &mut names, &mut call_stack, &mut scope_stack).unwrap();
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
                    res = parse_expr(&mut buf, &mut names, &mut call_stack, &mut scope_stack).unwrap();
                }

                println!("=> {:?}", res);
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