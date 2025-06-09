use crate::ray::Ray;

use nalgebra::Vector3;
use rand::Rng;
use std::f32;

/// 在单位圆内随机采样一点
fn random_in_unit_disk() -> Vector3<f32> {
    let mut rng = rand::rng();
    let theta = rng.random_range(0.0..std::f32::consts::PI * 2.0);
    let r = rng.random::<f32>().cbrt();

    Vector3::new(r * theta.sin(), r * theta.cos(), 0.0)
}

/// 相机
pub struct Camera {
    /// 位置
    origin: Vector3<f32>,

    /// 像平面的左下角
    lower_left_corner: Vector3<f32>,

    /// 像平面的水平向量
    horizontal: Vector3<f32>,

    /// 像平面的垂直向量
    vertical: Vector3<f32>,

    /// 相机的右向量
    u: Vector3<f32>,

    /// 相机的上向量 (vup)
    v: Vector3<f32>,

    /// 镜头半径 (景深)
    lens_radius: f32,
}

impl Camera {
    #[allow(unused)]
    pub fn from(
        look_from: Vector3<f32>,
        look_at: Vector3<f32>,
        view_up: Vector3<f32>,
        vertical_fov: f32,
        aspect: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Self {
        // 像平面的高度和宽度
        let theta = vertical_fov.to_radians();
        let half_height = focus_dist * f32::tan(theta / 2.0);
        let half_width = aspect * half_height;

        // 相机坐标系
        let w = (look_from - look_at).normalize();
        let u = view_up.cross(&w).normalize();
        let v = w.cross(&u);

        Self {
            origin: look_from,
            lower_left_corner: look_from - half_width * u - half_height * v - focus_dist * w,
            horizontal: 2.0 * half_width * u,
            vertical: 2.0 * half_height * v,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }

    /// 不带有景深的相机
    #[allow(unused)]
    pub fn from_without_focus(
        look_from: Vector3<f32>,
        look_at: Vector3<f32>,
        view_up: Vector3<f32>,
        vertical_fov: f32,
        aspect: f32,
    ) -> Self {
        // 像平面的高度和宽度
        let theta = vertical_fov.to_radians();
        let half_height = f32::tan(theta / 2.0);
        let half_width = aspect * half_height;

        // 相机坐标系
        let w = (look_from - look_at).normalize();
        let u = view_up.cross(&w).normalize();
        let v = w.cross(&u);

        Self {
            origin: look_from,
            lower_left_corner: look_from - half_width * u - half_height * v - w,
            horizontal: 2.0 * half_width * u,
            vertical: 2.0 * half_height * v,
            u,
            v,
            lens_radius: 0.0,
        }
    }

    /// 从相机发出光线
    pub fn camera_ray(&self, s: f32, t: f32) -> Ray {
        // 在镜头平面上采样
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;

        // 从镜头平面采样点到像平面采样点的光线
        Ray::from(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}
