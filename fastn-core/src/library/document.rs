/*
document filename
foo/abc.ftd

document id
/foo/abc/
/foo/abc/-/x/y/ --> full id
/x/y/ - suffix
*/

/// converts the document_name/document-full-id to document_id
/// and returns it as String
///
///
/// ## Examples
/// ```rust
/// # use fastn_core::library::convert_to_document_id;
///assert_eq!(convert_to_document_id("/bar/index.ftd/"), "/bar/");
///assert_eq!(convert_to_document_id("index.ftd"), "/");
///assert_eq!(convert_to_document_id("/foo/-/x/"), "/foo/");
///assert_eq!(convert_to_document_id("/fastn.dev/doc.txt"), "/fastn.dev/doc/");
///assert_eq!(convert_to_document_id("foo.png/"), "/foo/");
///assert_eq!(convert_to_document_id("README.md"), "/README/");
/// ```
pub fn convert_to_document_id(doc_name: &str) -> String {
    let doc_name = ftd::regex::EXT.replace_all(doc_name, "");

    // Discard document suffix if there
    // Also discard trailing index
    let document_id = doc_name
        .split_once("/-/")
        .map(|x| x.0)
        .unwrap_or_else(|| doc_name.as_ref())
        .trim_end_matches("index")
        .trim_matches('/');

    // In case if doc_id = index.ftd
    if document_id.is_empty() {
        return "/".to_string();
    }

    // Attach /{doc_id}/ before returning
    format!("/{}/", document_id)
}

pub fn document_full_id(
    config: &fastn_core::Config,
    doc: &ftd::p2::TDoc,
) -> ftd::ftd2021::p1::Result<String> {
    let full_document_id = config.doc_id().unwrap_or_else(|| {
        doc.name
            .to_string()
            .replace(config.package.name.as_str(), "")
    });

    if full_document_id.trim_matches('/').is_empty() {
        return Ok("/".to_string());
    }

    Ok(format!("/{}/", full_document_id.trim_matches('/')))
}

#[allow(dead_code)]
pub mod processor {
    pub fn document_id(
        _section: &ftd::ftd2021::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fastn_core::Config,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        let doc_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        let document_id = doc_id
            .split_once("/-/")
            .map(|x| x.0)
            .unwrap_or_else(|| &doc_id)
            .trim_matches('/');

        Ok(ftd::Value::String {
            text: format!("/{}/", document_id),
            source: ftd::TextSource::Default,
        })
    }
    pub fn document_full_id(
        _section: &ftd::ftd2021::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fastn_core::Config,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        Ok(ftd::Value::String {
            text: super::document_full_id(config, doc)?,
            source: ftd::TextSource::Default,
        })
    }

    pub async fn document_name<'a>(
        section: &ftd::ftd2021::p1::Section,
        doc: &ftd::p2::TDoc<'a>,
        config: &fastn_core::Config,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        let doc_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        let file_path = config.get_file_path(&doc_id).await.map_err(|e| {
            ftd::ftd2021::p1::Error::ParseError {
                message: e.to_string(),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            }
        })?;

        Ok(ftd::Value::String {
            text: file_path.trim().to_string(),
            source: ftd::TextSource::Default,
        })
    }

    pub fn document_suffix(
        _section: &ftd::ftd2021::p1::Section,
        doc: &ftd::p2::TDoc,
        config: &fastn_core::Config,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        let doc_id = config.doc_id().unwrap_or_else(|| {
            doc.name
                .to_string()
                .replace(config.package.name.as_str(), "")
        });

        let value = doc_id
            .split_once("/-/")
            .map(|(_, y)| y.trim().to_string())
            .map(|suffix| ftd::Value::String {
                text: suffix,
                source: ftd::TextSource::Default,
            });

        Ok(ftd::Value::Optional {
            data: Box::new(value),
            kind: ftd::p2::Kind::String {
                caption: false,
                body: false,
                default: None,
                is_reference: false,
            },
        })
    }
}
