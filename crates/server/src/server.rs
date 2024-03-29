use std::{net::UdpSocket, time::SystemTime};

use bevy::{prelude::*, utils::HashMap, window::PresentMode};
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_renet::{
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ClientId, RenetServer, ServerEvent,
    },
    transport::NetcodeServerPlugin,
    RenetServerPlugin,
};
use bevy_xpbd_3d::{
    components::LinearVelocity,
    plugins::{PhysicsDebugPlugin, PhysicsPlugins},
};
use isotokyo::{
    config, generate_map,
    networking::NetworkedEntities,
    player::{self, server_spawn_player},
};
use isotokyo::{
    networking::{
        connection_config, ClientChannel, Player, PlayerCommand, ServerChannel, ServerMessages,
        PROTOCOL_ID,
    },
    player::PlayerInput,
};
use renet_visualizer::RenetServerVisualizer;

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Entity>,
}

#[derive(Debug, Default, Resource)]
struct NetworkTick(u32);

// Clients last received ticks
#[derive(Debug, Default, Resource)]
struct ClientTicks(HashMap<u64, Option<u32>>);

fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time: std::time::Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    (server, transport)
}

fn main() {
    let (client, transport) = new_renet_server();
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Isotokyo Server".into(),
                        resolution: (1280., 720.).into(),
                        present_mode: PresentMode::Mailbox,
                        ..default()
                    }),
                    ..default()
                }),
            RenetServerPlugin,
            NetcodeServerPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            EguiPlugin,
            config::ConfigPlugin,
            player::ServerPlayerPlugin,
        ))
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .insert_resource(ServerLobby::default())
        .insert_resource(NetworkTick(0))
        .insert_resource(ClientTicks::default())
        .insert_resource(client)
        .insert_resource(transport)
        .insert_resource(RenetServerVisualizer::<200>::default())
        .add_systems(Startup, (generate_map, setup_simple_camera))
        .add_systems(
            Update,
            (
                (
                    server_update_system,
                    player::player_move,
                    server_network_sync,
                )
                    .chain(),
                update_visualizer_system,
            ),
        )
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
    players: Query<(Entity, &Player, &Transform)>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);
                visualizer.add_client(*client_id);

                // Initialize other players for this new client
                for (entity, player, transform) in players.iter() {
                    let translation: [f32; 3] = transform.translation.into();
                    let message = bincode::serialize(&ServerMessages::PlayerCreate {
                        id: player.id,
                        entity,
                        translation,
                    })
                    .unwrap();
                    server.send_message(*client_id, ServerChannel::ServerMessages, message);
                }

                // Spawn new player
                let transform = Transform::from_xyz(0.0, 0.51, 0.0);
                let player_entity = server_spawn_player(
                    &mut commands,
                    &mut materials,
                    &mut meshes,
                    *client_id,
                    transform,
                );

                lobby.players.insert(*client_id, player_entity);

                let translation: [f32; 3] = transform.translation.into();
                let message = bincode::serialize(&ServerMessages::PlayerCreate {
                    id: *client_id,
                    entity: player_entity,
                    translation,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);
                visualizer.remove_client(*client_id);
                if let Some(player_entity) = lobby.players.remove(client_id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessages::PlayerRemove { id: *client_id }).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }
        }
    }

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            let command: PlayerCommand = bincode::deserialize(&message).unwrap();
            match command {
                _ => (),
            }
        }
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input: PlayerInput = bincode::deserialize(&message).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_contexts.ctx_mut());
}

#[allow(clippy::type_complexity)]
fn server_network_sync(
    mut server: ResMut<RenetServer>,
    query: Query<(Entity, &Transform, &LinearVelocity, &player::IsGrounded), With<Player>>,
) {
    let mut networked_entities = NetworkedEntities::default();
    for (entity, transform, velocity, is_grounded) in query.iter() {
        networked_entities.entities.push(entity);
        networked_entities
            .translations
            .push(transform.translation.into());
        networked_entities.rotations.push(transform.rotation.into());
        networked_entities.velocities.push(velocity.to_array());
        networked_entities.groundeds.push(is_grounded.0);
    }

    let sync_message = bincode::serialize(&networked_entities).unwrap();
    server.broadcast_message(ServerChannel::NetworkedEntities, sync_message);
}

pub fn setup_simple_camera(mut commands: Commands) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
