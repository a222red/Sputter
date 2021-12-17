use super::*;
use crate::{
    object::{
        Type,
        Object
    },
    call::{
        Param,
        Func
    }
};

pub fn param_list<'a>(buf: &'a mut Buffer, tok: &mut Token) -> Result<Vec<Param>, Box<dyn Error>> {
    let mut params = Vec::<Param>::new();
    let mut idx: usize;
    loop {
        let name: String;
        let mut arg_type = Type::Any;

        *tok = get_tok(buf)?;
        if let Token::Name(s) = tok {name = s.clone()}
        else {break}

        idx = buf.index;
        *tok = get_tok(buf)?;

        match tok {
            Token::Colon => {
                *tok = get_tok(buf)?;

                if let Token::Typename(t) = tok {arg_type = t.clone()}
                else {output::error(buf, "Expected type after `:`".to_owned())?}
            },
            _ => buf.index = idx
        }

        params.push(Param {
            name,
            arg_type
        });
    }

    return Ok(params);
}

pub fn parse_def_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>) -> Result<Object, Box<dyn Error>> {
    let mut tok = get_tok(buf)?;
    match tok {
        Token::LParen => (),
        _ => output::error(buf, format!("Expected `(`, got `{:?}`", tok))?
    }
    
    tok = get_tok(buf)?;
    let name = match &tok {
        Token::Name(s) => s.clone(),
        _ => {output::error(buf, format!("Expected name, got `{:?}`", tok))?; panic!("")}
    };

    let params = param_list(buf, &mut tok)?;

    match tok {
        Token::RParen => (),
        _ => output::error(buf, format!("Expected `)`, got `{:?}`", tok))?
    }

    let addr = buf.index;

    eat_expr(buf)?;

    names.insert(name.clone(), Object::Function(Func {
        addr,
        name: name.clone(),
        params
    }));

    return Ok(Object::None);
}

pub fn parse_lambda_expr<'a>(buf: &'a mut Buffer) -> Result<Object, Box<dyn Error>> {
    let mut tok = get_tok(buf)?;
    match tok {
        Token::LParen => (),
        _ => output::error(buf, format!("Expected `(`, got `{:?}`", tok))?
    }

    let params = param_list(buf, &mut tok)?;

    match tok {
        Token::RParen => (),
        _ => output::error(buf, format!("Expected `)`, got `{:?}`", tok))?
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
