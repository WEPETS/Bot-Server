use super::filter::{new_filter, SuiStructTag};
use crate::{
    config::Config,
    get_config,
    sui_call::{
        sui_move_object::{
            admin_obj::SuiAdminObject, bot_obj::SuiBotObject, pet_obj::SuiPetObject,
            FromSuiMoveStruct,
        },
        Result, ADMIN_OBJECT_NAME, BOT_OBJECT_NAME, MODULE_NAME,
    },
};
use sqlx::any;
use sui_json_rpc_types::{
    SuiMoveStruct, SuiObjectData, SuiObjectDataFilter, SuiObjectDataOptions, SuiObjectResponse,
    SuiObjectResponseQuery, SuiParsedData, SuiParsedMoveObject,
};
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, SuiAddress};

pub struct WePetGame {
    sui: SuiClient,
    adrr: SuiAddress,
    package_id: String,
}

impl WePetGame {
    pub fn new(sui: SuiClient, adrr: SuiAddress, package_id: &str) -> Self {
        WePetGame {
            sui,
            adrr,
            package_id: package_id.to_string(),
        }
    }
}

impl WePetGame {
    pub async fn get_sui_obj_first<T>(&self, name: &str) -> Result<T>
    where
        T: FromSuiMoveStruct,
    {
        let sui_data_filter = new_filter(
            SuiStructTag::builder()
                .package(self.package_id.as_str())
                .module(MODULE_NAME)
                .name(name)
                .build(),
        );

        let query = new_default_query(sui_data_filter);

        let response = get_objects_response(&self.sui, &self.adrr, query)
            .await?
            .into_iter()
            .next();

        process_response::<T>(response).await
    }

    pub async fn get_sui_objs<T>(&self, name: &str) -> Result<Vec<T>>
    where
        T: FromSuiMoveStruct,
    {
        let sui_data_filter = new_filter(
            SuiStructTag::builder()
                .package(self.package_id.as_str())
                .module(MODULE_NAME)
                .name(name)
                .build(),
        );

        let query = new_default_query(sui_data_filter.clone());

        let response = get_objects_response(&self.sui, &self.adrr, query).await?;

        let mut vec = Vec::new();

        for i in response {
            vec.push(process_response::<T>(Some(i)).await?)
        }

        Ok(vec)
    }
}

fn new_default_query(sui_data_filter: SuiObjectDataFilter) -> SuiObjectResponseQuery {
    SuiObjectResponseQuery::new(
        Some(sui_data_filter),
        Some(SuiObjectDataOptions::new().with_content()),
    )
}

async fn get_objects_response(
    sui: &SuiClient,
    adrr: &SuiAddress,
    query: SuiObjectResponseQuery,
) -> Result<Vec<SuiObjectResponse>> {
    Ok(sui
        .read_api()
        .get_owned_objects(*adrr, Some(query), None, Some(10))
        .await?
        .data)
}

async fn process_response<T>(response: Option<SuiObjectResponse>) -> Result<T>
where
    T: FromSuiMoveStruct,
{
    if let Some(SuiObjectResponse {
        data:
            Some(SuiObjectData {
                content:
                    Some(SuiParsedData::MoveObject(SuiParsedMoveObject {
                        fields: SuiMoveStruct::WithFields(field_map),
                        ..
                    })),
                ..
            }),
        ..
    }) = response
    {
        println!("{:?}", field_map);
        let obj: T = FromSuiMoveStruct::from_sui_move_struct(field_map);
        Ok(obj)
    } else {
        Err(anyhow::Error::msg("No valid response"))
    }
}

// region:    --- Tests
#[cfg(test)]
mod tests {
    use std::{path::Path, str::FromStr};

    use dotenvy::dotenv;
    use serial_test::serial;
    use sui_keys::keystore::{FileBasedKeystore, Keystore};
    use sui_sdk::{SuiClient, SuiClientBuilder};
    use sui_types::base_types::{ObjectID, SuiAddress};
    use tracing::{debug, info};

    use crate::{
        get_config,
        sui_call::{
            sui_move_object::{
                admin_obj::SuiAdminObject, bot_obj::SuiBotObject, hero_obj::SuiHeroObject,
                pet_obj::SuiPetObject,
            },
            ADMIN_OBJECT_NAME, BOT_OBJECT_NAME, HERO_OBJECT_NAME, PET_OBJECT_NAME,
        },
    };

    use super::WePetGame;

    #[serial]
    #[tokio::test]
    async fn test_get_bot_obj_success() {
        dotenv().ok();

        let config = get_config();

        let keystore_path = Path::new("/home/ganzzi/.sui/sui_config/sui.keystore");
        let mut keystore =
            Keystore::from(FileBasedKeystore::new(&keystore_path.to_path_buf()).unwrap());

        let sui_client = SuiClientBuilder::default().build_devnet().await.unwrap();
        let package_id = ObjectID::from_str(&config.PACKAGE).unwrap();
        let player = SuiAddress::from_str(
            "0x64f804ad5f8bf531d507a2dd4e00c7de041c8c6ced7744bbe66d93fedf8dfb7f",
        )
        .unwrap();
        let admin = SuiAddress::from_str(
            "0xdb96399b7daeac4613a8494a30cf371206cff2ea4d19924d87ddf151d0d3a1c7",
        )
        .unwrap();

        let obj = WePetGame::new(sui_client.clone(), player, package_id.to_string().as_str());
        let obj_admin = WePetGame::new(sui_client, admin, package_id.to_string().as_str());

        let admin = obj_admin
            .get_sui_obj_first::<SuiAdminObject>(ADMIN_OBJECT_NAME)
            .await
            .map_err(|e| println!("{e:?}"));

        println!("admin data: \n{admin:?}\n");

        let pet = obj
            .get_sui_obj_first::<SuiPetObject>(PET_OBJECT_NAME)
            .await
            .map_err(|e| println!("{e:?}"));

        println!("pet data: \n{pet:?}\n");

        let bot = obj
            .get_sui_obj_first::<SuiBotObject>(BOT_OBJECT_NAME)
            .await
            .map_err(|e| println!("{e:?}"));
        println!("bot data: \n{bot:?}\n");

        let hero = obj
            .get_sui_obj_first::<SuiHeroObject>(HERO_OBJECT_NAME)
            .await
            .map_err(|e| println!("{e:?}"));
        println!("hero data: \n{hero:?}\n");
    }
}
// endregion:    --- Tests
