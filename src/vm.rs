use crate::parser::Statement;

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

enum CompilerState {
    Initial,
    Increment(i32),
    Shift(i32),
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    NegativeRegister,
    NoMoreCells,
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
            match self.eval_one(instruction) {
                Ok(next_instruction) => self.i_ptr = next_instruction,
                Err(err) => panic!("Runtime error: {:?}", err),
            }
        }
    }

    pub fn eval_one(&mut self, instruction: &Instruction) -> Result<usize, RuntimeError> {
        let cell = &mut self.memory[self.mem_ptr];

        match instruction {
            Instruction::Increment(n) => {
                if *n < 0 && n.abs() > (*cell).into() {
                    return Err(RuntimeError::NegativeRegister);
                }
                *cell = cell.saturating_add_signed(*n as i8);
            }
            Instruction::Shift(n) => {
                let target = if *n > 0 {
                    self.mem_ptr.checked_add(*n as usize)
                } else {
                    self.mem_ptr.checked_sub(n.abs() as usize)
                };

                if let Some(target) = target {
                    if target >= MEMORY_CELLS {
                        return Err(RuntimeError::NoMoreCells);
                    }
                    self.mem_ptr = target;
                } else {
                    return Err(RuntimeError::NoMoreCells);
                }
            }
            Instruction::Print => {
                print!("{}", *cell as char);
            }
            Instruction::JumpIfZero(target) if *cell == 0 => {
                return Ok(*target);
            }
            Instruction::JumpIfNotZero(target) if *cell != 0 => {
                return Ok(*target);
            }
            _ => {}
        };

        Ok(self.i_ptr + 1)
    }
}

pub fn compile(instructions: &[Statement]) -> Vec<Instruction> {
    let mut state: CompilerState = CompilerState::Initial;
    let mut bytecode: Vec<Instruction> = vec![];

    let flush_and_reset = |state: &mut CompilerState, bytecode: &mut Vec<Instruction>| {
        match state {
            CompilerState::Increment(n) if *n != 0 => {
                bytecode.push(Instruction::Increment(*n));
            }
            CompilerState::Shift(n) if *n != 0 => {
                bytecode.push(Instruction::Shift(*n));
            }
            _ => {}
        }
        *state = CompilerState::Initial;
    };

    for instruction in instructions {
        match (instruction, &state) {
            (Statement::Increment, CompilerState::Increment(n)) => {
                state = CompilerState::Increment(n + 1)
            }
            (Statement::Decrement, CompilerState::Increment(n)) => {
                state = CompilerState::Increment(n - 1)
            }
            (Statement::ShiftRight, CompilerState::Shift(n)) => state = CompilerState::Shift(n + 1),
            (Statement::ShiftLeft, CompilerState::Shift(n)) => state = CompilerState::Shift(n - 1),

            _ => {
                flush_and_reset(&mut state, &mut bytecode);

                match instruction {
                    Statement::ShiftRight => state = CompilerState::Shift(1),
                    Statement::ShiftLeft => state = CompilerState::Shift(-1),
                    Statement::Increment => state = CompilerState::Increment(1),
                    Statement::Decrement => state = CompilerState::Increment(-1),

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

    #[test]
    fn cant_decrement_zero_mem_register() {
        let mut vm = VM::default();
        vm.eval_one(&Instruction::Increment(-1))
            .expect_err("Expected operation to fail");
    }

    #[test]
    fn cant_move_past_last_memory_cell() {
        let mut vm = VM {
            i_ptr: 0,
            mem_ptr: 29999,
            memory: [0; MEMORY_CELLS],
        };
        vm.eval_one(&Instruction::Shift(1))
            .expect_err("Expected operation to fail");
    }

    #[test]
    fn cant_move_before_first_memory_cell() {
        let mut vm = VM::default();
        vm.eval_one(&Instruction::Shift(-1))
            .expect_err("Expected operation to fail");
    }
}
