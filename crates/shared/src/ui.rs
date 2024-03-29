use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_xpbd_3d::components::LinearVelocity;

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
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (update_fps, update_speed, max_speed));
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
                top: Val::Px(0.0),
                left: Val::Px(12.0),
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
                top: Val::Px(20.0),
                left: Val::Px(12.0),
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
                top: Val::Px(40.0),
                left: Val::Px(12.0),
                ..default()
            }),
        )
        .insert(MaxSpeed::default());
}

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsCounter>>) {
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
    player_query: Query<&LinearVelocity, With<LocalPlayer>>,
    mut query: Query<&mut Text, With<Speedometer>>,
) {
    for mut text in query.iter_mut() {
        if let Ok(velocity) = player_query.get_single() {
            // Update the value of the second section
            text.sections[1].value = format!("{:.2}", velocity.xz().length());
        }
    }
}

fn max_speed(
    player_query: Query<&LinearVelocity, With<LocalPlayer>>,
    mut query: Query<(&mut Text, &mut MaxSpeed), With<MaxSpeed>>,
) {
    for (mut text, mut max_speed) in query.iter_mut() {
        if let Ok(velocity) = player_query.get_single() {
            if velocity.xz().length() > max_speed.0 {
                max_speed.0 = velocity.xz().length();
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", max_speed.0);
            }
        }
    }
}
