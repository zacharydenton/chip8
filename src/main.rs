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

fn main() {
    let mut rng = rand::thread_rng();
    let mut chip8 = Chip8::new();

    let args: Vec<_> = std::env::args().collect();
    if args.len() == 2 {
        let filename = &args[1];
        let mut f = File::open(filename).expect("Unable to open program file.");
        let mut program: Vec<u8> = vec![];
        f.read_to_end(&mut program).expect("Error reading program file.");
        chip8.load(&program);
    } else {
        let logo = include_bytes!("logo.ch8");
        chip8.load(logo);
    }

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut closed = false;
    while !closed {
        let mut framebuffer: Vec<u8> = vec![0; 3 * chip8.graphics.len()];
        render(&chip8, &mut framebuffer);

        let image = glium::texture::RawImage2d::from_raw_rgb(framebuffer, (64, 32));
        let texture = glium::Texture2d::new(&display, image).unwrap();
        let surface = texture.as_surface();

        let target = display.draw();
        surface.fill(&target, glium::uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();

        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => closed = true,
                _ => (),
            },
            _ => (),
        });

        chip8.cycle(&mut rng);
    }
}
