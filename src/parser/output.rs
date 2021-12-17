use super::*;
use crate::object::Object;

pub fn repr_object(obj: &Object, buf: &mut String) {
    match obj {
        Object::Int(i) => buf.push_str(i.to_string().as_str()),
        Object::Str(s) => buf.push_str(s.as_str()),
        Object::Bool(b) => buf.push_str(format!("{}", b).as_str()),
        Object::Function(f) => buf.push_str(format!("{:?}", f).as_str()),
        Object::List(l) => {
            buf.push_str("[");
            for item in l {
                repr_object(item, buf);
                buf.push_str(" ");
            }
            buf.pop();
            buf.push_str("]");
        },
        Object::None => buf.push_str("none")
    }
}

pub fn error(buf: &Buffer, msg: String) -> Result<(), Box<dyn Error>> {
    let line: String;
    let mut line_start = 0usize;
    let mut line_end = buf.len;
    let mut line_num = 1;
    let mut carat = String::new();
    
    for i in 0..buf.len {
        if buf.bytes[i] == b'\n' {
            if i < buf.index {
                line_start = i + 1;
                line_num += 1;
            }

            if i >= buf.index && i > line_start {line_end = i};
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
