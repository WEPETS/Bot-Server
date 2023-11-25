use std::collections::BTreeMap;

use sui_json_rpc_types::SuiMoveValue;

use super::FromSuiMoveStruct;

#[derive(Debug, serde::Deserialize)]
pub struct SuiHeroObject {
    pub id: String,
    pub level: u32,
    pub game_id: String,
}

impl FromSuiMoveStruct for SuiHeroObject {
    fn from_sui_move_struct(field_map: BTreeMap<String, SuiMoveValue>) -> Self {
        SuiHeroObject {
            game_id: field_map
                .get("game_id")
                .and_then(|v| match v {
                    SuiMoveValue::Address(id) => Some(id.to_string()),
                    _ => None,
                })
                .unwrap_or_default(),
            level: field_map
                .get("hp")
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
