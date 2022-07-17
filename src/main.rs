mod sprites;
mod player;
mod utils;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::*;
use sprites::*;
use player::*;
// use rand::{thread_rng, Rng};

const MAP_SIZE: i32 = 128;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Isotokyo".into(),
            width: 960.,
            height: 540.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(Sprite3dPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .add_startup_system(generate_map)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 540.0 / 2.0 / 64.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);

    // camera
    commands.spawn_bundle(camera);

    let texture_handle = asset_server.load("textures/props/sakura1.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let mesh_handle = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(1.5, 2.0),
        ..default()
    }));

    // props
    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(1.5, 1.0, 1.5),
            ..Default::default()
        })
        .insert(Billboard);
    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(1.5, 1.0, -1.5),
            ..Default::default()
        })
        .insert(Billboard);
    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(-1.5, 1.0, 1.5),
            ..Default::default()
        })
        .insert(Billboard);
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(-1.5, 0.5, -1.5),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            radius: 0.25,
            depth: 0.5,
            ..Default::default()
        })),
        material: materials.add(Color::rgb(0.0, 0.7, 0.0).into()),
        transform: Transform::from_xyz(0., 0.5, 0.),
        ..Default::default()
    });
}



fn generate_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture_handle = asset_server.load("textures/tiles/grass1.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let mesh_handle = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    // plane
    for x in -MAP_SIZE..MAP_SIZE {
        for y in -MAP_SIZE..MAP_SIZE {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh_handle.clone(),
                material: material_handle.clone(),
                transform: Transform::from_xyz(x as f32, 0.0, y as f32),
                ..Default::default()
            });
        }
    }

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.05,
    });
}

#[derive(Deref, DerefMut)]
struct PrintingTimer(Timer);

impl Default for PrintingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, true))
    }
}
