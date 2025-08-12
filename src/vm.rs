use crate::parser::Statement;
use std::ops::Neg;

const MEMORY_CELLS: usize = 30000;

pub struct VM {
    i_ptr: usize,
    mem_ptr: usize,
    memory: [u8; MEMORY_CELLS],
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Increment(i32),
    Shift(i32),
    Print,
    JumpIfZero(usize),
    JumpIfNotZero(usize),
}

enum State {
    Initial,
    Increment(i32),
    Shift(i32),
}

impl Default for VM {
    fn default() -> Self {
        Self {
            i_ptr: 0,
            mem_ptr: 0,
            memory: [0; MEMORY_CELLS],
        }
    }
}

impl VM {
    pub fn eval(&mut self, expr: &[Instruction]) {
        while let Some(instruction) = expr.get(self.i_ptr) {
            let cell = &mut self.memory[self.mem_ptr];

            match instruction {
                Instruction::Increment(n) => {
                    // TODO: Handle memory cell becoming negative
                    *cell = (*cell).saturating_add_signed(*n as i8);
                }
                Instruction::Shift(n) => {
                    let target: usize = if *n > 0 {
                        // TODO: handle going past the edge
                        self.mem_ptr.saturating_add(*n as usize)
                    } else {
                        // TODO: handle going below zero
                        self.mem_ptr.saturating_sub(n.neg().try_into().unwrap())
                    };
                    self.mem_ptr = target;
                }
                Instruction::Print => {
                    print!("{}", *cell as char);
                }
                Instruction::JumpIfZero(target) if *cell == 0 => {
                    self.i_ptr = *target;
                    continue;
                    // These are matched, otherwise we'd have gotten a syntax error
                }
                Instruction::JumpIfNotZero(target) if *cell != 0 => {
                    self.i_ptr = *target;
                    continue;
                }
                _ => {}
            }

            self.i_ptr += 1;
        }
    }
}

pub fn compile(instructions: &[Statement]) -> Vec<Instruction> {
    let mut state: State = State::Initial;
    let mut bytecode: Vec<Instruction> = vec![];

    let flush_and_reset = |state: &mut State, bytecode: &mut Vec<Instruction>| {
        match state {
            State::Increment(n) if *n != 0 => {
                bytecode.push(Instruction::Increment(*n));
            }
            State::Shift(n) if *n != 0 => {
                bytecode.push(Instruction::Shift(*n));
            }
            _ => {}
        }
        *state = State::Initial;
    };

    for instruction in instructions {
        match (instruction, &state) {
            (Statement::Increment, State::Increment(n)) => state = State::Increment(n + 1),
            (Statement::Decrement, State::Increment(n)) => state = State::Increment(n - 1),
            (Statement::ShiftRight, State::Shift(n)) => state = State::Shift(n + 1),
            (Statement::ShiftLeft, State::Shift(n)) => state = State::Shift(n - 1),

            _ => {
                flush_and_reset(&mut state, &mut bytecode);

                match instruction {
                    Statement::ShiftRight => state = State::Shift(1),
                    Statement::ShiftLeft => state = State::Shift(-1),
                    Statement::Increment => state = State::Increment(1),
                    Statement::Decrement => state = State::Increment(-1),

                    Statement::Print => bytecode.push(Instruction::Print),
                    Statement::JumpIfZero(n) => bytecode.push(Instruction::JumpIfZero((*n).into())),
                    Statement::JumpIfNotZero(n) => {
                        bytecode.push(Instruction::JumpIfNotZero((*n).into()))
                    }
                    _ => todo!("{:?} not implemented", instruction),
                }
            }
        }
    }
    flush_and_reset(&mut state, &mut bytecode);

    resolve_jumps(&mut bytecode);
    // TODO: Add a "program return" instruction at the end of every program
    bytecode
}

fn resolve_jumps(bytecode: &mut [Instruction]) -> () {
    for i in 0..bytecode.len() {
        match bytecode[i] {
            Instruction::JumpIfZero(n) => {
                let (_before, after) = bytecode.split_at_mut(i);
                let offset = after
                    .iter()
                    .position(|i| i == &Instruction::JumpIfNotZero(n))
                    .expect("Expected to find matching jump");

                bytecode[i] = Instruction::JumpIfZero(offset + i);
                bytecode[offset + i] = Instruction::JumpIfNotZero(i);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_multiple_increments() {
        let got = compile(&vec![
            Statement::Increment,
            Statement::Increment,
            Statement::Increment,
        ]);

        assert_eq!(got, vec![Instruction::Increment(3)]);
    }

    #[test]
    fn net_multiple_increments() {
        let got = compile(&vec![
            Statement::Increment,
            Statement::Decrement,
            Statement::Increment,
        ]);

        assert_eq!(got, vec![Instruction::Increment(1)]);
    }

    #[test]
    fn ignore_noops() {
        let got = compile(&vec![Statement::Increment, Statement::Decrement]);
        assert_eq!(got, vec![]);
    }

    #[test]
    fn compile_multiple_shifts() {
        let got = compile(&vec![
            Statement::ShiftRight,
            Statement::ShiftLeft,
            Statement::ShiftRight,
        ]);

        assert_eq!(got, vec![Instruction::Shift(1)]);
    }

    #[test]
    fn compile_shifts_and_increments() {
        use Statement::*;

        let got = compile(&vec![Increment, ShiftRight, ShiftRight, Increment]);

        assert_eq!(
            got,
            vec![
                Instruction::Increment(1),
                Instruction::Shift(2),
                Instruction::Increment(1)
            ]
        );
    }

    #[test]
    fn compile_with_print() {
        let got = compile(&vec![Statement::Increment, Statement::Print]);

        assert_eq!(got, vec![Instruction::Increment(1), Instruction::Print]);
    }

    #[test]
    fn nested_jump_expressions() {
        let got = compile(&vec![
            Statement::Increment,
            Statement::JumpIfZero(0),
            Statement::JumpIfZero(1),
            Statement::ShiftLeft,
            Statement::JumpIfNotZero(1),
            Statement::Decrement,
            Statement::JumpIfNotZero(0),
        ]);

        assert_eq!(
            got,
            vec![
                Instruction::Increment(1),
                Instruction::JumpIfZero(6),
                Instruction::JumpIfZero(4),
                Instruction::Shift(-1),
                Instruction::JumpIfNotZero(2),
                Instruction::Increment(-1),
                Instruction::JumpIfNotZero(1)
            ]
        );
    }
}
