use crate::Vec3;

// Rays
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3
}


pub fn ray_at(r: &Ray, t: f64) -> Vec3 {
    r.origin + r.direction * t
}