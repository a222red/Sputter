mod tokenizer;
mod object;
mod call;
mod parser;

use crate::{
    tokenizer::{
        Buffer,
        get_tok
    },
    object::{
        Type,
        Object
    },
    call::{
        Func,
        CallInfo,
        Param,
    },
    parser::match_expr
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

use clap::{
    Arg,
    App,
    crate_version,
    crate_authors,
    crate_description
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

/// Shorthand macro for spawning a thread
macro_rules! spawn_thread {
    ($name:literal, $stack_size:expr, $f:expr) => {
        Builder::new().name($name.to_owned()).stack_size($stack_size).spawn(move || $f)?
    };
}

/// Constant specifying the amount of stack space available to the execution thread
const DEFAULT_STACK_SIZE: usize = 32 * 1024 * 1024;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Sputter")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to use")
            .required(false)
            .index(1)
        )
        .arg(Arg::with_name("STACK_SIZE")
            .short("s")
            .long("stack-size")
            .value_name("SIZE")
            .help("Specify stack space for the execution thread (in megabytes)")
            .takes_value(true)
        )
        .arg(Arg::with_name("COLOR_OFF")
            .short("o")
            .long("color-off")
            .help("Don't use color in REPL output")
            .takes_value(false)
        )
        .get_matches();

    let child = spawn_thread!("Sputter", match matches.value_of("STACK_SIZE") {
        Some(s) => s.parse::<usize>().unwrap() * 1024 * 1024,
        None => DEFAULT_STACK_SIZE
    }, {
        let mut names = HashMap::<String, Object>::new();
        let mut call_stack = Vec::<CallInfo>::new();
        let mut scope_stack = Vec::<Vec<String>>::new();

        gen_builtin!(names {
            (print content: Any)
            (println content: Any)
            (readln)
            (format format_str: Str object: Any)
            (exit code: Int)
            (get ls: Any idx: Int)
            (len ls: Any)
            (range start: Int end: Int)
        });

        // Run file specified by command line arg
        if let Some(filename) = matches.value_of("INPUT") {
            let mut buf = Buffer::new(read(filename).unwrap()).unwrap();

            while buf.index < buf.len {
                let tok = get_tok(&mut buf).unwrap();
                match_expr(&mut buf, &mut names, &mut call_stack, &mut scope_stack, tok).unwrap();
            }
        }
        // REPL
        else {
            let stdin = stdin();
            let no_color = matches.is_present("COLOR_OFF");
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
                
                if !no_color {println!("\u{001b}[36m=> {:?}\u{001b}[0m", res);}
                else {println!("=> {:?}", res)}
            }
        }
    });

    child.join().unwrap();

    return Ok(());
}

/// Reads the contents of a file to an allocated buffer
fn read(filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let path = Path::new(filename);
    
    let mut file = File::open(&path)?;
  
    let mut v = Vec::<u8>::new();
    file.read_to_end(&mut v)?;
    
    return Ok(v);
}
