use std::time::Instant;

/// 朴素的 FPS 监控器.
pub struct FpsCounter {
    /// 初始化时刻
    init_time: Instant,

    /// 上次刷新时刻
    last_update_time: Instant,

    /// 本轮帧数
    frames: u32,

    /// 总帧数
    frames_total: u32,

    /// 本轮 FPS, 近似为瞬时 FPS
    fps: f32,

    /// 总平均 FPS
    fps_average: f32,
}

impl FpsCounter {
    /// 初始化一个 FPS 监控器.
    pub fn init() -> Self {
        let init_time = Instant::now();

        Self {
            init_time,
            last_update_time: init_time,
            frames: 0,
            frames_total: 0,
            fps: 0.0,
            fps_average: 0.0,
        }
    }

    /// 刷新 FPS, 通过控制传入的 `threshold_secs` 调整刷新时间间隔,
    /// 如传入 0.2 表示每 0.2s 刷新一次 FPS 的瞬时值和平均值,
    /// 返回的布尔值表示是否已刷新.
    pub fn update(&mut self, threshold_secs: f32) -> bool {
        self.frames += 1;

        let current_time = Instant::now();
        let delta = current_time
            .duration_since(self.last_update_time)
            .as_secs_f32();

        if delta >= threshold_secs {
            self.frames_total += self.frames;
            let delta_total = current_time.duration_since(self.init_time).as_secs_f32();
            self.fps_average = (self.frames_total as f32) / delta_total;

            self.fps = (self.frames as f32) / delta;
            self.frames = 0;
            self.last_update_time = current_time;

            true
        } else {
            false
        }
    }

    /// 查询当前的瞬时 FPS 和总平均 FPS.
    pub fn fps(&self) -> (f32, f32) {
        (self.fps, self.fps_average)
    }
}
