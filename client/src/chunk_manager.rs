use bevy::{tasks::{Task, AsyncComputeTaskPool}, prelude::{Component, Commands, Mesh, Query, Entity, ResMut, Assets, Res, Transform}, math::Vec3, pbr::{StandardMaterial, PbrBundle}, sprite::TextureAtlas};
use futures_lite::future;

use crate::{chunky::{Chunk, CHUNK_SIZE, build_chunk_mesh, ChunkMesh}, procedural::ProcGen, identifier::Identifier, registry::{get_block_from_registry, get_block_from_registry_by_string}, texture_atlas::TextureAtlasHandles, ToggleWireframe};

#[derive(Component)]
pub struct ComputeChunk(Task<(Chunk, Mesh)>);

pub fn spawn_ex_chunk_tasks(mut commands: Commands) {
    let threadpool = AsyncComputeTaskPool::get();

    //let mut rng = rand::thread_rng();

    let genner = ProcGen::new(2342537, CHUNK_SIZE);

    //let texture_atlas = texture_atlases.get(&our_atlases.block_atlas.as_ref().unwrap()).unwrap();

    let size_min = -24;
    let size_max = 24;

    for z in size_min..size_max {
        for x in size_min..size_max {
            for y in 0..1 {
                // spawn new task on the threadpool
                let task = threadpool.spawn(async move {
                    let block = get_block_from_registry_by_string("blocky:grass_block").unwrap();

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

                    let chunk_mesh = build_chunk_mesh(&chunk);

                    (chunk, chunk_mesh)
                });

                commands.spawn()
                    .insert(Transform::from_xyz(x as f32, 0., z as f32))
                    .insert(ComputeChunk(task));
            }
        }
    }
}

pub fn handle_chunk_tasks(
    mut commands: Commands,
    mut chunk_tasks: Query<(Entity, &mut ComputeChunk)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    our_atlases: Res<TextureAtlasHandles>,
    texture_atlases: Res<Assets<TextureAtlas>>,
) {
    let texture_atlas = texture_atlases.get(&our_atlases.block_atlas.as_ref().unwrap()).unwrap();
    
    for (entity, mut chunk_task) in &mut chunk_tasks {
        if let Some((chunk, chunk_mesh)) = future::block_on(future::poll_once(&mut chunk_task.0)) {
            let mesh_handle = meshes.add(chunk_mesh);
            
            commands.entity(entity)
                .insert_bundle(PbrBundle {
                    mesh: mesh_handle.clone_weak(),
                    material: materials.add(
                        StandardMaterial {
                            base_color_texture: Some(texture_atlas.texture.clone()),
                            ..Default::default()
                    }),
                    ..Default::default()
                })
                .insert(chunk)
                .insert(ChunkMesh(mesh_handle))
                .insert(ToggleWireframe(true))
                .remove::<ComputeChunk>();
        }
    }
}