use crate::auth::email_password::key_expired;

pub(crate) async fn confirm_email(
    req: &fastn_core::http::Request,
    req_config: &mut fastn_core::RequestConfig,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let code = match req_config.request.query().get("code") {
        Some(code) => code,
        None => {
            tracing::info!("finishing response due to bad code");

            // [ERROR] logging (bad-request: QueryNotFound)
            req.log(
                "confirm-email",
                fastn_core::log::BadRequestOutcome::QueryNotFoundError {
                    query: "code".to_string(),
                }
                .into_kind(),
                file!(),
                line!(),
            );
            return Ok(fastn_core::http::api_error("Bad Request")?);
        }
    };

    let code = match code {
        serde_json::Value::String(c) => c,
        _ => {
            tracing::info!("failed to Deserialize ?code as string");

            // [ERROR] logging (bad-request: QueryDeserializeError)
            req.log(
                "confirm-email",
                fastn_core::log::BadRequestOutcome::QueryDeserializeError {
                    query: "code".to_string(),
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return Ok(fastn_core::http::api_error("Bad Request")?);
        }
    };

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            // [ERROR] logging (server-error: PoolError)
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            req.log(
                "confirm-email",
                fastn_core::log::ServerErrorOutcome::PoolError {
                    message: err_message.clone(),
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    let conf_data: Option<(i64, i64, chrono::DateTime<chrono::Utc>)> =
        match fastn_core::schema::fastn_email_confirmation::table
            .select((
                fastn_core::schema::fastn_email_confirmation::email_id,
                fastn_core::schema::fastn_email_confirmation::session_id,
                fastn_core::schema::fastn_email_confirmation::sent_at,
            ))
            .filter(fastn_core::schema::fastn_email_confirmation::key.eq(&code))
            .first(&mut conn)
            .await
            .optional()
        {
            Ok(v) => v,
            Err(e) => {
                // [ERROR] logging (server-error: DatabaseQueryError)
                let err_message = format!("{:?}", &e);
                req.log(
                    "confirm-email",
                    fastn_core::log::ServerErrorOutcome::DatabaseQueryError {
                        message: err_message,
                    }
                    .into_kind(),
                    file!(),
                    line!(),
                );
                return Err(e.into());
            }
        };

    let (email_id, session_id, sent_at) = match conf_data {
        Some(values) => values,
        None => {
            tracing::info!("invalid code value. No entry exists for the given code in db");
            tracing::info!("provided code: {}", &code);

            // [ERROR] logging (bad-request: InvalidCode)
            req.log(
                "confirm-email",
                fastn_core::log::BadRequestOutcome::InvalidCode {
                    code: code.to_string(),
                }
                .into_kind(),
                file!(),
                line!(),
            );

            return Ok(fastn_core::http::api_error("Bad Request")?);
        }
    };

    if key_expired(&req_config.config.ds, sent_at).await {
        // TODO: this redirect route should be configurable
        tracing::info!("provided code has expired.");

        // [SUCCESS] logging: redirect ResendConfirmationEmail (key expired)
        let log_success_message = "confirm-email: redirect to ResendConfirmationEmail (key expired: EMAIL_CONFIRMATION_EXPIRE_DAYS)".to_string();
        req.log(
            "confirm-email",
            fastn_core::log::OutcomeKind::Success(fastn_core::log::SuccessOutcome::Descriptive(
                log_success_message,
            )),
            file!(),
            line!(),
        );

        return Ok(fastn_core::http::temporary_redirect(format!(
            "{scheme}://{host}{resend_confirmation_email_route}",
            scheme = req_config.request.connection_info.scheme(),
            host = req_config.request.connection_info.host(),
            resend_confirmation_email_route = fastn_core::auth::Route::ResendConfirmationEmail
        )));
    }

    let email: fastn_core::utils::CiString =
        match diesel::update(fastn_core::schema::fastn_user_email::table)
            .set(fastn_core::schema::fastn_user_email::verified.eq(true))
            .filter(fastn_core::schema::fastn_user_email::id.eq(email_id))
            .returning(fastn_core::schema::fastn_user_email::email)
            .get_result(&mut conn)
            .await
        {
            Ok(email) => email,
            Err(e) => {
                // [ERROR] logging (server-error: DatabaseQueryError)
                let err_message = format!("{:?}", &e);
                req.log(
                    "confirm-email",
                    fastn_core::log::ServerErrorOutcome::DatabaseQueryError {
                        message: err_message,
                    }
                    .into_kind(),
                    file!(),
                    line!(),
                );
                return Err(e.into());
            }
        };

    let user_id: i64 = match diesel::update(fastn_core::schema::fastn_user::table)
        .set(fastn_core::schema::fastn_user::verified_email.eq(true))
        .filter(fastn_core::schema::fastn_user::email.eq(&email))
        .returning(fastn_core::schema::fastn_user::id)
        .get_result(&mut conn)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            // [ERROR] logging (server-error: DatabaseQueryError)
            let err_message = format!("{:?}", &e);
            req.log(
                "confirm-email",
                fastn_core::log::ServerErrorOutcome::DatabaseQueryError {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    // Onboarding step is opt-in
    let onboarding_enabled = req_config
        .config
        .ds
        .env("FASTN_AUTH_ADD_ONBOARDING_STEP")
        .await
        .is_ok();

    let next_path = if onboarding_enabled {
        format!(
            "{onboarding_route}?next={next}",
            onboarding_route = fastn_core::auth::Route::Onboarding
        )
    } else {
        next.to_string()
    };

    let now = chrono::Utc::now();

    // session always exists for new unverified user since it is created during `create-account`
    let affected = match diesel::update(fastn_core::schema::fastn_auth_session::table)
        .set((
            fastn_core::schema::fastn_auth_session::user_id.eq(&user_id),
            fastn_core::schema::fastn_auth_session::updated_at.eq(&now),
        ))
        .filter(fastn_core::schema::fastn_auth_session::id.eq(session_id))
        .execute(&mut conn)
        .await
    {
        Ok(affected) => affected,
        Err(e) => {
            // [ERROR] logging (server-error: DatabaseQueryError)
            let err_message = format!("{:?}", &e);
            req.log(
                "confirm-email",
                fastn_core::log::ServerErrorOutcome::DatabaseQueryError {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    tracing::info!("updated session. affected: {}", affected);

    // redirect to onboarding route with a GET request
    // if some user is already logged in, this will override their session with this one
    let mut resp = match fastn_core::auth::set_session_cookie_and_redirect_to_next(
        &req_config.request,
        "confirm-email",
        &req_config.config.ds,
        session_id,
        next_path,
    )
    .await
    {
        Ok(response) => response,
        Err(e) => {
            // [ERROR] logging (server-error: CookieError)
            let err_message = format!("session cookie: {:?}", &e);
            req.log(
                "confirm-email",
                fastn_core::log::ServerErrorOutcome::CookieError {
                    message: err_message,
                }
                .into_kind(),
                file!(),
                line!(),
            );
            return Err(e);
        }
    };

    if onboarding_enabled {
        resp.add_cookie(
            &actix_web::cookie::Cookie::build(
                fastn_core::auth::FIRST_TIME_SESSION_COOKIE_NAME,
                "1",
            )
            .domain(fastn_core::auth::utils::domain(
                req_config.request.connection_info.host(),
            ))
            .path("/")
            .finish(),
        )
        .map_err(|e| {
            // [ERROR] logging (Set Cookie Error)
            let err_message = format!("failed to set cookie: {:?}", &e);
            req.log(
                "confirm-email",
                fastn_core::log::ServerErrorOutcome::CookieError {
                    message: err_message.clone(),
                }
                .into_kind(),
                file!(),
                line!(),
            );

            fastn_core::Error::generic(err_message)
        })?;
    }

    Ok(resp)
}
