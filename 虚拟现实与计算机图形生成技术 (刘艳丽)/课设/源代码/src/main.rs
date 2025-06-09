mod bvh;
mod camera;
mod hittable;
mod material;
mod ray;
mod rng;
mod sphere;

use std::fs::File;
use std::iter::repeat_n;
use std::sync::Arc;
#[cfg(not(feature = "benchmark"))]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{
    f32,
    io::{self, Write},
};

use crate::bvh::{BVHNode, Bounded};
use crate::camera::Camera;
use crate::hittable::{Hittable, HittableList};
use crate::material::Material;
use crate::ray::Ray;
use crate::rng::get_rng;
use crate::sphere::Sphere;

use clap::Parser;
use material::Scatter;
use nalgebra::Vector3;
use rand::Rng;
use rand::seq::IndexedRandom;
use rayon::prelude::*;

// 小球材质的比例
const LAMBERTIAN_PROP: usize = 10;
const METAL_PROP: usize = 3;
const DIELECTRIC_PROP: usize = 2;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(name = "ray-tracing")]
#[command(about = "Rust 实现的迷你光线追踪器", long_about = None)]
struct Args {
    /// 图像宽度
    #[arg(long, default_value_t = 1200)]
    nx: usize,

    /// 图像高度
    #[arg(long, default_value_t = 800)]
    ny: usize,

    /// 像素采样率
    #[arg(long, default_value_t = 50)]
    ns: usize,

    /// 最大追踪深度
    #[arg(long, default_value_t = 50)]
    depth: usize,
}

/// 终章的场景
#[allow(unused)]
fn final_scene() -> HittableList {
    let mut rng = get_rng();
    let origin = Vector3::new(4.0, 0.2, 0.0);
    let mut scene = HittableList::default();

    // 地面
    scene.push(Sphere::from(
        Vector3::new(0.0, -1000.0, 0.0),
        1000.0,
        Material::lambertian(Vector3::new(0.5, 0.5, 0.5)),
    ));

    let mut materials_list = vec![];
    materials_list.extend(repeat_n(0, LAMBERTIAN_PROP));
    materials_list.extend(repeat_n(1, METAL_PROP));
    materials_list.extend(repeat_n(2, DIELECTRIC_PROP));

    // 小球
    for a in -11..11 {
        for b in -11..11 {
            let center = Vector3::new(
                a as f32 + 0.9 * rng.random::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.random::<f32>(),
            );

            if (center - origin).magnitude() > 0.9 {
                let material_pick = *materials_list.choose(&mut rng).unwrap();

                let material: Material = if material_pick == 0 {
                    Material::lambertian(Vector3::new(
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>(),
                    ))
                } else if material_pick == 1 {
                    Material::metal(
                        Vector3::new(
                            0.5 * (1.0 + rng.random::<f32>()),
                            0.5 * (1.0 + rng.random::<f32>()),
                            0.5 * (1.0 + rng.random::<f32>()),
                        ),
                        0.5 * rng.random::<f32>(),
                    )
                } else {
                    Material::dielectric(1.5)
                };

                scene.push(Sphere::from(center, 0.2, material));
            }
        }
    }

    // 大球
    scene.push(Sphere::from(
        Vector3::new(0.0, 1.0, 0.0),
        1.0,
        Material::dielectric(1.5),
    ));

    scene.push(Sphere::from(
        Vector3::new(-4.0, 1.0, 0.0),
        1.0,
        Material::lambertian(Vector3::new(0.4, 0.2, 0.1)),
    ));

    scene.push(Sphere::from(
        Vector3::new(4.0, 1.0, 0.0),
        1.0,
        Material::metal(Vector3::new(0.7, 0.6, 0.5), 0.0),
    ));

    scene
}

/// 大球横排场景
#[allow(unused)]
fn lined_up_scene() -> HittableList {
    let mut rng = get_rng();
    let mut scene = HittableList::default();
    let mut list = vec![];

    // 地面
    let plane = Sphere::from(
        Vector3::new(0.0, -1000.0, 0.0),
        1000.0,
        Material::lambertian(Vector3::new(0.5, 0.5, 0.5)),
    );
    scene.push(plane.clone());

    // 大球
    let dielectric = Sphere::from(Vector3::new(0.4, 1.0, 3.2), 1.0, Material::dielectric(1.5));
    scene.push(dielectric.clone());
    list.push(dielectric);

    let lambertian = Sphere::from(
        Vector3::new(0.0, 1.0, 0.0),
        1.0,
        Material::lambertian(Vector3::new(0.0, 0.5, 1.0)),
    );
    scene.push(lambertian.clone());
    list.push(lambertian);

    let metal = Sphere::from(
        Vector3::new(3.2, 1.0, 0.4),
        1.0,
        Material::metal(Vector3::new(1.0, 1.0, 1.0), 0.0),
    );
    scene.push(metal.clone());
    list.push(metal);

    // 小球
    let radius = 0.2;
    let wander = 0.75;
    let edge = 11;
    let mut materials_list = vec![];
    materials_list.extend(repeat_n(0, LAMBERTIAN_PROP));
    materials_list.extend(repeat_n(1, METAL_PROP));
    materials_list.extend(repeat_n(2, DIELECTRIC_PROP));

    for a in -edge..edge {
        'positions: for b in -edge..edge {
            let x = a as f32 + wander * rng.random::<f32>();
            let z = b as f32 + wander * rng.random::<f32>();
            let center = Sphere::correct_center(Vector3::new(x, radius, z), radius, &plane);

            for obj in &list {
                if Sphere::overlaps(center, radius, obj) {
                    continue 'positions;
                }
            }

            let material_pick = *materials_list.choose(&mut rng).unwrap();
            let material: Material = if material_pick == 0 {
                Material::lambertian(Vector3::new(
                    rng.random::<f32>() * rng.random::<f32>(),
                    rng.random::<f32>() * rng.random::<f32>(),
                    rng.random::<f32>() * rng.random::<f32>(),
                ))
            } else if material_pick == 1 {
                Material::metal(
                    Vector3::new(
                        0.5 * (1.0 + rng.random::<f32>()),
                        0.5 * (1.0 + rng.random::<f32>()),
                        0.5 * (1.0 + rng.random::<f32>()),
                    ),
                    0.5 * rng.random::<f32>(),
                )
            } else {
                Material::dielectric(1.5)
            };

            let sphere = Sphere::from(center, radius, material);
            scene.push(sphere.clone());
            list.push(sphere);
        }
    }

    scene
}

fn build_camera(nx: usize, ny: usize) -> Camera {
    let look_from = if cfg!(feature = "benchmark") {
        Vector3::new(13.0, 2.0, 3.0)
    } else {
        Vector3::new(12.0, 2.0, 12.0)
    };
    let look_at = if cfg!(feature = "benchmark") {
        Vector3::new(0.0, 0.0, 0.0)
    } else {
        Vector3::new(0.0, 1.0, 0.0)
    };

    if cfg!(feature = "course") {
        Camera::from_without_focus(
            look_from,
            look_at,
            Vector3::new(0.0, 1.0, 0.0),
            20.0,
            nx as f32 / ny as f32,
        )
    } else {
        let focus_dist = if cfg!(feature = "benchmark") {
            10.0
        } else {
            14.5
        };
        let aperture = 0.1;

        Camera::from(
            look_from,
            look_at,
            Vector3::new(0.0, 1.0, 0.0),
            20.0,
            nx as f32 / ny as f32,
            aperture,
            focus_dist,
        )
    }
}

fn write_image(image: Vec<u8>, nx: usize, ny: usize) -> io::Result<()> {
    eprint!("Writing file...");
    let image = image
        .chunks(3)
        .map(|col| format!("{} {} {}", col[0], col[1], col[2]))
        .collect::<Vec<_>>()
        .join("\n");

    let file_name = if cfg!(feature = "benchmark") {
        "benchmark"
    } else if cfg!(feature = "course") {
        "course"
    } else {
        "result"
    };
    let file_path = format!("{file_name}.ppm");
    writeln!(
        &mut File::create(&file_path)?,
        "P3\n{nx} {ny}\n255\n{image}",
    )?;
    eprintln!("\rFile written{}", " ".repeat(10));

    Ok(())
}

/// 光线颜色
fn ray_color(mut ray: Ray, scene: &impl Hittable, max_depth: usize) -> Vector3<f32> {
    let mut color = Vector3::new(1.0, 1.0, 1.0);

    // 在设定的深度以内
    for _ in 0..max_depth {
        if let Some(hit) = scene.hit(&ray, 0.001, f32::MAX) {
            // 击中: 更新颜色和光线
            if let Some((scattered, attenuation)) = hit.material.scatter(&ray, &hit) {
                color = color.zip_map(&attenuation, |l, r| l * r);
                ray = scattered;
            } else {
                break;
            }
        } else {
            // 未击中: 打到天空, 设为背景颜色
            let unit_direction = ray.direction().normalize();
            let t = 0.5 * (unit_direction[1] + 1.0);
            let sky = (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);

            return color.zip_map(&sky, |l, r| l * r);
        }
    }

    Vector3::zeros()
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let (nx, ny, ns, max_depth) = (args.nx, args.ny, args.ns, args.depth);

    // 构建场景
    eprint!("Constructing scene...");
    let scene_list = if cfg!(feature = "benchmark") {
        final_scene()
    } else {
        lined_up_scene()
    };
    eprintln!("\rScene constructed{}", " ".repeat(10));

    // 构建 BVH
    eprint!("Building BVH...");
    let objects: Vec<_> = scene_list
        .list
        .into_iter()
        .filter_map(|obj| {
            let hittable_ref = obj.as_ref();
            (hittable_ref as &dyn std::any::Any)
                .downcast_ref::<Sphere>()
                .map(|sphere| Arc::new(sphere.clone()) as Arc<dyn Bounded + Sync + Send>)
        })
        .collect();
    let scene = BVHNode::build(objects);
    eprintln!("\rBVH built{}", " ".repeat(10));

    // 构建相机
    let camera = build_camera(nx, ny);

    // gamma 修正闭包
    let correct_gamma = |c: &f32| (255.99 * (c / ns as f32).sqrt().clamp(0.0, 1.0)) as u8;

    // 跟踪渲染进度
    #[cfg(not(feature = "benchmark"))]
    let finished_count = Arc::new(AtomicUsize::new(0));
    let timer = Instant::now();

    // 并行渲染
    let sqrt_ns = (ns as f32).sqrt() as usize;
    let image = (0..ny)
        .into_par_iter()
        .rev()
        .flat_map(|y| {
            let rng = &mut get_rng();

            // 更新进度
            #[cfg(not(feature = "benchmark"))]
            {
                let count = finished_count.fetch_add(1, Ordering::SeqCst) + 1;
                let elapsed = timer.elapsed().as_millis() as usize;
                let avg_speed = elapsed / count;
                let remaining = ny - count;
                eprint!(
                    "\rRemaining: {:>4} | ETA: {:>4}s",
                    remaining,
                    remaining * avg_speed / 1000
                );
            }

            // 渲染
            (0..nx)
                .flat_map(|x| {
                    // 对每个像素进行多次采样
                    let mut col = Vector3::zeros();
                    for sy in 0..sqrt_ns {
                        for sx in 0..sqrt_ns {
                            let u = (x as f32 + (sx as f32 + rng.random::<f32>()) / sqrt_ns as f32)
                                / nx as f32;
                            let v = (y as f32 + (sy as f32 + rng.random::<f32>()) / sqrt_ns as f32)
                                / ny as f32;
                            col += ray_color(camera.camera_ray(u, v), &scene, max_depth);
                        }
                    }

                    // gamma 修正
                    col.iter().map(correct_gamma).collect::<Vec<u8>>()
                })
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();

    eprintln!(
        "\rRendered in {:.1}s{}",
        timer.elapsed().as_secs_f32(),
        " ".repeat(20)
    );

    // 写入结果
    write_image(image, nx, ny)
}
