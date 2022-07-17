use bevy::prelude::*;
use crate::sprites::*;
use crate::utils::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_system(update_crosshair)
            .add_system(camera_follow_player.after(update_crosshair))
            .add_system(look_at_crosshair.after(update_crosshair));
    }
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Player
    let texture_handle = asset_server.load("textures/player/jinrai_walk.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let mesh_handle = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(1.0, 1.0),
        ..default()
    }));

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform: Transform::from_xyz(4.0, 0.5, 4.0),
            ..default()
        })
        .insert(Player)
        // .insert(Wireframe)
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: mesh_handle,
                    material: material_handle,
                    ..default()
                })
                .insert(Billboard)
                .insert(Animation::new(
                    8,
                    0.1,
                    true
                ));
        });

    // Crosshair
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.05,
                ..default()
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            }),
            ..default()
        })
        .insert(Crosshair);
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Crosshair;

fn update_crosshair(
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut query: Query<&mut Transform, (With<Crosshair>, Without<Camera>)>,
    cam_query: Query<(&Camera, &Transform)>,
) {
    let (camera, camera_transform) = cam_query.single();
    let mut crosshair_transform = query.single_mut();

    if let Some(ray) = Ray3d::from_screenspace(&windows, &images, &camera, &camera_transform) {
        if let Some(aim_point) = ray.intersect_y_plane(0.5) {
            crosshair_transform.translation = aim_point;
        }
    }
}

fn look_at_crosshair(
    mut query: Query<&mut Transform, With<Player>>,
    crosshair_query: Query<&Transform, (With<Crosshair>, Without<Player>)>,
) {
    let mut transform = query.single_mut();
    let crosshair_transform = crosshair_query.single();

    transform.look_at(crosshair_transform.translation, Vec3::Y);
}

fn camera_follow_player(
    mut query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    crosshair_query: Query<&Transform, (With<Crosshair>, Without<Camera>, Without<Player>)>,
) {
    let player_transform = player_query.single();
    let crosshair_transform = crosshair_query.single();
    let mut transform = query.single_mut();
    let camera_offset = Vec3::new(5.0, 5.0, 5.0);
    transform.translation = player_transform.translation
        + (crosshair_transform.translation - player_transform.translation) / 6.0
        + camera_offset;
}
