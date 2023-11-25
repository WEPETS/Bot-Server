// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

mod utils;

use std::str::FromStr;

use sui_json_rpc_types::{SuiObjectDataFilter, SuiObjectDataOptions, SuiObjectResponseQuery};
use sui_types::{base_types::ObjectID, Identifier};
// use sui_sdk::types::base_types::ObjectID;
use move_core_types::language_storage::StructTag;
use utils::setup_for_read;

// This example uses the Read API to get owned objects of an address,
// the dynamic fields of an object,
// past objects, information about the chain
// and the protocol configuration,
// the transaction data after executing a transaction,
// and finally, the number of transaction blocks known to the server.

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (sui, active_address) = setup_for_read().await?;

    // ************ READ API ************ //
    println!("// ************ READ API ************ //\n");

    let sui_data_filter = SuiObjectDataFilter::StructType(StructTag {
        address: *ObjectID::from_str(
            "0xbd3de1b61b1a52f8dae85081d5cc8078d36f1af4dcc8a265a8c902cf0d887ad2",
        )
        .unwrap(),
        module: Identifier::from_str("we_pet_game").unwrap(),
        name: Identifier::from_str("Pet").unwrap(),
        type_params: Vec::new(),
    });

    let query = SuiObjectResponseQuery::new(
        Some(sui_data_filter),
        Some(SuiObjectDataOptions::new().with_display().with_content()),
    );

    // Owned Objects
    let owned_objects = sui
        .read_api()
        .get_owned_objects(active_address, Some(query), None, Some(10))
        .await?
        .data;
    println!(" *** address ***");
    println!("{:?}", active_address.to_string());
    println!(" *** Owned Objects ***\n");
    println!("{:?}", owned_objects);

    Ok(())
}
