use crate::hittable::HitRecord;
use crate::ray::Ray;

use nalgebra::Vector3;
use rand::Rng;

/// 在单位球内随机采样一点
fn random_in_unit_sphere() -> Vector3<f32> {
    let mut rng = rand::rng();
    let theta = rng.random_range(0.0..std::f32::consts::PI * 2.0);
    let phi = rng.random_range(0.0..std::f32::consts::PI);
    let r = rng.random::<f32>().cbrt();

    Vector3::new(
        r * theta.sin() * phi.cos(),
        r * theta.sin() * phi.sin(),
        r * theta.cos(),
    )
}

/// 反射向量
fn reflect(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * v.dot(n) * n
}

/// 折射向量
fn refract(v: &Vector3<f32>, n: &Vector3<f32>, ni_over_nt: f32) -> Option<Vector3<f32>> {
    let uv = v.normalize();
    let dt = uv.dot(n);
    let disc = 1.0 - ni_over_nt.powi(2) * (1.0 - dt.powi(2));

    if disc > 0.0 {
        Some(ni_over_nt * (uv - n * dt) - n * disc.sqrt())
    } else {
        None
    }
}

/// Schlick 近似下的反射系数
fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);

    (1.0 - r0) * (1.0 - cosine).powi(5) + r0
}

/// 可散射表面
pub trait Scatter: Send + Sync {
    /// 光线散射
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f32>)>;
}

/// 材质
#[derive(Clone, Copy)]
pub enum Material {
    /// 漫反射
    Lambertian { albedo: Vector3<f32> },

    /// 金属
    Metal { albedo: Vector3<f32>, fuzz: f32 },

    /// 玻璃
    Dielectric { ref_idx: f32 },
}

impl Material {
    /// 构建漫反射
    pub const fn lambertian(albedo: Vector3<f32>) -> Self {
        Self::Lambertian { albedo }
    }

    /// 构建金属
    pub const fn metal(albedo: Vector3<f32>, fuzz: f32) -> Self {
        Self::Metal { albedo, fuzz }
    }

    /// 构建玻璃
    pub const fn dielectric(ref_idx: f32) -> Self {
        Self::Dielectric { ref_idx }
    }
}

impl Scatter for Material {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f32>)> {
        match self {
            Self::Lambertian { albedo } => {
                // 随机反射
                let target = hit.position + hit.normal + random_in_unit_sphere();
                let scattered = Ray::from(hit.position, target - hit.position);

                Some((scattered, *albedo))
            }

            Self::Metal { albedo, fuzz } => {
                let mut reflected = reflect(&ray.direction().normalize(), &hit.normal);

                // 模糊
                if *fuzz > 0.0 {
                    reflected += *fuzz * random_in_unit_sphere();
                }

                // 检查反射方向是否在表面上方
                if reflected.dot(&hit.normal) > 0.0 {
                    let scattered = Ray::from(hit.position, reflected);
                    Some((scattered, *albedo))
                } else {
                    None
                }
            }

            Self::Dielectric { ref_idx } => {
                let attenuation = Vector3::new(1.0, 1.0, 1.0);

                // 入射方向 (从空气到材质或从材质到空气)
                let (outward_normal, ni_over_nt, cosine) = if ray.direction().dot(&hit.normal) > 0.0
                {
                    let cosine =
                        ref_idx * ray.direction().dot(&hit.normal) / ray.direction().magnitude();
                    (-hit.normal, *ref_idx, cosine)
                } else {
                    let cosine = -ray.direction().dot(&hit.normal) / ray.direction().magnitude();
                    (hit.normal, 1.0 / *ref_idx, cosine)
                };

                // 尝试折射
                if let Some(refracted) = refract(&ray.direction(), &outward_normal, ni_over_nt) {
                    let reflect_prob = schlick(cosine, *ref_idx);
                    if rand::rng().random::<f32>() >= reflect_prob {
                        let scattered = Ray::from(hit.position, refracted);
                        return Some((scattered, attenuation));
                    }
                }

                let reflected = reflect(&ray.direction(), &hit.normal);
                let scattered = Ray::from(hit.position, reflected);

                Some((scattered, attenuation))
            }
        }
    }
}
