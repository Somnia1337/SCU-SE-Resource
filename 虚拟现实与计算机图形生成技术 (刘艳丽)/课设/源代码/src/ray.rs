use nalgebra::Vector3;

/// 光线
pub struct Ray {
    /// 起点
    origin: Vector3<f32>,

    /// 方向
    direction: Vector3<f32>,
}

impl Ray {
    pub const fn from(origin: Vector3<f32>, direction: Vector3<f32>) -> Self {
        Self { origin, direction }
    }

    pub const fn origin(&self) -> Vector3<f32> {
        self.origin
    }

    pub const fn direction(&self) -> Vector3<f32> {
        self.direction
    }

    /// 光线上 t 处的点
    pub fn point_at_t(&self, t: f32) -> Vector3<f32> {
        self.origin + t * self.direction
    }
}
