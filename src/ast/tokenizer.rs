use std::io::{self, Read, Write};
use std::str::FromStr;
use std::{char, fmt};

#[derive(Debug)]
pub enum Token {
    IncPtr,
    DecPtr,
    Inc,
    Dec,
    Out,
    In,
    JmpFwd,
    JmpBck,
    Nop(u8),
}

impl From<u8> for Token {
    fn from(s: u8) -> Self {
        match s {
            b'>' => Token::IncPtr,
            b'<' => Token::DecPtr,
            b'+' => Token::Inc,
            b'-' => Token::Dec,
            b'.' => Token::Out,
            b',' => Token::In,
            b'[' => Token::JmpFwd,
            b']' => Token::JmpBck,
            _ => Token::Nop(s),
        }
    }
}

impl From<Token> for u8 {
    fn from(t: Token) -> Self {
        match t {
            Token::IncPtr => b'>',
            Token::DecPtr => b'<',
            Token::Inc => b'+',
            Token::Dec => b'-',
            Token::Out => b'.',
            Token::In => b',',
            Token::JmpFwd => b'[',
            Token::JmpBck => b']',
            Token::Nop(v) => v,
        }
    }
}

impl From<&Token> for u8 {
    fn from(t: &Token) -> Self {
        match t {
            Token::IncPtr => b'>',
            Token::DecPtr => b'<',
            Token::Inc => b'+',
            Token::Dec => b'-',
            Token::Out => b'.',
            Token::In => b',',
            Token::JmpFwd => b'[',
            Token::JmpBck => b']',
            Token::Nop(v) => *v,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c: u8 = self.into();
        write!(f, "{}", c as char)
    }
}

#[derive(Default)]
pub struct Tokenizer(Vec<Token>);

impl Tokenizer {
    fn new(tokens: Vec<Token>) -> Self {
        Self(tokens)
    }

    pub fn inner(self) -> Vec<Token> {
        self.0
    }
}

impl FromStr for Tokenizer {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tokenizer::new(
            s.bytes().map(|c| c.into()).collect::<Vec<Token>>(),
        ))
    }
}

impl From<String> for Tokenizer {
    fn from(s: String) -> Tokenizer {
        Tokenizer::new(
            s.into_bytes()
                .into_iter()
                .map(|c| c.into())
                .collect::<Vec<Token>>(),
        )
    }
}

impl From<&str> for Tokenizer {
    fn from(s: &str) -> Tokenizer {
        Tokenizer::new(s.bytes().map(|c| c.into()).collect::<Vec<Token>>())
    }
}

impl From<&[u8]> for Tokenizer {
    fn from(s: &[u8]) -> Tokenizer {
        Tokenizer::new(s.iter().map(|c| (*c).into()).collect::<Vec<Token>>())
    }
}

impl From<Vec<u8>> for Tokenizer {
    fn from(s: Vec<u8>) -> Tokenizer {
        Tokenizer::new(s.into_iter().map(Token::from).collect::<Vec<Token>>())
    }
}

impl Write for Tokenizer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .extend(buf.iter().map(|val| (*val).into()).collect::<Vec<Token>>());
        Ok(self.0.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.clear();
        Ok(())
    }
}

impl Read for Tokenizer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.0.len().min(buf.len());
        self.0
            .drain(..len)
            .zip(buf.iter_mut())
            .for_each(|(src, dst)| *dst = src.into());
        Ok(len)
    }
}
