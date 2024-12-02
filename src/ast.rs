mod operators;
mod tokenizer;

pub use operators::Operator;
pub use tokenizer::{Token, Tokenizer};

#[derive(Debug)]
pub struct Ast(Vec<Operator>);

impl From<Tokenizer> for Ast {
    fn from(t: Tokenizer) -> Ast {
        Ast::load(&t.inner())
    }
}

impl Ast {
    pub fn inner(&self) -> &[Operator] {
        &self.0
    }

    fn load(tokens: &[Token]) -> Ast {
        let mut sp = 0;
        let mut stack = 0;
        let mut ops = Vec::new();

        for (pc, token) in tokens.iter().enumerate() {
            if stack == 0 {
                let ins = match token {
                    Token::IncPtr => Some(Operator::IncPtr),
                    Token::DecPtr => Some(Operator::DecPtr),
                    Token::Inc => Some(Operator::Inc),
                    Token::Dec => Some(Operator::Dec),
                    Token::In => Some(Operator::In),
                    Token::Out => Some(Operator::Out),
                    Token::JmpFwd => {
                        sp = pc;
                        stack += 1;
                        None
                    }
                    Token::JmpBck => panic!("found extra ]"),
                    Token::Nop(_) => None,
                };

                if let Some(ins) = ins {
                    ops.push(ins)
                };
            } else {
                match token {
                    Token::JmpFwd => {
                        stack += 1;
                    }
                    Token::JmpBck => {
                        stack -= 1;
                        if stack == 0 {
                            ops.push(Operator::Loop(Self::load(&tokens[sp + 1..pc])))
                        }
                    }
                    _ => {}
                }
            }
        }

        Ast(ops)
    }
}
