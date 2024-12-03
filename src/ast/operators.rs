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

impl From<u8> for Operator {
    fn from(s: u8) -> Self {
        match s {
            b'>' => Operator::IncPtr,
            b'<' => Operator::DecPtr,
            b'+' => Operator::Inc,
            b'-' => Operator::Dec,
            b'.' => Operator::Out,
            b',' => Operator::In,
            s => panic!("invalid type {}", s),
        }
    }
}
