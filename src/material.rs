use crate::{Vec3, Ray};

pub trait Material {
    fn scatter(&self, r_in: &Ray, hit: &Vec3, attenuation: &Vec3, scattered: &Vec3);
}