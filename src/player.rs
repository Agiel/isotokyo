use crate::sprites::*;
use crate::utils::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const JUMP_HEIGHT: f32 = 0.5;
const GROUND_SPEED: f32 = 3.0;
const AIR_SPEED: f32 = 0.5;
const GROUND_ACCEL: f32 = 10.0;
const AIR_ACCEL: f32 = 1.0;
const GROUND_FRICTION: f32 = 5.0;
const AIR_FRICTION: f32 = 0.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInput::default())
            .add_startup_system(setup_player)
            .add_system(update_crosshair)
            .add_system(player_input.after(update_crosshair))
            .add_system(camera_follow_player.after(update_crosshair))
            .add_system(look_at_crosshair.after(update_crosshair))
            .add_system(player_move.after(look_at_crosshair));
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
        perceptual_roughness: 1.0,
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
            transform: Transform::from_xyz(4.0, 5.5, 4.0),
            ..default()
        })
        .insert(Player::default())
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule_y(0.25, 0.25))
        .insert(Velocity::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Friction {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: mesh_handle,
                    material: material_handle,
                    ..default()
                })
                .insert(Billboard)
                .insert(Animation::new(8, 0.1, true));
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

#[derive(Component, Default)]
struct Player {
    is_grounded: bool,
}

#[derive(Component)]
struct Crosshair;

#[derive(Component, Default)]
struct PlayerInput {
    forward: f32,
    right: f32,
    jump: bool,
    aim_ray: Ray3d,
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut player_input: ResMut<PlayerInput>,
    _mouse_button_input: Res<Input<MouseButton>>,
    cam_query: Query<(&Camera, &Transform)>,
) {
    player_input.forward = 0.0;
    player_input.right = 0.0;
    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
        player_input.forward += 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
        player_input.forward += -1.0;
    }
    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        player_input.right += 1.0;
    }
    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        player_input.right += -1.0;
    }
    player_input.jump = (player_input.jump || keyboard_input.just_pressed(KeyCode::Space))
        && !keyboard_input.just_released(KeyCode::Space);

    let (camera, camera_transform) = cam_query.single();
    if let Some(ray) = Ray3d::from_screenspace(&windows, &images, &camera, &camera_transform) {
        player_input.aim_ray = ray;
    }
}

fn update_crosshair(
    player_input: Res<PlayerInput>,
    mut query: Query<&mut Transform, With<Crosshair>>,
) {
    let mut transform = query.single_mut();
    if let Some(aim_point) = player_input.aim_ray.intersect_y_plane(0.0) {
        transform.translation = aim_point;
    }
}

fn look_at_crosshair(
    mut query: Query<&mut Transform, With<Player>>,
    crosshair_query: Query<&Transform, (With<Crosshair>, Without<Player>)>,
) {
    let mut transform = query.single_mut();
    let mut target = crosshair_query.single().translation.clone();
    target.y = transform.translation.y;
    transform.look_at(target, Vec3::Y);
}

fn player_move(
    mut player_input: ResMut<PlayerInput>,
    physics_config: Res<RapierConfiguration>,
    physics_context: Res<RapierContext>,
    time: Res<Time>,
    mut query: Query<(&mut Player, &mut Velocity, Entity, &Transform)>,
) {
    for (mut player, mut velocity, entity, transform) in query.iter_mut() {
        player.is_grounded = check_grounded(entity, &velocity, &transform, &physics_context);

        if player.is_grounded && player_input.jump {
            player_input.jump = false;
            player.is_grounded = false;
            velocity.linvel.y = (2.0 * JUMP_HEIGHT * -physics_config.gravity.y).sqrt();
        }

        friction(&mut velocity, player.is_grounded, time.delta_seconds());

        let wish_dir =
            transform.forward() * player_input.forward + transform.right() * player_input.right;
        let wish_speed = GROUND_SPEED;

        accelerate(
            &mut velocity,
            wish_dir,
            wish_speed,
            player.is_grounded,
            time.delta_seconds(),
        );
    }
}

fn check_grounded(
    entity: Entity,
    velocity: &Velocity,
    transform: &Transform,
    physics_context: &RapierContext,
) -> bool {
    if velocity.linvel.y > f32::EPSILON {
        return false;
    }

    if let Some((_entity, _toi)) = physics_context.cast_ray(
        transform.translation,
        -Vec3::Y,
        0.5,
        true,
        QueryFilter::new().exclude_rigid_body(entity),
    ) {
        return true;
    }

    return false;
}

fn friction(velocity: &mut Velocity, is_grounded: bool, delta_time: f32) {
    let current_speed = velocity.linvel.length();
    if current_speed == 0.0 {
        return;
    }

    let friction = if is_grounded {
        GROUND_FRICTION
    } else {
        AIR_FRICTION
    };

    // TODO: Use stop_speed instead of walk_speed?
    let drop = current_speed.max(GROUND_SPEED) * friction * delta_time;
    let new_speed = (current_speed - drop).max(0.0);
    velocity.linvel *= new_speed / current_speed;
}

fn accelerate(
    velocity: &mut Velocity,
    wish_dir: Vec3,
    wish_speed: f32,
    is_grounded: bool,
    delta_time: f32,
) {
    let wsh_speed = if !is_grounded { AIR_SPEED } else { wish_speed };
    let current_speed = velocity.linvel.dot(wish_dir);
    let add_speed = wsh_speed - current_speed;
    if add_speed <= 0.0 {
        return;
    }

    let accel = if is_grounded { GROUND_ACCEL } else { AIR_ACCEL };

    let accel_speed = add_speed.min(accel * wish_speed * delta_time);

    velocity.linvel += wish_dir * accel_speed;
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
