use std::fs::File;
use std::io::prelude::*;

extern crate chip8;
use chip8::Chip8;

extern crate glium;
use glium::{glutin, Surface};

extern crate rand;

fn render(chip8: &Chip8, framebuffer: &mut [u8]) {
    for x in 0..64 {
        for y in 0..32 {
            let ti = 3 * ((31 - y) * 64 + x);
            let source = chip8.graphics[64 * y + x];
            framebuffer[ti + 0] = 0 * source;
            framebuffer[ti + 1] = 255 * source;
            framebuffer[ti + 2] = 0 * source;
        }
    }
}

fn keymap(scancode: u32) -> Option<u8> {
    match scancode {
        2 => Some(0x1),
        3 => Some(0x2),
        4 => Some(0x3),
        5 => Some(0xC),
        16 => Some(0x4),
        17 => Some(0x5),
        18 => Some(0x6),
        19 => Some(0xD),
        30 => Some(0x7),
        31 => Some(0x8),
        32 => Some(0x9),
        33 => Some(0xE),
        44 => Some(0xA),
        45 => Some(0x0),
        46 => Some(0xB),
        47 => Some(0xF),
        _ => None,
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut chip8 = Chip8::new();

    let args: Vec<_> = std::env::args().collect();
    if args.len() == 2 {
        let filename = &args[1];
        let mut f = File::open(filename).expect("Unable to open program file.");
        let mut program: Vec<u8> = vec![];
        f.read_to_end(&mut program)
            .expect("Error reading program file.");
        chip8.load(&program);
    } else {
        let logo = include_bytes!("../data/logo.ch8");
        chip8.load(logo);
    }

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut closed = false;
    while !closed {
        if chip8.needs_redraw {
            let mut framebuffer: Vec<u8> = vec![0; 3 * chip8.graphics.len()];
            render(&chip8, &mut framebuffer);

            let image = glium::texture::RawImage2d::from_raw_rgb(framebuffer, (64, 32));
            let texture = glium::Texture2d::new(&display, image).unwrap();
            let surface = texture.as_surface();

            let target = display.draw();
            surface.fill(&target, glium::uniforms::MagnifySamplerFilter::Nearest);
            target.finish().unwrap();
        }

        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => closed = true,
                glutin::WindowEvent::KeyboardInput { input, .. } => match keymap(input.scancode) {
                    Some(keycode) => match input.state {
                        glium::glutin::ElementState::Pressed => {
                            chip8.key_down(keycode);
                        }
                        glium::glutin::ElementState::Released => {
                            chip8.key_up(keycode);
                        }
                    },
                    None => (),
                },
                _ => (),
            },
            _ => (),
        });

        chip8.cycle(&mut rng);
    }
}
