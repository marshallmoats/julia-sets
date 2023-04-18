use std::io::{self, Write};
use minifb::{Key, ScaleMode, Window, WindowOptions};
use num_complex::Complex64;
use rayon::prelude::{IntoParallelRefMutIterator, IndexedParallelIterator, ParallelIterator};

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

fn divergence(z: Complex64, max_iter: u32, a: f64, b: f64) -> u32 {
    let _rng = rand::thread_rng();
    let mut z = z;
    let c = Complex64::new(a, b);
    for i in 0..max_iter {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return i;
        }
    }
    max_iter
}

fn color(i: u32, max_iter: u32) -> u32 {
    let x = (255.0 * (i as f64 / max_iter as f64)) as u32 ;
    let r = (x % 255)as f64;
    let g = ((x + 80) % 255) as f64;
    let b = ((x + 160) % 255) as f64;

    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[derive(Debug)]
struct ViewBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl ViewBox {
    fn new_default() -> Self {
        Self {
            x: -2.0,
            y: -2.0,
            width: 4.0,
            height: 4.0,
        }
    }

    fn zoom(&mut self, factor: f64, fixed_point: (f64, f64)) {
        self.x = (self.x - fixed_point.0) * factor + fixed_point.0;
        self.y = (self.y - fixed_point.1) * factor + fixed_point.1;

        self.width *= factor;
        self.height *= factor;
    }
}

fn main() {
    println!("This program draws Julia sets, which are parameterized by two variables.\n\
    Enter them below to start, and use hjkl to modify them while the program is running.\n\
    Use the arrow keys to pan, and z and x to zoom in and out.");
    print!("Enter parameter a: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let mut a: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => panic!("Invalid input"),
    };
    print!("Enter parameter b: ");
    io::stdout().flush().unwrap();
    input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let mut b: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => panic!("Invalid input"),
    };


    let mut window = Window::new(
        "Julia Sets",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);

    let mut size = (0, 0);

    let mut view_box = ViewBox::new_default();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let new_size = (window.get_size().0, window.get_size().1);
        if new_size != size {
            size = new_size;
            buffer.resize(size.0 * size.1, 0);
        }

        let iter = buffer.par_iter_mut().enumerate();

        let num_iter = (100.0 - view_box.width.log2()) as u32;

        iter.for_each(|(idx, i)| {
            let (x, y) = ((idx % size.0) as u32, (idx / size.0) as u32);
            *i = color(divergence(Complex64::new(x as f64 / size.0 as f64 * view_box.width + view_box.x, y as f64 / size.1 as f64 * view_box.height + view_box.y), num_iter, a, b), num_iter);
        });

        window.get_keys().iter().for_each(|key| match key {
            Key::Z => {
                view_box.zoom(0.95, (view_box.x + view_box.width / 2.0, view_box.y + view_box.height / 2.0));
            },
            Key::X => {
                view_box.zoom(1.05, (view_box.x + view_box.width / 2.0, view_box.y + view_box.height / 2.0));
            },
            Key::Left => {
                view_box.x -= view_box.width / 50.0;
            },
            Key::Right => {
                view_box.x += view_box.width / 50.0;
            },
            Key::Up => {
                view_box.y -= view_box.height / 50.0;
            },
            Key::Down => {
                view_box.y += view_box.height / 50.0;
            },
            Key::H => {
                a -= 0.01;
            }
            Key::J => {
                a += 0.01;
            }
            Key::K => {
                b -= 0.01;
            }
            Key::L => {
                b += 0.01;
            }
            _ => (),
        });

        window
            .update_with_buffer(&buffer, new_size.0, new_size.1)
            .unwrap();
    }
}