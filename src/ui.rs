use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::player::LocalPlayer;

#[derive(Component)]
struct FpsCounter;

#[derive(Component)]
struct Speedometer;

#[derive(Component, Default)]
struct MaxSpeed(f32);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup_ui)
            .add_system(update_fps)
            .add_system(update_speed)
            .add_system(max_speed);
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let style = TextStyle {
        font: asset_server.load("fonts/X-SCALE_.TTF"),
        font_size: 24.0,
        ..default()
    };

    commands
        .spawn(
            TextBundle::from_sections([
                TextSection::new("FPS: ", style.clone()),
                TextSection::new("", style.clone()),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(0.0),
                    left: Val::Px(12.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(FpsCounter);
    commands
        .spawn(
            TextBundle::from_sections([
                TextSection::new("Speed: ", style.clone()),
                TextSection::new("", style.clone()),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(20.0),
                    left: Val::Px(12.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(Speedometer);
    commands
        .spawn(
            TextBundle::from_sections([
                TextSection::new("Max: ", style.clone()),
                TextSection::new("", style),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(40.0),
                    left: Val::Px(12.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(MaxSpeed::default());
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

fn update_speed(
    player_query: Query<&Velocity, With<LocalPlayer>>,
    mut query: Query<&mut Text, With<Speedometer>>,
) {
    for mut text in query.iter_mut() {
        if let Ok(velocity) = player_query.get_single() {
            let mut velocity = *velocity;
            velocity.linvel.y = 0.0;
            // Update the value of the second section
            text.sections[1].value = format!("{:.2}", velocity.linvel.length());
        }
    }
}

fn max_speed(
    player_query: Query<&Velocity, With<LocalPlayer>>,
    mut query: Query<(&mut Text, &mut MaxSpeed), With<MaxSpeed>>,
) {
    for (mut text, mut max_speed) in query.iter_mut() {
        if let Ok(velocity) = player_query.get_single() {
            let mut velocity = *velocity;
            velocity.linvel.y = 0.0;
            if velocity.linvel.length() > max_speed.0 {
                max_speed.0 = velocity.linvel.length();
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", max_speed.0);
            }
        }
    }
}
