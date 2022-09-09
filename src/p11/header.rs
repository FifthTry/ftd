#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Header {
    KV(ftd::p11::header::KV),
    Section(ftd::p11::header::Section),
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct KV {
    line_number: usize,
    key: String,
    kind: Option<String>,
    value: Option<String>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Section {
    line_number: usize,
    key: String,
    kind: Option<String>,
    section: Vec<ftd::p11::Section>,
}

impl Header {
    pub(crate) fn from_string(
        key: &str,
        kind: Option<String>,
        value: &str,
        line_number: usize,
    ) -> Header {
        Header::KV(KV {
            line_number,
            key: key.to_string(),
            kind,
            value: Some(value.to_string()),
        })
    }

    pub(crate) fn from_caption(value: &str, line_number: usize) -> Header {
        Header::from_string("$caption$", None, value, line_number)
    }

    pub(crate) fn kv(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        value: Option<String>,
    ) -> Header {
        Header::KV(KV {
            line_number,
            key: key.to_string(),
            kind,
            value,
        })
    }

    pub(crate) fn section(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd::p11::Section>,
    ) -> Header {
        Header::Section(Section {
            line_number,
            key: key.to_string(),
            kind,
            section,
        })
    }

    pub fn without_line_number(&self) -> Self {
        use itertools::Itertools;

        match self {
            Header::KV(kv) => {
                let mut kv = (*kv).clone();
                kv.line_number = 0;
                Header::KV(kv)
            }
            Header::Section(s) => {
                let mut s = (*s).clone();
                s.line_number = 0;
                s.section = s
                    .section
                    .iter()
                    .map(|v| v.without_line_number())
                    .collect_vec();
                Header::Section(s)
            }
        }
    }

    pub(crate) fn get_key(&self) -> String {
        match self {
            Header::KV(ftd::p11::header::KV { key, .. })
            | Header::Section(ftd::p11::header::Section { key, .. }) => key.to_string(),
        }
    }

    pub(crate) fn set_key(&mut self, value: &str) {
        match self {
            Header::KV(ftd::p11::header::KV { key, .. })
            | Header::Section(ftd::p11::header::Section { key, .. }) => {
                *key = value.to_string();
            }
        }
    }

    pub(crate) fn get_value(&self, doc_id: &str) -> ftd::p11::Result<Option<String>> {
        match self {
            Header::KV(ftd::p11::header::KV { value, .. }) => Ok(value.to_owned()),
            Header::Section(_) => Err(ftd::p11::Error::ParseError {
                message: format!(
                    "Expected Header of type: KV, found: Section {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number: self.get_line_number(),
            }),
        }
    }

    pub(crate) fn get_sections(&self, doc_id: &str) -> ftd::p11::Result<&Vec<ftd::p11::Section>> {
        match self {
            Header::KV(_) => Err(ftd::p11::Error::ParseError {
                message: format!(
                    "Expected Header of type: Sections, found: KV {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number: self.get_line_number(),
            }),
            Header::Section(ftd::p11::header::Section { section, .. }) => Ok(section),
        }
    }

    pub(crate) fn get_line_number(&self) -> usize {
        match self {
            Header::KV(ftd::p11::header::KV { line_number, .. })
            | Header::Section(ftd::p11::header::Section { line_number, .. }) => *line_number,
        }
    }

    pub(crate) fn get_kind(&self) -> Option<String> {
        match self {
            Header::KV(ftd::p11::header::KV { kind, .. })
            | Header::Section(ftd::p11::header::Section { kind, .. }) => kind.to_owned(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Header::KV(ftd::p11::header::KV { value, .. }) => value.is_none(),
            Header::Section(ftd::p11::header::Section { section, .. }) => section.is_empty(),
        }
    }

    pub fn remove_comments(&self) -> Option<Header> {
        let mut header = self.clone();
        let key = header.get_key().trim().to_string();
        if key.starts_with('/') {
            return None;
        }

        if key.starts_with(r"\/") {
            header.set_key(key.trim_start_matches('\\'));
        }

        match &mut header {
            Header::KV(ftd::p11::header::KV { value, .. }) => {
                ftd::p11::utils::remove_value_comment(value)
            }
            Header::Section(ftd::p11::header::Section { section, .. }) => {
                *section = section
                    .iter_mut()
                    .filter_map(|s| s.remove_comments())
                    .collect();
            }
        }
        Some(header)
    }
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Headers(pub Vec<Header>);

impl Headers {
    pub fn find(&self, key: &str) -> Vec<&ftd::p11::Header> {
        use itertools::Itertools;

        self.0.iter().filter(|v| v.get_key().eq(key)).collect_vec()
    }

    pub fn find_once(
        &self,
        key: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::p11::Result<&ftd::p11::Header> {
        let headers = self.find(key);
        let header = headers.first().ok_or(ftd::p11::Error::HeaderNotFound {
            key: key.to_string(),
            doc_id: doc_id.to_string(),
            line_number,
        })?;
        if headers.len() > 1 {
            return Err(ftd::p11::Error::MoreThanOneHeader {
                key: key.to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            });
        }
        Ok(header)
    }

    pub fn push(&mut self, item: ftd::p11::Header) {
        self.0.push(item)
    }

    /// returns a copy of Header after processing comments "/" and escape "\\/" (if any)
    ///
    /// only used by [`Section::remove_comments()`] and [`SubSection::remove_comments()`]
    ///
    /// [`SubSection::remove_comments()`]: ftd::p1::sub_section::SubSection::remove_comments
    /// [`Section::remove_comments()`]: ftd::p1::section::Section::remove_comments
    pub fn remove_comments(self) -> Headers {
        use itertools::Itertools;

        Headers(
            self.0
                .into_iter()
                .filter_map(|h| h.remove_comments())
                .collect_vec(),
        )
    }
}
