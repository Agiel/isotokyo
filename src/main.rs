mod config;
mod input;
mod player;
mod sprites;
mod ui;
mod utils;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use config::ConfigPlugin;
use input::InputPlugin;
use player::*;
use rand::{thread_rng, Rng};
use sprites::*;
use ui::UiPlugin;

const MAP_SIZE: i32 = 64;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Isotokyo".into(),
            width: 1280.,
            height: 720.,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ConfigPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(Sprite3dPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(UiPlugin)
        .add_startup_system(setup)
        .add_startup_system(generate_map)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    // Set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 720.0 / 2.0 / 64.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn_bundle(camera).insert(MainCamera);
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
        alpha_mode: AlphaMode::Opaque,
        reflectance: 0.0,
        metallic: 0.0,
        perceptual_roughness: 1.0,
        ..default()
    });

    let mesh_handle = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    // Plane
    for x in -MAP_SIZE / 2..MAP_SIZE / 2 {
        for y in -MAP_SIZE / 2..MAP_SIZE / 2 {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh_handle.clone(),
                material: material_handle.clone(),
                transform: Transform::from_xyz(x as f32, 0.0, y as f32),
                ..default()
            });
        }
    }

    // Ground collider
    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(-0.5, -0.1, -0.5)))
        .insert(Collider::cuboid(
            (MAP_SIZE / 2) as f32,
            0.1,
            (MAP_SIZE / 2) as f32,
        ))
        .insert(CollisionGroups::new(0b0001, 0b1111));

    // Light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.05,
    });

    // // directional 'sun' light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Props
    let texture_handle = asset_server.load("textures/props/sakura1.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.0,
        metallic: 0.0,
        perceptual_roughness: 1.0,
        ..default()
    });
    let mesh_handle = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(1.5, 2.0),
        ..default()
    }));
    let plane_handle = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));
    for _ in 0..128 {
        let mut rng = thread_rng();
        let x = rng.gen::<f32>() * MAP_SIZE as f32 - (MAP_SIZE / 2) as f32;
        let z = rng.gen::<f32>() * MAP_SIZE as f32 - (MAP_SIZE / 2) as f32;
        // Tree
        commands
            .spawn_bundle(TransformBundle {
                local: Transform::from_xyz(x, 1.0, z),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: material_handle.clone(),
                        ..default()
                    })
                    .insert(Billboard);
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: plane_handle.clone(),
                        material: materials.add(StandardMaterial {
                            base_color: Color::BLACK,
                            base_color_texture: Some(
                                asset_server.load("textures/fx/blob_shadow.png"),
                            ),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        }),
                        transform: Transform::from_xyz(0.0, -1.0, 0.0),
                        ..default()
                    })
                    .insert(BlobShadow);
            });
    }

    let mesh_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material_handle = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    for _ in 0..32 {
        let mut rng = thread_rng();
        let x = rng.gen::<f32>() * MAP_SIZE as f32 - (MAP_SIZE / 2) as f32;
        let z = rng.gen::<f32>() * MAP_SIZE as f32 - (MAP_SIZE / 2) as f32;
        commands.spawn_bundle(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(x, 0.5, z),
            ..default()
        }).insert(Collider::cuboid(0.5, 0.5, 0.5));
    }
}
