use bevy::sprite::Rect;
use serde::{Deserialize, Serialize};

use crate::{chunky::{Chunk, CHUNK_SIZE}, identifier::Identifier};

pub const FACE_INDICES: &[u32; 6] = &[
    0, 1, 2, // triangle 1
    2, 3, 0, // triangle 2
];

pub const VERTICES_TOP: &[([f32;3], [f32;3]); 4] = &[
    ([ 0.5, 0.5, -0.5], [0., 1., 0.]),
    ([-0.5, 0.5, -0.5], [0., 1., 0.]),
    ([-0.5, 0.5,  0.5], [0., 1., 0.]),
    ([ 0.5, 0.5,  0.5], [0., 1., 0.]),
];

pub const VERTICES_BOTTOM: &[([f32;3], [f32;3]); 4] = &[
    ([ 0.5, -0.5,  0.5], [0., -1., 0.]),
    ([-0.5, -0.5,  0.5], [0., -1., 0.]),
    ([-0.5, -0.5, -0.5], [0., -1., 0.]),
    ([ 0.5, -0.5, -0.5], [0., -1., 0.]),
];

pub const VERTICES_RIGHT: &[([f32;3], [f32;3]); 4] = &[
    ([-0.5,  0.5,  0.5], [1., 0., 0.]),
    ([-0.5,  0.5, -0.5], [1., 0., 0.]),
    ([-0.5, -0.5, -0.5], [1., 0., 0.]),
    ([-0.5, -0.5,  0.5], [1., 0., 0.]),
];

pub const VERTICES_LEFT: &[([f32;3], [f32;3]); 4] = &[
    ([0.5,  0.5, -0.5], [1., 0., 0.]),
    ([0.5,  0.5,  0.5], [1., 0., 0.]),
    ([0.5, -0.5,  0.5], [1., 0., 0.]),
    ([0.5, -0.5, -0.5], [1., 0., 0.]),
];

pub const VERTICES_FRONT: &[([f32;3], [f32;3]); 4] = &[
    ([-0.5,  0.5, -0.5], [0., 0., -1.]),
    ([ 0.5,  0.5, -0.5], [0., 0., -1.]),
    ([ 0.5, -0.5, -0.5], [0., 0., -1.]),
    ([-0.5, -0.5, -0.5], [0., 0., -1.]),
];

pub const VERTICES_BACK: &[([f32;3], [f32;3]); 4] = &[
    ([-0.5, -0.5, 0.5], [0., 0., 1.]),
    ([ 0.5, -0.5, 0.5], [0., 0., 1.]),
    ([ 0.5,  0.5, 0.5], [0., 0., 1.]),
    ([-0.5,  0.5, 0.5], [0., 0., 1.]),
];

pub enum BlockFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockDefinition {
    pub id: String,

    #[serde(default)]
    pub texture: String,

    #[serde(default)]
    pub top_texture: String,
    #[serde(default)]
    pub bottom_texture: String,
    #[serde(default)]
    pub left_texture: String,
    #[serde(default)]
    pub right_texture: String,
    #[serde(default)]
    pub front_texture: String,
    #[serde(default)]
    pub back_texture: String,
}

impl BlockDefinition {
    pub fn get_texture_for_face(&self, block_face: BlockFace) -> Option<String> {
        match Identifier::from_str(&self.id) {
            Ok(id) => {
                let texture_face = match block_face {
                    BlockFace::Top => &self.top_texture,
                    BlockFace::Bottom => &self.bottom_texture,
                    BlockFace::Left => &self.left_texture,
                    BlockFace::Right => &self.right_texture,
                    BlockFace::Front => &self.front_texture,
                    BlockFace::Back => &self.back_texture,
                }.clone();
        
                let texture = if !self.texture.is_empty() {
                    self.texture.clone()
                } else {
                    if texture_face.is_empty() {
                        id.get_name()
                    } else {
                        texture_face
                    }
                };
        
                Some(format!("textures/block\\{}.png", texture))
            },
            Err(err) => {
                println!("[Error] {err}");
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextureCoords {
    pub bottom_left_x: f32,
    pub bottom_left_y: f32,

    pub top_right_x: f32,
    pub top_right_y: f32
}

#[derive(Clone)]
pub struct Block {
    pub(crate) id: Identifier,

    pub(crate) texture_front: Rect, //TextureCoords,
    pub(crate) texture_back: Rect, //TextureCoords,
    pub(crate) texture_top: Rect, //TextureCoords,
    pub(crate) texture_btm: Rect, //TextureCoords,
    pub(crate) texture_left: Rect, //TextureCoords,
    pub(crate) texture_right: Rect, //TextureCoords,
}

impl Block {
    pub fn get_identifier(&self) -> Identifier { self.id.clone() }

    pub fn get_uvs_top(&self) -> Vec<[f32;2]> { 
        let mut uvs: Vec<[f32;2]> = Vec::new();

        uvs.push([ self.texture_top.min.x, self.texture_top.min.y ]);
        uvs.push([ self.texture_top.max.x, self.texture_top.min.y ]);
        uvs.push([ self.texture_top.max.x, self.texture_top.max.y ]);
        uvs.push([ self.texture_top.min.x, self.texture_top.max.y ]);

        uvs
    }

    pub fn get_uvs_bottom(&self) -> Vec<[f32;2]> {
        let mut uvs: Vec<[f32;2]> = Vec::new();

        uvs.push([ self.texture_btm.max.x, self.texture_btm.min.y ]);
        uvs.push([ self.texture_btm.min.x, self.texture_btm.min.y ]);
        uvs.push([ self.texture_btm.min.x, self.texture_btm.max.y ]);
        uvs.push([ self.texture_btm.max.x, self.texture_btm.max.y ]);

        uvs
    }

    pub fn get_uvs_left(&self) -> Vec<[f32;2]> {
        let mut uvs: Vec<[f32;2]> = Vec::new();

        uvs.push([ self.texture_left.max.x, self.texture_left.min.y ]);
        uvs.push([ self.texture_left.min.x, self.texture_left.min.y ]);
        uvs.push([ self.texture_left.min.x, self.texture_left.max.y ]);
        uvs.push([ self.texture_left.max.x, self.texture_left.max.y ]);

        uvs
    }

    pub fn get_uvs_right(&self) -> Vec<[f32;2]> {
        let mut uvs: Vec<[f32;2]> = Vec::new();

        uvs.push([ self.texture_right.min.x, self.texture_right.min.y ]);
        uvs.push([ self.texture_right.max.x, self.texture_right.min.y ]);
        uvs.push([ self.texture_right.max.x, self.texture_right.max.y ]);
        uvs.push([ self.texture_right.min.x, self.texture_right.max.y ]);

        uvs
    }

    pub fn get_uvs_front(&self) -> Vec<[f32;2]> {
        let mut uvs: Vec<[f32;2]> = Vec::new();

        uvs.push([ self.texture_front.max.x, self.texture_front.min.y ]);
        uvs.push([ self.texture_front.min.x, self.texture_front.min.y ]);
        uvs.push([ self.texture_front.min.x, self.texture_front.max.y ]);
        uvs.push([ self.texture_front.max.x, self.texture_front.max.y ]);

        uvs
    }

    pub fn get_uvs_back(&self) -> Vec<[f32;2]> {
        let mut uvs: Vec<[f32;2]> = Vec::new();

        uvs.push([ self.texture_back.max.x, self.texture_back.max.y ]);
        uvs.push([ self.texture_back.min.x, self.texture_back.max.y ]);
        uvs.push([ self.texture_back.min.x, self.texture_back.min.y ]);
        uvs.push([ self.texture_back.max.x, self.texture_back.min.y ]);

        uvs
    }
}

// Credit: https://www.reddit.com/r/Unity3D/comments/5ys3vc/voxel_face_culling/desvzlu/
// Archived at: https://web.archive.org/web/20210528184220/https://www.reddit.com/r/Unity3D/comments/5ys3vc/voxel_face_culling/desvzlu/

pub enum VoxelCullCode
{
    Default = 0, //0000
    U = 1,       //0001
    D = 2,       //0010
    R = 4,       //0100
    L = 8,       //1000
    RL = 12,     //1100
    UD = 3,      //0011
    UR = 5,      //0101
    UL = 9,      //1001
    DR = 6,      //0110
    DL = 10,     //1010
    UDR = 7,     //0111
    UDL = 11,    //1011
    URL = 13,    //1101
    DRL = 14,    //1110
    UDRL = 15,   //1111

    B = 16,      //0001 0000
    BU = 17,     //0001 0001
    BD = 18,     //0001 0010
    BR = 20,     //0001 0100
    BL = 24,     //0001 1000
    BRL = 28,    //0001 1100
    BUD = 19,    //0001 0011
    BUR = 21,    //0001 0101
    BUL = 25,    //0001 1001
    BDR = 22,    //0001 0110
    BDL = 26,    //0001 1010
    BUDR = 23,   //0001 0111
    BUDL = 27,   //0001 1011
    BURL = 29,   //0001 1101
    BDRL = 30,   //0001 1110
    BUDRL = 31,  //0001 1111

    F = 32,      //0010 0000
    FU = 33,     //0010 0001
    FD = 34,     //0010 0010
    FR = 36,     //0010 0100
    FL = 40,     //0010 1000
    FRL = 44,    //0010 1100
    FUD = 35,    //0010 0011
    FUR = 37,    //0010 0101
    FUL = 41,    //0010 1001
    FDR = 38,    //0010 0110
    FDL = 42,    //0010 1010
    FUDR = 39,   //0010 0111
    FUDL = 43,   //0010 1011
    FURL = 45,   //0010 1101
    FDRL = 46,   //0010 1110
    FUDRL = 47,  //0010 1111

    BF = 48,     //0011 0000
    BFU = 49,    //0011 0001
    BFD = 50,    //0011 0010
    BFR = 52,    //0011 0100
    BFL = 56,    //0011 1000
    BFRL = 60,   //0011 1100
    BFUD = 51,   //0011 0011
    BFUR = 53,   //0011 0101
    BFUL = 57,   //0011 1001
    BFDR = 54,   //0011 0110
    BFDL = 58,   //0011 1010
    BFUDR = 55,  //0011 0111
    BFUDL = 59,  //0011 1011
    BFURL = 61,  //0011 1101
    BFDRL = 62,  //0011 1110
    BFUDRL = 63, //0011 1111
}

pub fn cull_neighbors(chunk: &Chunk, x: usize, y: usize, z: usize) -> u8 {
    let mut code = 0;

    if x > 0 {
        code = if chunk.has_block_at(x - 1, y, z) { code } else { code | (VoxelCullCode::R as u8) }
    } else {
        code |= VoxelCullCode::R as u8;
    }

    if z > 0 {
        code = if chunk.has_block_at(x, y, z - 1) { code } else { code | VoxelCullCode::F as u8 }
    }
    else {
        code |= VoxelCullCode::F as u8;
    }


    if x < CHUNK_SIZE - 1 {
        code = if chunk.has_block_at(x + 1, y, z) { code } else { code | VoxelCullCode::L as u8 }
    } else {
        code |= VoxelCullCode::L as u8;
    }

    if z < CHUNK_SIZE - 1 {
        code = if chunk.has_block_at(x, y, z + 1) { code } else { code | VoxelCullCode::B as u8 }
    } else {
        code |= VoxelCullCode::B as u8;
    }


    if y < CHUNK_SIZE - 1 {
        code = if chunk.has_block_at(x, y + 1, z) { code } else { code | VoxelCullCode::U as u8 }
    } else {
        code |= VoxelCullCode::U as u8;
    }
    
    if y > 0 {
        code = if chunk.has_block_at(x, y - 1, z) { code } else { code | VoxelCullCode::D as u8 }
    } else {
        code |= VoxelCullCode::D as u8;
    }

    code
}