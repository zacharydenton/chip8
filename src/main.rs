extern crate chip8;

#[macro_use]
extern crate glium;

fn main() {
    use chip8::Chip8;
    use glium::{glutin, Surface};

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut chip8 = Chip8::new();
    for i in 0..10 {
        for j in 0..12 {
            chip8.graphics[(j * 64) + i] = 1;
        }
        for j in 29..32 {
            chip8.graphics[(j * 64) + i] = 1;
        }
    }
    for i in 50..64 {
        for j in 0..32 {
            chip8.graphics[(j * 64) + i] = 1;
        }
    }

    let mut framebuffer: Vec<u8> = vec![0; 3 * chip8.graphics.len()];
    for x in 0..64 {
        for y in 0..32 {
            let ti = 3 * ((31 - y) * 64 + x);
            let source = chip8.graphics[64 * y + x];
            framebuffer[ti + 0] = 0 * source;
            framebuffer[ti + 1] = 255 * source;
            framebuffer[ti + 2] = 0 * source;
        }
    }

    let image = glium::texture::RawImage2d::from_raw_rgb(framebuffer, (64, 32));
    let texture = glium::Texture2d::new(&display, image).unwrap();
    let surface = texture.as_surface();

    let mut closed = false;
    while !closed {
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
    }
}
