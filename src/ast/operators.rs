use crate::ast::Ast;

#[derive(Debug)]
pub enum Operator {
    IncPtr,
    DecPtr,
    Inc,
    Dec,
    Out,
    In,
    Loop(Ast),
}
