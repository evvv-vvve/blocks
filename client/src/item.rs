use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ToolType {
    Sword,
    Axe,
    Pickaxe,
    Hoe,
    Shovel,
    Bow
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ItemTrait {
    Durability {
        max: f32,
        break_on_empty: bool
    },
    Consumable {
        restoration: f32,
        consumption_time: f32,
    },
    Tool {
        base_damage: f32,
        tool_type: ToolType,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemDefinition {
    pub id: String,
    pub stack_size: i32,
    pub item_traits: Vec<ItemTrait>,
}