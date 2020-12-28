mod ram;
mod cpu;

use std::fs::File;
use std::io::Read;
use minifb::{
    Window,
    WindowOptions,
    Scale
};
use rodio::Sink;

use ram::Ram;
use cpu::Cpu;

const NUM_GPR: usize = 16;
const RAM_SIZE: usize = 4096;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const TIMER_DEFAULT: usize = 8;
const PX_OFF: u32 = 0;
const PX_ON: u32 = 0xFFFFFF;
const PROGRAM_START_ADDR: u16 = 0x0200;

fn load_rom(data: &Vec<u8>, ram: &mut Ram) {
    for i in 0..data.len() {
        ram.write_byte(PROGRAM_START_ADDR + i as u16, data[i]);
    }
}

fn main() {
    let mut file = File::open("data/breakout").expect("Could not open file.");
    let mut data = Vec::<u8>::new();
    file.read_to_end(&mut data).expect("Could not read file.");

    let mut ram = Ram::new();
    let mut cpu = Cpu::new();

    // load rom into Chip8
    load_rom(&data, &mut ram);

    // setup audio
    let audio_device = rodio::default_output_device().unwrap();
    let audio_sink = Sink::new(&audio_device);
    let audio_source = rodio::source::SineWave::new(440);
    audio_sink.append(audio_source);
    audio_sink.pause();

    let mut runloop_time = TIMER_DEFAULT;

    let mut window = Window::new(
        &format!("chip-8 rust"),
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        }
    ).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(2083)));

    cpu.handle_keypress(&window);

    while window.is_open() && (cpu.reg_pc() as usize) <= RAM_SIZE {
        cpu.run_instruction(&mut ram);

        // reset timers
        if runloop_time == 0 {
            if cpu.reg_dt() > 0 {
                cpu.set_reg_dt(cpu.reg_dt() - 1);
            }
            if cpu.reg_st() > 0 {
                audio_sink.play();
                cpu.set_reg_st(cpu.reg_st() - 1);
            } else if cpu.reg_st() == 0 {
                audio_sink.pause();
            }

            window.update_with_buffer(&cpu.gfx(), WIDTH, HEIGHT).unwrap();

            runloop_time = TIMER_DEFAULT;
        } else {
            runloop_time -= 1;
        }
    }
}