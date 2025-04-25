/*
use crate::gaussian::settings::DoGSettings;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContextPass, EguiContextSettings, EguiContexts, EguiPlugin};

struct Images {
    bevy_icon: Handle<Image>,
    bevy_icon_inverted: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            bevy_icon: asset_server.load("icon.png"),
            bevy_icon_inverted: asset_server.load("icon_inverted.png"),
        }
    }
}
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
                    ui_example_system,
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
    label: String,
    value: f32,
    inverted: bool,
    egui_texture_handle: Option<egui::TextureHandle>,
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
    mut query: Query<&mut DoGSettings, With<Camera3d>>,
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
) {
    let ctx = contexts.ctx_mut();
    let Ok(mut edge_detection_settings) = query.single_mut() else {
        return;
    };

    egui::Window::new("Edge Detection")
        .vscroll(true)
        .open(&mut ui_state.is_edge_window_open)
        .show(ctx, |ui| {
            ui.heading("EdgeDetection");
            ui.style_mut().spacing.slider_width = 100.0;
            /*
            ui.add(
                egui::Slider::new(&mut edge_detection_settings.depth_threshold, 0.0..=1.0)
                    .text("Depth Threshold"),
            );
            ui.add(
                egui::Slider::new(&mut edge_detection_settings.normal_threshold, 0.0..=1.0)
                    .text("Normal Threshold"),
            );
            ui.add(
                egui::Slider::new(&mut edge_detection_settings.color_threshold, 0.0..=1.0)
                    .text("Color Threshold"),
            );
            ui.add(egui::Slider::new(&mut edge_detection_settings.enabled, 0..=1).text("Enabled"));
            */
        });
}

fn ui_example_system(
    mut ui_state: ResMut<UiState>,
    mut rendered_texture_id: Local<egui::TextureId>,
    mut is_initialized: Local<bool>,
    images: Local<Images>,
    image_assets: ResMut<Assets<Image>>,
    mut contexts: EguiContexts,
) {
    let egui_texture_handle = ui_state
        .egui_texture_handle
        .get_or_insert_with(|| {
            contexts.ctx_mut().load_texture(
                "example-image",
                egui::ColorImage::example(),
                Default::default(),
            )
        })
        .clone();

    let mut load = false;
    let mut copy = false;
    let mut remove = false;
    let mut invert = false;

    if !*is_initialized {
        *is_initialized = true;
        *rendered_texture_id = contexts.add_image(images.bevy_icon.clone_weak());
    }

    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("side_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut ui_state.label);
            });

            ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
                egui_texture_handle.id(),
                egui_texture_handle.size_vec2(),
            )));

            ui.add(egui::Slider::new(&mut ui_state.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                ui_state.value += 1.0;
            }

            ui.allocate_space(egui::Vec2::new(1.0, 100.0));
            ui.horizontal(|ui| {
                load = ui.button("Load").clicked();
                copy = ui.button("Copy").clicked();
                invert = ui.button("Invert").clicked();
                remove = ui.button("Remove").clicked();
            });

            ui.add(egui::widgets::Image::new(egui::load::SizedTexture::new(
                *rendered_texture_id,
                [256.0, 256.0],
            )));

            ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            ui.checkbox(&mut ui_state.is_edge_window_open, "Edge-Window Is Open");
            ui.checkbox(
                &mut ui_state.is_camera_light_window_open,
                "Camera/Light-Window Is Open",
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(egui::Hyperlink::from_label_and_url(
                    "powered by egui",
                    "https://github.com/emilk/egui/",
                ));
            });
        });

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
        });
    });

    if invert {
        ui_state.inverted = !ui_state.inverted;
    }
    let bevy_icon_handle = if ui_state.inverted {
        images.bevy_icon_inverted.clone_weak()
    } else {
        images.bevy_icon.clone_weak()
    };
    if load || invert {
        // If an image is already added to the context, it'll return an existing texture id.
        *rendered_texture_id = contexts.add_image(bevy_icon_handle.clone_weak());
    }
    if copy {
        let image = image_assets
            .get(&bevy_icon_handle)
            .expect("images should be created");

        contexts
            .ctx_mut()
            .copy_image(egui::ColorImage::from_rgba_unmultiplied(
                image.size().to_array().map(|a| a as usize),
                image.data.as_ref().expect("image data"),
            ));
    }
    if remove {
        contexts.remove_image(&images.bevy_icon);
        contexts.remove_image(&images.bevy_icon_inverted);
    }
}
*/
