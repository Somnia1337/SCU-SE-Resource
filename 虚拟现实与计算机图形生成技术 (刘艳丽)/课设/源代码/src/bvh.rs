use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;

use nalgebra::Vector3;
use std::cmp::Ordering;
use std::sync::Arc;

/// 一个结点最多包含的实体
const MAX_OBJECTS: usize = 7;

/// 轴对齐包围盒
#[derive(Clone)]
pub struct AaBb {
    /// 最小点
    pub min: Vector3<f32>,

    /// 最大点
    pub max: Vector3<f32>,
}

impl AaBb {
    const fn new() -> Self {
        Self {
            min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    /// 能包裹两个包围盒的最小包围盒
    fn surrounding_box(box0: &Self, box1: &Self) -> Self {
        let small = box0.min.zip_map(&box1.min, f32::min);
        let big = box0.max.zip_map(&box1.max, f32::max);

        Self {
            min: small,
            max: big,
        }
    }

    /// 能包裹多个实体的最小包围盒
    fn all_surrounding_box(objects: &[Arc<dyn Bounded + Sync + Send>]) -> Self {
        let mut surround = Self::new();

        for obj in objects {
            let bbox = obj.bounding_box();
            surround.min = surround.min.zip_map(&bbox.min, f32::min);
            surround.max = surround.max.zip_map(&bbox.max, f32::max);
        }

        surround
    }

    /// 光线与包围盒相交
    pub fn hit(&self, ray: &Ray) -> bool {
        let inv_d = ray.direction().map(|d| 1.0 / d);
        let t0s = (self.min - ray.origin()).component_mul(&inv_d);
        let t1s = (self.max - ray.origin()).component_mul(&inv_d);

        let t_min_vec = t0s.zip_map(&t1s, f32::min);
        let t_max_vec = t0s.zip_map(&t1s, f32::max);

        let t_min = t_min_vec.max();
        let t_max = t_max_vec.min();

        t_max > t_min.max(t_min)
    }

    /// 选取分割轴 (包围盒最长边所在的轴)
    fn split_axis(&self) -> usize {
        let x = self.max.x - self.min.x;
        let y = self.max.y - self.min.y;
        let z = self.max.z - self.min.z;
        let max = x.max(y).max(z);

        if x == max {
            0
        } else if y == max {
            1
        } else {
            2
        }
    }
}

/// 可被 BVH 管理的有界实体
pub trait Bounded: Hittable + Send {
    /// 实体的包围盒
    fn bounding_box(&self) -> AaBb;
}

impl Hittable for Vec<Arc<dyn Bounded + Sync + Send>> {
    /// 光线与结点中的最近包围盒相交
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut closest_hit: Option<HitRecord> = None;

        // 与结点中包围盒最近的相交点
        for obj in self {
            if let Some(hit) = obj.hit(ray, t_min, closest) {
                closest = hit.distance;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }
}

/// BVH 结点
pub enum BVHNode {
    /// 叶子结点, 包含一个实体
    Leaf {
        objects: Vec<Arc<dyn Bounded + Sync + Send>>,
    },

    /// 内部结点, 包含左右子树和包围盒
    Node {
        left: Arc<BVHNode>,
        right: Arc<BVHNode>,
        bbox: AaBb,
    },
}

impl BVHNode {
    /// 构建 BVH 树
    pub fn build(mut objects: Vec<Arc<dyn Bounded + Sync + Send>>) -> Self {
        if objects.len() <= MAX_OBJECTS {
            Self::Leaf { objects }
        } else {
            let surround = AaBb::all_surrounding_box(&objects);
            let axis = surround.split_axis();

            objects.sort_by(|a, b| {
                let box_a = a.bounding_box();
                let box_b = b.bounding_box();

                box_a.min[axis]
                    .partial_cmp(&box_b.min[axis])
                    .unwrap_or(Ordering::Equal)
            });

            let right = objects.split_off(objects.len() / 2);
            let left = objects;

            let left = Self::build(left);
            let right = Self::build(right);
            let bbox = AaBb::surrounding_box(&left.bounding_box(), &right.bounding_box());

            Self::Node {
                left: Arc::new(left),
                right: Arc::new(right),
                bbox,
            }
        }
    }

    /// 当前结点的包围盒
    fn bounding_box(&self) -> AaBb {
        match self {
            Self::Leaf { objects } => AaBb::all_surrounding_box(objects),
            Self::Node { bbox, .. } => bbox.clone(),
        }
    }
}

impl Hittable for BVHNode {
    /// 光线与 BVH 结点相交
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self {
            Self::Leaf { objects } => objects.hit(ray, t_min, t_max),

            Self::Node { left, right, bbox } => {
                if !bbox.hit(ray) {
                    return None;
                }

                left.hit(ray, t_min, t_max).map_or_else(
                    || right.hit(ray, t_min, t_max),
                    |l| match right.hit(ray, t_min, l.distance) {
                        Some(r) => {
                            if l.distance < r.distance {
                                Some(l)
                            } else {
                                Some(r)
                            }
                        }
                        None => Some(l),
                    },
                )
            }
        }
    }
}
