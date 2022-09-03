use bevy::{
    asset::LoadState,
    prelude::*, sprite::{TextureAtlasBuilderError, Rect}
};
use iyes_loopless::prelude::*;

use crate::{AppState, registry::register_block_texture_coords};

#[derive(Default)]
pub struct TextureHandles {
    pub block_texture_handles: Vec<HandleUntyped>,
    pub item_texture_handles: Vec<HandleUntyped>,
}

#[derive(PartialEq)]
pub enum TextureBuildState {
    LoadTextures,
    BuildAtlas,
}

pub struct TextureAtlasesPlugin;

impl Plugin for TextureAtlasesPlugin {
    fn build(&self, app: &mut App) {
		app.insert_resource(TextureAtlasHandles::default())
           .insert_resource(TextureBuildState::LoadTextures)
           .add_enter_system(AppState::LoadResources, load_textures)
           .add_system_set(
              ConditionSet::new()
                .run_in_state(AppState::LoadResources)
                .label("load-textures")
                .with_system(check_textures)
                .with_system(build_texture_atlas)
                .into()
        );
	}
}

pub fn load_textures(
    mut texture_handles: ResMut<TextureHandles>,
    asset_server: Res<AssetServer>,
) {
    texture_handles.block_texture_handles = asset_server.load_folder("textures/block").unwrap();
    texture_handles.item_texture_handles = asset_server.load_folder("textures/item").unwrap();

    /*for handle in &texture_handles.block_texture_handles {
        if let Some(img) = asset_server.get_handle_path(handle) {
            if let Some(label) = img.label() {
                println!("{label}");
            }

            if let Some(path) = img.path().to_str() {
                println!("{path}");
            }
        }
    }*/
}

pub fn check_textures(
    mut texture_build_state: ResMut<TextureBuildState>,
    texture_handles: ResMut<TextureHandles>,
    asset_server: Res<AssetServer>,
) {
    let block_textures_states = asset_server.get_group_load_state(
        texture_handles.block_texture_handles.iter()
          .map(|handle| handle.id)
    );

    let item_textures_states = asset_server.get_group_load_state(
        texture_handles.item_texture_handles.iter()
          .map(|handle| handle.id)
    );
    
    if LoadState::Loaded == block_textures_states &&
       LoadState::Loaded == item_textures_states {
        *texture_build_state = TextureBuildState::BuildAtlas;
    }
}

pub struct TextureAtlasHandles {
    pub block_atlas: Option<Handle<TextureAtlas>>,
    pub item_atlas: Option<Handle<TextureAtlas>>,
}

impl Default for TextureAtlasHandles {
    fn default() -> Self {
        Self { block_atlas: None, item_atlas: None }
    }
}

pub fn build_texture_atlas(
    mut commands: Commands,
    texture_build_state: Res<TextureBuildState>,
    asset_server: Res<AssetServer>,
    texture_handles: Res<TextureHandles>,
    mut our_atlases: ResMut<TextureAtlasHandles>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Image>>,
) {
    if *texture_build_state != TextureBuildState::BuildAtlas {
        return;
    }

    let block_texture_atlas = build_atlas(
        &asset_server,
        &texture_handles.block_texture_handles,
        &mut textures
    ).unwrap();
    
    //let block_texture_atlas_texture = block_texture_atlas.texture.clone();
    //let grass_block_handle = asset_server.get_handle("textures/block/grass_block_side.png");
    //let grass_block_index = block_texture_atlas.get_texture_index(&grass_block_handle).unwrap();
    let block_atlas_handle = texture_atlases.add(block_texture_atlas.clone());

    our_atlases.block_atlas = Some(block_atlas_handle);

    for (texture_handle, _) in &block_texture_atlas.texture_handles.clone().unwrap() {
        if let Some(asset_path) = asset_server.get_handle_path(texture_handle) {
            let tex_path = String::from(asset_path.path().to_str().unwrap());
            register_block_texture_coords(tex_path, &block_texture_atlas, &asset_server);
        }
    }
    

    let item_texture_atlas = build_atlas(
        &asset_server,
        &texture_handles.item_texture_handles,
        &mut textures
    ).unwrap();

    //let item_texture_atlas_texture = item_texture_atlas.texture.clone();
    //let itemtest_block_handle = asset_server.get_handle("textures/item/item_test.png");
    //let itemtest_block_index = item_texture_atlas.get_texture_index(&itemtest_block_handle).unwrap();
    let item_atlas_handle = texture_atlases.add(item_texture_atlas);

    our_atlases.item_atlas = Some(item_atlas_handle);

    commands.insert_resource(NextState(AppState::Registry))
}

/// Returned when there is an error when loading textures/
/// creating a texture atlas
#[derive(thiserror::Error, Debug)]
pub enum TextureError {
    #[error("{0} did not create an `Image` asset")]
    ImageAssetError(String),
    
    #[error("An error occurred while building texture atlas: {0}")]
    TextureAtlasBuilderError(TextureAtlasBuilderError),
}

fn build_atlas(
    asset_server: &Res<AssetServer>,
    texture_handles: &Vec<HandleUntyped>,
    mut textures: &mut ResMut<Assets<Image>>,
) -> Result<TextureAtlas, TextureError> {
    let mut atlas_builder = TextureAtlasBuilder::default();
    for handle in texture_handles {
        let handle = handle.typed_weak();
        
        match textures.get(&handle) {
            Some(texture) => atlas_builder.add_texture(handle, texture),
            None => return Err(TextureError::ImageAssetError(
                match asset_server.get_handle_path(handle) {
                    Some(path) => String::from(path.path().to_str().unwrap()),
                    None => String::from("{unknown path}")
                }
            ))
        }
    }

    match atlas_builder.finish(&mut textures) {
        Ok(atlas) => Ok(atlas),
        // we love writing 'err' :o)
        Err(atlas_err) => Err(TextureError::TextureAtlasBuilderError(atlas_err))
    }
}

/// Makes sure that texture UVs are between 0 and 1
pub fn atlas_coords_fix(texture_pos: Rect, size: Vec2) -> Rect {
    Rect {
        min: texture_pos.min / size,
        max: texture_pos.max / size
    }
}