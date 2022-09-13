use crate::{vector::{Vec3, unit_vector, cross, length_squared}, ray::Ray};
use std::f64::consts::PI;
use rand::Rng;

fn degrees_to_radians(d: f64) -> f64 {
    d * PI / 180.0
}

fn random_unit_in_disk() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let p = Vec3 {x: rng.gen_range(-1.0..1.0), y: rng.gen_range(-1.0..1.0), z: 0.0};
        if length_squared(&p) >= 1.0 {continue;}
        return p
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    origin: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    llc: Vec3,
    lens_radius: f64,
    u: Vec3,
    v: Vec3,
    w: Vec3
}

impl Camera {
    pub fn new(look_from: Vec3, look_at: Vec3, vup: Vec3, vfov: f64, aspect_ratio: f64, aperture: f64, focus_dist: f64) -> Self {
        let theta = degrees_to_radians(vfov);
        let h = (theta/2.0).tan();
        let view_height = 2.0 * h;
        let view_width = aspect_ratio * view_height;

        let w = unit_vector(look_from - look_at);
        let u = unit_vector(cross(&vup, &w));
        let v = cross(&w, &u);


        let origin = look_from;
        let horizontal = u * view_width * focus_dist;
        let vertical = v * view_height * focus_dist;
        let llc = origin - horizontal/2.0 - vertical/2.0 - w*focus_dist;
        let lens_radius = aperture / 2.0;

        Camera {
            origin: origin,
            horizontal: horizontal,
            vertical: vertical,
            llc: llc,
            lens_radius: lens_radius,
            u: u,
            v: v,
            w: w
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = random_unit_in_disk() * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;
        Ray {
            origin: self.origin + offset,
            direction: self.llc + self.horizontal*s + self.vertical*t - self.origin - offset
        }
    }
}