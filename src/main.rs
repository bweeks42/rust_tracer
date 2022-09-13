use std::io::Write;
use std::fs::File;
use rand::{Rng, thread_rng};
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::sync::{Arc};
use std::env;

mod vector;
use vector::{Vec3, dot, unit_vector, length};

mod material;
use material::{Material, scatter_for_material};

mod shape;
use shape::{Shape, Sphere};

mod camera;
use camera::Camera;

mod ray;
use ray::{Ray};

use crate::material::MaterialType;

type ShapeList = Vec<Box<dyn Shape>>;

// Hit
#[derive(Default, Clone)]
pub struct Hit {
    point: Vec3,
    normal: Vec3,
    t: f64,
    front_face: bool,
    material: Option<Material>
}

impl Hit {
    fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = dot(&r.direction, &outward_normal) < 0.0;
        self.normal = if self.front_face {outward_normal} else {-outward_normal}
    }
}


fn hit_in_list(l: &ShapeList, r: &Ray, t_min: f64, t_max: f64, hit_record: &mut Hit) -> bool { 
    let mut hit_anything = false;
    let mut closest = t_max;
    let mut temp_hit = Default::default();

    for shape in l { 
        if (shape.hit(r, t_min, closest, &mut temp_hit)) {
            hit_anything = true;
            closest = temp_hit.t;
            
            // move values to hit record
            hit_record.point = temp_hit.point;
            hit_record.normal = temp_hit.normal;
            hit_record.t = temp_hit.t;
            hit_record.front_face = temp_hit.front_face;
            hit_record.material = temp_hit.material;
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
    let mut hit: Hit = Default::default();
    
    if depth <= 0 {
        return Color {x: 0.0, y: 0.0, z: 0.0}
    }
    if hit_in_list(v, r, 0.001, f64::INFINITY, &mut hit) {
        let mut scatter: Ray = Default::default();
        let mut color: Color = Default::default();
        if scatter_for_material(hit.material.unwrap(), r, &hit, &mut color, &mut scatter) {
            return color * ray_color(&scatter, v, depth - 1);
        }
        return Color {x: 0.0, y: 0.0, z: 0.0}
    }
    let unit_direction = unit_vector(r.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    let a = Vec3{x: 1.0, y: 1.0, z: 1.0} * (1.0-t);
    let b = Vec3{x: 0.5, y: 0.7, z: 1.0} * t;
    a + b
}

fn random_double() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..=1.0)
}

fn random_shapes() -> ShapeList {
    // Set ground
    let mut shape_list = ShapeList::new();
    let ground_material = Material {
        material_type: MaterialType::Matte, 
        color: Vec3 { x: 0.5, y: 0.5, z: 0.5 },
        fuzz: 0.0,
        refraction_index: 0.0
    };
    shape_list.push(
        Box::new(Sphere {
            center: Vec3 {x: 0.0, y: -1000.0, z: 0.0},
            radius: 1000.0,
            material: ground_material
        })
    );

    // Random shapes
    for a in -11..11 {
        for b in -11..11 {
            let mat = random_double();
            let center = Vec3 {x: a as f64 + 0.9*random_double(), y:0.2, z:b as f64 + 0.9*random_double()};
            let k = Vec3 {x: 4.0, y: 0.2, z: 0.0};
            if length(&(center - k)) > 0.9 {
                let material: Material = if mat < 0.8 {
                    // matte
                    let mut color = Color {x: random_double(), y: random_double(), z: random_double()};
                    color = color * color;
                    Material {
                        material_type: MaterialType::Matte,
                        color: color,
                        fuzz: 0.0,
                        refraction_index: 0.0
                    }
                } else if mat < 0.95 {
                    // metal
                    let mut rng = rand::thread_rng();
                    let color = Color {
                        x: rng.gen_range(0.0..0.5),
                        y: rng.gen_range(0.0..0.5),
                        z: rng.gen_range(0.0..0.5)
                    };
                    let fuzz = rng.gen_range(0.0..0.5);
                    Material {
                        material_type: MaterialType::Metal,
                        color: color,
                        fuzz: fuzz,
                        refraction_index: 0.0
                    }
                } else {
                    // glass
                    Material {
                        material_type: MaterialType::Dialectric,
                        color: Default::default(),
                        fuzz: 0.0,
                        refraction_index: 1.5
                    }
                };
                shape_list.push(Box::new(Sphere {
                    center: center,
                    radius: 0.2,
                    material: material
                }))
            }
        }
    }

    // Big 3
    let glass = Material {
        material_type: MaterialType::Dialectric,
        color: Default::default(),
        fuzz: 0.0,
        refraction_index: 1.5
    };
    shape_list.push(Box::new(Sphere {
        center: Vec3 {x: 0.0, y: 1.0, z: 0.0},
        radius: 1.0,
        material: glass
    }));

    let matte = Material {
        material_type: MaterialType::Matte,
        color: Color {x: 0.4, y: 0.2, z: 0.1},
        fuzz: 0.0,
        refraction_index: 0.0
    };
    shape_list.push(Box::new(Sphere {
        center: Vec3 { x: -4.0, y: 1.0, z: 0.0 },
        radius: 1.0,
        material: matte
    }));

    let metal = Material {
        material_type: MaterialType::Metal,
        color: Color {x: 0.7, y: 0.6, z: 0.5},
        fuzz: 0.0,
        refraction_index: 0.0
    };
    shape_list.push(Box::new(Sphere {
        center: Vec3 {x: 4.0, y: 1.0, z: 0.0},
        radius: 1.0,
        material: metal
    }));

    shape_list
}

fn main() {
    
    // Args
    let args: Vec<String> = env::args().collect();
    let mut pixels:i64 = 1000;
    if args.len() == 3 && args[1] == "-p" {
        pixels = args[2].parse().unwrap();
    }

    // Image
    let aspect_ratio = 16.0/10.0;//16.0 / 9.0;
    let image_width = pixels;
    let image_height = (image_width as f64 / aspect_ratio) as i64;
    let samples_per_pixel = 500;
    let max_depth = 50;

    // Camera
    let origin = Vec3 { x: 13.0, y: 2.0, z: 3.0 };
    let look_at = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let vup = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    let vfov = 20.0;
    let distance_to_focus = 10.0;//length(&(origin - look_at));
    let aperture = 0.1;
    
    let camera = Camera::new(
        origin,
        look_at,
        vup,
        vfov,
        aspect_ratio,
        aperture,
        distance_to_focus
    );

    // // Materials
    // let ground_material = Material {material_type: MaterialType::Matte, color: Color {x: 0.8, y: 0.8, z: 0.0}, fuzz: 0.0, refraction_index: 0.0};
    // let material_center = Material {material_type: MaterialType::Matte, color: Color {x: 0.1, y: 0.2, z: 0.5}, fuzz: 0.0, refraction_index: 0.0};
    // let material_left = Material {material_type: MaterialType::Dialectric, color: Color {x: 1.0, y: 1.0, z: 1.0}, fuzz: 0.03, refraction_index: 1.5};
    // let material_right = Material {material_type: MaterialType::Metal, color: Color {x: 0.8, y: 0.6, z: 0.2}, fuzz: 0.05, refraction_index: 0.0};

    // // World contents
    // let world: Arc<ShapeList> = Arc::new(
    //     vec![
    //         Box::new(Sphere{
    //             center: Vec3 { x: 0.0, y: 0.0, z: -1.0 },
    //             radius: 0.5,
    //             material: material_center 
    //         }),
    //         Box::new(Sphere{
    //             center: Vec3 { x: 0.0, y: -100.5, z: -2.0 },
    //             radius: 100.0,
    //             material: ground_material 
    //         }),
    //         Box::new(Sphere{
    //             center: Vec3 { x: -1.0, y: 0.0, z: -1.0 },
    //             radius: 0.5,
    //             material: material_left
    //         }),
    //         Box::new(Sphere{
    //             center: Vec3 { x: -1.0, y: 0.0, z: -1.0 },
    //             radius: -0.45,
    //             material: material_left
    //         }),
    //         Box::new(Sphere{
    //             center: Vec3 { x: 1.0, y: 0.0, z: -1.0 },
    //             radius: 0.5,
    //             material: material_right
    //         })
    //     ]
    // );
    let world = Arc::new(random_shapes());
        
        

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
        let cam = camera.clone();
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
                    let ray = cam.get_ray(u, v);
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
