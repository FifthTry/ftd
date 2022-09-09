use itertools::Itertools;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Section {
    pub name: String,
    pub kind: Option<String>,
    pub caption: Option<ftd::p11::Header>,
    pub headers: ftd::p11::Headers,
    pub body: Option<Body>,
    pub sub_sections: Vec<Section>,
    pub is_commented: bool,
    pub line_number: usize,
    pub block_body: bool,
}

impl Section {
    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: None,
            caption: None,
            body: None,
            sub_sections: vec![],
            is_commented: false,
            line_number: 0,
            headers: ftd::p11::Headers(vec![]),
            block_body: false,
        }
    }

    pub fn add_sub_section(mut self, sub: Self) -> Self {
        self.sub_sections.push(sub);
        self
    }

    pub fn without_line_number(&self) -> Self {
        Self {
            name: self.name.to_string(),
            kind: self.kind.to_owned(),
            caption: self.caption.as_ref().map(|v| v.without_line_number()),
            headers: ftd::p11::Headers(
                self.headers
                    .0
                    .iter()
                    .map(|v| v.without_line_number())
                    .collect_vec(),
            ),
            body: self.body.as_ref().map(|v| v.without_line_number()),
            sub_sections: self
                .sub_sections
                .iter()
                .map(|v| v.without_line_number())
                .collect_vec(),
            is_commented: self.is_commented.to_owned(),
            line_number: 0,
            block_body: false,
        }
    }

    pub fn and_caption(mut self, caption: &str) -> Self {
        self.caption = Some(ftd::p11::Header::from_caption(caption, self.line_number));
        self
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }

    pub fn add_header_str(mut self, key: &str, value: &str) -> Self {
        self.headers.push(ftd::p11::Header::kv(
            0,
            key,
            None,
            if value.trim().is_empty() {
                None
            } else {
                Some(value.to_string())
            },
        ));
        self
    }

    pub fn add_header_section(
        mut self,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd::p11::Section>,
    ) -> Self {
        self.headers
            .push(ftd::p11::Header::section(0, key, kind, section));
        self
    }

    pub fn and_body(mut self, body: &str) -> Self {
        self.body = Some(ftd::p11::Body::new(0, body));
        self
    }

    pub fn kind(mut self, kind: &str) -> Self {
        self.kind = Some(kind.to_string());
        self
    }

    /// returns a copy of Section after processing comments
    ///
    /// ## NOTE: This function is only called by [`ParsedDocument::ignore_comments()`]
    ///
    /// [`ParsedDocument::ignore_comments()`]: ftd::p2::interpreter::ParsedDocument::ignore_comments
    pub fn remove_comments(&self) -> Option<Section> {
        if self.is_commented {
            return None;
        }
        Some(Section {
            name: self.name.to_string(),
            kind: self.kind.to_owned(),
            caption: self.caption.as_ref().and_then(|v| v.remove_comments()),
            headers: self.headers.clone().remove_comments(),
            body: self.body.as_ref().and_then(|v| v.remove_comments()),
            sub_sections: self
                .sub_sections
                .iter()
                .filter_map(|s| s.remove_comments())
                .collect::<Vec<ftd::p11::Section>>(),
            is_commented: false,
            line_number: self.line_number,
            block_body: self.block_body,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub line_number: usize,
    pub value: String,
}

impl Body {
    pub(crate) fn new(line_number: usize, value: &str) -> Body {
        Body {
            line_number,
            value: value.trim().to_string(),
        }
    }
    pub fn without_line_number(&self) -> Self {
        Body {
            line_number: 0,
            value: self.value.to_string(),
        }
    }

    pub(crate) fn remove_comments(&self) -> Option<Self> {
        let mut value = Some(self.value.to_owned());
        ftd::p11::utils::remove_value_comment(&mut value);
        value.map(|value| Body {
            line_number: self.line_number,
            value,
        })
    }
}
