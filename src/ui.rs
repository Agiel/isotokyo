use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::player::Player;

#[derive(Component)]
struct FpsCounter;

#[derive(Component)]
struct Speedometer;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup_ui)
            .add_system(update_fps)
            .add_system(update_speed);
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    let style = TextStyle {
        font: asset_server.load("fonts/X-SCALE_.TTF"),
        font_size: 24.0,
        ..default()
    };

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: style.clone(),
                    },
                    TextSection {
                        value: "".to_string(),
                        style: style.clone(),
                    }
                ],
                ..default()
            },
            ..default()
        })
        .insert(FpsCounter);
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(20.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Speed: ".to_string(),
                        style: style.clone(),
                    },
                    TextSection {
                        value: "".to_string(),
                        style: style.clone(),
                    }
                ],
                ..default()
            },
            ..default()
        })
        .insert(Speedometer);
}

fn update_fps(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsCounter>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}

fn update_speed(player_query: Query<&Velocity, With<Player>>, mut query: Query<&mut Text, With<Speedometer>>) {
    for mut text in query.iter_mut() {
        if let Ok(velocity) = player_query.get_single() {
            let mut velocity = velocity.clone();
            velocity.linvel.y = 0.0;
            // Update the value of the second section
            text.sections[1].value = format!("{:.2}", velocity.linvel.length());
        }
    }
}
