use std::error::Error;

#[derive(Debug)]
pub enum Token {
    None,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
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
    Nop,
    Use
}

pub fn get_tok(buf: &mut Buffer) -> Result<Token, Box<dyn Error>> {
    let mut i = buf.index;
    let start: usize;
    let mut tok = Token::None;
    
    // Ignore leading whitespace
    while i < buf.len {
        if buf.bytes[i] != b' ' && buf.bytes[i] != b'\t' && buf.bytes[i] != b'\n' && buf.bytes[i] != b'\r' {
            break;
        }
        i += 1;
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
                "none" => Token::Nop,
                "use" => Token::Use,
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
            len
        })
    }

    pub fn new_empty() -> Buffer {
        Buffer {
            bytes: Vec::new(),
            index: 0,
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

        self.len += bytes.len();
        self.bytes.extend(bytes);

        return Ok(());
    }

    pub fn splice(&mut self, slice: &[u8]) {
        self.bytes.splice(self.index..self.index, slice.iter().cloned());
        self.len += slice.len();
    }
}
