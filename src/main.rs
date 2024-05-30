use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use std::io::Read;

const PPM_MAX_LINE_WIDTH: usize = 70;

fn load_image() -> (Vec<u8>, String) {
    let mut buf: Vec<u8> = Vec::new();
    let mut args = std::env::args();
    args.next().unwrap();
    let path = args.next().expect("Expected .ppm file");
    let mut file = std::fs::File::open(path.as_str()).expect(format!("Could not open file {}", path).as_str());
    file.read_to_end(&mut buf).expect("Could not read file {path}");
    (buf, path)
}


fn main() -> Result<(), Error> {
    let (raw, file) = load_image();
    let img = Image::from_buffer(raw, file);
    let event_loop = EventLoop::new();
    let title = format!("{} - {} x {}",img.file, img.width, img.height);
    let window = {
        let size = LogicalSize::new(img.width as f64,img.height as f64);
        WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(img.width, img.height, surface_texture)?
    };
    event_loop.run(move |event, _, control_flow| {
        img.draw(pixels.frame_mut());
        pixels.render().expect("Render error");
        *control_flow = winit::event_loop::ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {*control_flow = winit::event_loop::ControlFlow::Exit; return},
            _ => (),
        }
    });
}

#[derive(Debug)]
struct Image {
    file: String,
    max: u32,
    width: u32,
    height: u32,
    buffer: Vec<[u8; 4]>
}

enum IterState {
    Magic,
    Size,
    Max,
    Data,
    Done
}

impl IterState {
    fn next(&self) -> Self {
        match self {
            IterState::Magic => IterState::Size,
            IterState::Size => IterState::Max,
            IterState::Max => IterState::Data,
            IterState::Data => IterState::Done,
            _ => unreachable!()
        }
    }
}


impl Image {
    fn from_buffer(buffer: Vec<u8>, file: String) -> Self {

        let mut bytes = buffer.iter().peekable();
        let mut line = Vec::with_capacity(PPM_MAX_LINE_WIDTH);
        let mut img = Image {
            file,
            width: 0,
            height: 0,
            max: 0,
            buffer: Vec::with_capacity(512*512),
        };
        let mut state = IterState::Magic;
        while bytes.peek().is_some() {
            while bytes.peek().is_some() && *bytes.peek().unwrap() != &b'\n' {
                line.push(bytes.next().unwrap());
            }
            //eat the new line
            bytes.next();
            //check for comment
            if line[0] == &b'#' {
                line.clear();
                continue;
            }
            match state {
                IterState::Magic => {
                    if line.len() == 2 && line[0] == &b'P' && line[1] == &b'3' {
                        line.clear();
                        state = state.next();
                        continue;
                    } else {
                        panic!("File is not P3");
                    }
                },
                IterState::Size => {
                    //split on whitespace
                    let mut splits = line.splitn(2, |byte| *byte == &b' ');
                    let width_b  = splits.next().unwrap();
                    let height_b = splits.next().unwrap();
                    let mut width = String::new();
                    for b in width_b {
                        width.push(**b as char);
                    }
                    let mut height = String::new();
                    for b in height_b {
                        height.push(**b as char);
                    }
                    img.width = width.as_str().parse::<u32>().unwrap();
                    img.height = height.as_str().parse::<u32>().unwrap();
                    line.clear();
                    state = state.next();
                    continue;
                },
                IterState::Max => {
                    let mut max = String::new();
                    for b in &line {
                        max.push(**b as char);
                    }
                    img.max = max.as_str().parse::<u32>().unwrap();
                    line.clear();
                    state = state.next();
                },
                IterState::Data => {
                    let mut i = 0;
                    let mut characters = String::new();
                    let mut values = vec![0u8,0u8,0u8];
                    for b in &line {
                        if *b == &b' ' {
                            values[i] = characters.as_str().parse::<u8>().unwrap();
                            characters.clear();
                            i += 1;
                        } else {
                            characters.push(**b as char);
                        }
                        if i == 3 {
                            img.push_pixel(values[0], values[1], values[2]);
                            i = 0;
                        }
                    }
                    img.push_pixel(values[0], values[1], characters.as_str().parse::<u8>().unwrap());
                    line.clear();
                },
                _ => unimplemented!()
            }
        }
        img
    }

    fn push_pixel(&mut self, r: u8, g: u8, b: u8) {
        let mut pix = [0,0,0, 255];
        pix[0] = r;
        pix[1] = g;
        pix[2] = b;
        self.buffer.push(pix);
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&self.buffer[i]);
        }
    }
}




