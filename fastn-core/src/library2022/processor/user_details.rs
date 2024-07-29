/// returns details of the logged in user
pub async fn process(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    match req_config
        .config
        .ds
        .ud(
            req_config.config.get_db_url().await.as_str(),
            &req_config
                .request
                .cookie(fastn_core::http::SESSION_COOKIE_NAME),
        )
        .await
    {
        Ok(ud) => doc.from_json(&ud, &kind, &value),
        Err(e) => ftd::interpreter::utils::e2(
            format!("failed to get user data: {e:?}"),
            doc.name,
            value.line_number(),
        ),
    }
}
