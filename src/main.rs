#![allow(warnings)]
use glam::*;
use raylib::ffi::ImageCopy;
use raylib::prelude::*;
use std::fs::File;
use std::fs::create_dir;
use std::fs::remove_dir_all;
use std::io::prelude::*;
use std::time::*;

#[derive(Clone, Copy, Debug)]
struct Header {
    signature: [u8; 2],
    file_size: u32,
    reserved: u32,
    data_offset: u32,
}

#[derive(Clone, Copy, Debug)]
struct InfoHeader {
    size: u32,
    width: u32,
    height: u32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    image_size: u32,
    x_pixels_per_m: u32,
    y_pixels_per_m: u32,
    colors_used: u32,
    important_colors: u32,
}

#[derive(Clone, Debug)]
struct BMP {
    header: Header,
    info_header: InfoHeader,
    pixel_data: Vec<u8>,
}

impl BMP {
    pub fn new(resolution: Resolution) -> Self {
        let header = Header {
            signature: [b'B', b'M'],
            file_size: 58,
            reserved: 0,
            data_offset: 54,
        };
        let info_header = InfoHeader {
            size: 40,
            width: resolution.width,
            height: resolution.height,
            planes: 1,
            bits_per_pixel: 24,
            compression: 0,
            image_size: 0,
            x_pixels_per_m: 2835,
            y_pixels_per_m: 2835,
            colors_used: 0,
            important_colors: 0,
        };

        Self {
            header,
            info_header,
            pixel_data: vec![],
        }
    }

    pub fn set_pixel_data(&mut self, pixel_data: Vec<Color>) {
        let mut count = 0;
        self.pixel_data = vec![];
        let width = self.info_header.width;
        for i in pixel_data {
            self.pixel_data.push(i.b);
            self.pixel_data.push(i.g);
            self.pixel_data.push(i.r);
            count += 1;
            if count % width == 0 && self.pixel_data.len() % 4 != 0 {
                self.pixel_data.push(0x00);
                self.pixel_data.push(0x00);
            }
        }

        // Size of Header + Size of Info Header + Length of Pixel Data
        self.header.file_size = 54 + 4 * self.info_header.height * self.info_header.width;
    }

    pub fn write_to_file(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        let header = self.header;

        let mut buffer = vec![];
        // BMP Header
        buffer.extend_from_slice(&header.signature);
        buffer.extend_from_slice(&header.file_size.to_le_bytes());
        buffer.extend_from_slice(&header.reserved.to_le_bytes());
        buffer.extend_from_slice(&header.data_offset.to_le_bytes());
        // BMP Info Header
        let info_header = self.info_header;
        buffer.extend_from_slice(&info_header.size.to_le_bytes());
        buffer.extend_from_slice(&info_header.width.to_le_bytes());
        buffer.extend_from_slice(&info_header.height.to_le_bytes());
        buffer.extend_from_slice(&info_header.planes.to_le_bytes());
        buffer.extend_from_slice(&info_header.bits_per_pixel.to_le_bytes());
        buffer.extend_from_slice(&info_header.compression.to_le_bytes());
        buffer.extend_from_slice(&info_header.image_size.to_le_bytes());
        buffer.extend_from_slice(&info_header.x_pixels_per_m.to_le_bytes());
        buffer.extend_from_slice(&info_header.y_pixels_per_m.to_le_bytes());
        buffer.extend_from_slice(&info_header.colors_used.to_le_bytes());
        buffer.extend_from_slice(&info_header.important_colors.to_le_bytes());
        // Pixel Data
        let data = self.pixel_data.clone();
        for i in data.iter() {
            buffer.extend_from_slice(&[*i]);
        }
        file.write_all(buffer.as_slice())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Clone, Copy, Debug)]
struct Triangle {
    a: Vec2,
    b: Vec2,
    c: Vec2,
    color: Color,
}

impl Triangle {
    pub fn new(a: Vec2, b: Vec2, c: Vec2, color: Color) -> Self {
        Self { a, b, c, color }
    }

    pub fn area(&self, a: Vec2, b: Vec2, c: Vec2) -> f32 {
        return ((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2.0).abs();
    }

    pub fn contains_point_area(&self, point: Vec2) -> bool {
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

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Debug)]
struct Resolution {
    width: u32,
    height: u32,
}

struct App {
    resolution: Resolution,
    bmp: BMP,
}

impl App {
    pub fn new(resolution: Resolution) -> Self {
        let bmp = BMP::new(resolution);
        Self { resolution, bmp }
    }
}

fn main() {
    let mut app = App::new(Resolution {
        width: 480,
        height: 240,
    });

    let mut triangle1 = Triangle::new(
        Vec2::new(160.0, 120.0),
        Vec2::new(160.0, 180.0),
        Vec2::new(200.0, 120.0),
        Color::BLUE,
    );
    let mut triangle2 = Triangle::new(
        Vec2::new(300.0, 10.0),
        Vec2::new(300.0, 120.0),
        Vec2::new(200.0, 10.0),
        Color::RED,
    );
    let mut triangle3 = Triangle::new(
        Vec2::new(160.0, 180.0),
        Vec2::new(160.0, 240.0),
        Vec2::new(200.0, 120.0),
        Color::GREEN,
    );

    let (mut rl, thread) = raylib::init()
        .size(app.resolution.width as i32, app.resolution.height as i32)
        .title("Software Renderer")
        .resizable()
        .build();

    while !rl.window_should_close() {
        let mut time_accumulation = 0;

        let time1 = Instant::now();
        let data = draw(&mut app, vec![triangle1, triangle2, triangle3]);
        app.bmp.set_pixel_data(data);
        app.bmp.write_to_file("image0.bmp").unwrap();
        println!("{}", app.bmp.pixel_data.len());
        triangle1.rotate(std::f32::consts::TAU / 14.0);
        triangle2.rotate(std::f32::consts::TAU / 300.0);
        triangle3.rotate(std::f32::consts::TAU / 5.0);
        let time2 = Instant::now();
        let diff = time2 - time1;
        time_accumulation += diff.as_millis();
        let mut origin = Image::load_image("frames/image0.bmp").unwrap();
        origin.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        let mut texture = rl.load_texture_from_image(&thread, &origin).unwrap();
        let mut d = rl.begin_drawing(&thread);
        // println!("Frame {x} completed in {} ms", diff.as_millis());
        d.clear_background(raylib::prelude::Color::WHITE);
        d.draw_texture(&texture, 0, 0, raylib::prelude::Color::WHITE);
        // texture.update_texture(&app.bmp.pixel_data).unwrap();
        // println!("Completed in {} ms", time_accumulation);
    }
}

fn draw(app: &mut App, triangles: Vec<Triangle>) -> Vec<Color> {
    let width = app.resolution.width;
    let height = app.resolution.height;
    let mut pixel_data = vec![];
    for i in 0..height {
        for j in 0..width {
            let mut found_color = false;
            for triangle in triangles.iter() {
                if triangle.contains_point_area(Vec2::new(j as f32, i as f32)) {
                    pixel_data.push(triangle.color);
                    found_color = true;
                    break;
                }
            }
            if !found_color {
                pixel_data.push(Color::BLACK);
            }
        }
    }
    pixel_data
}
