use std::collections::BTreeMap;

use super::FromSuiMoveStruct;
use sui_json_rpc_types::SuiMoveValue;
use sui_types::id::UID;

#[derive(Debug, serde::Deserialize)]
pub struct SuiAdminObject {
    pub id: String,
    pub bot_animal_created: u32,
    pub game_id: String,
}

impl FromSuiMoveStruct for SuiAdminObject {
    fn from_sui_move_struct(field_map: BTreeMap<String, SuiMoveValue>) -> Self {
        SuiAdminObject {
            game_id: field_map
                .get("game_id")
                .and_then(|v| match v {
                    SuiMoveValue::Address(id) => Some(id.to_string()),
                    _ => None,
                })
                .unwrap_or_default(),
            bot_animal_created: field_map
                .get("bot_animal_created")
                .and_then(|v| match v {
                    SuiMoveValue::Number(s) => Some(*s),
                    _ => None,
                })
                .unwrap_or_default(),
            id: field_map
                .get("id")
                .and_then(|v| match v {
                    SuiMoveValue::UID { id } => Some(id.to_string()),
                    _ => None,
                })
                .unwrap_or_default(),
        }
    }
}
