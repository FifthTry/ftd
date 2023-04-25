pub fn fastn_ftd() -> &'static str {
    include_str!("../fastn_2021.ftd")
}

pub fn parse_old_fastn(
    source: &str,
) -> Result<ftd::ftd2021::p2::Document, fastn_issues::initialization::OldFastnParseError> {
    let mut s = ftd::ftd2021::interpret("FASTN", source, &None)?;
    let document;
    loop {
        match s {
            ftd::ftd2021::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::ftd2021::Interpreter::StuckOnProcessor { section, .. } => {
                return Err(
                    fastn_issues::initialization::OldFastnParseError::ProcessorUsed {
                        processor: section
                            .header
                            .str("FASTN.ftd", section.line_number, ftd::PROCESSOR_MARKER)
                            .expect("we cant get stuck on processor without processor marker")
                            .to_string(),
                    },
                )
            }
            ftd::ftd2021::Interpreter::StuckOnImport { module, state: st } => {
                let source = if module == "fastn" {
                    fastn_ftd()
                } else {
                    return Err(
                        fastn_issues::initialization::OldFastnParseError::InvalidImport { module },
                    );
                };
                s = st.continue_after_import(module.as_str(), source)?;
            }
            ftd::ftd2021::Interpreter::StuckOnForeignVariable { .. } => {
                unreachable!("we never register any foreign variable so we cant come here")
            }
            ftd::ftd2021::Interpreter::CheckID { .. } => {
                unimplemented!()
            }
        }
    }
    Ok(document)
}

pub fn get_name(
    doc: ftd::ftd2021::p2::Document,
) -> Result<String, fastn_issues::initialization::GetNameError> {
    let op: Option<PackageTemp> = doc.get(fastn_package::FASTN_PACKAGE_VARIABLE)?;
    match op {
        Some(p) => Ok(p.name),
        None => Err(fastn_issues::initialization::GetNameError::PackageIsNone),
    }
}

/// Backend Header is a struct that is used to read and store the backend-header from the FASTN.ftd file
#[derive(serde::Deserialize, Debug, Clone)]
pub struct BackendHeader {
    #[serde(rename = "header-key")]
    pub header_key: String,
    #[serde(rename = "header-value")]
    pub header_value: String,
}

/// PackageTemp is a struct that is used for mapping the `fastn.package` data in FASTN.ftd file. It is
/// not used elsewhere in program, it is immediately converted to `fastn_core::Package` struct during
/// deserialization process
#[derive(serde::Deserialize, Debug, Clone)]
pub struct PackageTemp {
    pub name: String,
    pub versioned: bool,
    #[serde(rename = "translation-of")]
    pub translation_of: Option<String>,
    #[serde(rename = "translation")]
    pub translations: Vec<String>,
    #[serde(rename = "language")]
    pub language: Option<String>,
    pub about: Option<String>,
    pub zip: Option<String>,
    #[serde(rename = "download-base-url")]
    pub download_base_url: Option<String>,
    #[serde(rename = "canonical-url")]
    pub canonical_url: Option<String>,
    #[serde(rename = "inherit-auto-imports-from-original")]
    pub import_auto_imports_from_original: bool,
    #[serde(rename = "favicon")]
    pub favicon: Option<String>,
    #[serde(rename = "endpoint")]
    pub endpoint: Option<String>,
    #[serde(rename = "backend")]
    pub backend: bool,
    #[serde(rename = "backend-headers")]
    pub backend_headers: Option<Vec<BackendHeader>>,
    #[serde(rename = "icon")]
    pub icon: Option<ftd::ImageSrc>,
}
