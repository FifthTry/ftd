pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    config: &fastn_core::Config,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (headers, line_number) = if let Ok(val) = value.get_record(doc.name) {
        (val.2.to_owned(), val.5.to_owned())
    } else {
        (ftd::ast::HeaderValues::new(vec![]), value.line_number())
    };

    let method = headers
        .get_optional_string_by_key("method", doc.name, line_number)?
        .unwrap_or_else(|| "GET".to_string())
        .to_lowercase();

    if method.as_str().ne("get") && method.as_str().ne("post") {
        return ftd::interpreter::utils::e2(
            format!("only GET and POST methods are allowed, found: {}", method),
            doc.name,
            line_number,
        );
    }

    let url = match headers.get_optional_string_by_key("url", doc.name, line_number)? {
        Some(v) => v,
        None => {
            return ftd::interpreter::utils::e2(
                format!(
                    "'url' key is required when using `{}: http`",
                    ftd::PROCESSOR_MARKER
                ),
                doc.name,
                line_number,
            )
        }
    };

    let (_, mut url, conf) = fastn_core::config::utils::get_clean_url(config, url.as_str())
        .map_err(|e| ftd::interpreter::Error::ParseError {
            message: format!("invalid url: {:?}", e),
            doc_id: doc.name.to_string(),
            line_number,
        })?;

    let mut body = vec![];
    for header in headers.0 {
        if header.key.as_str() == ftd::PROCESSOR_MARKER
            || header.key.as_str() == "url"
            || header.key.as_str() == "method"
        {
            continue;
        }

        let value = header.value.string(doc.name)?;

        // 1 id: $query.id
        // After resolve headers: id:1234(value of $query.id)
        if value.starts_with('$') {
            if let Some(value) = doc
                .get_value(header.line_number, value.as_str())?
                .to_string()
            {
                if method.as_str().eq("post") {
                    body.push(format!("\"{}\": {}", header.key, value));
                    continue;
                }
                url.query_pairs_mut()
                    .append_pair(header.key.as_str(), &value);
            }
        } else {
            if method.as_str().eq("post") {
                body.push(format!("\"{}\": {}", header.key, value));
                continue;
            }
            url.query_pairs_mut()
                .append_pair(header.key.as_str(), value.as_str());
        }
    }

    println!("calling `http` processor with url: {}", &url);

    let response = if method.as_str().eq("post") {
        fastn_core::http::http_post_with_cookie(
            url.as_str(),
            config.request.as_ref().and_then(|v| v.cookies_string()),
            &conf,
            dbg!(format!("{{{}}}", body.join(","))).as_str(),
        )
        .await
    } else {
        fastn_core::http::http_get_with_cookie(
            url.as_str(),
            config.request.as_ref().and_then(|v| v.cookies_string()),
            &conf,
        )
        .await
    };

    let response = match response {
        Ok(v) => v,
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("HTTP::get failed: {:?}", e),
                doc.name,
                line_number,
            )
        }
    };

    let response_string =
        String::from_utf8(response).map_err(|e| ftd::interpreter::Error::ParseError {
            message: format!("`http` processor API response error: {}", e),
            doc_id: doc.name.to_string(),
            line_number,
        })?;
    let response_json: serde_json::Value = serde_json::from_str(&response_string)
        .map_err(|e| ftd::interpreter::Error::Serde { source: e })?;

    doc.from_json(&response_json, &kind, &value)
}
