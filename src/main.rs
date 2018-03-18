extern crate chip8;
use chip8::Chip8;

#[macro_use]
extern crate glium;

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
    use glium::{glutin, Surface};

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut chip8 = Chip8::new();

    // Set V2 = rocket X coordinate = 0x00;
    chip8.memory[0x200] = 0x62;
    chip8.memory[0x201] = 0x00;

    // Set V3 = rocket Y coordinate = 0x00;
    chip8.memory[0x202] = 0x63;
    chip8.memory[0x203] = 0x00;

    // Set I = rocket pattern address = 0x020A;
    chip8.memory[0x204] = 0xA2;
    chip8.memory[0x205] = 0x0A;

    // Display 6 byte rocket pattern.
    chip8.memory[0x206] = 0xD2;
    chip8.memory[0x207] = 0x36;

    // Loop forever (goto 0x208).
    chip8.memory[0x208] = 0x12;
    chip8.memory[0x209] = 0x08;

    // Rocket pattern.
    chip8.memory[0x20A] = 0x20;
    chip8.memory[0x20B] = 0x70;
    chip8.memory[0x20C] = 0x70;
    chip8.memory[0x20D] = 0xF8;
    chip8.memory[0x20E] = 0xD8;
    chip8.memory[0x20F] = 0x88;

    let mut closed = false;
    while !closed {
        let mut framebuffer: Vec<u8> = vec![0; 3 * chip8.graphics.len()];
        render(&chip8, &mut framebuffer);

        let mut image = glium::texture::RawImage2d::from_raw_rgb(framebuffer, (64, 32));
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

        chip8.cycle();
    }
}
