use std::{
    error::Error,
    fmt::{Debug, Formatter}
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Function,
    Int,
    Bool,
    Str,
    List,
    None,
    Any
}

impl Debug for Type {
    fn fmt(&self, form: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(form, "{}", match self {
            Type::Any => "any",
            Type::Function => "function",
            Type::Int => "int",
            Type::Bool => "bool",
            Type::Str => "string",
            Type::List => "list",
            Type::None => "none_t"
        })
    }
}

pub enum Token {
    Unknown(String),
    Empty,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    Name(String),
    Num(String),
    Str(String),
    Op(String),
    Def,
    Let,
    Lambda,
    If,
    Else,
    True,
    False,
    None,
    Use,
    Typename(Type)
}

impl Debug for Token {
    fn fmt(&self, form: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(form, "{}", match self {
            Token::Empty => "".to_owned(),
            Token::Unknown(s) => s.clone(),
            Token::LParen => "(".to_owned(),
            Token::RParen => ")".to_owned(),
            Token::LBrace => "{".to_owned(),
            Token::RBrace => "}".to_owned(),
            Token::LBracket => "[".to_owned(),
            Token::RBracket => "]".to_owned(),
            Token::Colon => ":".to_owned(),
            Token::Name(s) => s.clone(),
            Token::Num(s) => s.clone(),
            Token::Str(s) => format!("\"{}\"", s),
            Token::Op(s) => s.clone(),
            Token::Def => "def".to_owned(),
            Token::Let => "let".to_owned(),
            Token::Lambda => "lambda".to_owned(),
            Token::If => "if".to_owned(),
            Token::Else => "else".to_owned(),
            Token::True => "true".to_owned(),
            Token::False => "false".to_owned(),
            Token::None => "none".to_owned(),
            Token::Use => "use".to_owned(),
            Token::Typename(t) => (match t {
                Type::Function => "function",
                Type::Int => "int",
                Type::Bool => "bool",
                Type::Str => "string",
                Type::List => "list",
                Type::None => "none_t",
                Type::Any => "any"
            }).to_owned()
        })
    }
}

pub fn get_tok(buf: &mut Buffer) -> Result<Token, Box<dyn Error>> {
    buf.last_index = buf.index;
    let mut i = buf.index;
    let start: usize;
    let mut tok = Token::Empty;
    
    // Ignore leading whitespace
    while i < buf.len {
        if buf.bytes[i] != b' ' && buf.bytes[i] != b'\t' && buf.bytes[i] != b'\n' && buf.bytes[i] != b'\r' {
            break;
        }
        i += 1;
    }

    if i == buf.len {
        buf.index = i;
        return Ok(tok);
    }

    // Ignore comments
    while buf.bytes[i] == b';' {
        while i < buf.len {
            i += 1;
            if buf.bytes[i] == b'\n' {
                break;
            }
        }
        // Ignore whitespace after comment
        while i < buf.len {
            if buf.bytes[i] != b' ' && buf.bytes[i] != b'\t' && buf.bytes[i] != b'\n' && buf.bytes[i] != b'\r' {
                break;
            }
            i += 1;
        }
    }

    if i >= buf.len {
        buf.index = i;
        return Ok(tok);
    }
    start = i;

    if buf.bytes[i].is_ascii_digit() {
        while i < buf.len {
            if !buf.bytes[i].is_ascii_digit() {break;}
            i += 1;
        }

        tok = Token::Num(String::from_utf8(buf.bytes[start..i].to_vec())?);
    }
    else if buf.bytes[i] == b'"' {
        let mut s = String::new();
        i += 1;

        while i < buf.len {
            if buf.bytes[i] == b'"' {break;}
            if buf.bytes[i] == b'\\' {
                if buf.bytes[i + 1] == b'\\' {
                    s.push('\\');
                }
                if buf.bytes[i + 1] == b'%' {
                    s.push('%');
                }
                else if buf.bytes[i + 1] == b'n' {
                    s.push('\n');
                }
                
                i += 1;
            }
            else {s.push(buf.bytes[i] as char);}

            i += 1;
        }

        tok = Token::Str(s);
        i += 1;
    }
    else if buf.bytes[i] == b'(' {
        i += 1;
        tok = Token::LParen;
    }
    else if buf.bytes[i] == b')' {
        i += 1;
        tok = Token::RParen;
    }
    else if buf.bytes[i] == b'[' {
        i += 1;
        tok = Token::LBracket;
    }
    else if buf.bytes[i] == b']' {
        i += 1;
        tok = Token::RBracket;
    }
    else if buf.bytes[i] == b'{' {
        i += 1;
        tok = Token::LBrace;
    }
    else if buf.bytes[i] == b'}' {
        i += 1;
        tok = Token::RBrace;
    }
    else if buf.bytes[i] == b':' {
        i += 1;
        tok = Token::Colon;
    }
    else if buf.bytes[i] == b'+' || buf.bytes[i] == b'-' || buf.bytes[i] == b'*' || buf.bytes[i] == b'/' 
        || buf.bytes[i] == b'=' || buf.bytes[i] == b'<' || buf.bytes[i] == b'>'
    {
        i += 1;
        tok = Token::Op(String::from_utf8(buf.bytes[start..i].to_vec())?);
    }
    else if buf.bytes[i].is_ascii_alphabetic() || buf.bytes[i] == b'_' {
        while i < buf.len {
            if (buf.bytes[i].is_ascii_alphanumeric() || buf.bytes[i] == b'_') == false {break;}
            i += 1;
        }
        tok = Token::Name(String::from_utf8(buf.bytes[start..i].to_vec())?);
    }
    else {
        i += 1;
        tok = Token::Unknown(String::from_utf8(buf.bytes[start..i].to_vec())?);
    }

    tok = match tok {
        Token::Name(ref s) => {
            match s.as_str() {
                "def" => Token::Def,
                "let" => Token::Let,
                "lambda" => Token::Lambda,
                "if" => Token::If,
                "else" => Token::Else,
                "true" => Token::True,
                "false" => Token::False,
                "none" => Token::None,
                "use" => Token::Use,
                "function" => Token::Typename(Type::Function),
                "int" => Token::Typename(Type::Int),
                "bool" => Token::Typename(Type::Bool),
                "string" => Token::Typename(Type::Str),
                "list" => Token::Typename(Type::List),
                "none_t" => Token::Typename(Type::None),
                _ => tok
            }
        },
        _ => tok
    };

    buf.index = i;
    return Ok(tok);
}

pub struct Buffer {
    pub bytes: Vec<u8>,
    pub index: usize,
    pub last_index: usize,
    pub len: usize
}

impl Buffer {
    pub fn new(raw_buf: Vec<u8>) -> Result<Buffer, Box<dyn Error>> {
        let bytes = String::from_utf8(raw_buf)?
            .replace("\r", "")
            .trim_end()
            .as_bytes()
            .to_vec();
        let len = bytes.len();

        return Ok(Buffer {
            bytes,
            index: 0,
            last_index: 0,
            len
        })
    }

    pub fn new_empty() -> Buffer {
        Buffer {
            bytes: Vec::new(),
            index: 0,
            last_index: 0,
            len: 0
        }
    }

    pub fn add_line(&mut self, stdin: &std::io::Stdin) -> Result<(), Box<dyn Error>> {
        let mut io_buf = String::new();
        stdin.read_line(&mut io_buf)?;

        let bytes = io_buf
            .trim_end()
            .as_bytes()
            .to_vec();

        self.len += bytes.len() + 1;
        self.bytes.push(b'\n');
        self.bytes.extend(bytes);

        return Ok(());
    }

    pub fn splice(&mut self, slice: &[u8]) {
        self.bytes.push(b'\n');
        self.index += 1;
        self.len += 1;

        self.bytes.splice(self.index..self.index, slice.iter().cloned());
        self.len += slice.len();
    }
}
