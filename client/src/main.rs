use std::time::Instant;

use bevy::{
    prelude::*,
    diagnostic::FrameTimeDiagnosticsPlugin,
    render::{settings::WgpuSettings, render_resource::WgpuFeatures, texture::ImageSettings},
    pbr::wireframe::{WireframePlugin, WireframeConfig, Wireframe}
};

use bevy_atmosphere::prelude::*;
use bevy_egui::EguiPlugin;
use chunk_manager::{spawn_ex_chunk_tasks, handle_chunk_tasks};
use chunky::{Chunk, CHUNK_SIZE};
use identifier::Identifier;
use iyes_loopless::prelude::*;
use player_cam::*;
use registry::*;
use texture_atlas::*;
use ui::*;

use crate::{chunky::build_chunk_mesh, procedural::ProcGen};

pub mod player_cam;
pub mod chunky;
pub mod block;
pub mod registry;
pub mod identifier;
pub mod item;
pub mod procedural;
pub mod texture_atlas;
pub mod ui;
pub mod custom_material;
pub mod chunk_manager;

/// Returned when there is an error reading a file or directory
#[derive(thiserror::Error, Debug)]
pub enum BlockyPathError {
    #[error("An error occurred while reading directory {0}: {1}")]
    DirectoryReadError(String, std::io::Error),
    
    #[error("An error occurred while reading file in path {0}: {1}")]
    PathReadError(String, std::io::Error),
    
    #[error("An error occurred while parsing ron file {0}: {1}")]
    FileParseError(String, ron::error::Error),
}

#[derive(Debug)]
pub struct GameVersion {
    pub state: String,

    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    LoadResources,
    Registry,
    Finished,
}

impl Default for GameVersion {
    fn default() -> Self {
        Self {
            state: String::from("Pre-Alpha"),
            
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
}

fn main() {
    App::new()
      .init_resource::<TextureHandles>()
      .insert_resource(ImageSettings::default_nearest())
      .insert_resource(WgpuSettings {
        features: WgpuFeatures::POLYGON_MODE_LINE,
        ..default()
      })
      .insert_resource(WindowDescriptor {
        title: String::from("Blocky"),
        ..Default::default()
      })
      .insert_resource(GameVersion::default())
      .insert_resource(WorldGenSettings::default())
      .add_loopless_state(AppState::LoadResources)
      .add_plugins(DefaultPlugins)
      .add_plugin(WireframePlugin)
      .add_plugin(PlayerCameraPlugin)
      .add_plugin(AtmospherePlugin)
      .add_plugin(FrameTimeDiagnosticsPlugin::default())
      .add_plugin(EguiPlugin)
      .add_plugin(TextureAtlasesPlugin)
      .add_plugin(UIPlugin)
      .add_plugin(RegistryPlugin)
      .add_startup_system(spawn_player)
      //.add_startup_system(init_setup)
      .add_exit_system_set(
        AppState::Registry,
        ConditionSet::new()
          .label("setup")
          //.with_system(registry_init)
          .with_system(spawn_ui)
          .with_system(world_setup)
          .with_system(spawn_ex_chunk_tasks)
          .into()
      )
      .add_system_set(
        ConditionSet::new()
          .run_in_state(AppState::Finished)
          .with_system(toggle_wireframe)
          .with_system(handle_chunk_tasks)
          .into()
      )
      //.add_system(ui_world_gen)
      .run();
}

#[derive(Component)]
pub struct ToggleWireframe(bool);

fn toggle_wireframe(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut ToggleWireframe)>
) {
    if kb.just_pressed(KeyCode::K) {
        for (entity, mut wireframe_toggle) in query.iter_mut() {
            if wireframe_toggle.0 {
                commands.entity(entity)
                  .insert(Wireframe);
            } else {
                commands.entity(entity)
                .remove::<Wireframe>();
            }

            wireframe_toggle.0 = !wireframe_toggle.0;
        }
    }
}

pub fn spawn_player(mut commands: Commands) {
    // directional 'sun' light
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: false,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    commands.spawn()
        .insert_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PlayerCamera {
            accel: 0.7,
            max_speed: 0.4,
            sensitivity: 6.,
            friction: 0.6,
            ..Default::default()
    })
    .insert(UiCameraConfig { show_ui: true })
    .insert(AtmosphereCamera(None));
}

pub fn world_setup(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    our_atlases: Res<TextureAtlasHandles>,
    mut wireframe_config: ResMut<WireframeConfig>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    wireframe_config.global = false;

    //let block_atlas = texture_atlases.get(&our_atlases.block_atlas.as_ref().unwrap()).unwrap();
    
    //register_items_in_dir("data/blocky/items/");
    //register_blocks_in_dir(asset_server, block_atlas, "data/blocky/blocks/");

    //let gen_start = Instant::now();
    //gen_chunks(commands, meshes, materials, our_atlases, texture_atlases);
    //println!("Took {}ms to mesh and place 17^2 chunks!", gen_start.elapsed().as_millis());
}

pub fn gen_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    our_atlases: Res<TextureAtlasHandles>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    
    let block = get_block_from_registry(&Identifier::new("blocky", "grass_block")).unwrap();

    //let mut rng = rand::thread_rng();

    let genner = ProcGen::new(2342537, CHUNK_SIZE);

    let texture_atlas = texture_atlases.get(&our_atlases.block_atlas.as_ref().unwrap()).unwrap();

    let size_min = 0;//-4;
    let size_max = 1;//12;

    for x in size_min..size_max {
        for z in size_min..size_max {
            for y in 0..1 {
                //let chunker_start = Instant::now();

                let chunk_pos = Vec3::new(x as f32, y as f32, z as f32);

                let mut chunk = Chunk::new(chunk_pos);

                let chunk_noise_map = genner.gen_noise_map(chunk_pos);

                for z in 0..CHUNK_SIZE as i32 {
                    for x in 0..CHUNK_SIZE as i32 {
                        let y_pos = chunk_noise_map[z as usize * CHUNK_SIZE + x as usize];
                        
                        chunk.add_block(
                            x as usize,
                            (y_pos) as usize,
                            z as usize,
                            Some(block.clone())
                        );
                    }
                }

                //println!("Took {}ms to place 16^3 blocks!", chunker_start.elapsed().as_millis());

                //let mesh_start = Instant::now();

                let mesh = build_chunk_mesh(&chunk);
                let mesh_handle = meshes.add(mesh);

                //println!("Took {}ms to build mesh!", mesh_start.elapsed().as_millis());

                commands.spawn()
                .insert(Transform::from_xyz(x as f32, 0., z as f32))
                .insert_bundle(PbrBundle {
                    mesh: mesh_handle,//.clone_weak(),
                    material: materials.add(
                        StandardMaterial {
                            base_color_texture: Some(texture_atlas.texture.clone()),
                            ..Default::default()
                    }),
                    ..Default::default()
                })
                .insert(chunk)
                //.insert(ChunkMesh(mesh_handle))
                .insert(ToggleWireframe(true));
            }
        }   
    }
}