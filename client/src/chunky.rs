use bevy::{math::Vec3, prelude::*, render::mesh::Indices};

use crate::{block::*, registry, identifier::Identifier};

pub const CHUNK_SIZE: usize = 16;

pub const BLOCK_Y_SHIFT: usize = 4;
pub const BLOCK_Z_SHIFT: usize = 8;

#[derive(Component)]
pub struct Chunk {
    /// Whether or not the chunk is all air (0) or not
    is_empty: bool,

    /// Total amount of air blocks inside a chunk.
    air_count: usize,


    chunk_pos: Vec3,

    /// 0 is reserved for air:<br>
    /// Local block IDs are index + 1
    ids: Vec<Option<String>>,

    /// Blocks in this chunk.<br>
    /// Local block id is index + 1
    blocks: Vec<u16>,
}

/// used for storing a chunks mesh
/// so we can modify it later
#[derive(Component)]
pub struct ChunkMesh(pub Handle<Mesh>);

impl Chunk {
    pub fn new(pos: Vec3) -> Self {
        Self {
            is_empty: true,
            air_count: CHUNK_SIZE.pow(3),
            
            chunk_pos: pos,

            ids: Vec::new(),
            blocks: vec![0; CHUNK_SIZE.pow(3)],
        }
    }

    pub fn get_chunk_pos(&self) -> Vec3 { self.chunk_pos }

    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    pub fn add_block(&mut self, x: usize, y: usize, z: usize, block: Option<Block>) -> bool {
        let pos = pos_as_index(x, y, z);

        if pos < self.blocks.len() {
            if let Some(data) = &block {
                self.is_empty = false;

                if self.air_count > 0 {
                    self.air_count -= 1;
                }

                let id_string = data.get_identifier().as_string();

                // check if a block with this id exists already
                // if not, add it to our collection
                if !self.ids.contains(&Some(id_string.clone())) {
                    // check if there are any free slots to place our block id
                    let mut found_id = false;

                    for (index, block_id) in self.ids.clone().into_iter().enumerate() {
                        // if an empty spot was found,
                        // insert our block id and exit the loop
                        if let None = block_id {
                            found_id = true;
                            self.ids[index] = Some(id_string.clone());
                            break;
                        }
                    }

                    // if no free slots were found,
                    // create a new one
                    if !found_id {
                        self.ids.push(Some(id_string.clone()))
                    }
                }

                // get the local numerical id for this block
                let block_id = self.ids.iter().position(|x|
                    x == &Some(id_string.clone())
                ).unwrap() as u16 + 1;

                // add it to our chunk at the correct position
                self.blocks[pos] = block_id;
            } else {
                // only if we're replacing a block
                if let Some(_) = self.get_block(x, y, z) {
                    self.air_count += 1;

                    let curr_block_id = self.get_local_block_id(x, y, z);

                    // add air
                    self.blocks[pos] = 0;

                    // check if there are any blocks left in the chunk with
                    // the local id of the block we just removed
                    let mut has_block = false;

                    for block_id in &self.blocks {
                        // exit the loop if we've found another block with the current
                        // block id, and set our boolean to true
                        if *block_id == curr_block_id {
                            has_block = true;
                            break;
                        }
                    }

                    // clear the global block id if no
                    // blocks of its type exist inside the chunk anymore
                    if !has_block {
                        // id is local_id + 1: '0' is reserved for air
                        self.ids[(curr_block_id + 1) as usize] = None;
                    }
                }
            }

            self.is_empty = self.air_count == self.blocks.len();

            true
        } else {
            false
        }
    }

    pub fn remove_block(&mut self, x: usize, y: usize, z: usize) -> bool {
        self.add_block(x, y, z, None)
    }

    pub fn get_local_block_id(&self, x: usize, y: usize, z: usize) -> u16 {
        let index = pos_as_index(x, y, z);

        if index < self.blocks.len() {
            self.blocks[index]
        } else {
            0
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        let block_id = self.get_local_block_id(x, y, z);
        
        if block_id > 0 {
            if let Some(block_string_id) = &self.ids[block_id as usize - 1] {
                registry::get_block_from_registry_by_string(&block_string_id)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns `false` if the local block ID at `(x, y, z)` is 0 (air).
    /// Returns `true` otherwise.
    pub fn has_block_at(&self, x: usize, y: usize, z: usize) -> bool {
        self.get_local_block_id(x, y, z) > 0
    }

    pub fn local_to_world_pos(&self, x: usize, y: usize, z: usize) -> Vec3 {
        Vec3::new(
            self.chunk_pos.x as f32 * CHUNK_SIZE as f32 + x as f32,
            self.chunk_pos.y as f32 * CHUNK_SIZE as f32 + y as f32,
            self.chunk_pos.z as f32 * CHUNK_SIZE as f32 + z as f32,
        )
    }
}

pub fn pos_as_index(local_x: usize, local_y: usize, local_z: usize) -> usize {
    //local_x + local_y * CHUNK_SIZE + local_z * CHUNK_SIZE * CHUNK_SIZE
    local_x | local_y << BLOCK_Y_SHIFT | local_z << BLOCK_Z_SHIFT
}

pub fn index_as_pos(index: usize) -> [usize; 3] {
    let block_x = index & 0xF;
    let block_y = (index >> BLOCK_Y_SHIFT) & 0xF;
    let block_z = (index >> BLOCK_Z_SHIFT) & 0xF;

    [block_x, block_y, block_z]
}

pub fn build_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let index = pos_as_index(x, y, z);

                // TODO: return Err if index > blocks.len()
                if index > chunk.blocks.len() {
                    continue;
                }

                let is_block = chunk.blocks[index] > 0;

                if is_block {
                    let cull_code = cull_neighbors(&chunk, x, y, z);

                    let block_pos = chunk.local_to_world_pos(x, y, z);

                    if let Some(block_id) = &chunk.ids[chunk.blocks[index] as usize - 1] {
                        if let Some(block) = registry::get_block_from_registry(&Identifier::from(block_id).unwrap()) {
                            if (cull_code & (VoxelCullCode::U as u8)) == VoxelCullCode::U as u8 {
                                build_face(
                                    &mut positions,
                                    &mut normals,
                                    &mut uvs,
                                    &mut indices,
                                    VERTICES_TOP,
                                    &mut block.get_uvs_top(),
                                    &block_pos,
                                );
                            }
    
                            if (cull_code & (VoxelCullCode::D as u8)) == VoxelCullCode::D as u8 {
                                build_face(
                                    &mut positions,
                                    &mut normals,
                                    &mut uvs,
                                    &mut indices,
                                    VERTICES_BOTTOM,
                                    &mut block.get_uvs_bottom(),
                                    &block_pos,
                                );
                            }
    
                            if (cull_code & (VoxelCullCode::R as u8)) == VoxelCullCode::R as u8 {
                                build_face(
                                    &mut positions,
                                    &mut normals,
                                    &mut uvs,
                                    &mut indices,
                                    VERTICES_RIGHT,
                                    &mut block.get_uvs_right(),
                                    &block_pos,
                                );
                            }
    
                            if (cull_code & (VoxelCullCode::L as u8)) == VoxelCullCode::L as u8 {
                                build_face(
                                    &mut positions,
                                    &mut normals,
                                    &mut uvs,
                                    &mut indices,
                                    VERTICES_LEFT,
                                    &mut block.get_uvs_left(),
                                    &block_pos,
                                );
                            }
    
                            if (cull_code & (VoxelCullCode::F as u8)) == VoxelCullCode::F as u8 {
                                build_face(
                                    &mut positions,
                                    &mut normals,
                                    &mut uvs,
                                    &mut indices,
                                    VERTICES_FRONT,
                                    &mut block.get_uvs_front(),
                                    &block_pos,
                                );
                            }
    
                            if (cull_code & (VoxelCullCode::B as u8)) == VoxelCullCode::B as u8 {
                                build_face(
                                    &mut positions,
                                    &mut normals,
                                    &mut uvs,
                                    &mut indices,
                                    VERTICES_BACK,
                                    &mut block.get_uvs_back(),
                                    &block_pos,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    let mut mesh = Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleList);

    //println!("{:?}", uvs);
    //panic!("at the disco");

    mesh.set_indices(Some(Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}

fn build_face(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indicies: &mut Vec<u32>,
    block_face: &[([f32; 3], [f32; 3]); 4],
    block_uvs: &mut Vec<[f32;2]>,
    block_pos: &Vec3
) {
    let block_indicies: Vec<u32> = vec![
        0, 1, 2, // triangle 1
        2, 3, 0, // triangle 2
    ];

    let index = positions.len() as u32;
    
    for (position, normal) in block_face {
        let pos = [
            block_pos.x + position[0],
            block_pos.y + position[1],
            block_pos.z + position[2]
        ];

        let norm = [
            block_pos.x + normal[0],
            block_pos.y + normal[1],
            block_pos.z + normal[2]
        ];

        positions.push(pos);
        normals.push(norm);
    }
    
    uvs.append(block_uvs);

    for f_index in &block_indicies {
        indicies.push(f_index.clone() + index);
    }
}