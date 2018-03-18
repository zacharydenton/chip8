extern crate chip8;
use chip8::Chip8;

extern crate glium;
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
    use glium::{glutin, Surface};

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let mut rng = rand::thread_rng();
    let mut chip8 = Chip8::new();

    let program: Vec<u8> = vec![
      // Set V2 = rocket X coordinate = 0x00;
      0x62, 0x00,
      // Set V3 = rocket Y coordinate = 0x00;
      0x63, 0x00,
      // Set I = rocket pattern address = 0x020A;
      0xA2, 0x0A,
      // Display 6 byte rocket pattern.
      0xD2, 0x36,
      // Loop forever (goto 0x208).
      0x12, 0x08,
      // Rocket pattern.
      0x20,
      0x70,
      0x70,
      0xF8,
      0xD8,
      0x88,
    ];
    chip8.load(&program);

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
