use bevy::{prelude::*, utils::HashMap};
use bevy_renet::renet::{
    ChannelConfig, ReliableChannelConfig, RenetConnectionConfig, UnreliableChannelConfig,
    NETCODE_KEY_BYTES,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const PRIVATE_KEY: &[u8; NETCODE_KEY_BYTES] = b"an example very very secret key."; // 32-bytes
pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Default, Component)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug, Default)]
pub struct MostRecentTick(pub Option<u32>);

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum PlayerCommand {
    BasicAttack { cast_at: Vec3 },
}

pub enum ClientChannel {
    Input,
    Command,
}

pub enum ServerChannel {
    ServerMessages,
    NetworkFrame,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerCreate {
        entity: Entity,
        id: u64,
        translation: [f32; 3],
    },
    PlayerRemove {
        id: u64,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<[f32; 3]>,
    pub rotations: Vec<[f32; 4]>,
    pub velocities: Vec<[f32; 3]>,
    pub groundeds: Vec<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkFrame {
    pub tick: u32,
    pub entities: NetworkedEntities,
}

impl ClientChannel {
    pub fn id(&self) -> u8 {
        match self {
            Self::Input => 0,
            Self::Command => 1,
        }
    }

    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ReliableChannelConfig {
                channel_id: Self::Input.id(),
                message_resend_time: Duration::ZERO,
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Command.id(),
                message_resend_time: Duration::ZERO,
                ..Default::default()
            }
            .into(),
        ]
    }
}

impl ServerChannel {
    pub fn id(&self) -> u8 {
        match self {
            Self::NetworkFrame => 0,
            Self::ServerMessages => 1,
        }
    }

    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            UnreliableChannelConfig {
                channel_id: Self::NetworkFrame.id(),
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::ServerMessages.id(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn client_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ClientChannel::channels_config(),
        receive_channels_config: ServerChannel::channels_config(),
        ..Default::default()
    }
}

pub fn server_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ServerChannel::channels_config(),
        receive_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}

#[derive(Default)]
pub struct NetworkMapping(pub HashMap<Entity, Entity>);

#[derive(Debug)]
pub struct PlayerInfo {
    pub client_entity: Entity,
    pub server_entity: Entity,
}

#[derive(Debug, Default)]
pub struct ClientLobby {
    pub players: HashMap<u64, PlayerInfo>,
}
