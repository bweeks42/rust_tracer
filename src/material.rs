use crate::{Vec3, Ray, Hit, Color, vector::{reflect, unit_vector, dot, refract}};
use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub enum MaterialType {
    Matte,
    Metal,
    Dialectric
}

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub material_type: MaterialType,
    pub color: Color,
    pub fuzz: f64,
    pub refraction_index: f64
}

pub fn scatter_for_material(m: Material, ray_in: &Ray, hit: &Hit, color: &mut Color, scatter: &mut Ray) -> bool {
    match m.material_type {
        MaterialType::Matte => {
            let mut scatter_direction = hit.normal + Vec3::random_unit_vector();
            if scatter_direction.near_zero() {
                scatter_direction = hit.normal;
            }
            scatter.origin = hit.point;
            scatter.direction = scatter_direction;
            color.x = m.color.x;
            color.y = m.color.y;
            color.z = m.color.z;

            true
        },
        MaterialType::Metal => {
            let reflected = reflect(unit_vector(ray_in.direction), hit.normal);
            scatter.origin = hit.point;
            scatter.direction = reflected + Vec3::random_in_unit_sphere()*m.fuzz;
            color.x = m.color.x;
            color.y = m.color.y;
            color.z = m.color.z; 
            dot(&scatter.direction, &hit.normal) > 0.0
        },
        MaterialType::Dialectric => {
            color.x = 1.0;
            color.y = 1.0;
            color.z = 1.0;
            
            let refraction_ratio = if hit.front_face {1.0/m.refraction_index} else {m.refraction_index};
            
            let unit_direction = unit_vector(ray_in.direction);
            let cos_theta = f64::min(dot(&(-unit_direction), &hit.normal), 1.0);
            let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();
            
            let cannot_refract = (refraction_ratio * sin_theta) > 1.0;
            let mut rng = rand::thread_rng();
            let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.gen_range(0.0..1.0)  {
                reflect(unit_direction, hit.normal)
            } else {
                refract(unit_direction, hit.normal, refraction_ratio)
            };

            scatter.origin = hit.point;
            scatter.direction = direction;

            true 
        }
    }
}

fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0*r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}