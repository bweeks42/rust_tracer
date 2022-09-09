use std::ops;
use rand::Rng;

// Vectors
#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl Vec3 {
    fn random_unit() -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3 { x: rng.gen_range(0.0..1.0), y: rng.gen_range(0.0..1.0), z: rng.gen_range(0.0..1.0) }
    }

    fn random(min:f64, max:f64) -> Vec3 {
        let mut rng = rand::thread_rng();
        Vec3 { x: rng.gen_range(min..max), y: rng.gen_range(min..max), z: rng.gen_range(min..max) }
    }

    fn random_in_unit_sphere() -> Vec3 {
        loop {
            let p = Vec3::random(-1.0, 1.0);
            if length(&p) * length(&p) >= 1.0 {continue};
            return p
        }
    }
    
    pub fn random_unit_vector() -> Vec3 {
        unit_vector(Vec3::random_in_unit_sphere())
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
    }
}

impl ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {x: self.x * rhs.x, y: self.y * rhs.y, z: self.z * rhs.z}
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Vec3 {
        Vec3 { x: self.x * rhs, y: self.y * rhs, z: self.z * rhs }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Vec3 {
       self * (1.0/rhs)
    }
}

pub fn dot(u: &Vec3, v: &Vec3) -> f64 {
    u.x * v.x + u.y * v.y + u.z * v.z
}

pub fn cross(u: &Vec3, v: &Vec3) -> Vec3 {
    Vec3 { 
        x: u.y * v.z - u.z * v.y, 
        y: u.z * v.x - u.x * v.z, 
        z: u.x * v.y - u.y * v.x 
    }
}

pub fn length(u: &Vec3) -> f64 {
    f64::sqrt(u.x*u.x + u.y*u.y + u.z*u.z)
}

pub fn unit_vector(u: Vec3) -> Vec3 {
    u / length(&u)
}