use bevy::{
    core_pipeline::{
        fxaa::{Fxaa, Sensitivity},
        prepass::{DepthPrepass, NormalPrepass},
    },
    log::{Level, LogPlugin},
    prelude::*,
};
mod debug_ui;
mod gaussian;
// use debug_ui::DebugUIPlugin;
use gaussian::{plugin::DoGPlugin, settings::DoGSettings};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    filter: "warn,ui=info".to_string(),
                    level: Level::INFO,
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // You may want this set to `true` if you need virtual keyboard work in mobile browsers.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
            // DebugUIPlugin,
            DoGPlugin,
        ))
        .register_type::<Rotates>()
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate, quit))
        .run();
}

fn quit(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Rotates,
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(20.0, 18.0, 0.0))
            .looking_at(Vec3::default(), Vec3::Y),
        /*
        Msaa::Off,
        Fxaa {
            enabled: true,
            edge_threshold: Sensitivity::Extreme,
            edge_threshold_min: Sensitivity::Extreme,
        },
        */
        DoGSettings::default(),
    ));

    // light
    commands.spawn((
        SpotLight {
            intensity: 30_000_000.,
            // shadows_enabled: true,
            inner_angle: 0.0,
            outer_angle: 0.8,
            ..default()
        },
        Transform::from_xyz(5.0, 18.5, -5.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Rotates;

/// Rotates any entity around the x and y axis
fn rotate(time: Res<Time>, mut query: Query<&mut Transform, With<Rotates>>) {
    for mut transform in &mut query {
        // transform.rotate_x(0.55 * time.delta_secs());
        // transform.rotate_z(0.15 * time.delta_secs());
        transform.rotate_y(0.55 * time.delta_secs());
    }
}
