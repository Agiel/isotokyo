use crate::config::Config;
use crate::input::*;
use crate::networking::ClientLobby;
use crate::networking::MostRecentTick;
use crate::networking::NetworkMapping;
use crate::networking::Player;
use crate::networking::PlayerInfo;
use crate::sprites::*;
use crate::MainCamera;
use bevy::prelude::*;
use bevy::prelude::shape::Icosphere;
use bevy::prelude::shape::Plane;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};

pub struct ClientPlayerPlugin;

impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPlayer>()
            .add_systems(Startup, setup_player);
    }
}

pub struct ServerPlayerPlugin;

impl Plugin for ServerPlayerPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Resource)]
struct PlayerPreload(Vec<Handle<Image>>);

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(PlayerPreload(vec![
        asset_server.load("textures/player/jinrai_idle.png"),
        asset_server.load("textures/player/jinrai_walk.png"),
        asset_server.load("textures/player/nsf_idle.png"),
        asset_server.load("textures/player/nsf_walk.png"),
    ]));

    // Crosshair
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::try_from(Icosphere {
                radius: 0.05,
                ..default()
            }).unwrap()),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            }),
            ..default()
        })
        .insert(Crosshair);
}

#[derive(Event)]
pub struct SpawnPlayer {
    pub id: ClientId,
    pub entity: Entity,
    pub position: Vec3,
    pub is_local: bool,
}

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component)]
pub struct IsGrounded(pub bool);

pub fn client_spawn_players(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut spawn_events: EventReader<SpawnPlayer>,
) {
    for spawn in spawn_events.read() {
        // Player
        let material_handle = materials.add(StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            reflectance: 0.0,
            metallic: 0.0,
            perceptual_roughness: 1.0,
            ..default()
        });
        let mesh_handle = meshes.add(Mesh::from(shape::Quad {
            size: Vec2::new(1.0, 1.0),
            ..default()
        }));

        let mut player = commands.spawn(SpatialBundle {
            transform: Transform::from_translation(spawn.position),
            ..default()
        });
        player
            .insert(Player { id: spawn.id })
            .insert(Collider::capsule_y(0.25, 0.25))
            .insert(CollisionGroups::new(Group::GROUP_2, Group::all()))
            .insert(Velocity::default())
            .insert(LockedAxes::ROTATION_LOCKED)
            .insert(Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            })
            .insert(IsGrounded(true))
            .with_children(|parent| {
                // Sprite
                parent
                    .spawn(PbrBundle {
                        mesh: mesh_handle,
                        material: material_handle,
                        ..default()
                    })
                    .insert(Billboard)
                    .insert(Animator::new(asset_server.load("animations/nsf.anim")))
                    .insert(Sequence::None);
                // Blob shadow
                parent
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(Plane::from_size(1.0))),
                        material: materials.add(StandardMaterial {
                            base_color: Color::BLACK,
                            base_color_texture: Some(
                                asset_server.load("textures/fx/blob_shadow.png"),
                            ),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        }),
                        transform: Transform::from_xyz(0.0, -0.5, 0.0),
                        ..default()
                    })
                    .insert(BlobShadow);
            });

        if spawn.is_local {
            player
                .insert(LocalPlayer)
                .insert(PlayerInput::default())
                .with_children(|parent| {
                    // Light
                    parent.spawn(PointLightBundle {
                        point_light: PointLight {
                            intensity: 2400.0,
                            ..default()
                        },
                        transform: Transform::from_xyz(0.0, 10.0, 0.0),
                        ..default()
                    });
                });
        }

        let player_info = PlayerInfo {
            server_entity: spawn.entity,
            client_entity: player.id(),
        };
        lobby.players.insert(spawn.id, player_info);
        network_mapping.0.insert(spawn.entity, player.id());
    }
}

#[derive(Component)]
pub struct Crosshair;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct PlayerInput {
    forward: f32,
    right: f32,
    jump: bool,
    aim_ray: Ray,
    pub most_recent_tick: Option<u32>,
}

pub fn player_input(
    input: Res<Input<InputAction>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut player_query: Query<&mut PlayerInput>,
    most_recent_tick: Res<MostRecentTick>,
    _mouse_button_input: Res<Input<MouseButton>>,
    cam_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if let Ok(mut player_input) = player_query.get_single_mut() {
        player_input.most_recent_tick = most_recent_tick.0;

        player_input.forward = 0.0;
        player_input.right = 0.0;
        if input.pressed(InputAction::Forward) {
            player_input.forward += 1.0;
        }
        if input.pressed(InputAction::Back) {
            player_input.forward += -1.0;
        }
        if input.pressed(InputAction::Right) {
            player_input.right += 1.0;
        }
        if input.pressed(InputAction::Left) {
            player_input.right += -1.0;
        }
        player_input.jump = (player_input.jump || input.just_pressed(InputAction::Jump))
            && !input.just_released(InputAction::Jump);

        let (camera, camera_transform) = cam_query.single();
        if let Some(cursor_pos) = primary_window.single().cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            player_input.aim_ray = ray;
        }
    }
    }
}

pub fn update_crosshair(
    query: Query<&PlayerInput, With<LocalPlayer>>,
    mut crosshair_query: Query<&mut Transform, (With<Crosshair>, Without<LocalPlayer>)>,
) {
    let mut crosshair_transform = crosshair_query.single_mut();
    if let Ok(player_input) = query.get_single() {
        let aim_ray = player_input.aim_ray;
        if let Some(distance) = aim_ray.intersect_plane(Vec3::ZERO, Vec3::Y) {
            crosshair_transform.translation = aim_ray.origin + aim_ray.direction * distance;
        }
    }
}

pub fn player_move(
    config: Res<Config>,
    physics_config: Res<RapierConfiguration>,
    physics_context: Res<RapierContext>,
    time: Res<Time>,
    mut query: Query<
        (
            &mut PlayerInput,
            &mut IsGrounded,
            &mut Velocity,
            &mut Transform,
        ),
        With<Player>,
    >,
) {
    for (mut player_input, mut is_grounded, mut velocity, mut transform) in query.iter_mut() {
        rotate(&mut transform, &player_input.aim_ray);

        is_grounded.0 = check_grounded(&transform, &physics_context);

        if is_grounded.0 && player_input.jump {
            player_input.jump = false;
            is_grounded.0 = false;
            velocity.linvel.y =
                (2.0 * config.physics.jump_height * -physics_config.gravity.y).sqrt();
        }

        friction(&mut velocity, is_grounded.0, &config, time.delta_seconds());

        let wish_dir = (transform.forward() * player_input.forward
            + transform.right() * player_input.right)
            .normalize_or_zero();
        let wish_speed = config.physics.ground_speed;

        accelerate(
            &mut velocity,
            wish_dir,
            wish_speed,
            is_grounded.0,
            &config,
            time.delta_seconds(),
        );
    }
}

fn rotate(transform: &mut Transform, aim_ray: &Ray) {
    if let Some(distance) = aim_ray.intersect_plane(Vec3::ZERO, Vec3::Y) {
        let mut aim_point = aim_ray.origin + aim_ray.direction * distance;
        aim_point.y = transform.translation.y;
        transform.look_at(aim_point, Vec3::Y);
    }
}

fn check_grounded(transform: &Transform, physics_context: &RapierContext) -> bool {
    if let Some((_entity, _toi)) = physics_context.cast_ray(
        transform.translation,
        -Vec3::Y,
        0.5,
        true,
        QueryFilter::new().groups(CollisionGroups::new(Group::GROUP_1, Group::GROUP_1)),
    ) {
        return true;
    }

    false
}

fn friction(velocity: &mut Velocity, is_grounded: bool, config: &Config, delta_time: f32) {
    let current_speed = velocity.linvel.length();
    if current_speed == 0.0 {
        return;
    }

    let friction = if is_grounded {
        config.physics.ground_friction
    } else {
        config.physics.air_friction
    };

    // TODO: Use stop_speed instead of walk_speed?
    let drop = current_speed.max(config.physics.ground_speed) * friction * delta_time;
    let new_speed = (current_speed - drop).max(0.0);
    velocity.linvel *= new_speed / current_speed;
}

fn accelerate(
    velocity: &mut Velocity,
    wish_dir: Vec3,
    wish_speed: f32,
    is_grounded: bool,
    config: &Config,
    delta_time: f32,
) {
    let wsh_speed = if !is_grounded {
        config.physics.air_speed
    } else {
        wish_speed
    };
    let current_speed = velocity.linvel.dot(wish_dir);
    let add_speed = wsh_speed - current_speed;
    if add_speed <= 0.0 {
        return;
    }

    let accel = if is_grounded {
        config.physics.ground_accel
    } else {
        config.physics.air_accel
    };

    let accel_speed = add_speed.min(accel * wish_speed * delta_time);

    velocity.linvel += wish_dir * accel_speed;
}

pub fn update_sequence(
    mut query: Query<(&mut Sequence, &Parent), Without<Player>>,
    p_query: Query<(&IsGrounded, &Velocity), With<Player>>,
) {
    for (mut sequence, parent) in query.iter_mut() {
        if let Ok((is_grounded, velocity)) = p_query.get(parent.get()) {
            let new_sequence = if !is_grounded.0 {
                Sequence::Jump
            } else if velocity.linvel.length() > f32::EPSILON {
                Sequence::Walk
            } else {
                Sequence::Idle
            };
            if new_sequence != *sequence {
                *sequence = new_sequence;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn camera_follow_player(
    mut query: Query<&mut Transform, With<MainCamera>>,
    player_query: Query<&Transform, (With<LocalPlayer>, Without<MainCamera>)>,
    crosshair_query: Query<
        &Transform,
        (With<Crosshair>, Without<MainCamera>, Without<LocalPlayer>),
    >,
) {
    if let (Ok(player_transform), Ok(crosshair_transform), Ok(mut transform)) = (
        player_query.get_single(),
        crosshair_query.get_single(),
        query.get_single_mut(),
    ) {
        let camera_offset = Vec3::ONE * 6.0;
        let mut translation = player_transform.translation;
        translation.y = 0.0;
        transform.translation = translation
            + (crosshair_transform.translation - translation) / 6.0
            + camera_offset;
    }
}
