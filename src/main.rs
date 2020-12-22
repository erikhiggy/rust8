use std::fs::File;
use std::io::Read;
use std::ops::Deref;

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
    sp: u8,

    // stack
    stack: [u8; 16]
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            reg_gpr: [0; 16],
            reg_i: 0,
            reg_pc: PROGRAM_START_ADDR,
            sp: 0,
            stack: [0; 16]
        }
    }

    pub fn run_instruction(&mut self, ram: &mut Ram) {
        // fetch opcode Big Endian
        let hi = ram.read_byte(self.reg_pc) as u16;
        let lo = ram.read_byte(self.reg_pc+1) as u16;
        let instruction: u16 = (hi << 8) | lo;
        // decode and execute the opcode
        match instruction & 0xF000 {
            0x0000 => match instruction & 0x000F {
                0x0000 => {
                    // 0x00E0: clear screen
                },
                0x000E => {
                    // 0x00EE: return from subroutine
                },
                _ => println!("Invalid opcode {}", instruction)
            },
            0x1000 => {
                // 0x1NNN: jumps to address NNN
            },
            0x2000 => {
                // 0x2NNN: calls subroutine at NNN
                self.stack[self.sp] = self.reg_pc;
                self.sp += 1;
                self.reg_pc = instruction & 0x0FFF;
            },
            0x3000 => {
                // 0x3XNN: skips the next instruction if VX === NNN
            },
            0x4000 => {
                // 0x4XNN: skips the next instruction if VX !== NNN
            },
            0x5000 => {
                // 0x5XY0: skips the next instruction if VX === VY
            },
            0x6000 => {
                // 0x6XNN: sets VX to NN
            },
            0x7000 => {
                // 0x7XNN: Adds NN to VX (carry flag is not changed)
            },
            0x8000 => {
                match instruction & 0x000F {
                    0x0000 => {
                        // 0x8XY0: sets VX = VY
                    },
                    0x0001 => {
                        // 0x8XY1: bitwise OR -> VX | VY
                    },
                    0x0002 => {
                        // 0x8XY2: bitwise AND -> VX & VY
                    },
                    0x0003 => {
                        // 0x8XY3: XOR -> VX XOR VY
                    },
                    0x0004 => {
                        // 0x8XY4: sets VX = VY
                    },
                    0x0005 => {
                        // 0x8XY5: adds VY to VX. VF is set to 1 when there's a carry
                        // and a 0 when when there isn't
                    },
                    0x0006 => {
                        // 0x8XY6: stores the LSB of VX in VF and then shifts VX to the right by 1
                    },
                    0x0007 => {
                        // 0x8XY7: sets VX to VY minus VX.
                        // VF is set to 0 when there's a borrow, and 1 when there isn't
                    },
                    0x000E => {
                        // 0x8XYE: stores the MSB of VX in VF and then shifts VX to the left by 1
                    },
                    _ => println!("Invalid opcode {}", instruction)
                }
            },
            0x9000 => {
                // 0x9XY0: skips the next instruction if VX doesn't equal VY
            },
            0xA000 => {
                // 0xANNN: sets I to the address NNN
            },
            0xB000 => {
                // 0xBNNN: jumps to the address NNN plus V0
            },
            0xC000 => {
                // 0xCXNN: sets VX to the result of a bitwise AND operation
                // on a random number (Typically: 0 to 255) and NN
            },
            0xD000 => {
                // 0xDXYN: draws a sprite at coordinate (VX, VY), has a width of 8 pixels and
                // a height of N + 1 pixels.
                // TODO: Figure out GFX later
            },
            0xE000 => {
                match instruction & 0x000F {
                    0x000E => {
                        // 0xEX9E: skips the next instruction if the key stored in VX is pressed
                    },
                    0x0001 => {
                        // 0xEXA1: skips the next instruction if the key stored in VX isn't pressed
                    }
                }
            }
            _ => println!("Invalid opcode! {}", instruction)
        }
        // TODO: update timers
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
