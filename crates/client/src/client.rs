use std::{net::UdpSocket, time::SystemTime};

use bevy::{prelude::*, window::PresentMode};
use bevy_egui::{EguiPlugin, EguiContexts};
use bevy_rapier3d::prelude::*;
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        RenetClient,
    },
    RenetClientPlugin, transport::NetcodeClientPlugin, client_connected,
};
use isotokyo::{
    networking::{
        connection_config, ClientChannel, ClientLobby, MostRecentTick,
        NetworkMapping, PlayerCommand, PlayerInfo, ServerChannel, ServerMessages,
        PROTOCOL_ID, NetworkedEntities,
    },
    player::{client_spawn_players, SpawnPlayer, PlayerInput},
    *,
};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};

fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let client = RenetClient::new(connection_config());

    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    (client, transport)
}

fn main() {
    let (client, transport) = new_renet_client();
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Isotokyo".into(),
                        resolution: (1280., 720.).into(),
                        present_mode: PresentMode::Fifo,
                        ..default()
                    }),
                    ..default()
                }),
            RenetClientPlugin,
            NetcodeClientPlugin,
            EguiPlugin,
            config::ConfigPlugin,
            input::InputPlugin,
            sprites::Sprite3dPlugin,
            player::ClientPlayerPlugin,
            ui::UiPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(ClientLobby::default())
        .insert_resource(client)
        .insert_resource(transport)
        .insert_resource(RenetClientVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ))
        .insert_resource(NetworkMapping::default())
        .insert_resource(MostRecentTick::default())
        .add_event::<PlayerCommand>()
        .add_systems(Startup, (
            setup_camera,
            generate_map,
        ))
        .add_systems(Update, (
            (
                client_sync_players,
                client_send_input.after(player::player_input),
                client_send_player_commands,
            ).run_if(client_connected()),
            (
                client_spawn_players,
                (
                    player::player_input,
                    player::update_crosshair,
                ).chain(),
                player::update_sequence,
            ).after(client_sync_players),
            update_visualizer_system,
            panic_on_error_system,
            bevy::window::close_on_esc,
        ))
        .add_systems(PostUpdate, player::camera_follow_player)
        .run();
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    for e in renet_error.read() {
        panic!("{}", e);
    }
}

fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetClientVisualizer<200>>,
    client: Res<RenetClient>,
    mut show_visualizer: Local<bool>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    visualizer.add_network_info(client.network_info());
    if keyboard_input.just_pressed(KeyCode::F1) {
        *show_visualizer = !*show_visualizer;
    }
    if *show_visualizer {
        visualizer.show_window(egui_contexts.ctx_mut());
    }
}

fn client_send_input(
    player_query: Query<&PlayerInput, With<player::LocalPlayer>>,
    mut client: ResMut<RenetClient>,
) {
    if let Ok(player_input) = player_query.get_single() {
        let input_message = bincode::serialize(player_input).unwrap();
        client.send_message(ClientChannel::Input, input_message);
    }
}

fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.read() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);
    }
}

fn client_sync_players(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    transport: Res<NetcodeClientTransport>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut spawn_events: EventWriter<SpawnPlayer>,
) {
    let client_id = transport.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessages::PlayerCreate {
                id,
                translation,
                entity,
            } => {
                println!("Player {} connected.", id);
                spawn_events.send(SpawnPlayer {
                    id,
                    entity,
                    position: translation.into(),
                    is_local: client_id == id.raw(),
                });
            }
            ServerMessages::PlayerRemove { id } => {
                println!("Player {} disconnected.", id);
                if let Some(PlayerInfo {
                    server_entity,
                    client_entity,
                }) = lobby.players.remove(&id)
                {
                    commands.entity(client_entity).despawn_recursive();
                    network_mapping.0.remove(&server_entity);
                }
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities) {
        let networked_entities: NetworkedEntities = bincode::deserialize(&message).unwrap();

        for i in 0..networked_entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&networked_entities.entities[i]) {
                let translation = networked_entities.translations[i].into();
                let rotation = Quat::from_array(networked_entities.rotations[i]);
                let transform = Transform {
                    translation,
                    rotation,
                    ..Default::default()
                };
                let velocity = Velocity::linear(networked_entities.velocities[i].into());
                let is_grounded = player::IsGrounded(networked_entities.groundeds[i]);
                commands.entity(*entity)
                    .insert(transform)
                    .insert(velocity)
                    .insert(is_grounded);
            }
        }
    }
}
