use std::io::Read;
use std::{fs, io};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum Operation {
    Halt,
    Set(u16, u16),
    Push(u16),
    Pop(u16),
    Eq(u16, u16, u16),
    Gt(u16, u16, u16),
    Jmp(u16),
    Jt(u16, u16),
    Jf(u16, u16),
    Add(u16, u16, u16),
    Mult(u16, u16, u16),
    Mod(u16, u16, u16),
    And(u16, u16, u16),
    Or(u16, u16, u16),
    Not(u16, u16),
    Rmem(u16, u16),
    Wmem(u16, u16),
    Call(u16),
    Ret,
    Out(u16),
    In(u16),
    Noop,
}

impl Operation {
    fn new(opcode: u16, args: Vec<u16>) -> Self {
        match opcode {
            0 => Operation::Halt,
            1 => Operation::Set(args[0], args[1]),
            2 => Operation::Push(args[0]),
            3 => Operation::Pop(args[0]),
            4 => Operation::Eq(args[0], args[1], args[2]),
            5 => Operation::Gt(args[0], args[1], args[2]),
            6 => Operation::Jmp(args[0]),
            7 => Operation::Jt(args[0], args[1]),
            8 => Operation::Jf(args[0], args[1]),
            9 => Operation::Add(args[0], args[1], args[2]),
            10 => Operation::Mult(args[0], args[1], args[2]),
            11 => Operation::Mod(args[0], args[1], args[2]),
            12 => Operation::And(args[0], args[1], args[2]),
            13 => Operation::Or(args[0], args[1], args[2]),
            14 => Operation::Not(args[0], args[1]),
            15 => Operation::Rmem(args[0], args[1]),
            16 => Operation::Wmem(args[0], args[1]),
            17 => Operation::Call(args[0]),
            18 => Operation::Ret,
            19 => Operation::Out(args[0]),
            20 => Operation::In(args[0]),
            21 => Operation::Noop,
            _ => panic!("Invalid opcode.")
        }
    }

    fn num_arguments(opcode: u16) -> u16 {
        match opcode {
            0 | 18 | 21 => 0,
            2 | 3 | 6 | 17 | 19 | 20 => 1,
            1 | 7 | 8 | 14 | 15 | 16 => 2,
            4 | 5 | 9 | 10 | 11 | 12 | 13 => 3,
            _ => panic!("Invalid opcode."),
        }
    }    
}

#[derive(Debug, Default)]
struct VM {
    instruction_ptr: u16,
    mem: HashMap<u16, u16>,
    registers: [u16; 8],
    stack: Vec<u16>,
    halted: bool,
}

impl VM {
    // TODO: should this return a value?
    fn run_binary(&mut self, filename: &str) {
        self.read_binary(filename);
        while !self.halted {
            self.execute_next_operation();
        }
    }

    fn read_binary(&mut self, filename: &str) {
        fs::read(filename).unwrap()
            .chunks_mut(2)
            .enumerate()
            .for_each(|(idx, bytes)| {
                self.mem.insert(
                    idx.try_into().unwrap(),
                    u16::from_le_bytes(bytes.try_into().unwrap()),
                );
            });
    }

    fn execute_next_operation(&mut self) {
        let operation = self.parse_next_operation();
        self.execute_operation(operation);
    }

    fn parse_next_operation(&self) -> Operation {
        // TODO: assert?
        let mut args = vec![];
        let opcode = *self.mem.get(&self.instruction_ptr).unwrap();
        for i in 0..Operation::num_arguments(opcode) {
            args.push(*self.mem.get(&(self.instruction_ptr + 1 + i)).unwrap());
        }
        Operation::new(opcode, args)
    }

    fn execute_operation(&mut self, operation: Operation) {
        match operation {
            Operation::Halt => self.halted = true,
            Operation::Set(register, value) => {
                let register = Self::register_idx(register)
                    .expect("First argument for set operation should be a register.");
                self.registers[register] = self.get_value(value);
            },
            Operation::Push(value) => self.stack.push(self.get_value(value)),
            Operation::Pop(address) => {
                let val = self.stack.pop().expect("Stack should be non-empty for pop operation.");
                self.set_register(address, val);
            },
            Operation::Eq(address, b, c) => {
                self.set_register(address, if self.get_value(b) == self.get_value(c) { 1 } else { 0 });
            },
            Operation::Gt(address, b, c) => {
                self.set_register(address, if self.get_value(b) > self.get_value(c) { 1 } else { 0 });
            },
            Operation::Jmp(address) => self.instruction_ptr = self.get_value(address),
            Operation::Jt(value, address) => {
                if self.get_value(value) != 0 { self.instruction_ptr = self.get_value(address); }
                else { self.instruction_ptr += 3; }
            },
            Operation::Jf(value, address) => {
                if self.get_value(value) == 0 { self.instruction_ptr = self.get_value(address); }
                else { self.instruction_ptr += 3; }
            },
            Operation::Add(address, b, c) => {
                self.set_register(address, (self.get_value(b) + self.get_value(c)) % 32_768);
            },
            Operation::Mult(address, b, c) => {
                self.set_register(address, ((self.get_value(b) as u32 * self.get_value(c) as u32) % 32_768) as u16);
            },
            Operation::Mod(address, b, c) => {
                self.set_register(address, self.get_value(b) % self.get_value(c));
            },
            Operation::And(address, b, c) => {
                self.set_register(address, self.get_value(b) & self.get_value(c));
            },
            Operation::Or(address, b, c) => {
                self.set_register(address, self.get_value(b) | self.get_value(c));
            },
            Operation::Not(address, b) => {
                self.set_register(address, (self.get_value(b) ^ 0xffff) & 0x7fff);
            },
            Operation::Rmem(write_address, read_address) => {
                self.set_register(write_address, *self.mem.get(&self.get_value(read_address)).unwrap());
            },
            Operation::Wmem(write_address, read_address) => {
                self.mem.insert(self.get_value(write_address), self.get_value(read_address));
            },
            Operation::Call(address) => {
                self.stack.push(self.instruction_ptr + 2);
                self.instruction_ptr = self.get_value(address);
            },
            Operation::Ret => {
                if let Some(next) = self.stack.pop() {
                    self.instruction_ptr = next;
                } else {
                    self.halted = true;
                }
            },
            Operation::Out(value) => {
                let value: u8 = self.get_value(value).try_into().unwrap();
                print!("{}", value as char);
            },
            Operation::In(address) => {
                let mut buffer = [0u8; 1];
                io::stdin().read_exact(&mut buffer).unwrap();
                self.set_register(address, buffer[0] as u16);
            },
            Operation::Noop => (),
        }
        match operation {
            Operation::Halt | Operation::Jmp(_) | Operation::Jt(_, _) | Operation::Jf(_, _) | Operation::Call(_) | Operation::Ret => (),
            Operation::Noop => self.instruction_ptr += 1,
            Operation::Push(_) | Operation::Pop(_) | Operation::Out(_) | Operation::In(_) => self.instruction_ptr += 2,
            Operation::Set(_, _) | Operation::Not(_, _) | Operation::Rmem(_, _) | Operation::Wmem(_, _) => self.instruction_ptr += 3,
            Operation::Eq(_, _, _) | Operation::Gt(_, _, _) | Operation::Add(_, _, _) | Operation::Mult(_, _, _) | Operation::Mod(_, _, _) | Operation::And(_, _, _) | Operation::Or(_, _, _) => self.instruction_ptr += 4,
        }
    }

    fn register_idx(value: u16) -> Option<usize> {
        if value < 32_768 || value > 32_775 {
            return None;
        }
        Some((value % 32_768).try_into().unwrap())
    }

    fn set_register(&mut self, address: u16, value: u16) {
        let register_idx = Self::register_idx(address).unwrap();
        self.registers[register_idx] = value;
    }

    fn get_value(&self, value: u16) -> u16 {
        if let Some(val_register) = Self::register_idx(value) {
            self.registers[val_register]
        } else {
            value
        }
    }
}

fn main() {
    let mut vm = VM::default();
    vm.run_binary("input/challenge.bin");
}