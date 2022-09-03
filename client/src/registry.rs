use std::sync::Mutex;

use bevy::{sprite::{TextureAtlas, Rect}, prelude::{Res, AssetServer, Plugin, Commands}};
use hashbrown::HashMap;
use iyes_loopless::{prelude::AppLooplessStateExt, state::NextState};
use lazy_static::lazy_static;

use crate::{item::ItemDefinition, identifier::Identifier, BlockyPathError, block::{Block, BlockDefinition, BlockFace}, texture_atlas::atlas_coords_fix, AppState};

lazy_static! {
    static ref ITEM_REGISTRY: Mutex<HashMap<String, ItemDefinition>> = Mutex::new(HashMap::new());
    static ref BLOCK_REGISTRY: Mutex<HashMap<String, Block>> = Mutex::new(HashMap::new());
    static ref BLOCK_TEXTURE_COORDS: Mutex<HashMap<String, Rect>> = Mutex::new(HashMap::new());
}

pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_enter_system(AppState::Registry, registry_init);
    }
}

pub fn registry_init(mut commands: Commands) {
    register_items_in_dir("data/blocky/items/");
    register_blocks_in_dir("data/blocky/blocks/");

    commands.insert_resource(NextState(AppState::Finished))
}

/// Store coordinates for a block texture
/// for retrieval later
pub fn register_block_texture_coords(
    texture_path: String,
    atlas: &TextureAtlas,
    asset_server: &Res<AssetServer>,
) {
    let mut tex_coords_registry = BLOCK_TEXTURE_COORDS.lock().unwrap();

    let tex_handle = asset_server.get_handle(&texture_path);
    let tex_index = atlas.get_texture_index(&tex_handle).unwrap();

    let texture_size = atlas_coords_fix(atlas.textures[tex_index], atlas.size);

    if !tex_coords_registry.contains_key(&texture_path) {
        tex_coords_registry.insert(texture_path.clone(), texture_size);

        println!("Registered atlas coords for \"{}\"", texture_path);
    } else {
        // overwrite key if it exists
        tex_coords_registry.entry(texture_path.clone()).and_modify(|e| *e = texture_size);

        println!("atlas coords for \"{}\" already registered; overwriting!", texture_path);
    }
}

pub fn get_block_texture_coords(texture_path: String) -> Option<Rect> {
    let tex_coords_registry = BLOCK_TEXTURE_COORDS.lock().unwrap();

    if tex_coords_registry.contains_key(&texture_path) {
        let registered_tex = tex_coords_registry[&texture_path].clone();
        
        Some(registered_tex)
    } else {
        None
    }
}

/// Adds a block to the block registry
pub fn register_block(
    block_def: BlockDefinition
) {
    let mut block_registry = BLOCK_REGISTRY.lock().unwrap();

    let id = match Identifier::from_str(&block_def.id) {
        Ok(id) => Some(id),
        Err(err) => {
            println!("{err}");
            None
        }
    };
    
    if let Some(id) = id {
        let top_texture_path = block_def.get_texture_for_face(BlockFace::Top);
        let btm_texture_path = block_def.get_texture_for_face(BlockFace::Bottom);
        let left_texture_path = block_def.get_texture_for_face(BlockFace::Left);
        let right_texture_path = block_def.get_texture_for_face(BlockFace::Right);
        let front_texture_path = block_def.get_texture_for_face(BlockFace::Front);
        let back_texture_path = block_def.get_texture_for_face(BlockFace::Back);

        // unwrap shouldnt fail here
        let texture_top = get_block_texture_coords(top_texture_path.unwrap()).unwrap();
        let texture_btm = get_block_texture_coords(btm_texture_path.unwrap()).unwrap();
        let texture_left = get_block_texture_coords(left_texture_path.unwrap()).unwrap();
        let texture_right = get_block_texture_coords(right_texture_path.unwrap()).unwrap();
        let texture_front = get_block_texture_coords(front_texture_path.unwrap()).unwrap();
        let texture_back = get_block_texture_coords(back_texture_path.unwrap()).unwrap();

        let block = Block {
            id: id.clone(),
            texture_front,
            texture_back,
            texture_top,
            texture_btm,
            texture_left,
            texture_right,
        };

        if !block_registry.contains_key(&id.as_string()) {
            block_registry.insert(id.as_string(), block);

            println!("Registered block \"{}\"", id.as_string());
        } else {
            // overwrite key if it exists
            block_registry.entry(id.as_string()).and_modify(|e| *e = block);

            println!("block \"{}\" already registered; overwriting!", id.as_string());
        }
    }
}

/// Registers any blocks found in a folder relative to the `assets` folder
pub fn register_blocks_in_dir(path: &str) {
    // load items
    match load_blocks_from_path(&format!("assets/{path}")) {
        Ok(block_defs) => {
            for block_def_res in block_defs {
                match block_def_res {
                    Ok(block_def) => {
                        register_block(block_def)
                    },
                    Err(err) => println!("{}", err)
                }
            }
        },
        Err(err) => println!("{}", err)
    }
}

pub fn load_blocks_from_path(path: &str) -> Result<Vec<Result<BlockDefinition, BlockyPathError>>, BlockyPathError> {
    let mut block_defs = Vec::new();
    
    let block_paths = std::fs::read_dir(path).map_err(|source|
        BlockyPathError::DirectoryReadError(String::from(path), source)
    )?;

    for block_def_path in block_paths {
        let block_path_res = block_def_path.map_err(|source|
            BlockyPathError::PathReadError(String::from(path), source)
        );

        match block_path_res {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                let file_path = path.to_str().unwrap();


                match ron::from_str::<BlockDefinition>(&std::fs::read_to_string(file_path).unwrap()) {
                    Ok(block_def) => block_defs.push(Ok(block_def)),
                    Err(err) => block_defs.push(Err(BlockyPathError::FileParseError(String::from(file_path), err)))
                }
            },
            Err(err) => block_defs.push(Err(err))
        }
    }

    Ok(block_defs)
}

pub fn get_block_from_registry(block_id: &Identifier) -> Option<Block> {
    get_block_from_registry_by_string(&block_id.as_string())
}

pub fn get_block_from_registry_by_string(block_id: &str) -> Option<Block> {
    let block_registry = BLOCK_REGISTRY.lock().unwrap();

    if block_registry.contains_key(block_id) {
        let registered_block = block_registry[block_id].clone();
        
        Some(registered_block)
    } else {
        None
    }
}

/// Adds an item to the item registry
pub fn register_item(item_def: ItemDefinition) {
    let mut item_registry = ITEM_REGISTRY.lock().unwrap();

    let id = match Identifier::from_str(&item_def.id) {
        Ok(id) => Some(id),
        Err(err) => {
            println!("{err}");
            None
        }
    };
    
    if let Some(id) = id {
        if !item_registry.contains_key(&id.as_string()) {
            item_registry.insert(id.as_string(), item_def);

            println!("Registered item \"{}\"", id.as_string());
        } else {
            // overwrite key if it exists
            item_registry.entry(id.as_string()).and_modify(|e| *e = item_def);

            println!("item \"{}\" already registered; overwriting!", id.as_string());
        }
    }
}

/// Registers any items found in a folder relative to the `assets` folder
pub fn register_items_in_dir(path: &str) {
    // load items
    match load_items_from_path(&format!("assets/{path}")) {
        Ok(item_defs) => {
            for item_def_res in item_defs {
                match item_def_res {
                    Ok(item_def) => {
                        register_item(item_def)
                    },
                    Err(err) => println!("{}", err)
                }
            }
        },
        Err(err) => println!("{}", err)
    }
}

pub fn load_items_from_path(path: &str) -> Result<Vec<Result<ItemDefinition, BlockyPathError>>, BlockyPathError> {
    let mut item_defs = Vec::new();
    
    let item_paths = std::fs::read_dir(path).map_err(|source|
        BlockyPathError::DirectoryReadError(String::from(path), source)
    )?;

    for item_def_path in item_paths {
        let item_path_res = item_def_path.map_err(|source|
            BlockyPathError::PathReadError(String::from(path), source)
        );

        match item_path_res {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                let file_path = path.to_str().unwrap();


                match ron::from_str::<ItemDefinition>(&std::fs::read_to_string(file_path).unwrap()) {
                    Ok(item_def) => item_defs.push(Ok(item_def)),
                    Err(err) => item_defs.push(Err(BlockyPathError::FileParseError(String::from(file_path), err)))
                }
            },
            Err(err) => item_defs.push(Err(err))
        }
    }

    Ok(item_defs)
}