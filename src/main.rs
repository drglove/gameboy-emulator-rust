mod cpu;
mod ppu;
mod apu;
mod memory;
mod utils;

struct DMG01 {
    cpu: cpu::CPU,
}

impl DMG01 {
    fn new(cart: Option<Cartridge>) -> DMG01 {
        use std::cmp::min;

        let mut memory: [u8; 0x10000] = [0; 0x10000];
        let size_to_copy = min(0x10000, cart.as_ref().unwrap().rom.len());
        memory[0..size_to_copy].copy_from_slice(&cart.unwrap().rom.as_slice());

        DMG01 {
            cpu: cpu::CPU::new(memory),
        }
    }
}

struct Cartridge {
    rom: Vec<u8>,
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str), long)]
    rom: Option<std::path::PathBuf>,
}

fn write_audio<T: cpal::Sample>(data: &mut [T], next_sample: &mut dyn FnMut() -> f32) {
    for sample in data.iter_mut() {
        *sample = cpal::Sample::from::<f32>(&next_sample());
    }
}

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::BufferSize;

fn main() {
    let args = Cli::from_args();

    use std::fs;
    let cart = if let Some(rom_path) = args.rom {
        Some(Cartridge {
            rom: fs::read(rom_path).expect("Could not open rom file!"),
        })
    } else {
        None
    };

    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % 48000.0;
        (sample_clock * 45.0 * 2.0 * 3.141592 / 48000.0).sin()
    };

    use cpal::SampleFormat;
    use cpal::traits::HostTrait;
    let audio_host = cpal::default_host();
    let audio_device = audio_host.default_output_device().expect("No audio output devices available!");
    let mut audio_supported_configs_range = audio_device.supported_output_configs().expect("Error while querying audio configurations");
    let audio_supported_config = audio_supported_configs_range.next().expect("No supported audio configs found").with_max_sample_rate();
    let mut audio_config = (&audio_supported_config.config()).clone();
    audio_config.buffer_size = match audio_supported_config.buffer_size() {
        cpal::SupportedBufferSize::Range { min, .. } => BufferSize::Fixed(*min),
        cpal::SupportedBufferSize::Unknown => BufferSize::Default,
    };
    let sample_format = &audio_supported_config.sample_format();
    let write_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        write_audio(data, &mut next_value);
    };
    let stream = match sample_format {
        SampleFormat::F32 => audio_device.build_output_stream(&audio_config, write_fn, |err| eprintln!("Error occurred on the output audio stream: {:?}", err)),
        SampleFormat::I16 => audio_device.build_output_stream(&audio_config, write_fn, |err| eprintln!("Error occurred on the output audio stream: {:?}", err)),
        SampleFormat::U16 => audio_device.build_output_stream(&audio_config, write_fn, |err| eprintln!("Error occurred on the output audio stream: {:?}", err)),
    }.unwrap();
    stream.play().unwrap();

    use ppu::{LCD_WIDTH, LCD_HEIGHT};
    use minifb::{Window, WindowOptions};
    let mut window = match Window::new("DMG-01", LCD_WIDTH as usize, LCD_HEIGHT as usize, WindowOptions::default()) {
        Ok(win) => win,
        Err(_) => panic!("Could not create window!"),
    };
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    let mut gameboy = DMG01::new(cart);
    gameboy.cpu.bus.apu.initialize_buffers(audio_config.sample_rate.0, cpu::CPU_CLOCK_RATE_HZ);
    while window.is_open() {
        gameboy.cpu.step_frame();

        window
            .update_with_buffer(gameboy.cpu.bus.ppu.framebuffer.as_slice(), LCD_WIDTH as usize, LCD_HEIGHT as usize)
            .unwrap();
    }
}
