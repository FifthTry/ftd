/// route handler: /-/auth/login/
pub async fn login(
    req: &fastn_core::http::Request,
    ds: &fastn_ds::DocumentStore,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    if fastn_core::auth::utils::is_authenticated(req) {
        return Ok(fastn_core::http::redirect(next));
    }

    let provider = req.q("provider", "github".to_string())?;

    match provider.as_str() {
        "github" => fastn_core::auth::github::login(ds, req, next).await,
        // client should handle redirects to next for email_password login
        "emailpassword" => fastn_core::auth::email_password::login(req, ds, db_pool, next).await,
        _ => Ok(fastn_core::not_found!("unknown provider: {}", provider)),
    }
}

// route: /-/auth/logout/
pub async fn logout(
    req: &fastn_core::http::Request,
    ds: &fastn_ds::DocumentStore,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    if let Some(session_data) = req.cookie(fastn_core::auth::SESSION_COOKIE_NAME) {
        let session_data = fastn_core::auth::utils::decrypt(ds, &session_data)
            .await
            .unwrap_or_default();

        #[derive(serde::Deserialize)]
        struct SessionData {
            session_id: i32,
        }

        if let Ok(data) = serde_json::from_str::<SessionData>(session_data.as_str()) {
            let session_id = data.session_id;

            let mut conn = db_pool
                .get()
                .await
                .map_err(|e| fastn_core::Error::DatabaseError {
                    message: format!("Failed to get connection to db. {:?}", e),
                })?;

            let affected = diesel::delete(fastn_core::schema::fastn_session::table)
                .filter(fastn_core::schema::fastn_session::id.eq(&session_id))
                .execute(&mut conn)
                .await?;

            tracing::info!("session destroyed for {session_id}. Rows affected {affected}.");
        }
    }

    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::SESSION_COOKIE_NAME, "")
                .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, next))
        .finish())
}

// handle: if request.url starts with /-/auth/
#[tracing::instrument(skip_all)]
pub async fn handle_auth(
    req: fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    config: &fastn_core::Config,
) -> fastn_core::Result<fastn_core::http::Response> {
    let next = req.q("next", "/".to_string())?;

    let pool = fastn_core::db::pool(&req_config.config.ds)
        .await
        .as_ref()
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    match req.path() {
        "/-/auth/login/" => login(&req, &req_config.config.ds, pool, next).await,
        // TODO: This has be set while creating the GitHub OAuth Application
        "/-/auth/github/" => {
            fastn_core::auth::github::callback(&req, &req_config.config.ds, pool, next).await
        }
        "/-/auth/logout/" => logout(&req, &req_config.config.ds, pool, next).await,

        "/-/auth/create-user/" => {
            fastn_core::auth::email_password::create_user(&req, req_config, config, pool, next)
                .await
        }
        "/-/auth/confirm-email/" => {
            fastn_core::auth::email_password::confirm_email(&req, &req_config.config.ds, pool, next)
                .await
        }
        "/-/auth/resend-confirmation-email/" => {
            fastn_core::auth::email_password::resend_email(&req, &req_config.config.ds, pool, next)
                .await
        }
        "/-/auth/onboarding/" => {
            fastn_core::auth::email_password::onboarding(&req, req_config, config, next).await
        }
        // "/-/auth/send-email-login-code/" => todo!(),
        // "/-/auth/add-email/" => todo!(),
        // "/-/auth/update-name/" => todo!(),
        // "/-/auth/update-password/" => todo!(),
        // "/-/auth/update-username/" => todo!(),
        // "/-/auth/update-email/" => todo!(),
        // "/-/auth/disable-account/" => todo!(),
        // "/-/auth/close-sessions/?session=<session-id|all>" => todo!(),
        _ => Ok(fastn_core::not_found!("route not found: {}", req.path())),
    }
}
