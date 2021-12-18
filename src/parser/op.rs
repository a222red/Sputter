use super::*;

use crate::{
    tokenizer::{
        Buffer,
        get_tok
    }
};

pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
    Or,
    And
}

pub fn parse_op_expr<'a>(buf: &'a mut Buffer, names: &mut HashMap<String, Object>, call_stack: &mut Vec<CallInfo>, scope_stack: &mut Vec<Vec<String>>, op: Op) -> Result<Object, Box<dyn Error>> {
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
        Op::Or => Object::Bool({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Bool(b) => b,
                _ => {output::error(buf, format!("Expected bool, got `{:?}`", lhs_obj))?; false}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Bool(b) => b,
                _ => {output::error(buf, format!("Expected bool, got `{:?}`", rhs_obj))?; false}
            };

            lhs || rhs
        }),
        Op::And => Object::Bool({
            tok = get_tok(buf)?;
            let lhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let lhs = match lhs_obj {
                Object::Bool(b) => b,
                _ => {output::error(buf, format!("Expected bool, got `{:?}`", lhs_obj))?; false}
            };

            tok = get_tok(buf)?;
            let rhs_obj = match_expr(buf, names, call_stack, scope_stack, tok)?;
            let rhs = match rhs_obj {
                Object::Bool(b) => b,
                _ => {output::error(buf, format!("Expected bool, got `{:?}`", rhs_obj))?; false}
            };

            lhs && rhs
        })
    })
}
