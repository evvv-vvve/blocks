use bevy::{prelude::*, diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin}};
use bevy_egui::{EguiContext, egui::{DragValue, Slider}};
use iyes_loopless::prelude::ConditionSet;

use crate::{player_cam::PlayerCamera, AppState};

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct PlayerPosText;

#[derive(Component)]
pub struct PlayerFacingDirText;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
		app.add_system_set(
            ConditionSet::new()
              .run_in_state(AppState::Finished)
              .label("ui-update")
              .with_system(draw_fps)
              .with_system(draw_player_pos)
              .with_system(draw_player_facing_dir)
              .into()
        );
	}
}

pub struct WorldGenSettings {
    regen_chunks: bool,

    scale: f64,
    octaves: i32,
    persistence: f32,
    lacunarity: f32,
    //offset: Vec2,
}

impl Default for WorldGenSettings {
    fn default() -> Self {
        Self {
            regen_chunks: false,
            scale: 25.,
            octaves: 5,
            persistence: 0.5,
            lacunarity: 2.,
            //offset: Vec2 { x: 51.11, y: 0. }
          }
    }
}

pub fn ui_world_gen(
    mut egui_context: ResMut<EguiContext>,
    mut world_gen_settings: ResMut<WorldGenSettings>
) {
    bevy_egui::egui::Window::new("World Generator").show(egui_context.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Scale");
            ui.add(DragValue::new(&mut world_gen_settings.scale));
        });

        ui.horizontal(|ui| {
            ui.label("Octaves");
            ui.add(DragValue::new(&mut world_gen_settings.octaves));
        });

        ui.horizontal(|ui| {
            ui.label("Persistence");
            ui.add(Slider::new(&mut world_gen_settings.persistence, 0.0..=1.0));
        });

        ui.horizontal(|ui| {
            ui.label("Lacunarity");
            ui.add(DragValue::new(&mut world_gen_settings.lacunarity));
        });

        ui.separator();

        if ui.button("Generate!").clicked() {
            world_gen_settings.regen_chunks = true;
        }
    });

    if world_gen_settings.regen_chunks {
        println!("regen");
    }
}

pub fn draw_player_pos(
    query: Query<&GlobalTransform, With<PlayerCamera>>,
    mut text_query: Query<&mut Text, With<PlayerPosText>>
) {
    let player = query.single();

    let player_pos = player.translation();

    // Update the value of the second section
    for mut text in text_query.iter_mut() {
        text.sections[0].value = format!("XYZ: {:.2} {:.2} {:.2}", player_pos.x, player_pos.y, player_pos.z);
    }
}

pub fn draw_fps(
    diagnostics: Res<Diagnostics>,
    //mut windows: ResMut<Windows>,
    mut query: Query<&mut Text, With<FpsText>>
) {
    //let window = windows.primary_mut();

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            // Update the value of the second section
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("FPS: {:.2}", average);
                //println!("FPS: {average:.2}");
            }
        }
    }
}

pub fn draw_player_facing_dir(
    player_cam_query: Query<&PlayerCamera>,
    //mut windows: ResMut<Windows>,
    mut query: Query<&mut Text, With<PlayerFacingDirText>>,
) {
    //let window = windows.primary_mut();

    for player_cam in player_cam_query.iter() {
        // Update the value of the second section
        for mut text in query.iter_mut() {
            text.sections[0].value = format!("Facing: {:.2}", player_cam.yaw);
            //println!("FPS: {average:.2}");
        }
    }
}

pub fn spawn_ui(
    mut commands: Commands,
    //game_version: Res<GameVersion>,
    asset_server: Res<AssetServer>,
) {
    // UI camera
    //commands.spawn_bundle(UiCameraBundle::default());

    // fps text
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("data/blocky/fonts/main.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        })
        .insert(FpsText);

    // player pos text
    commands
    .spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(25.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        },
        // Use `Text` directly
        text: Text {
            // Construct a `Vec` of `TextSection`s
            sections: vec![
                TextSection {
                    value: "XYZ: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("data/blocky/fonts/main.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
            ..default()
        },
        ..default()
    })
    .insert(PlayerPosText);

    // player facing text
    commands
    .spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(45.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        },
        // Use `Text` directly
        text: Text {
            // Construct a `Vec` of `TextSection`s
            sections: vec![
                TextSection {
                    value: "Facing: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("data/blocky/fonts/main.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
            ..default()
        },
        ..default()
    })
    .insert(PlayerFacingDirText);
    
    // game vers
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: format!("Dev Test"),
                        style: TextStyle {
                            font: asset_server.load("data/blocky/fonts/main.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                ..default()
            },
            ..default()
        }
    );
}