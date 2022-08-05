use std::{net::UdpSocket, time::SystemTime};

use bevy::{prelude::*, window::PresentMode, render::texture::ImageSettings};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_rapier3d::prelude::*;
use bevy_renet::{
    renet::{ClientAuthentication, RenetClient, RenetError},
    RenetClientPlugin, run_if_client_connected,
};
use isotokyo::{
    networking::{
        client_connection_config, ClientChannel, ClientLobby, MostRecentTick, NetworkFrame,
        NetworkMapping, PlayerCommand, PlayerInfo, ServerChannel, ServerMessages,
        PROTOCOL_ID,
    },
    player::{client_spawn_players, SpawnPlayer, PlayerInput},
    *,
};
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};

fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let connection_config = client_connection_config();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    RenetClient::new(
        current_time,
        socket,
        client_id,
        connection_config,
        authentication,
    )
    .unwrap()
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Isotokyo".into(),
            width: 1280.,
            height: 720.,
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(RenetClientPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(config::ConfigPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(sprites::Sprite3dPlugin)
        .add_plugin(player::ClientPlayerPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_event::<PlayerCommand>()
        .insert_resource(ClientLobby::default())
        .insert_resource(new_renet_client())
        .insert_resource(RenetClientVisualizer::<200>::new(
            RenetVisualizerStyle::default(),
        ))
        .insert_resource(NetworkMapping::default())
        .insert_resource(MostRecentTick::default())
        .add_system(client_sync_players.with_run_criteria(run_if_client_connected))
        .add_system(client_spawn_players.after(client_sync_players))
        .add_system(player::player_input.after(client_sync_players))
        .add_system(player::update_crosshair.after(player::player_input))
        .add_system(player::camera_follow_player.after(player::update_crosshair))
        .add_system(player::update_sequence.after(client_sync_players))
        .add_system(client_send_input.with_run_criteria(run_if_client_connected).after(player::player_input))
        .add_system(client_send_player_commands.with_run_criteria(run_if_client_connected))
        .add_system(update_visulizer_system)
        .add_startup_system(setup_camera)
        .add_startup_system(generate_map)
        .add_system(panic_on_error_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

fn update_visulizer_system(
    mut egui_context: ResMut<EguiContext>,
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
        visualizer.show_window(egui_context.ctx_mut());
    }
}

fn client_send_input(
    player_query: Query<&PlayerInput, With<player::LocalPlayer>>,
    mut client: ResMut<RenetClient>,
) {
    if let Ok(player_input) = player_query.get_single() {
        let input_message = bincode::serialize(&*player_input).unwrap();
        client.send_message(ClientChannel::Input.id(), input_message);
    }
}

fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command.id(), command_message);
    }
}

fn client_sync_players(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<ClientLobby>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut most_recent_tick: ResMut<MostRecentTick>,
    mut spawn_events: EventWriter<SpawnPlayer>,
) {
    let client_id = client.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages.id()) {
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
                    is_local: client_id == id,
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

    while let Some(message) = client.receive_message(ServerChannel::NetworkFrame.id()) {
        let frame: NetworkFrame = bincode::deserialize(&message).unwrap();
        match most_recent_tick.0 {
            None => most_recent_tick.0 = Some(frame.tick),
            Some(tick) if tick < frame.tick => most_recent_tick.0 = Some(frame.tick),
            _ => continue,
        }

        for i in 0..frame.entities.entities.len() {
            if let Some(entity) = network_mapping.0.get(&frame.entities.entities[i]) {
                let translation = frame.entities.translations[i].into();
                let rotation = Quat::from_array(frame.entities.rotations[i]);
                let transform = Transform {
                    translation,
                    rotation,
                    ..Default::default()
                };
                let velocity = Velocity::linear(frame.entities.velocities[i].into());
                let is_grounded = player::IsGrounded(frame.entities.groundeds[i]);
                commands.entity(*entity).insert(transform).insert(velocity).insert(is_grounded);
            }
        }
    }
}
