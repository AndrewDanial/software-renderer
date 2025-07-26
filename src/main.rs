use glam::*;
use raylib::prelude::{Image, PixelFormat, RaylibDraw};
use std::fs::File;
use std::io::prelude::*;
mod meshes;
use meshes::triangle::*;
mod color;
use color::*;

use crate::meshes::Mesh;

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

    pub fn set_pixel_data(&mut self, pixel_data: Vec<crate::color::Color>) {
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
        let mut data = self.pixel_data.clone();
        buffer.append(&mut data);
        file.write_all(buffer.as_slice())?;
        Ok(())
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
        width: 640,
        height: 480,
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
        // draw to image
        let data = draw(&mut app, vec![triangle1, triangle2, triangle3]);
        app.bmp.set_pixel_data(data);
        app.bmp.write_to_file("image0.bmp").unwrap();
        // display image on raylib window
        let mut origin = Image::load_image("image0.bmp").unwrap();
        origin.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        let texture = rl.load_texture_from_image(&thread, &origin).unwrap();
        // show fps in title
        rl.set_window_title(&thread, (1.0 / rl.get_frame_time()).to_string().as_str());
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(raylib::prelude::Color::WHITE);
        d.draw_texture(&texture, 0, 0, raylib::prelude::Color::WHITE);
        // rotate triangles
        triangle1.rotate(0.01);
        triangle2.rotate(std::f32::consts::TAU / 300.0);
        triangle3.rotate(std::f32::consts::TAU / 5.0);
    }
    std::fs::remove_file("image0.bmp").unwrap();
}

fn draw(app: &mut App, triangles: Vec<Triangle>) -> Vec<Color> {
    let width = app.resolution.width;
    let height = app.resolution.height;
    let mut pixel_data = vec![Color::BLACK; (width * height) as usize];
    for triangle in triangles.iter() {
        for i in (triangle.aabb.min_y as usize)..(triangle.aabb.max_y as usize) {
            for j in (triangle.aabb.min_x as usize)..(triangle.aabb.max_x as usize) {
                let index = i * width as usize + j;
                if triangle.contains_point(Vec2::new(j as f32, i as f32)) {
                    pixel_data[index] = triangle.color;
                }
            }
        }
    }

    pixel_data
}
