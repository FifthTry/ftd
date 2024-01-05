extern crate self as fastn_core;

#[macro_use]
pub mod utils;
mod apis;
mod auto_import;
pub mod commands;
mod config;
#[cfg(feature = "auth")]
mod db;
mod doc;
mod file;
mod font;
mod package;
pub mod tutor;
pub(crate) mod watcher;
#[macro_use]
pub mod http;
#[cfg(feature = "auth")]
mod auth;
mod ds;
mod error;
mod i18n;
pub mod library;
mod proxy;
#[cfg(feature = "auth")]
mod schema;
pub mod sitemap;
mod snapshot;
mod tracker;
mod translation;
mod version;
// mod wasm;
pub mod catch_panic;
pub(crate) mod google_sheets;
mod library2022;
#[cfg(feature = "auth")]
mod mail;

pub(crate) use auto_import::AutoImport;
pub use commands::{
    build::build, check::post_build_check, create_package::create_package, query::query,
    serve::listen, test::test, update::update,
};
pub use config::{Config, FTDEdition, RequestConfig};
pub use error::Error;
pub use file::File;
pub(crate) use file::{get_file, paths_to_files, Document, Static};
pub(crate) use font::Font;
pub use library::{FastnLibrary, Library, Library2};
pub use library2022::Library2022;
pub(crate) use package::dependency::Dependency;
pub use package::user_group;
pub(crate) use package::Package;
pub(crate) use snapshot::Snapshot;
pub(crate) use tracker::Track;
pub(crate) use translation::{TranslatedDocument, TranslationData};
pub use {doc::resolve_foreign_variable2, doc::resolve_import};

pub const FASTN_UI_INTERFACE: &str = "fastn-stack.github.io/fastn-ui";
pub const PACKAGE_THEME_INTERFACE: &str = "ftd-lang.github.io/theme";
pub const NUMBER_OF_CRS_TO_RESERVE: usize = 5;

pub const IMAGE_EXT: &[&str] = &["jpg", "png", "svg"];

pub const VIDEO_EXT: &[&str] = &["mp4", "ogg", "webm"];

pub fn ftd_html() -> &'static str {
    include_str!("../ftd_2022.html")
}

fn processor_ftd() -> &'static str {
    include_str!("../ftd/processors.ftd")
}

fn design_ftd() -> &'static str {
    include_str!("../ftd/design.ftd")
}

fn fastn_2022_js() -> &'static str {
    if fastn_core::utils::is_test() {
        return "FASTN_JS";
    }
    include_str!("../fastn2022.js")
}

fn fastn_lib_ftd() -> &'static str {
    include_str!("../ftd/fastn-lib.ftd")
}

#[allow(dead_code)]
async fn original_package_status(config: &fastn_core::Config) -> fastn_core::Result<String> {
    let path = config
        .ds
        .root()
        .join("fastn")
        .join("translation")
        .join("original-status.ftd");
    Ok(if config.ds.exists(&path).await {
        config.ds.read_to_string(&path).await?
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        format!(
            "{}\n\n-- import: {}/original-status as pi\n\n-- pi.original-status-page:",
            body_prefix,
            config.package_info_package()
        )
    })
}

#[allow(dead_code)]
async fn translation_package_status(config: &fastn_core::Config) -> fastn_core::Result<String> {
    let path = config
        .ds
        .root()
        .join("fastn")
        .join("translation")
        .join("translation-status.ftd");
    Ok(if config.ds.exists(&path).await {
        config.ds.read_to_string(&path).await?
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        format!(
            "{}\n\n-- import: {}/translation-status as pi\n\n-- pi.translation-status-page:",
            body_prefix,
            config.package_info_package()
        )
    })
}

async fn get_messages(
    status: &fastn_core::TranslatedDocument,
    config: &fastn_core::Config,
) -> fastn_core::Result<String> {
    Ok(match status {
        TranslatedDocument::Missing { .. } => {
            let path = config.ds.root().join("fastn/translation/missing.ftd");
            if config.ds.exists(&path).await {
                config.ds.read_to_string(&path).await?
            } else {
                include_str!("../ftd/translation/missing.ftd").to_string()
            }
        }
        TranslatedDocument::NeverMarked { .. } => {
            let path = config.ds.root().join("fastn/translation/never-marked.ftd");
            if config.ds.exists(&path).await {
                config.ds.read_to_string(&path).await?
            } else {
                include_str!("../ftd/translation/never-marked.ftd").to_string()
            }
        }
        TranslatedDocument::Outdated { .. } => {
            let path = config.ds.root().join("fastn/translation/out-of-date.ftd");
            if config.ds.exists(&path).await {
                config.ds.read_to_string(&path).await?
            } else {
                include_str!("../ftd/translation/out-of-date.ftd").to_string()
            }
        }
        TranslatedDocument::UptoDate { .. } => {
            let path = config.ds.root().join("fastn/translation/upto-date.ftd");
            if config.ds.exists(&path).await {
                config.ds.read_to_string(&path).await?
            } else {
                include_str!("../ftd/translation/upto-date.ftd").to_string()
            }
        }
    })
}

pub fn get_env_ftd_file() -> String {
    std::env::vars()
        .filter(|(key, val)| {
            ["CARGO", "VERGEN", "FASTN"]
                .iter()
                .any(|prefix| !key.is_empty() && key.starts_with(prefix) && !val.is_empty())
        })
        .fold(String::new(), |accumulator, (key, value)| {
            format!("{accumulator}\n-- string {key}: {value}")
        })
}

pub fn debug_env_vars() -> String {
    std::env::vars()
        .filter(|(key, _)| {
            ["CARGO", "VERGEN", "FASTN"]
                .iter()
                .any(|prefix| key.starts_with(prefix))
        })
        .fold(String::new(), |consolidated_res, (key, value)| {
            format!("{}\n{}: {}", consolidated_res, key, value)
        })
}

// fn default_markdown() -> &'static str {
//     include_str!("../ftd/markdown.ftd")
// }

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn usage_error<T>(message: String) -> Result<T> {
    Err(Error::UsageError { message })
}

pub(crate) fn generic_error<T>(message: String) -> Result<T> {
    Error::generic_err(message)
}

pub(crate) fn assert_error<T>(message: String) -> Result<T> {
    Err(Error::AssertError { message })
}

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
