pub use crate::p1::{Error, Header, Result, SubSection, SubSections};

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Section {
    pub name: String,
    pub caption: Option<String>,
    pub header: Header,
    pub body: Option<(usize, String)>,
    pub sub_sections: SubSections,
    pub is_commented: bool,
    pub line_number: usize,
}

impl Section {
    pub fn body_without_comment(&self) -> Option<(usize, String)> {
        let body = match &self.body {
            None => return None,
            Some(b) => b,
        };
        match body {
            _ if body.1.starts_with(r"\/") =>
            {
                #[allow(clippy::single_char_pattern)]
                Some((body.0, body.1.strip_prefix(r"\").expect("").to_string()))
            }
            _ if body.1.starts_with('/') => None,
            _ => Some((body.0, body.1.to_string())),
        }
    }

    pub fn remove_comments(&self) -> Section {
        let mut headers = vec![];
        for (i, k, v) in self.header.0.iter() {
            if !k.starts_with('/') {
                headers.push((i.to_owned(), k.to_string(), v.to_string()));
            }
        }

        let body = match &self.body {
            None => None,
            Some(body) => match body {
                _ if body.1.starts_with(r"\/") =>
                {
                    #[allow(clippy::single_char_pattern)]
                    Some((body.0, body.1.strip_prefix(r"\").expect("").to_string()))
                }
                _ if body.1.starts_with('/') => None,
                _ => self.body.clone(),
            },
        };

        Section {
            name: self.name.to_string(),
            caption: self.caption.to_owned(),
            header: Header(headers),
            body,
            sub_sections: SubSections(
                self.sub_sections
                    .0
                    .iter()
                    .filter(|s| !s.is_commented)
                    .map(|s| s.remove_comments())
                    .collect::<Vec<SubSection>>(),
            ),
            is_commented: false,
            line_number: self.line_number,
        }
    }

    pub fn caption(&self, line_number: usize, doc_id: &str) -> Result<String> {
        match self.caption {
            Some(ref v) => Ok(v.to_string()),
            None => Err(Error::ParseError {
                message: format!("caption is missing in {}", self.name.as_str(),),
                doc_id: doc_id.to_string(),
                line_number,
            }),
        }
    }

    pub fn body(&self, line_number: usize, doc_id: &str) -> Result<String> {
        match self.body_without_comment() {
            Some(ref v) => Ok(v.1.to_string()),
            None => Err(Error::ParseError {
                message: format!("body is missing in {}", self.name.as_str(),),
                doc_id: doc_id.to_string(),
                line_number,
            }),
        }
    }

    pub fn assert_missing(&self, line_number: usize, key: &str, doc_id: &str) -> Result<()> {
        if self
            .header
            .str_optional(doc_id.to_string(), line_number, key)?
            .is_some()
        {
            return Err(Error::ParseError {
                message: format!("'{}' is not expected in {}", key, self.name.as_str()),
                doc_id: doc_id.to_string(),
                line_number,
            });
        }

        Ok(())
    }

    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            caption: None,
            header: Header::default(),
            body: None,
            sub_sections: SubSections::default(),
            is_commented: false,
            line_number: 0,
        }
    }

    pub fn without_line_number(&self) -> Self {
        Self {
            name: self.name.to_string(),
            caption: self.caption.to_owned(),
            header: self.header.without_line_number(),
            body: self.body.to_owned().map(|v| (0, v.1)),
            sub_sections: self.sub_sections.without_line_number(),
            is_commented: self.is_commented.to_owned(),
            line_number: 0,
        }
    }

    pub fn and_caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn and_optional_caption(mut self, value: &Option<ftd_rt::Rendered>) -> Self {
        if let Some(v) = value {
            self = self.and_caption(v.original.as_str());
        }
        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> Self {
        self.header.0.push((0, key.to_string(), value.to_string()));
        self
    }

    pub fn add_optional_header_bool(mut self, key: &str, value: Option<bool>) -> Self {
        if let Some(v) = value {
            self = self.add_header(key, v.to_string().as_str());
        }
        self
    }
    pub fn add_optional_header_i32(mut self, key: &str, value: &Option<i32>) -> Self {
        if let Some(v) = value {
            self = self.add_header(key, v.to_string().as_str());
        }
        self
    }

    pub fn add_header_if_not_equal<T>(self, key: &str, value: T, reference: T) -> Self
    where
        T: ToString + std::cmp::PartialEq,
    {
        if value != reference {
            self.add_header(key, value.to_string().as_str())
        } else {
            self
        }
    }

    pub fn add_optional_header(mut self, key: &str, value: &Option<String>) -> Self {
        if let Some(v) = value {
            self = self.add_header(key, v.as_str());
        }
        self
    }

    pub fn and_body(mut self, body: &str) -> Self {
        self.body = Some((0, body.to_string()));
        self
    }

    pub fn and_optional_body(mut self, body: &Option<String>) -> Self {
        self.body = body.as_ref().map(|v| (0, v.to_string()));
        self
    }

    pub fn add_sub_section(mut self, sub: SubSection) -> Self {
        self.sub_sections.0.push(sub);
        self
    }

    pub fn sub_section_by_name(
        &self,
        name: &str,
        doc_id: String,
    ) -> crate::p1::Result<&crate::p1::SubSection> {
        let mut count = 0;
        for s in self.sub_sections.0.iter() {
            if s.is_commented {
                continue;
            }
            if s.name == name {
                count += 1;
            }
        }
        if count > 1 {
            return Err(crate::p1::Error::MoreThanOneSubSections {
                key: name.to_string(),
                doc_id,
                line_number: self.line_number,
            });
        }

        for s in self.sub_sections.0.iter() {
            if s.is_commented {
                continue;
            }
            if s.name == name {
                return Ok(s);
            }
        }

        Err(crate::p1::Error::NotFound {
            doc_id,
            line_number: self.line_number,
            key: name.to_string(),
        })
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }
}
