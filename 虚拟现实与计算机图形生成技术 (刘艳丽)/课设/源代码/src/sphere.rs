use crate::bvh::{AaBb, Bounded};
use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;

use nalgebra::Vector3;

/// 球体
#[derive(Clone)]
pub struct Sphere {
    /// 球心
    center: Vector3<f32>,

    /// 半径
    radius: f32,

    /// 材质
    material: Material,
}

impl Sphere {
    pub const fn from(center: Vector3<f32>, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    /// 球体是否重合
    pub fn overlaps(center: Vector3<f32>, radius: f32, other: &Self) -> bool {
        let d = center - other.center;

        d.magnitude() < radius + other.radius
    }

    /// 修正小球高度
    pub fn correct_center(center: Vector3<f32>, radius: f32, plane: &Self) -> Vector3<f32> {
        let d = plane.center - center;
        let dist = d.magnitude();
        let exp = radius + plane.radius;

        let diff = dist - exp;

        if diff <= 0.0 {
            center
        } else {
            center + (diff / dist) * d
        }
    }
}

impl Hittable for Sphere {
    /// 光线与球体相交
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // 光线起点到球心的向量
        let oc = ray.origin() - self.center;

        // 方程系数
        let d = ray.direction();
        let a = d.dot(&d);
        let b = oc.dot(&d);
        let c = oc.dot(&oc) - self.radius * self.radius;

        // 判定式
        let disc = b.powi(2) - a * c;

        if disc > 0.0 {
            let sqrt_disc = disc.sqrt();

            // 交点 1
            let t = (-b - sqrt_disc) / a;
            if t > t_min && t < t_max {
                let p = ray.point_at_t(t);
                let normal = (p - self.center) / self.radius;

                return Some(HitRecord {
                    distance: t,
                    position: p,
                    normal,
                    material: self.material,
                });
            }

            // 交点 2
            let t = (-b + sqrt_disc) / a;
            if t > t_min && t < t_max {
                let p = ray.point_at_t(t);
                let normal = (p - self.center) / self.radius;

                return Some(HitRecord {
                    distance: t,
                    position: p,
                    normal,
                    material: self.material,
                });
            }
        }

        None
    }
}

impl Bounded for Sphere {
    fn bounding_box(&self) -> AaBb {
        let min = self.center - Vector3::new(self.radius, self.radius, self.radius);
        let max = self.center + Vector3::new(self.radius, self.radius, self.radius);

        AaBb { min, max }
    }
}
