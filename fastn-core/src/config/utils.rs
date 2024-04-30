/// `find_root_for_file()` starts with the given path, which is the current directory where the
/// application started in, and goes up till it finds a folder that contains `FASTN.ftd` file.
/// TODO: make async
#[async_recursion::async_recursion]
pub(crate) async fn find_root_for_file(
    dir: &fastn_ds::Path,
    file_name: &str,
    ds: &fastn_ds::DocumentStore,
) -> Option<fastn_ds::Path> {
    if ds.exists(&dir.join(file_name)).await {
        Some(dir.clone())
    } else {
        if let Some(p) = dir.parent() {
            return find_root_for_file(&p, file_name, ds).await;
        };
        None
    }
}

pub fn insert_or_update(
    all_packages: &scc::HashMap<String, fastn_core::Package>,
    package_name: String,
    current_package: fastn_core::Package,
) {
    match all_packages.entry(package_name) {
        scc::hash_map::Entry::Occupied(mut ov) => {
            ov.insert(current_package);
        }
        scc::hash_map::Entry::Vacant(vv) => {
            vv.insert_entry(current_package);
        }
    }
}

pub async fn fastn_doc(
    ds: &fastn_ds::DocumentStore,
    path: &fastn_ds::Path,
) -> fastn_core::Result<ftd::ftd2021::p2::Document> {
    let doc = ds.read_to_string(path).await?;
    let lib = fastn_core::FastnLibrary::default();
    match fastn_core::doc::parse_ftd("fastn", doc.as_str(), &lib) {
        Ok(v) => Ok(v),
        Err(e) => Err(fastn_core::Error::PackageError {
            message: format!("failed to parse FASTN.ftd 3: {:?}", &e),
        }),
    }
}

// if path starts with /-/package-name or -/package-name,
// so it trims the package and return the remaining url
pub fn trim_package_name(path: &str, package_name: &str) -> Option<String> {
    let package_name1 = format!("-/{}", package_name.trim().trim_matches('/'));
    let path = path.trim().trim_start_matches('/');
    if path.starts_with(package_name1.as_str()) {
        return Some(path.trim_start_matches(package_name1.as_str()).to_string());
    }

    let package_name2 = format!("/-/{}", package_name.trim().trim_matches('/'));
    if path.starts_with(package_name2.as_str()) {
        return Some(path.trim_start_matches(package_name2.as_str()).to_string());
    }

    None
}

// url can be start with /-/package-name/ or  -/package-name/
// It will return url with end-point, if package or dependency contains endpoints in them
// url: /-/<package-name>/api/ => (package-name, endpoints/api/, app or package config)
// url: /-/<package-name>/api/ => (package-name, endpoints/api/, app or package config)
pub fn get_clean_url(
    config: &fastn_core::Config,
    url: &str,
) -> fastn_core::Result<(url::Url, std::collections::HashMap<String, String>)> {
    if url.starts_with("http") {
        return Ok((url::Url::parse(url)?, std::collections::HashMap::new()));
    }

    let url = if url.starts_with("/-/") || url.starts_with("-/") {
        url.to_string()
    } else {
        config
            .get_mountpoint_sanitized_path(&config.package, url)
            .map(|(u, _, _, _)| u)
            .unwrap_or_else(|| url.to_string()) // TODO: Error possibly, in that return 404 from proxy
    };

    // This is for current package
    if let Some(remaining_url) = trim_package_name(url.as_str(), config.package.name.as_str()) {
        if config.package.endpoints.is_empty() {
            return Err(fastn_core::Error::GenericError(format!(
                "package does not contain the endpoints: {:?}",
                config.package.name
            )));
        }

        let mut end_point = None;
        for e in config.package.endpoints.iter() {
            if remaining_url.starts_with(e.mountpoint.as_str()) {
                end_point = Some(e.endpoint.to_string());
                break;
            }
        }

        if end_point.is_none() {
            return Err(fastn_core::Error::GenericError(format!(
                "No mountpoint matched for url: {}",
                remaining_url.as_str()
            )));
        }

        return Ok((
            url::Url::parse(format!("{}{}", end_point.unwrap(), remaining_url).as_str())?,
            std::collections::HashMap::new(), // TODO:
        ));
    }

    // Handle logic for apps
    for app in config.package.apps.iter() {
        if let Some(ep) = &app.end_point {
            if let Some(remaining_url) = trim_package_name(url.as_str(), app.package.name.as_str())
            {
                let mut app_conf = app.config.clone();
                if let Some(user_id) = &app.user_id {
                    app_conf.insert("user-id".to_string(), user_id.clone());
                }
                return Ok((
                    url::Url::parse(format!("{}{}", ep, remaining_url).as_str())?,
                    app_conf,
                ));
            }
        }
    }

    if let Some(e) = config
        .package
        .endpoints
        .iter()
        .find(|&endpoint| url.starts_with(&endpoint.mountpoint))
    {
        let endpoint_url = e.endpoint.trim_end_matches('/');
        let relative_path = url.trim_start_matches(&e.mountpoint);
        let full_url = format!("{}/{}", endpoint_url, relative_path);
        return Ok((
            url::Url::parse(&full_url)?,
            std::collections::HashMap::new(),
        ));
    }

    let msg = format!("http-processor: end-point not found url: {}", url);
    tracing::error!(msg = msg);
    Err(fastn_core::Error::GenericError(msg))
}

pub(crate) fn is_http_url(url: &str) -> bool {
    url.starts_with("http")
}
