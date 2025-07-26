use glam::*;
pub mod aabb;
pub mod triangle;
pub trait Mesh {
    fn contains_point(&self, point: Vec2) -> bool;
}
