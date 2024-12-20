use std::io::{self, Write};

use three_d::*;

use three_d_scene::fps::FpsCounter;

#[tokio::main]
async fn main() -> io::Result<()> {
    run().await;

    // 阻塞命令行, 以便出错时显示错误提示再退出
    print!("Press ENTER to continue...");
    io::stdout().flush()?;
    let _ = io::stdin().read_line(&mut String::new())?;

    Ok(())
}

/// 为 egui 配置字体
fn setup_custom_fonts(ctx: &egui::Context) {
    use three_d::egui::{FontFamily::*, TextStyle::*, *};

    let mut fonts = FontDefinitions::default();

    // 中文字体
    fonts.font_data.insert(
        "Chinese_font".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/DingTalkJinBuTi.ttf")),
    );

    // 等宽字体
    fonts.font_data.insert(
        "Monospace".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-SemiBold.ttf")),
    );

    // 将等宽字体分配给比例字体是有意为之(防止 FPS 显示跳动)
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Monospace".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(1, "Chinese_font".to_owned());

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(20.0, Proportional)),
        (Body, FontId::new(18.0, Proportional)),
        (
            three_d::egui::TextStyle::Monospace,
            FontId::new(18.0, Proportional),
        ),
        (Button, FontId::new(18.0, Proportional)),
        (Small, FontId::new(14.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);

    ctx.set_fonts(fonts);
}

// 打开仓库链接
fn open_link(url: &str) -> Result<(), std::io::Error> {
    use std::process::Command;
    Command::new("cmd")
        .args(["/C", "start", url])
        .spawn()?
        .wait()?;
    Ok(())
}

pub async fn run() {
    //////////////////// ! 1. 控件 ////////////////////

    // 窗口及其图形上下文
    let window = Window::new(WindowSettings {
        title: String::from("3D-scene"),
        max_size: Some((1280, 720)),
        min_size: (720, 596),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    // 相机, 单独声明参数以备重置
    let position = vec3(20.0, 12.0, 20.0);
    let target = vec3(0.0, 7.0, 0.0);
    let up = vec3(0.0, 1.0, 0.0);
    let field_of_view_y = degrees(60.0);
    let (z_near, z_far) = (1.0, 1000.0);
    let mut camera = Camera::new_perspective(
        window.viewport(),
        position,
        target,
        up,
        field_of_view_y,
        z_near,
        z_far,
    );

    // 旋转控制
    let mut control = OrbitControl::new(camera.target(), 1.0, 100.0);

    // GUI 控制台
    let mut gui = three_d::GUI::new(&context);

    //////////////////// ! 2. 资源 ////////////////////

    // 加载资源文件
    let assets = [
        "assets/garden_4k.hdr",
        "assets/checkerboard.jpg",
        "assets/duck.glb",
        "assets/ferris.glb",
    ];
    let mut loaded = match three_d_asset::io::load_async(&assets).await {
        Ok(loaded) => loaded,
        Err(e) => {
            println!("资源加载失败: {}", e);
            return;
        }
    };

    // 天空盒
    let garden = loaded.deserialize("garden").unwrap();
    let skybox = Skybox::new_from_equirectangular(&context, &garden);

    // 平面: 纯色
    let mut plane_cpu = CpuMesh::square();
    plane_cpu
        .transform(
            Mat4::from_translation(vec3(0.0, -1.0, 0.0))
                * Mat4::from_scale(30.0)
                * Mat4::from_angle_x(degrees(-90.0)),
        )
        .unwrap();
    let plane = Gm::new(
        Mesh::new(&context, &plane_cpu),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::new_opaque(128, 128, 128),
                ..Default::default()
            },
        ),
    );

    // 平面: 棋盘
    let mut board_cpu: CpuTexture = loaded.deserialize("checkerboard").unwrap();
    board_cpu.data.to_color();
    let mut board_geometry = CpuMesh::square();
    board_geometry
        .uvs
        .as_mut()
        .unwrap()
        .iter_mut()
        .for_each(|uv| *uv = 5.0 * (*uv - vec2(0.4, 0.4)));
    let mut board = Gm::new(
        Mesh::new(&context, &board_geometry),
        ColorMaterial::default(),
    );
    board.set_transformation(
        Mat4::from_translation(vec3(0.0, -1.0, 0.0))
            * Mat4::from_scale(30.0)
            * Mat4::from_angle_x(degrees(-90.0)),
    );

    // 小黄鸭
    let mut duck_cpu: CpuModel = loaded.deserialize("duck").unwrap();
    duck_cpu
        .geometries
        .iter_mut()
        .for_each(|m| m.compute_tangents());
    let mut duck = Model::<PhysicalMaterial>::new(&context, &duck_cpu)
        .unwrap()
        .remove(0);
    duck.set_transformation(Mat4::from_translation(vec3(0.0, 4.0, 0.0)) * Mat4::from_scale(0.04));
    duck.material.roughness = 0.15;

    // Rust 吉祥物 Ferris
    let mut ferris_cpu: CpuModel = loaded.deserialize("ferris").unwrap();
    ferris_cpu
        .geometries
        .iter_mut()
        .for_each(|m| m.compute_tangents());
    let mut ferris = Model::<PhysicalMaterial>::new(&context, &ferris_cpu)
        .unwrap()
        .remove(0);
    ferris.set_transformation(
        Mat4::from_translation(vec3(12.0, 3.0, -12.0))
            * Mat4::from_angle_y(degrees(90.0))
            * Mat4::from_scale(10.0),
    );
    ferris.set_animation(|time| {
        Mat4::from_angle_y(radians(time))
            * Mat4::from_translation(vec3(0.0, radians(time).sin() * 0.2, 0.0))
    });

    // 镜面球
    let mut mirror_sphere = Gm::new(
        Mesh::new(&context, &CpuMesh::sphere(32)),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                roughness: 0.0,
                metallic: 1.0,
                ..Default::default()
            },
        ),
    );
    mirror_sphere
        .set_transformation(Mat4::from_translation(vec3(-16.0, 4.0, 0.0)) * Mat4::from_scale(4.0));

    // 彩色立方体
    // 8 个顶点
    let positions = vec![
        vec3(-0.5, -0.5, 0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(-0.5, 0.5, -0.5),
    ];

    // 6 个面的片元
    let indices = vec![
        0, 1, 2, 0, 2, 3, // F
        4, 5, 6, 4, 6, 7, // B
        0, 3, 7, 0, 7, 4, // L
        1, 5, 6, 1, 6, 2, // R
        3, 2, 6, 3, 6, 7, // U
        0, 1, 5, 0, 5, 4, // D
    ];

    // 8 个顶点的颜色
    let colors = vec![
        Srgba::BLACK,
        Srgba::RED,
        Srgba {
            r: 255,
            g: 255,
            b: 0,
            a: 1,
        },
        Srgba::GREEN,
        Srgba::BLUE,
        Srgba {
            r: 255,
            g: 0,
            b: 255,
            a: 1,
        },
        Srgba::WHITE,
        Srgba {
            r: 0,
            g: 255,
            b: 255,
            a: 1,
        },
    ];

    // 构建彩色立方体
    let cube_cpu = CpuMesh {
        positions: Positions::F32(positions),
        indices: Indices::U32(indices),
        colors: Some(colors),
        ..Default::default()
    };
    let mut cube = Gm::new(Mesh::new(&context, &cube_cpu), PhysicalMaterial::default());
    cube.set_transformation(Mat4::from_translation(vec3(12.0, 4.0, 12.0)) * Mat4::from_scale(4.0));
    cube.set_animation(|time| Mat4::from_angle_y(radians(time)));

    //////////////////// ! 3. 光照 ////////////////////

    // 环境光
    let mut ambient =
        AmbientLight::new_with_environment(&context, 0.6, Srgba::WHITE, skybox.texture());

    // 平行光
    let mut dir0 = DirectionalLight::new(&context, 1.0, Srgba::RED, vec3(0.0, -1.0, 0.0));
    let mut dir1 = DirectionalLight::new(&context, 1.0, Srgba::GREEN, vec3(0.0, -1.0, 0.0));

    // 点光源
    let mut point0 = PointLight::new(
        &context,
        1.0,
        Srgba::GREEN,
        vec3(0.0, 0.0, 0.0),
        Attenuation {
            constant: 0.5,
            linear: 0.05,
            quadratic: 0.005,
        },
    );
    let mut point1 = PointLight::new(
        &context,
        1.0,
        Srgba::RED,
        vec3(0.0, 0.0, 0.0),
        Attenuation {
            constant: 0.5,
            linear: 0.05,
            quadratic: 0.005,
        },
    );

    //////////////////// ! 4. 渲染 ////////////////////

    // FPS 监控器
    let mut fps_counter = FpsCounter::init();
    let mut current_fps = 0.0;
    let mut current_avg_fps = 0.0;

    // 动画开关
    let mut is_animated = true;

    // 解决动画突变
    let mut ani_lag = 0.0;
    let mut last_stop = 0.0;
    let mut ani_updated = false;

    // 光照强度的中间变量
    let mut dir_intensity = dir0.intensity;
    let mut point_intensity = point0.intensity;

    // 平面种类
    let mut is_checker = false;

    // Mipmap
    let mut mipmap_level = 1;
    let mut mipmap_filter = Interpolation::Nearest;

    // 渲染
    window.render_loop(move |mut frame_input| {
        // 更新 FPS
        if fps_counter.update(0.2) {
            let (fps, average_fps) = fps_counter.fps();
            current_fps = fps;
            current_avg_fps = average_fps;
        }

        // GUI 控制台
        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;

                SidePanel::left("side_panel").show(gui_context, |ui| {
                    setup_custom_fonts(gui_context);
                    ui.horizontal(|ui| {
                        // 标题
                        ui.label(RichText::new("控制面板").font(FontId::proportional(24.0)));

                        // 仓库链接
                        use egui::special_emojis::GITHUB;
                        if ui.button(format!("{GITHUB} GitHub")).clicked() {
                            let url = "https://github.com/Somnia1337/three-d-scene";
                            if let Err(e) = open_link(url) {
                                eprintln!("Failed to open browser: {}", e);
                            }
                        }
                    });

                    // FPS 监控
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading("FPS");
                        ui.toggle_value(&mut is_animated, "动画");
                    });
                    ui.label(format!(
                        "当前/平均: {:.1}/{:.1}",
                        current_fps, current_avg_fps
                    ));

                    // 表面
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading("小黄鸭表面");
                        if ui.button("重置").clicked() {
                            duck.material.metallic = 0.0;
                            duck.material.roughness = 0.15;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("金属度");
                        ui.add(Slider::new::<f32>(&mut duck.material.metallic, 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("粗糙度");
                        ui.add(Slider::new::<f32>(&mut duck.material.roughness, 0.0..=1.0));
                    });

                    // 光照强度
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading("光照强度");
                        if ui.button("重置").clicked() {
                            ambient.intensity = 0.6;
                            dir_intensity = 1.0;
                            dir0.intensity = dir_intensity;
                            dir1.intensity = dir_intensity;
                            point_intensity = 1.0;
                            point0.intensity = point_intensity;
                            point1.intensity = point_intensity;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("环境光");
                        ui.add(Slider::new(&mut ambient.intensity, 0.0..=1.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("平行光");
                        ui.add(Slider::new(&mut dir_intensity, 0.0..=1.0));
                    });
                    dir0.intensity = dir_intensity;
                    dir1.intensity = dir_intensity;
                    ui.horizontal(|ui| {
                        ui.label("点光源");
                        ui.add(Slider::new(&mut point_intensity, 0.0..=1.0));
                    });
                    point0.intensity = point_intensity;
                    point1.intensity = point_intensity;

                    // 光照模型
                    ui.separator();
                    ui.heading("光照模型");
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut duck.material.lighting_model,
                            LightingModel::Blinn,
                            "Blinn-Phong",
                        );
                        ui.radio_value(
                            &mut duck.material.lighting_model,
                            LightingModel::Cook(
                                NormalDistributionFunction::TrowbridgeReitzGGX,
                                GeometryFunction::SmithSchlickGGX,
                            ),
                            "Cook-Torrance",
                        );
                    });

                    // Mipmap
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.heading("平面");
                        ui.toggle_value(&mut is_checker, "棋盘格");
                        if is_checker {
                            dir0.clear_shadow_map();
                            dir1.clear_shadow_map();
                        }
                    });
                    ui.add_enabled_ui(is_checker, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Mipmap 等级");
                            ui.add(Slider::new(&mut mipmap_level, 1..=8));
                        });
                        ui.horizontal(|ui| {
                            ui.radio_value(
                                &mut mipmap_filter,
                                Interpolation::Nearest,
                                "最邻近插值",
                            );
                            ui.radio_value(&mut mipmap_filter, Interpolation::Linear, "线性插值");
                        });
                    });

                    // 控制说明
                    ui.separator();
                    ui.label("WASD/↑←↓→: 移动");
                    ui.label("空格(+Shift): 上升(下降)");
                    ui.label("R: 重置相机");
                });

                panel_width = gui_context.used_rect().width();
            },
        );

        // 视口
        let viewport = Viewport {
            x: (panel_width * frame_input.device_pixel_ratio) as i32,
            y: 0,
            width: frame_input.viewport.width
                - (panel_width * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };
        camera.set_viewport(viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        // 光照变化
        let ac_time = 0.001 * frame_input.accumulated_time;

        // 动画
        if is_animated {
            if ani_updated {
                ani_lag += ac_time - last_stop;
                ani_updated = false;
            }

            let c = (ac_time - ani_lag).cos() as f32;
            let s = (ac_time - ani_lag).sin() as f32;

            // 光照动画
            dir0.direction = vec3(-1.0 - c, -1.0, 1.0 + s);
            dir1.direction = vec3(1.0 + c, -1.0, -1.0 - s);
            point0.position = vec3(-5.0 * c, 5.0, -5.0 * s);
            point1.position = vec3(5.0 * c, 5.0, 5.0 * s);

            // 模型动画
            ferris.animate((ac_time - ani_lag) as f32);
            cube.animate((ac_time - ani_lag) as f32);
        } else if !ani_updated {
            last_stop = ac_time;
            ani_updated = true;
        }

        // 阴影
        if !is_checker {
            dir0.generate_shadow_map(1024, [&duck, &ferris, &mirror_sphere, &cube]);
            dir1.generate_shadow_map(1024, [&duck, &ferris, &mirror_sphere, &cube]);
        }

        // 光照组
        let lights = [&ambient as &dyn Light, &dir0, &dir1, &point0, &point1];

        // Mipmap
        let mipmap = Some(Mipmap {
            max_ratio: 1,
            max_levels: mipmap_level,
            filter: mipmap_filter,
        });
        if board_cpu.mipmap != mipmap {
            board_cpu.mipmap = mipmap;
            board.material.texture = Some(Texture2DRef::from_cpu_texture(&context, &board_cpu));
        }

        let screen = frame_input.screen();
        screen.clear(ClearState::default());

        // 响应键盘输入
        for event in frame_input.events.iter() {
            if let Event::KeyPress {
                kind, modifiers, ..
            } = *event
            {
                let mut move_direction = vec3(0.0, 0.0, 0.0);

                let mut forward = camera.view_direction();
                forward.y = 0.0;
                forward = forward.normalize();
                let right = forward.cross(vec3(0.0, 1.0, 0.0));
                let up = vec3(0.0, 1.0, 0.0);

                match kind {
                    // WASD: 移动
                    Key::W | Key::ArrowUp => move_direction += forward,
                    Key::S | Key::ArrowDown => move_direction -= forward,
                    Key::A | Key::ArrowLeft => move_direction -= right,
                    Key::D | Key::ArrowRight => move_direction += right,

                    // 空格: 上升 (Shift+空格: 下降)
                    Key::Space => {
                        if modifiers.shift {
                            move_direction -= up
                        } else {
                            move_direction += up
                        }
                    }

                    // R 重置相机
                    Key::R => {
                        camera.set_view(position, target, up);
                    }

                    // 忽略其余按键
                    _ => {}
                }

                if move_direction != vec3(0.0, 0.0, 0.0) {
                    camera.translate(move_direction * 0.4);
                }
            }
        }

        // 渲染
        let mut objects_to_render = vec![
            &duck as &dyn three_d::Object,
            &ferris,
            &mirror_sphere,
            &cube,
        ];

        // 动态选择平面并插入到对象列表
        if is_checker {
            objects_to_render.insert(0, &board);
        } else {
            objects_to_render.insert(0, &plane);
        }

        // 渲染到屏幕
        screen.render(
            &camera,
            skybox.into_iter().chain(objects_to_render.iter().copied()),
            &lights,
        );

        // 写入画布
        screen.write(|| gui.render()).unwrap();

        FrameOutput::default()
    });
}
