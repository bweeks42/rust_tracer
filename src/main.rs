use std::io::Write;
use std::ops;
use std::fs::File;
use rand::Rng;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

// Hit
struct Hit {
    point: Vec3,
    normal: Vec3,
    t: f64,
    front_face: bool
}

impl Hit {
    fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        let front_face = dot(&r.direction, &outward_normal) < 0.0;
        self.normal = if front_face {outward_normal} else {outward_normal * -1.0}
    }
}

trait Shape: Send + Sync {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, hit: &mut Hit) -> bool;
}

// Shapes

struct Sphere {
    center: Vec3,
    radius: f64
}


impl Shape for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, hit: &mut Hit) -> bool {
        let oc = r.origin - self.center;
        let a = length(&r.direction) * length(&r.direction);
        let half_b = dot(&oc, &r.direction);
        let c = length(&oc) * length(&oc) - self.radius*self.radius;
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



        true
    }
}


// Vectors
#[derive(Copy, Clone, Debug)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64
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

fn dot(u: &Vec3, v: &Vec3) -> f64 {
    u.x * v.x + u.y * v.y + u.z * v.z
}

fn cross(u: &Vec3, v: &Vec3) -> Vec3 {
    Vec3 { 
        x: u.y * v.z - u.z * v.y, 
        y: u.z * v.x - u.x * v.z, 
        z: u.x * v.y - u.y * v.x 
    }
}

fn length(u: &Vec3) -> f64 {
    f64::sqrt(u.x*u.x + u.y*u.y + u.z*u.z)
}

fn unit_vector(u: Vec3) -> Vec3 {
    u / length(&u)
}

// Rays
struct Ray {
    origin: Vec3,
    direction: Vec3
}


fn ray_at(r: &Ray, t: f64) -> Vec3 {
    r.origin + r.direction * t
}


// Color
type Color = Vec3;
fn to_color(u: &Color, n_samples: i64) -> String {
    let mut r = u.x;
    let mut g = u.y;
    let mut b = u.z;

    // Sample
    let scale = 1.0 / n_samples as f64;
    r *= scale;
    g *= scale;
    b *= scale;

    let ir = (256.0 * r.clamp(0.0, 0.999)) as i64;
    let ig = (256.0 * g.clamp(0.0, 0.999)) as i64;
    let ib = (256.0 * b.clamp(0.0, 0.999)) as i64;

    format!("{} {} {}\n", ir, ig, ib)
}

fn hit_in_list(l: &Vec<Box<dyn Shape>>, r: &Ray, t_min: f64, t_max: f64, hit: &mut Hit) -> bool {
    let mut temp_hit = Hit {
        point: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        normal: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        t: 0.0,
        front_face: false
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
        }
    }


    hit_anything
}

// Tracing
fn ray_color(r: &Ray, v: &Vec<Box<dyn Shape>>) -> Color {
    let mut hit = Hit {
        point: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        normal: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        t: 0.0,
        front_face: false
    };
    if (hit_in_list(v, r, 0.0, f64::MAX, &mut hit)) {
        return (hit.normal + Color {x: 1.0, y: 1.0, z: 1.0}) * 0.5;
    }
    let unit_direction = unit_vector(r.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    let a = Vec3{x: 1.0, y: 1.0, z: 1.0} * (1.0-t);
    let b = Vec3{x: 0.5, y: 0.7, z: 1.0} * t;
    a + b
}

fn write_color(f: &mut File, c: Color, samples: i64) {
    f.write_all(to_color(&c, samples).as_bytes());
}

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 5000;
    let image_height = (image_width as f64 / aspect_ratio) as i64;
    let samples_per_pixel = 100;

    // Camera
    let view_height = 2.0;
    let view_width = aspect_ratio * view_height;
    let focal_length = 1.0;
    let origin = Vec3{x: 0.0, y: 0.0, z: 0.0};
    let horizontal = Vec3{x: view_width, y: 0.0, z: 0.0};
    let vertical = Vec3{x: 0.0, y:view_height, z: 0.0};
    let lower_left = origin - (horizontal/2.0) - (vertical/2.0) - Vec3{x: 0.0, y: 0.0, z: focal_length};

    // World contents
    let world: Arc<Vec<Box<dyn Shape>>> = Arc::new(
        vec![
            Box::new(Sphere{
                center: Vec3 { x: 0.0, y: 0.0, z: -1.0 },
                radius: 0.5
            })
        ]
    );
        
        

    // File
    let mut file = File::create("image.ppm").unwrap();
    let header = format!("P3\n{} {}\n255\n", image_width, image_height);
    file.write_all(header.as_bytes());

    // Threadpool
    let n_workers = 32;
    let pool = ThreadPool::new(n_workers);
    let (tx, rx) = channel();

    // Render
    for j in 0..image_height {
        let tx = tx.clone();
        let world = world.clone();
        pool.execute(move|| {    
            println!("{}/{}", j, image_height);
            let mut row: Vec<Vec3> = Vec::with_capacity(image_width as usize);
            let mut rng = rand::thread_rng();
            let inv_j = (image_height) -1 - j;
            for i in 0..image_width {
                let mut pixel_color = Color {x: 0.0, y: 0.0, z: 0.0};
                for _ in 0..samples_per_pixel {
                    let u = ((i as f64) + rng.gen_range(0.0..1.0)) / (image_width - 1) as f64;
                    let v = ((inv_j as f64) + rng.gen_range(0.0..1.0)) / (image_height -1) as f64;
                    let ray = Ray {origin: origin, direction: lower_left + horizontal*u + vertical*v - origin};
                    let s_color = ray_color(&ray, &world);
                    pixel_color = pixel_color + s_color;
                }
                row.push(pixel_color);
            }
            tx.send((inv_j, row)).expect("Oops!");
        });
    }
    
    let mut ordered:Vec<(i64, Vec<Vec3>)> = Vec::with_capacity(image_height as usize);
    rx.iter().take(image_height as usize).for_each(|(i, v)| {
        ordered.push((i, v));
    });
    
    println!("Sorting image rows.");
    ordered.sort_by(| a, b| {
        b.0.cmp(&a.0)
    });

    println!("Constructing RGB String");
    let mut outs = String::new();
    for row in ordered {
        for c in row.1 {
            outs.push_str(to_color(&c, samples_per_pixel).as_str());
        }
    }

    println!("Writing to file");
    file.write_all(outs.as_bytes());
}
