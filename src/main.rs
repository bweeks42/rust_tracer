use std::io::Write;
use std::fs::File;
use rand::{Rng};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::sync::{Arc};
use std::env;

mod vector;
use vector::{Vec3, dot, unit_vector};

mod material;
use material::{Material, scatter_for_material};

mod shape;
use shape::{Shape, Sphere};

mod ray;
use ray::{Ray};

use crate::material::MaterialType;

// Hit
pub struct Hit {
    point: Vec3,
    normal: Vec3,
    t: f64,
    front_face: bool,
    material: Option<Material>
}

impl Hit {
    fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        let front_face = dot(&r.direction, &outward_normal) < 0.0;
        self.normal = if front_face {outward_normal} else {outward_normal * -1.0}
    }
}


fn hit_in_list(l: &Vec<Box<dyn Shape>>, r: &Ray, t_min: f64, t_max: f64, hit: &mut Hit) -> bool {
    let mut temp_hit = Hit {
        point: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        normal: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        t: 0.0,
        front_face: false,
        material: None
    };
    let mut hit_anything = false;
    let mut closest = t_max;

    for shape in l {
        if (shape.hit(r, t_min, closest, &mut temp_hit)) {
            hit_anything = true;
            closest = temp_hit.t;
            
            hit.front_face = temp_hit.front_face;
            hit.normal = temp_hit.normal;
            hit.point = temp_hit.point;
            hit.t = temp_hit.t;
            hit.material = Some(shape.material())
        }
    }
    hit_anything
}

// Color
type Color = Vec3;
fn to_color(u: &Color, n_samples: i64) -> String {
    let mut r = u.x;
    let mut g = u.y;
    let mut b = u.z;

    // Sample
    let scale = 1.0 / n_samples as f64;
    r = f64::sqrt(r * scale);
    g = f64::sqrt(g * scale);
    b = f64::sqrt(b * scale);

    let ir = (256.0 * r.clamp(0.0, 0.999)) as i64;
    let ig = (256.0 * g.clamp(0.0, 0.999)) as i64;
    let ib = (256.0 * b.clamp(0.0, 0.999)) as i64;

    format!("{} {} {}\n", ir, ig, ib)
}

fn ray_color(r: &Ray, v: &Vec<Box<dyn Shape>>, depth: i64) -> Color {
    if (depth <= 1) {
        return Color {x: 0.0, y: 0.0, z: 0.0}
    }
    let mut hit = Hit {
        point: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        normal: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        t: 0.0,
        front_face: false,
        material: None
    };
    if (hit_in_list(v, r, 0.001, f64::MAX, &mut hit)) {
        let (did_scatter, color, rn) = scatter_for_material(hit.material.unwrap(), r, &mut hit);
        if did_scatter {
            return color * ray_color(&rn, v, depth - 1) * 0.5;
        }
        return Color {x: 0.0, y: 0.0, z: 0.0}
    }
    let unit_direction = unit_vector(r.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    let a = Vec3{x: 1.0, y: 1.0, z: 1.0} * (1.0-t);
    let b = Vec3{x: 0.5, y: 0.7, z: 1.0} * t;
    a + b
}

fn main() {
    
    // Args
    let args: Vec<String> = env::args().collect();
    let mut pixels:i64 = 1000;
    if args.len() == 3 && args[1] == "-p" {
        pixels = args[2].parse().unwrap();
    }

    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = pixels;
    let image_height = (image_width as f64 / aspect_ratio) as i64;
    let samples_per_pixel = 100;
    let max_depth = 50;

    // Camera
    let view_height = 2.0;
    let view_width = aspect_ratio * view_height;
    let focal_length = 1.0;
    let origin = Vec3{x: 0.0, y: 0.0, z: 0.0};
    let horizontal = Vec3{x: view_width, y: 0.0, z: 0.0};
    let vertical = Vec3{x: 0.0, y:view_height, z: 0.0};
    let lower_left = origin - (horizontal/2.0) - (vertical/2.0) - Vec3{x: 0.0, y: 0.0, z: focal_length};

    // Materials
    let ground_material = Material {material_type: MaterialType::Matte, color: Color {x: 0.8, y: 0.8, z: 0.0}, fuzz: 0.0};
    let material_center = Material {material_type: MaterialType::Matte, color: Color {x: 0.7, y: 0.3, z: 0.3}, fuzz: 0.0};
    let material_left = Material {material_type: MaterialType::Metal, color: Color {x: 0.8, y: 0.8, z: 0.8}, fuzz: 0.3};
    let material_right = Material {material_type: MaterialType::Metal, color: Color {x: 0.8, y: 0.6, z: 0.2}, fuzz: 0.8};

    // World contents
    let world: Arc<Vec<Box<dyn Shape>>> = Arc::new(
        vec![
            Box::new(Sphere{
                center: Vec3 { x: 0.0, y: 0.0, z: -1.0 },
                radius: 0.5,
                material: material_center 
            }),
            Box::new(Sphere{
                center: Vec3 { x: 0.0, y: -100.5, z: -2.0 },
                radius: 100.0,
                material: ground_material 
            }),
            Box::new(Sphere{
                center: Vec3 { x: -1.0, y: 0.0, z: -1.0 },
                radius: 0.5,
                material: material_left
            }),
            Box::new(Sphere{
                center: Vec3 { x: 1.0, y: 0.0, z: -1.0 },
                radius: 0.5,
                material: material_right
            }),

        ]
    );
        
        

    // File
    let mut file = File::create("image.ppm").unwrap();
    let header = format!("P3\n{} {}\n255\n", image_width, image_height);
    file.write_all(header.as_bytes());

    // Threadpool
    let n_workers = 128;
    let pool = ThreadPool::new(n_workers);
    let (tx, rx) = channel();

    // Render
    for j in 0..image_height {
        let tx = tx.clone();
        let world = world.clone();
        pool.execute(move|| {    
            println!("{}/{}", j+1, image_height);
            let mut row: Vec<Vec3> = Vec::with_capacity(image_width as usize);
            let mut rng = rand::thread_rng();
            let inv_j = (image_height) -1 - j;
            for i in 0..image_width {
                let mut pixel_color = Color {x: 0.0, y: 0.0, z: 0.0};
                for _ in 0..samples_per_pixel {
                    let u = ((i as f64) + rng.gen_range(0.0..1.0)) / (image_width - 1) as f64;
                    let v = ((inv_j as f64) + rng.gen_range(0.0..1.0)) / (image_height -1) as f64;
                    let ray = Ray {origin: origin, direction: lower_left + horizontal*u + vertical*v - origin};
                    let s_color = ray_color(&ray, &world, max_depth);
                    pixel_color = pixel_color + s_color;
                }
                row.push(pixel_color);
            }
            let mut rows = String::new();
            for c in row {
                rows.push_str(to_color(&c, samples_per_pixel).as_str());
            }
            tx.send((inv_j, rows)).expect("Oops!");
        });
    }
    
    let mut ordered:Vec<(i64, String)> = Vec::with_capacity(image_height as usize);
    rx.iter().take(image_height as usize).for_each(|(i, v)| {
        ordered.push((i, v));
    });
    
    println!("Sorting image rows.");
    ordered.sort_by(| a, b| {
        b.0.cmp(&a.0)
    });

    println!("Writing to file");
    for o in ordered {
        file.write_all(o.1.as_bytes());
    }
}
