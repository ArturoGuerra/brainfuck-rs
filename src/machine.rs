use crate::ast::{Ast, Operator};
use crate::tape::Tape;
use std::io::{self, Read, Write};

#[derive(Default)]
pub struct Machine {
    pc: usize,
    tape: Tape,
}

impl Machine {
    pub fn run(&mut self, program: &Ast) {
        for op in program.inner() {
            match op {
                Operator::IncPtr => self.pc += 1,
                Operator::DecPtr => self.pc -= 1,
                Operator::Inc => *self.tape.get_mut(self.pc).unwrap() += 1,
                Operator::Dec => *self.tape.get_mut(self.pc).unwrap() -= 1,
                Operator::Out => io::stdout()
                    .write_all(&[*self.tape.get(self.pc).unwrap()])
                    .unwrap(),
                Operator::In => {
                    let mut buf = [0_u8, 1];
                    io::stdin().read_exact(&mut buf).unwrap();
                    *self.tape.get_mut(self.pc).unwrap() = buf[0];
                }
                Operator::Loop(program) => {
                    while *self.tape.get(self.pc).unwrap() != 0 {
                        self.run(program);
                    }
                }
            }
        }
    }
}
