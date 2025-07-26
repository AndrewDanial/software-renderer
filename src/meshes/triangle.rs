use crate::color::Color;
use crate::meshes::Mesh;
use crate::meshes::aabb::Aabb;
use glam::*;
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
    pub color: Color,
    pub aabb: Aabb,
}

impl Mesh for Triangle {
    fn contains_point(&self, point: Vec2) -> bool {
        let a = self.area(self.a, self.b, self.c);
        let a1 = self.area(point, self.b, self.c);
        let a2 = self.area(self.a, point, self.c);
        let a3 = self.area(self.a, self.b, point);
        return (a1 + a2 + a3 - a).abs() < 0.01;
    }
}

impl Triangle {
    pub fn new(a: Vec2, b: Vec2, c: Vec2, color: Color) -> Self {
        let mut triangle = Triangle {
            a,
            b,
            c,
            color,
            aabb: Aabb::default(),
        };
        triangle.calculate_aabb();
        triangle
    }

    pub fn area(&self, a: Vec2, b: Vec2, c: Vec2) -> f32 {
        return ((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2.0).abs();
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
        self.calculate_aabb();
    }

    pub fn calculate_aabb(&mut self) {
        fn min(values: &Vec<f32>) -> f32 {
            values.iter().fold(f32::INFINITY, |a, &b| a.min(b))
        }
        fn max(values: &Vec<f32>) -> f32 {
            values.iter().fold(f32::MIN, |a, &b| a.max(b))
        }
        let a = self.a.clone();
        let b = self.b.clone();
        let c = self.c.clone();
        let min_x = min(&vec![a.x, b.x, c.x]);
        let min_y = min(&vec![a.y, b.y, c.y]);
        let max_x = max(&vec![a.x, b.x, c.x]);
        let max_y = max(&vec![a.y, b.y, c.y]);

        let aabb = Aabb {
            min_x: min_x as f32,
            min_y: min_y as f32,
            max_x: max_x as f32,
            max_y: max_y as f32,
        };
        self.aabb = aabb;
    }
}
