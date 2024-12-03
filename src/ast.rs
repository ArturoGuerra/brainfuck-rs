mod operators;
mod tokenizer;

pub use operators::Operator;
use pest::iterators::Pairs;
pub use tokenizer::{Token, Tokenizer};

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct BrainFuckParser;

#[derive(Debug)]
pub struct Ast(Vec<Operator>);

impl From<Tokenizer> for Ast {
    fn from(t: Tokenizer) -> Ast {
        Ast::parse_from_tokenizer(&t.inner())
    }
}

impl From<Pairs<'_, Rule>> for Ast {
    fn from(mut pairs: Pairs<Rule>) -> Ast {
        Ast::parse_from_pest(pairs.next().unwrap().into_inner())
    }
}

impl Ast {
    pub fn inner(&self) -> &[Operator] {
        &self.0
    }

    fn parse_from_pest(pairs: Pairs<Rule>) -> Ast {
        let mut ops = vec![];
        for pair in pairs {
            match pair.as_rule() {
                Rule::Command => ops.push(Operator::from(
                    pair.as_span().as_str().bytes().next().unwrap(),
                )),
                Rule::Loop => ops.push(Operator::Loop(Ast::parse_from_pest(pair.into_inner()))),
                Rule::Program => panic!("error we should never see a program rule"),
                Rule::EOI => break,
                _ => {}
            }
        }

        Ast(ops)
    }

    fn parse_from_tokenizer(tokens: &[Token]) -> Ast {
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
                            ops.push(Operator::Loop(Self::parse_from_tokenizer(
                                &tokens[sp + 1..pc],
                            )))
                        }
                    }
                    _ => {}
                }
            }
        }

        Ast(ops)
    }
}
