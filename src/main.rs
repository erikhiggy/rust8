use std::fs::File;
use std::io::Read;
use rand::Rng;

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
    // delay and sound timers
    reg_dt: u8,
    reg_st: u8,

    // 16 bit program counter
    reg_pc: u16,

    // 8 bit stack pointer
    sp: u8,

    // stack
    stack: [u8; 16],

    // 8 bit graphics (gfx) array
    gfx: [u8; 64 * 32],

    // draw flag
    draw_flag: bool,

    // keyboard handling
    key: [u8; 16]

}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            reg_gpr: [0; 16],
            reg_i: 0,
            reg_pc: PROGRAM_START_ADDR,
            sp: 0,
            stack: [0; 16],
            reg_dt: 0,
            reg_st: 0,
            gfx: [0; 64 * 32],
            draw_flag: false,
            key: [0; 16]
        }
    }

    pub fn run_instruction(&mut self, ram: &mut Ram) {
        // fetch opcode Big Endian
        let hi = ram.read_byte(self.reg_pc) as u16;
        let lo = ram.read_byte(self.reg_pc+1) as u16;
        let instruction: u16 = (hi << 8) | lo;
        // decode and execute the opcode
        let mut reg_vx = self.reg_gpr[((instruction & 0x0F00) >> 8) as usize];
        let mut reg_vy = self.reg_gpr[((instruction & 0x00F0) >> 4) as usize];
        let nnn = instruction & 0x0FFF;
        let nn: u8 = (instruction & 0x00FF) as u8;
        let mut reg_v0 = self.reg_gpr[0] as u16;
        let mut reg_vf = self.reg_gpr[0x000F];

        match instruction & 0xF000 {
            0x0000 => match instruction & 0x000F {
                0x0000 => {
                    // 0x00E0: clear screen
                    // TODO: figure out how to clear screen
                },
                0x000E => {
                    // 0x00EE: return from subroutine
                    // restores program counter and then removes stack address
                    self.sp -= 1;
                    self.reg_pc = self.stack[self.sp as usize] as u16;
                },
                _ => println!("Invalid opcode {}", instruction)
            },
            0x1000 => {
                // 0x1NNN: jumps to address NNN
                self.reg_pc = nnn;
            },
            0x2000 => {
                // 0x2NNN: calls subroutine at NNN
                self.stack[self.sp as usize] = self.reg_pc as u8;
                self.sp += 1;
                self.reg_pc = instruction & 0x0FFF;
            },
            0x3000 => {
                // 0x3XNN: skips the next instruction if VX === NNN
                if reg_vx == nn {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0x4000 => {
                // 0x4XNN: skips the next instruction if VX !== NNN
                if reg_vx != nn {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0x5000 => {
                // 0x5XY0: skips the next instruction if VX === VY
                if reg_vx == reg_vy {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0x6000 => {
                // 0x6XNN: sets VX to NN
                reg_vx = nn;
                self.reg_pc += 2;
            },
            0x7000 => {
                // 0x7XNN: Adds NN to VX (carry flag is not changed)
                reg_vx += nn;
                self.reg_pc += 2;
            },
            0x8000 => {
                match instruction & 0x000F {
                    0x0000 => {
                        // 0x8XY0: sets VX = VY
                        reg_vx = reg_vy;
                        self.reg_pc += 2;
                    },
                    0x0001 => {
                        // 0x8XY1: bitwise OR -> VX | VY, store in VX
                        reg_vx |= reg_vy;
                        self.reg_pc += 2;
                    },
                    0x0002 => {
                        // 0x8XY2: bitwise AND -> VX & VY
                        reg_vx &= reg_vy;
                        self.reg_pc += 2;
                    },
                    0x0003 => {
                        // 0x8XY3: XOR -> VX XOR VY
                        reg_vx ^= reg_vy;
                        self.reg_pc += 2;
                    },
                    0x0004 => {
                        // 0x8XY4: adds VY to VX. VF is set to 1 when there's a carry
                        // and a 0 when when there isn't
                        if reg_vy > (0x00FF - reg_vx) {
                            reg_vf = 1;
                        } else {
                            reg_vf = 0;
                        }
                        reg_vx += reg_vy;
                        self.reg_pc += 2;

                    },
                    0x0005 => {
                        // 0x8XY5: subtracts VY from VX. VF is set to 1 when there's a carry
                        // and a 0 when when there isn't
                         if reg_vx > reg_vy {
                             reg_vf = 1;
                         } else {
                             reg_vf = 0;
                         }
                        reg_vx -= reg_vy;
                        self.reg_pc += 2;
                    },
                    0x0006 => {
                        // 0x8XY6: stores the LSB of VX in VF and then shifts VX to the right by 1
                        reg_vf = reg_vx & 1;
                        reg_vx = reg_vx >> 1;
                        self.reg_pc += 2;
                    },
                    0x0007 => {
                        // 0x8XY7: sets VX to VY minus VX.
                        // VF is set to 0 when there's a borrow, and 1 when there isn't
                        if reg_vy > reg_vx {
                            reg_vf = 1;
                        } else {
                            reg_vf = 0;
                        }
                        reg_vy -= reg_vx;
                        self.reg_pc += 2;
                    },
                    0x000E => {
                        // 0x8XYE: stores the MSB of VX in VF and then shifts VX to the left by 1
                        reg_vf = (reg_vx >> 3) & 1;
                        reg_vx = reg_vx << 1;
                        self.reg_pc += 2;
                    },
                    _ => println!("Invalid opcode {}", instruction)
                }
            },
            0x9000 => {
                // 0x9XY0: skips the next instruction if VX doesn't equal VY
                if reg_vx != reg_vy {
                    self.reg_pc += 2;
                }
                self.reg_pc += 2;
            },
            0xA000 => {
                // 0xANNN: sets I to the address NNN
                self.reg_i = nnn;
                self.reg_pc += 2;
            },
            0xB000 => {
                // 0xBNNN: jumps to the address NNN plus V0
                self.reg_pc = nnn + reg_v0;
            },
            0xC000 => {
                // 0xCXNN: sets VX to the result of a bitwise AND operation
                // on a random number (Typically: 0 to 255) and NN
                let mut rng = rand::thread_rng();
                let rand_num: u8 = rng.gen();
                reg_vx = rand_num & nn as u8;
                self.reg_pc += 2;
            },
            0xD000 => {
                // 0xDXYN: draws a sprite at coordinate (VX, VY), has a width of 8 pixels and
                // a height of N + 1 pixels.
                let x = reg_vx as u16;
                let y = reg_vy as u16;
                let height = instruction & 0x000F;
                let mut pixel: u8;
                reg_vf = 0;

                for y_line in 0..height {
                    pixel = ram.memory[(self.reg_i + y_line) as usize];
                    for x_line in 0..8 {
                        if (pixel & (0x80 >> x_line)) != 0 {
                            if self.gfx[(x + x_line as u16 + ((y + y_line) * 64)) as usize] == 1 {
                                reg_vf = 1;
                            }
                            self.gfx[(x + x_line as u16 + ((y + y_line) * 64)) as usize] ^= 1;
                        }
                    }
                }
                self.draw_flag = true;
                self.reg_pc += 2;
            },
            0xE000 => {
                match instruction & 0x000F {
                    0x000E => {
                        // 0xEX9E: skips the next instruction if the key stored in VX is pressed
                        if self.key[reg_vx as usize] != 0 {
                            self.reg_pc += 2;
                        }
                        self.reg_pc += 2;
                    },
                    0x0001 => {
                        // 0xEXA1: skips the next instruction if the key stored in VX isn't pressed
                        if self.key[reg_vx as usize] == 0 {
                            self.reg_pc += 2;
                        }
                        self.reg_pc += 2;
                    },
                    _ => println!("Invalid opcode! {}", instruction)
                }
            },
            0xF000 => {
                match instruction & 0x000F {
                    0x0007 => {
                        // 0xFX07: the value of DT is placed in VX
                        reg_vx = self.reg_dt;
                        self.reg_pc += 2;
                    },
                    0x000A => {
                        // 0xFX0A: wait for key press, store the value of key into VX
                        // keypad logic
                        let mut key_pressed = false;
                        for i in 0..self.key.len() {
                            if self.key[i] != 0 {
                                key_pressed = true;
                                reg_vx = i as u8;
                                break;
                            }
                        }

                        // if no key is pressed, try again
                        if !key_pressed {
                            return;
                        }

                        self.reg_pc += 2;
                    },
                    0x0005 => {
                        match instruction & 0x00FF {
                            0x0015 => {
                                // 0xFX15: set DT to VX
                                self.reg_dt = reg_vx;
                                self.reg_pc += 2;
                            },
                            0x0055 => {
                                // 0xFX55: store registers V0 -> VX in memory starting at
                                // location I
                                for x in 0..=self.reg_gpr.iter().position(|&val| val == reg_vx).unwrap() {
                                    ram.write_byte(self.reg_i + x as u16, self.reg_gpr[x])
                                }
                            },
                            0x0065 => {
                                // 0xFX65: read registers V0 -> Vx from memory starting at
                                // location
                                for x in self.reg_i..=ram.memory[self.reg_gpr.iter().position(|&s| s == reg_vx).unwrap()] as u16 {
                                    ram.read_byte(ram.memory[x as usize] as u16);
                                }
                            },
                            _ => println!("Invalid opcode! {}", instruction)
                        }
                    },
                    0x0008 => {
                        // 0xFX18: set ST to VX
                        self.reg_st = reg_vx;
                        self.reg_pc += 2;
                    },
                    0x000E => {
                        // 0xFX1E: set I = I + VX
                        self.reg_i += reg_vx as u16;
                        self.reg_pc += 2;
                    },
                    0x0009 => {
                        // 0xFX29: set I = location of sprite for digit VX
                        self.reg_i = self.gfx[reg_vx as usize] as u16;
                        self.reg_pc += 2;
                    },
                    0x0003 => {
                        // 0xFX33: store BCD representation of VX in memory locations
                        // I, I+1, I+2
                        ram.memory[self.reg_i as usize] = reg_vx / 100;
                        ram.memory[self.reg_i as usize + 1] = (reg_vx / 10) % 10;
                        ram.memory[self.reg_i as usize + 2] = (reg_vx % 100) % 10;
                        self.reg_pc += 2;
                    },
                    _ => println!("Invalid opcode! {}", instruction)
                }
            }
            _ => println!("Invalid opcode! {}", instruction)
        }
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
