use crate::{Vec3, Ray, Hit, Color, vector::{reflect, unit_vector, dot}};

#[derive(Copy, Clone, Debug)]
pub enum MaterialType {
    Matte,
    Metal
}

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub material_type: MaterialType,
    pub color: Color,
    pub fuzz: f64
}

pub fn scatter_for_material(m: Material, ray_in: &Ray, hit: &mut Hit) -> (bool, Vec3, Ray) {
    match m.material_type {
        MaterialType::Matte => {
            let mut scatter_direction = hit.normal + Vec3::random_unit_vector();
            if scatter_direction.near_zero() {
                scatter_direction = hit.normal;
            }
            let scattered = Ray {origin: hit.point, direction: scatter_direction};
            (true, m.color, scattered)
        },
        MaterialType::Metal => {
            let reflected = reflect(unit_vector(ray_in.direction), hit.normal);
            let scattered = Ray {origin: hit.point, direction: reflected + Vec3::random_in_unit_sphere()*m.fuzz};
            let ok = dot(&scattered.direction, &hit.normal) > 0.0;
            (ok, m.color, scattered)
        }
    }
}