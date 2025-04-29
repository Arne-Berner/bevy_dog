use bevy::prelude::*;
use bevy_dog::settings::{BlendMode, DoGSettings, PassesSettings, Thresholding};
use bevy_egui::{egui, EguiContextPass, EguiContextSettings, EguiContexts, EguiPlugin};

/// It is generally encouraged to set up post processing effects as a plugin
pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<UiState>()
            .add_plugins(EguiPlugin {
                enable_multipass_for_primary_context: true,
            })
            .add_systems(Startup, configure_visuals_system)
            .add_systems(Startup, configure_ui_state_system)
            .add_systems(
                EguiContextPass,
                (
                    update_ui_scale_factor_system,
                    edge_detection_window,
                    camera_window,
                    light_window,
                ),
            );
    }
}
#[derive(Default, Resource)]
struct UiState {
    is_edge_window_open: bool,
    is_camera_light_window_open: bool,
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_corner_radius: 0.0.into(),
        ..Default::default()
    });
}

fn configure_ui_state_system(mut ui_state: ResMut<UiState>) {
    ui_state.is_edge_window_open = true;
    ui_state.is_camera_light_window_open = true;
}

fn update_ui_scale_factor_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut contexts: Query<(&mut EguiContextSettings, &Window)>,
) {
    if keyboard_input.just_pressed(KeyCode::Slash) || toggle_scale_factor.is_none() {
        *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

        if let Ok((mut egui_settings, window)) = contexts.single_mut() {
            let scale_factor = if toggle_scale_factor.unwrap() {
                1.0
            } else {
                1.0 / window.scale_factor()
            };
            egui_settings.scale_factor = scale_factor;
        }
    }
}

fn light_window(
    mut light_query: Query<&mut Transform, With<SpotLight>>,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
) {
    let ctx = contexts.ctx_mut();
    let Ok(mut light_transform) = light_query.single_mut() else {
        return;
    };

    egui::Window::new("Light Control")
        .vscroll(true)
        .open(&mut ui_state.is_camera_light_window_open)
        .show(ctx, |ui| {
            ui.heading("Light");
            ui.style_mut().spacing.slider_width = 100.0;
            ui.add(egui::Slider::new(&mut light_transform.translation.x, -30.0..=30.0).text("X"));
            ui.add(egui::Slider::new(&mut light_transform.translation.y, -30.0..=30.0).text("Y"));
            ui.add(egui::Slider::new(&mut light_transform.translation.z, -30.0..=30.0).text("Z"));
        });
}

fn camera_window(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut x: Local<f32>,
    mut y: Local<f32>,
    mut z: Local<f32>,
    mut last_x: Local<f32>,
    mut last_y: Local<f32>,
    mut last_z: Local<f32>,
    mut lookat_y: Local<f32>,
    mut last_lookat_y: Local<f32>,
) {
    let ctx = contexts.ctx_mut();
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    egui::Window::new("Camera control")
        .vscroll(true)
        .open(&mut ui_state.is_camera_light_window_open)
        .show(ctx, |ui| {
            ui.heading("Camera");
            ui.style_mut().spacing.slider_width = 100.0;
            ui.add(egui::Slider::new(&mut *x, -30.0..=30.0).text("X"));
            ui.add(egui::Slider::new(&mut *y, 0.0..=30.0).text("Y"));
            ui.add(egui::Slider::new(&mut *z, -30.0..=30.0).text("Z"));
            ui.add(egui::Slider::new(&mut *lookat_y, 0.0..=30.0).text("Z"));
        });

    // weird workaround so that lookat works
    if (*last_x != *x) || (*last_y != *y) || (*last_z != *z) || (*last_lookat_y != *lookat_y) {
        *camera_transform = Transform::from_xyz(*x, *y, *z)
            .looking_at(Vec3::from_array([0., *lookat_y, 0.]), Vec3::Y);
        *last_x = *x;
        *last_y = *y;
        *last_z = *z;
        *last_lookat_y = *lookat_y;
    }
}

fn edge_detection_window(
    mut query: Query<(&mut DoGSettings, &mut PassesSettings), With<Camera3d>>,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
) {
    let ctx = contexts.ctx_mut();
    for (mut dog_settings, mut passes_settings) in &mut query {
        egui::Window::new("DoG Settings")
            .vscroll(true)
            .open(&mut ui_state.is_edge_window_open)
            .show(ctx, |ui| {
                ui.heading("Common Settings");
                ui.style_mut().spacing.slider_width = 100.0;
                ui.add(egui::Slider::new(&mut dog_settings.k, 0.1..=5.0).text("K"));
                ui.add(egui::Slider::new(&mut dog_settings.tau, 0.0..=120.0).text("Tau"));
                ui.add(
                    egui::Slider::new(&mut dog_settings.blend_strength, 0.0..=2.0)
                        .text("Blend Strength"),
                );
                ui.add(
                    egui::Slider::new(&mut dog_settings.dog_strength, 0.0..=5.0)
                        .text("Dog Strength"),
                );
                ui.add(
                    egui::Slider::new(&mut dog_settings.min_color.x, 0.0..=1.0).text("Min Color R"),
                );
                ui.add(
                    egui::Slider::new(&mut dog_settings.min_color.y, 0.0..=1.0).text("Min Color G"),
                );
                ui.add(
                    egui::Slider::new(&mut dog_settings.min_color.z, 0.0..=1.0).text("Min Color B"),
                );
                ui.add(
                    egui::Slider::new(&mut dog_settings.max_color.x, 0.0..=1.0).text("Max Color R"),
                );
                ui.add(
                    egui::Slider::new(&mut dog_settings.max_color.y, 0.0..=1.0).text("Max Color G"),
                );
                ui.add(
                    egui::Slider::new(&mut dog_settings.max_color.z, 0.0..=1.0).text("Max Color B"),
                );
                ui.add(egui::Slider::new(&mut dog_settings.invert, 0..=1).text("Invert"));
                egui::ComboBox::from_label("BlendMode")
                    .selected_text(format!(
                        "{:?}",
                        match dog_settings.blend_mode {
                            0 => BlendMode::NoBlend,
                            1 => BlendMode::Interpolate,
                            _ => BlendMode::TwoPointInterpolate,
                        }
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut dog_settings.blend_mode,
                            BlendMode::NoBlend as i32,
                            "No Blend",
                        );
                        ui.selectable_value(
                            &mut dog_settings.blend_mode,
                            BlendMode::Interpolate as i32,
                            "Interpolate",
                        );
                        ui.selectable_value(
                            &mut dog_settings.blend_mode,
                            BlendMode::TwoPointInterpolate as i32,
                            "TwoPointInterpolate",
                        );
                    });
                // horizontal line
                ui.heading("Thresholding Specific Settings");
                egui::ComboBox::from_label("Thresholding")
                    .selected_text(format!(
                        "{:?}",
                        match dog_settings.thresholding {
                            0 => Thresholding::NoThreshold,
                            1 => Thresholding::Tanh,
                            2 => Thresholding::Quantization,
                            3 => Thresholding::SmoothQuantization,
                            _ => Thresholding::SmoothQuantization, // Handle invalid values
                        }
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut dog_settings.thresholding,
                            Thresholding::NoThreshold as i32,
                            "NoThreshold",
                        );
                        ui.selectable_value(
                            &mut dog_settings.thresholding,
                            Thresholding::Tanh as i32,
                            "Tanh",
                        );
                        ui.selectable_value(
                            &mut dog_settings.thresholding,
                            Thresholding::Quantization as i32,
                            "Quantization",
                        );
                        ui.selectable_value(
                            &mut dog_settings.thresholding,
                            Thresholding::SmoothQuantization as i32,
                            "Smooth Quantization",
                        );
                    });
                if dog_settings.thresholding != 0 {
                    ui.add(egui::Slider::new(&mut dog_settings.phi, 0.0..=10.0).text("Phi"));
                }
                if dog_settings.thresholding == 1 {
                    ui.add(
                        egui::Slider::new(&mut dog_settings.enable_layers.x, 0.0..=1.0)
                            .text("Enable Layer 1"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.enable_layers.y, 0.0..=1.0)
                            .text("Enable Layer 2"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.enable_layers.z, 0.0..=1.0)
                            .text("Enable Layer 3"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.enable_layers.w, 0.0..=1.0)
                            .text("Enable Layer 4"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.thresholds.x, 0.0..=100.0)
                            .text("Threshold 1"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.thresholds.y, 0.0..=100.0)
                            .text("Threshold 2"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.thresholds.z, 0.0..=100.0)
                            .text("Threshold 3"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.thresholds.w, 0.0..=100.0)
                            .text("Threshold 4"),
                    );
                }
                if dog_settings.thresholding > 1 {
                    ui.add(
                        egui::Slider::new(&mut dog_settings.quantizer_step, 0.0..=5.0)
                            .text("Quantizer Step"),
                    );
                }
                // horizontal line
                ui.heading("Anti Aliasing Settings");
                ui.add(egui::Slider::new(&mut passes_settings.aa, 0..=1).text("Anti Aliasing"));
                if passes_settings.aa != 0 {
                    ui.add(
                        egui::Slider::new(&mut dog_settings.sigma_a, 0.0..=10.0).text("Sigma A"),
                    );
                }
                // horizontal line
                ui.heading("FDoG Settings");
                ui.add(egui::Slider::new(&mut passes_settings.tfm, 0..=1).text("Uses FDoG"));
                if passes_settings.tfm != 0 {
                    ui.add(egui::Slider::new(&mut dog_settings.sigma_c, 0.0..=7.0).text("Sigma C"));
                    ui.add(egui::Slider::new(&mut dog_settings.sigma_e, 0.0..=7.0).text("Sigma E"));
                    ui.add(
                        egui::Slider::new(&mut dog_settings.sigma_m, 0.0..=20.0).text("Sigma M"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.line_conv_step_sizes.x, 0.0..=3.0)
                            .text("Line Conv X"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.line_conv_step_sizes.y, 0.0..=3.0)
                            .text("Line Conv Y"),
                    );
                    ui.add(
                        egui::Slider::new(&mut dog_settings.calc_diff_before_convolution, 0..=1)
                            .text("Calc Difference before conv"),
                    );
                }
                // horizontal line
                ui.heading("Crosshatch Settings");
                ui.add(
                    egui::Slider::new(&mut dog_settings.enable_hatch, 0..=1).text("Enable Hatch"),
                );
                if dog_settings.enable_hatch == 1 {
                    ui.add(
                        egui::Slider::new(&mut dog_settings.hatch_resolutions.x, 0.1..=10.0)
                            .text("First Layer Hatch resolution"),
                    );
                    if dog_settings.thresholding == 1 {
                        ui.add(
                            egui::Slider::new(&mut dog_settings.hatch_resolutions.y, 0.1..=10.0)
                                .text("Second Layer Hatch resolution"),
                        );
                        ui.add(
                            egui::Slider::new(&mut dog_settings.hatch_resolutions.z, 0.1..=10.0)
                                .text("Third Layer Hatch resolution"),
                        );
                        ui.add(
                            egui::Slider::new(&mut dog_settings.hatch_resolutions.w, 0.1..=10.0)
                                .text("Fourth Layer Hatch resolution"),
                        );
                    }
                    ui.add(
                        egui::Slider::new(&mut dog_settings.hatch_rotations.x, 0.0..=180.0)
                            .text("First Layer Hatch Rotation"),
                    );
                    if dog_settings.thresholding == 1 {
                        ui.add(
                            egui::Slider::new(&mut dog_settings.hatch_rotations.y, 0.0..=180.0)
                                .text("Second Layer Hatch Rotation"),
                        );
                        ui.add(
                            egui::Slider::new(&mut dog_settings.hatch_rotations.z, 0.0..=180.0)
                                .text("Third Layer Hatch Rotation"),
                        );
                        ui.add(
                            egui::Slider::new(&mut dog_settings.hatch_rotations.w, 0.0..=180.0)
                                .text("Fourth Layer Hatch Rotation"),
                        );
                    }
                }
            });
    }
}
