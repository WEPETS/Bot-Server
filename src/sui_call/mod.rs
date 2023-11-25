pub mod call_api;
pub mod read_api;
pub mod sui_move_object;
pub mod utils;

pub type Result<T> = core::result::Result<T, anyhow::Error>;

pub const ADMIN_OBJECT_NAME: &str = "GameAdmin";
pub const BOT_OBJECT_NAME: &str = "Bot";
pub const PET_OBJECT_NAME: &str = "Pet";
pub const HERO_OBJECT_NAME: &str = "Hero";
pub const MODULE_NAME: &str = "we_pet_game";
