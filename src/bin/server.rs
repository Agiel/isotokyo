use std::{net::UdpSocket, time::SystemTime};

use bevy::{prelude::*, utils::HashMap, window::PresentMode, render::texture::ImageSettings};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_rapier3d::prelude::*;
use bevy_renet::{
    renet::{RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use isotokyo::{config, generate_map, player};
use isotokyo::{
    networking::{
        server_connection_config, ClientChannel, NetworkFrame, Player, PlayerCommand,
        ServerChannel, ServerMessages, PROTOCOL_ID,
    },
    player::PlayerInput,
};
use renet_visualizer::RenetServerVisualizer;

#[derive(Debug, Default)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>,
}

#[derive(Debug, Default)]
struct NetworkTick(u32);

// Clients last received ticks
#[derive(Debug, Default)]
struct ClientTicks(HashMap<u64, Option<u32>>);

fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = server_connection_config();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Isotokyo Server".into(),
            width: 1280.,
            height: 720.,
            present_mode: PresentMode::Mailbox,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(RenetServerPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EguiPlugin)
        .add_plugin(config::ConfigPlugin)
        .add_plugin(player::ServerPlayerPlugin)
        .insert_resource(ServerLobby::default())
        .insert_resource(NetworkTick(0))
        .insert_resource(ClientTicks::default())
        .insert_resource(new_renet_server())
        .insert_resource(RenetServerVisualizer::<200>::default())
        .add_system(server_update_system)
        .add_system(player::player_move.after(server_update_system))
        .add_system(server_network_sync.after(player::player_move))
        .add_system(update_visulizer_system)
        .add_startup_system(generate_map)
        .add_startup_system(setup_simple_camera)
        .run();
}

#[allow(clippy::too_many_arguments)]
fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lobby: ResMut<ServerLobby>,
    mut server: ResMut<RenetServer>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    mut client_ticks: ResMut<ClientTicks>,
    players: Query<(Entity, &Player, &Transform)>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);
                visualizer.add_client(*id);

                // Initialize other players for this new client
                for (entity, player, transform) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        id: player.id,
                        entity,
                        translation,
                    })
                    .unwrap();
                    server.send_message(*id, ServerChannel::ServerMessages.id(), message);
                }

                // Spawn new player
                let transform = Transform::from_xyz(0.0, 0.51, 0.0);
                let player_entity = commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            depth: 0.5,
                            radius: 0.25,
                            ..default()
                        })),
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform,
                        ..Default::default()
                    })
                    .insert(RigidBody::Dynamic)
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(Collider::capsule_y(0.25, 0.25))
                    .insert(CollisionGroups::new(0b0010, 0b1111))
                    .insert(PlayerInput::default())
                    .insert(Velocity::default())
                    .insert(player::IsGrounded(true))
                    .insert(Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    })
                    .insert(Player {
                        id: *id,
                        ..default()
                    })
                    .id();

                lobby.players.insert(*id, player_entity);

                let translation: [f32; 3] = transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *id,
                    entity: player_entity,
                    translation,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages.id(), message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                visualizer.remove_client(*id);
                client_ticks.0.remove(id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages.id(), message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command.id()) {
            let command: PlayerCommand = bincode::deserialize(&message).unwrap();
            match command {
                _ => (),
            }
        }
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input.id()) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            client_ticks.0.insert(client_id, input.most_recent_tick);
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

fn update_visulizer_system(
    mut egui_context: ResMut<EguiContext>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_context.ctx_mut());
}

#[allow(clippy::type_complexity)]
fn server_network_sync(
    mut tick: ResMut<NetworkTick>,
    mut server: ResMut<RenetServer>,
    networked_entities: Query<(Entity, &Transform, &Velocity, &player::IsGrounded), With<Player>>,
) {
    let mut frame = NetworkFrame::default();
    for (entity, transform, velocity, is_grounded) in networked_entities.iter() {
        frame.entities.entities.push(entity);
        frame
            .entities
            .translations
            .push(transform.translation.into());
        frame
            .entities
            .rotations
            .push(transform.rotation.into());
        frame
            .entities
            .velocities
            .push(velocity.linvel.into());
        frame
            .entities
            .groundeds
            .push(is_grounded.0);
    }

    frame.tick = tick.0;
    tick.0 += 1;
    let sync_message = bincode::serialize(&frame).unwrap();
    server.broadcast_message(ServerChannel::NetworkFrame.id(), sync_message);
}

pub fn setup_simple_camera(mut commands: Commands) {
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
