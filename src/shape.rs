use crate::material::Material;
use crate::{Vec3, Ray, Hit};
use crate::vector::{length, dot, length_squared};
use crate::ray::{ray_at};

// Shapes
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Material
}

pub trait Shape: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, hit: &mut Hit) -> bool;
    fn material(&self) -> Material;
}

impl Shape for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, hit: &mut Hit) -> bool {
        let oc = r.origin - self.center;
        let a = length_squared(&r.direction);
        let half_b = dot(&oc, &r.direction);
        let c = length_squared(&oc) - self.radius*self.radius;
        let discriminant = half_b*half_b - a*c;
        if discriminant < 0.0 {
            return false
        }
        let sqrtd = f64::sqrt(discriminant);
        let mut root = (-half_b - sqrtd) / a;
        if (root < t_min || t_max < root) {
            root = (-half_b + sqrtd) / a;
            if (root < t_min || t_max < root) {
                return false
            }
        }

        hit.t = root;
        hit.point = ray_at(r, hit.t);
        let outward_normal = (hit.point - self.center) / self.radius;
        hit.set_face_normal(r, outward_normal);
        hit.material = Some(self.material);
        true
    }

    fn material(&self) -> Material {
        self.material
    }
}