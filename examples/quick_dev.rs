#![allow(unused)] // For beginning only.

use anyhow::{Ok, Result};
use serde_json::{from_value, json, to_value, Value};
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    // public web folder Roote
    hc.do_get("/index.html").await?.print().await?;

    // 404 NotFound Roote
    hc.do_get("/abcxyz").await?.print().await?;

    // sign up route
    let req_sign_up = hc.do_post(
        "/auth/signup",
        json!({
            "username": "ganzzi22",
            "password": "ganzzi",
        }),
    );
    req_sign_up.await?.print().await?;

    // log in route
    let req_log_in = hc.do_post(
        "/auth/login",
        json!({
            "username": "ganzzi22",
            "password": "ganzzi",
        }),
    );
    req_log_in.await?.print().await?;

    // for rpc route: Please use api testing software like postman to add header { "Authentication": "Token" }

    Ok(())
}
