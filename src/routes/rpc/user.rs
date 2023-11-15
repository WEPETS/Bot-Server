use crate::{
    ctx::Ctx,
    models::{ModelManager, User, UserBmc, UserForCreate, UserForUpdate},
    routes::error::{Error, Result},
};
use axum::response::Response;

use super::params::{ParamsForCreate, ParamsForJustId, ParamsForUpdate};

pub async fn list_users(ctx: Ctx, mm: ModelManager) -> Result<Vec<User>> {
    Ok(UserBmc::list::<User>(&ctx, &mm).await?)
}

pub async fn get_user(ctx: Ctx, mm: ModelManager, params: ParamsForJustId) -> Result<User> {
    Ok(UserBmc::get::<User>(&ctx, &mm, params.id).await?)
}

pub async fn update_user(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForUpdate<UserForUpdate>,
) -> Result<()> {
    if ctx.user_id() != params.id {
        return Err(Error::RpcNoPermission);
    }
    UserBmc::update(&ctx, &mm, params.id, params.data).await?;
    Ok(())
}

pub async fn delete_user(ctx: Ctx, mm: ModelManager, params: ParamsForJustId) -> Result<()> {
    if ctx.user_id() != params.id {
        return Err(Error::RpcNoPermission);
    }
    UserBmc::delete(&ctx, &mm, params.id).await?;
    Ok(())
}
