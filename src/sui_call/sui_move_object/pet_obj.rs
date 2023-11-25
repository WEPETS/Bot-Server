use std::collections::BTreeMap;

use sui_json_rpc_types::SuiMoveValue;

use super::FromSuiMoveStruct;

#[derive(Debug, serde::Deserialize)]
pub struct SuiPetObject {
    pub id: String,
    pub hp: u32,
    pub exp: u32,
    pub game_id: String,
    pub strength: u32,
}

impl FromSuiMoveStruct for SuiPetObject {
    fn from_sui_move_struct(field_map: BTreeMap<String, SuiMoveValue>) -> Self {
        SuiPetObject {
            exp: field_map
                .get("exp")
                .and_then(|v| match v {
                    SuiMoveValue::Number(s) => Some(*s),
                    _ => None,
                })
                .unwrap_or_default(),
            game_id: field_map
                .get("game_id")
                .and_then(|v| match v {
                    SuiMoveValue::Address(id) => Some(id.to_string()),
                    _ => None,
                })
                .unwrap_or_default(),
            hp: field_map
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
            strength: field_map
                .get("strength")
                .and_then(|v| match v {
                    SuiMoveValue::Number(s) => Some(*s),
                    _ => None,
                })
                .unwrap_or_default(),
        }
    }
}
