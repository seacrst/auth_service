
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie, CookieJar};

use crate::{
    app::state::AppState, 
    services::{
        api::AuthApiError, 
        auth::validate_token, 
        constants::JWT_COOKIE_NAME
    }
};

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthApiError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthApiError::MissingToken))
    };

    // Validate token
    let token = cookie.value().to_owned();
    let _ = match validate_token(&token, state.banned_token_store.clone()).await {
        Ok(claims) => claims,
        Err(_) => return (jar, Err(AuthApiError::InvalidToken)),
    };

    // Add token to banned list
    if state
        .banned_token_store
        .write()
        .await
        .add_token(token.to_owned())
        .await
        .is_err()
    {
        return (jar, Err(AuthApiError::UnexpectedError));
    }

    // Remove jwt cookie
    let jar = jar.remove(cookie::Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}

