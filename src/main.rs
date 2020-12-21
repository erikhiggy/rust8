use std::fs::File;
use std::io::Read;

fn main() {
    let mut file = File::open("data/invaders").unwrap();
    let mut data = Vec::<u8>::new();
    file.read_to_end(&mut data);

    let mut chip8 = Chip8::new();
    chip8.load_rom(&data);

    loop {
        chip8.run_instruction()
    }
}

const NUM_GPR: usize = 16;
const NUM_SP: usize = 2;
const RAM_SIZE: usize = 4096;

pub const PROGRAM_START_ADDR: u16 = 0x200;

struct Cpu {
    // 16 8 bit general purpose registers
    reg_gpr: [u8; NUM_GPR],

    // 1 16 bit register, i
    reg_i: u16,

    // 2 special purpose 8 bit registers
    // reg_sp: [u8; NUM_SP],

    // 16 bit program counter
    reg_pc: u16,

    // 8 bit stack pointer
    // reg_stack_pointer: u8
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            reg_gpr: [0; 16],
            reg_i: 0,
            reg_pc: PROGRAM_START_ADDR,
        }
    }

    pub fn run_instruction(&mut self, ram: &mut Ram) {
        // each instr is 2 bytes long, stored most SIGNIFICANT BYTE FIRST --> Big Endian
        let hi = ram.read_byte(self.reg_pc) as u16;
        let lo = ram.read_byte(self.reg_pc+1) as u16;
        let instruction: u16 = (hi << 8) | lo;
        println!("Instruction read: {:#X}: hi:{:#X} lo:{:#X}", instruction, hi, lo);
        // increment the PC by 2 after each instruction
        self.reg_pc += 2;
    }
}

struct Chip8 {
    ram: Ram,
    cpu: Cpu
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            ram: Ram::new(),
            cpu: Cpu::new()
        }
    }

    pub fn load_rom(&mut self, data: &Vec<u8>) {
        for i in 0..data.len() {
            self.ram.write_byte(PROGRAM_START_ADDR + i as u16, data[i]);
        }
    }

    pub fn run_instruction(&mut self) {
        self.cpu.run_instruction(&mut self.ram);
    }
}

struct Ram {
    memory: [u8; RAM_SIZE]
}

impl Ram {
    fn new() -> Ram {
        let mut ram = Ram {
            memory: [0; RAM_SIZE]
        };

        let sprites: [[u8; 5]; 16] = [
            [0xF0, 0x90, 0x90, 0x90, 0xF0],
            [0x20, 0x60, 0x20, 0x20, 0x70],
            [0xF0, 0x10, 0xF0, 0x80, 0xF0],
            [0xF0, 0x10, 0xF0, 0x10, 0xF0],
            [0x90 ,0x90, 0xF0, 0x10, 0x10],
            [0xF0, 0x80, 0xF0, 0x10, 0xF0],
            [0xF0, 0x80, 0xF0, 0x90, 0xF0],
            [0xF0, 0x10, 0x20, 0x40, 0x40],
            [0xF0, 0x90, 0xF0, 0x90, 0xF0],
            [0xF0, 0x90, 0xF0, 0x10, 0xF0],
            [0xF0, 0x90, 0xF0, 0x90, 0x90],
            [0xE0, 0x90, 0xE0, 0x90, 0xE0],
            [0xF0, 0x80, 0x80, 0x80, 0xF0],
            [0xE0, 0x90, 0x90, 0x90, 0xE0],
            [0xF0, 0x80, 0xF0, 0x80, 0xF0],
            [0xF0, 0x80, 0xF0, 0x80, 0x80]
        ];

        let mut i = 0;
        for sprite in sprites.iter() {
            for ch in sprite {
                ram.memory[i] = *ch;
                i += 1;
            }
        }

        ram
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        return self.memory[addr as usize];
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }
}