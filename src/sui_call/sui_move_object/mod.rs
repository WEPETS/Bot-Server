pub mod admin_obj;
pub mod bot_obj;
pub mod hero_obj;
pub mod pet_obj;

use std::collections::BTreeMap;
use sui_json_rpc_types::SuiMoveValue;

pub trait FromSuiMoveStruct {
    fn from_sui_move_struct(fields: BTreeMap<String, SuiMoveValue>) -> Self;
}
