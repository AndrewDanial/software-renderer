use glam::*;
use std::fs::File;
use std::fs::create_dir;
use std::fs::remove_dir_all;
use std::io::prelude::*;
use std::time::*;

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u32,
    g: u32,
    b: u32,
}

struct Triangle {
    a: Vec2,
    b: Vec2,
    c: Vec2,
}

impl Triangle {
    pub fn new(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self { a, b, c }
    }

    pub fn area(&self, a: Vec2, b: Vec2, c: Vec2) -> f32 {
        return ((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2.0).abs();
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        let a = self.area(self.a, self.b, self.c);
        let a1 = self.area(point, self.b, self.c);
        let a2 = self.area(self.a, point, self.c);
        let a3 = self.area(self.a, self.b, point);
        return (a1 + a2 + a3 - a).abs() < 0.01;
    }

    pub fn rotate(&mut self, angle: f32) {
        let (sin_theta, cos_theta) = angle.sin_cos();
        let center = Vec2::new(
            (self.a.x + self.b.x + self.c.x) / 3.,
            (self.a.y + self.b.y + self.c.y) / 3.,
        );

        let mut a = self.a - center;
        a = Mat2::from_cols_array(&[cos_theta, -sin_theta, sin_theta, cos_theta]) * a;
        a += center;
        self.a = a;
        let mut b = self.b - center;
        b = Mat2::from_cols_array(&[cos_theta, -sin_theta, sin_theta, cos_theta]) * b;
        b += center;
        self.b = b;
        let mut c = self.c - center;
        c = Mat2::from_cols_array(&[cos_theta, -sin_theta, sin_theta, cos_theta]) * c;
        c += center;
        self.c = c;
    }
}

impl Color {
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);

    pub const fn new(r: u32, g: u32, b: u32) -> Self {
        Self { r, g, b }
    }
}

struct Resolution {
    width: u32,
    height: u32,
}

struct App {
    resolution: Resolution,
    file: File,
}

impl App {
    pub fn new(resolution: Resolution, file: File) -> Self {
        Self { resolution, file }
    }

    pub fn write_header(&mut self) -> std::io::Result<()> {
        let width = self.resolution.width;
        let height = self.resolution.height;

        // PPM Header info
        self.file.write_all(b"P3\n")?;
        // Width and height of the image
        self.file.write_all(width.to_string().as_bytes())?;
        self.file.write_all(b" ")?;
        self.file.write_all(height.to_string().as_bytes())?;
        self.file.write_all(b"\n")?;
        // Max value for each color
        self.file.write_all(b"255\n")?;
        Ok(())
    }

    pub fn write_to_file(&mut self, color: Color) -> std::io::Result<()> {
        let buf = format!("{} {} {}", color.r, color.g, color.b);
        self.file.write_all(buf.as_bytes())?;
        Ok(())
    }

    pub fn write_newline(&mut self) -> std::io::Result<()> {
        self.file.write_all(b"\n")?;
        Ok(())
    }
}

fn main() {
    remove_dir_all("frames").unwrap();
    create_dir("frames").unwrap();
    let total_frames = 15;
    let mut triangle = Triangle::new(
        Vec2::new(130.0, 150.0),
        Vec2::new(160.0, 150.0),
        Vec2::new(160.0, 200.0),
    );
    let mut time_accumulation = 0;
    for x in 0..total_frames {
        let time1 = Instant::now();
        let file = File::create(format!("frames/image{}.ppm", x)).unwrap();
        let mut app = App::new(
            Resolution {
                width: 480,
                height: 240,
            },
            file,
        );
        let _ = app.write_header();
        draw(&mut app, &mut triangle);
        triangle.rotate(std::f32::consts::TAU / 15.0);
        let time2 = Instant::now();
        let diff = time2 - time1;
        time_accumulation += diff.as_millis();
        println!("Frame {x} completed in {} ms", diff.as_millis());
    }
    println!("Completed in {} ms", time_accumulation);
}

fn draw(app: &mut App, triangle: &Triangle) {
    for i in 0..app.resolution.height {
        for j in 0..app.resolution.width {
            if triangle.contains_point(Vec2::new(i as f32, j as f32)) {
                app.write_to_file(Color::BLUE).unwrap();
            } else {
                app.write_to_file(Color::BLACK).unwrap();
            }
        }
        app.write_newline().unwrap();
    }
}
