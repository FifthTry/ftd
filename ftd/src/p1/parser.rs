#[derive(Debug, Clone)]
enum ParsingStateReading {
    Section,
    Header {
        key: String,
        kind: Option<String>,
        condition: Option<String>,
    },
    Caption,
    Body,
    Subsection,
}

#[derive(Debug)]
pub struct State {
    line_number: i32,
    sections: Vec<ftd::p1::Section>,
    content: String,
    doc_id: String,
    state: Vec<(ftd::p1::Section, Vec<ParsingStateReading>)>,
}

impl State {
    fn next(&mut self) -> ftd::p1::Result<()> {
        use itertools::Itertools;

        self.reading_section()?;

        while let Some((_, mut state)) = self.get_latest_state() {
            let mut change_state = None;
            self.end(&mut change_state)?;

            if self.content.trim().is_empty() {
                let sections = self.state.iter().map(|(v, _)| v.clone()).collect_vec();
                self.state = vec![];
                self.sections.extend(sections);

                continue;
            }

            if let Some(change_state) = change_state {
                state = change_state;
            }

            match state {
                ParsingStateReading::Section => {
                    self.reading_block_headers()?;
                }
                ParsingStateReading::Header {
                    key,
                    kind,
                    condition,
                } => {
                    self.reading_header_value(key.as_str(), kind, condition)?;
                }
                ParsingStateReading::Caption => {
                    self.reading_caption_value()?;
                }
                ParsingStateReading::Body => {
                    self.reading_body_value()?;
                }
                ParsingStateReading::Subsection => {
                    self.reading_section()?;
                }
            }
        }

        Ok(())
    }

    fn end(&mut self, change_state: &mut Option<ParsingStateReading>) -> ftd::p1::Result<()> {
        let (scan_line_number, content) = self.clean_content();
        let (start_line, rest_lines) = new_line_split(content.as_str());
        if !start_line.starts_with("-- ") {
            return Ok(());
        }
        let start_line = &start_line[2..];
        let (name, caption) = colon_separated_values(
            ftd::p1::utils::i32_to_usize(self.line_number + 1),
            start_line,
            self.doc_id.as_str(),
        )?;
        if is_end(name.as_str()) {
            let caption = caption.ok_or_else(|| ftd::p1::Error::ParseError {
                message: "section name not provided for `end`".to_string(),
                doc_id: self.doc_id.to_string(),
                line_number: ftd::p1::utils::i32_to_usize(self.line_number),
            })?;
            let mut sections = vec![];
            loop {
                let line_number = self.line_number;
                let (section, state) = if let Some(state) = self.remove_latest_state() {
                    state
                } else {
                    let section = self.remove_latest_section()?.ok_or_else(|| {
                        ftd::p1::Error::ParseError {
                            message: format!("No section found to end: {}", caption),
                            doc_id: self.doc_id.to_string(),
                            line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                        }
                    })?;
                    sections.push(section);
                    continue;
                };
                match state {
                    ParsingStateReading::Section if caption.eq(section.name.as_str()) => {
                        sections.reverse();
                        section.sub_sections.extend(sections);
                        *change_state = None;
                        break;
                    }
                    ParsingStateReading::Header {
                        key,
                        kind,
                        condition,
                    } if caption.eq(format!("{}.{}", section.name, key).as_str()) => {
                        sections.reverse();
                        section.headers.push(ftd::p1::Header::section(
                            ftd::p1::utils::i32_to_usize(line_number),
                            key.as_str(),
                            kind,
                            sections,
                            condition,
                        ));
                        *change_state = Some(ParsingStateReading::Section);
                        break;
                    }
                    _ => {}
                }
            }
            self.line_number += (scan_line_number as i32) + 1;
            self.content = rest_lines;
            return self.end(change_state);
        }

        Ok(())
    }

    fn clean_content(&mut self) -> (usize, String) {
        let mut valid_line_number = None;
        let new_line_content = self.content.split('\n');
        let mut scan_line_number = 0;
        for (line_number, line) in new_line_content.enumerate() {
            if valid_line(line) && !line.trim().is_empty() {
                valid_line_number = Some(line_number);
                break;
            }
            scan_line_number += 1;
        }
        (
            scan_line_number,
            content_index(self.content.as_str(), valid_line_number),
        )
    }

    fn reading_section(&mut self) -> ftd::p1::Result<()> {
        let (scan_line_number, content) = self.clean_content();
        let (start_line, rest_lines) = new_line_split(content.as_str());
        let start_line = start_line.trim();

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            return if start_line.is_empty() {
                Ok(())
            } else {
                Err(ftd::p1::Error::SectionNotFound {
                    // TODO: context should be a few lines before and after the input
                    doc_id: self.doc_id.to_string(),
                    line_number: ftd::p1::utils::i32_to_usize(
                        self.line_number + (scan_line_number as i32) + 1,
                    ),
                })
            };
        }

        let start_line = clean_line(start_line);

        let is_commented = start_line.starts_with("/-- ");
        let line = if is_commented {
            &start_line[3..]
        } else {
            &start_line[2..]
        };

        let (name_with_kind, caption) =
        //  section-kind section-name: caption
            colon_separated_values(ftd::p1::utils::i32_to_usize(self.line_number), line, self
                .doc_id.as_str())?;
        let (section_name, kind) = get_name_and_kind(name_with_kind.as_str());
        let last_section = self.get_latest_state().map(|v| v.0);
        match last_section {
            Some(section) if section_name.starts_with(format!("{}.", section.name).as_str()) => {
                return Err(ftd::p1::Error::SectionNotFound {
                    doc_id: self.doc_id.to_string(),
                    line_number: ftd::p1::utils::i32_to_usize(
                        self.line_number + (scan_line_number as i32) + 1,
                    ),
                });
            }
            _ => {}
        }

        self.line_number += (scan_line_number as i32) + 1;
        let section = ftd::p1::Section {
            name: section_name,
            kind,
            caption: caption.map(|v| {
                ftd::p1::Header::from_caption(
                    v.as_str(),
                    ftd::p1::utils::i32_to_usize(self.line_number),
                )
            }),
            headers: Default::default(),
            body: None,
            sub_sections: Default::default(),
            is_commented,
            line_number: ftd::p1::utils::i32_to_usize(self.line_number),
            block_body: false,
        };

        self.state
            .push((section, vec![ParsingStateReading::Section]));
        self.content = rest_lines;
        self.reading_inline_headers()?;
        Ok(())
    }

    fn reading_block_headers(&mut self) -> ftd::p1::Result<()> {
        self.end(&mut None)?;
        let (scan_line_number, content) = self.clean_content();
        let (section, parsing_states) =
            self.state
                .last_mut()
                .ok_or_else(|| ftd::p1::Error::SectionNotFound {
                    doc_id: self.doc_id.to_string(),
                    line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                })?;

        let header_not_found_next_state = if !section.block_body {
            ParsingStateReading::Body
        } else {
            ParsingStateReading::Subsection
        };

        let (start_line, rest_lines) = new_line_split(content.as_str());

        let start_line = start_line.trim();

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            parsing_states.push(header_not_found_next_state);
            return Ok(());
        }

        let is_commented = start_line.starts_with("/-- ");
        let line = if is_commented {
            &start_line[3..]
        } else {
            &start_line[2..]
        };

        let (name_with_kind, value) = colon_separated_values(
            ftd::p1::utils::i32_to_usize(self.line_number),
            line,
            self.doc_id.as_str(),
        )?;
        let (key, kind) = get_name_and_kind(name_with_kind.as_str());

        let key = if let Some(key) = key.strip_prefix(format!("{}.", section.name).as_str()) {
            key
        } else {
            parsing_states.push(header_not_found_next_state);
            return Ok(());
        };

        self.line_number += (scan_line_number as i32) + 1;
        self.content = rest_lines;
        section.block_body = true;

        let condition = get_block_header_condition(
            &mut self.content,
            &mut self.line_number,
            self.doc_id.as_str(),
        )?;

        if is_caption(key) && kind.is_none() && section.caption.is_some() {
            return Err(ftd::p1::Error::MoreThanOneCaption {
                doc_id: self.doc_id.to_string(),
                line_number: section.line_number,
            });
        }
        if let Some(value) = value {
            section.headers.push(ftd::p1::Header::kv(
                ftd::p1::utils::i32_to_usize(self.line_number),
                key,
                kind,
                Some(value),
                condition,
            ))
        } else {
            parsing_states.push(if is_caption(key) {
                ParsingStateReading::Caption
            } else if is_body(key) {
                ParsingStateReading::Body
            } else {
                ParsingStateReading::Header {
                    key: key.to_string(),
                    kind,
                    condition,
                }
            });
        }
        Ok(())
    }

    fn reading_header_value(
        &mut self,
        header_key: &str,
        header_kind: Option<String>,
        header_condition: Option<String>,
    ) -> ftd::p1::Result<()> {
        if let Err(ftd::p1::Error::SectionNotFound { .. }) = self.reading_section() {
            let mut value = vec![];
            let mut new_line_number = None;
            let mut first_line = true;
            let split_content = self.content.as_str().split('\n');
            for (line_number, line) in split_content.enumerate() {
                if line.starts_with("-- ") || line.starts_with("/-- ") {
                    new_line_number = Some(line_number);
                    break;
                }
                self.line_number += 1;
                if !valid_line(line) {
                    continue;
                }
                if first_line {
                    if !line.trim().is_empty() {
                        return Err(ftd::p1::Error::ParseError {
                            message: format!("start section header '{}' after a newline!!", line),
                            doc_id: self.doc_id.to_string(),
                            line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                        });
                    }
                    first_line = false;
                }
                value.push(clean_line(line));
            }
            self.content = content_index(self.content.as_str(), new_line_number);
            let doc_id = self.doc_id.to_string();
            let line_number = self.line_number;
            let section = self
                .remove_latest_state()
                .ok_or(ftd::p1::Error::SectionNotFound {
                    doc_id,
                    line_number: ftd::p1::utils::i32_to_usize(line_number),
                })?
                .0;
            let value = value.join("\n").trim().to_string();
            section.headers.push(ftd::p1::Header::kv(
                ftd::p1::utils::i32_to_usize(line_number),
                header_key,
                header_kind,
                if value.is_empty() { None } else { Some(value) },
                header_condition,
            ));
        }
        Ok(())
    }

    fn reading_caption_value(&mut self) -> ftd::p1::Result<()> {
        let mut value = vec![];
        let mut new_line_number = None;
        let mut first_line = true;
        let split_content = self.content.as_str().split('\n');
        for (line_number, line) in split_content.enumerate() {
            if line.starts_with("-- ") || line.starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            self.line_number += 1;
            if !valid_line(line) {
                continue;
            }
            if first_line {
                if !line.trim().is_empty() {
                    return Err(ftd::p1::Error::ParseError {
                        message: format!("start section caption '{}' after a newline!!", line),
                        doc_id: self.doc_id.to_string(),
                        line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                    });
                }
                first_line = false;
            }
            value.push(clean_line(line));
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;
        let section = self
            .remove_latest_state()
            .ok_or(ftd::p1::Error::SectionNotFound {
                doc_id,
                line_number: ftd::p1::utils::i32_to_usize(line_number),
            })?
            .0;

        let value = value.join("\n").trim().to_string();
        section.caption = Some(ftd::p1::Header::from_caption(
            value.as_str(),
            ftd::p1::utils::i32_to_usize(line_number),
        ));
        Ok(())
    }

    fn reading_body_value(&mut self) -> ftd::p1::Result<()> {
        let mut value = vec![];
        let mut new_line_number = None;
        let mut first_line = true;
        let split_content = self.content.as_str().split('\n');
        for (line_number, line) in split_content.enumerate() {
            if line.starts_with("-- ") || line.starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            self.line_number += 1;
            if !valid_line(line) {
                continue;
            }
            if first_line {
                if !line.trim().is_empty() {
                    return Err(ftd::p1::Error::ParseError {
                        message: format!("start section body '{}' after a newline!!", line),
                        doc_id: self.doc_id.to_string(),
                        line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                    });
                }
                first_line = false;
            }

            value.push(clean_line(line));
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;
        let section = self
            .remove_latest_state()
            .ok_or(ftd::p1::Error::SectionNotFound {
                doc_id,
                line_number: ftd::p1::utils::i32_to_usize(line_number),
            })?
            .0;
        let value = value.join("\n").trim().to_string();
        if !value.is_empty() {
            section.body = Some(ftd::p1::Body::new(
                ftd::p1::utils::i32_to_usize(line_number),
                value.as_str(),
            ));
        }
        let (section, parsing_state) = self.state.last_mut().unwrap();
        if !section.block_body {
            parsing_state.push(ParsingStateReading::Subsection);
        }
        Ok(())
    }

    // There should not be no new line in the headers
    fn reading_inline_headers(&mut self) -> ftd::p1::Result<()> {
        let mut headers = vec![];
        let mut new_line_number = None;
        for (line_number, line) in self.content.split('\n').enumerate() {
            if line.trim().is_empty() || line.starts_with("-- ") || line.starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            if !valid_line(line) {
                self.line_number += 1;
                continue;
            }
            let line = clean_line(line);
            if let Ok((name_with_kind, caption)) = colon_separated_values(
                ftd::p1::utils::i32_to_usize(self.line_number),
                line.as_str(),
                self.doc_id.as_str(),
            ) {
                let (header_key, kind, condition) =
                    get_name_kind_and_condition(name_with_kind.as_str());
                self.line_number += 1;
                headers.push(ftd::p1::Header::kv(
                    ftd::p1::utils::i32_to_usize(self.line_number),
                    header_key.as_str(),
                    kind,
                    caption,
                    condition,
                ));
            } else {
                new_line_number = Some(line_number);
                break;
            }
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;

        let section = self
            .mut_latest_state()
            .ok_or(ftd::p1::Error::SectionNotFound {
                doc_id,
                line_number: ftd::p1::utils::i32_to_usize(line_number),
            })?
            .0;
        section.headers.0.extend(headers);
        Ok(())
    }

    fn mut_latest_state(&mut self) -> Option<(&mut ftd::p1::Section, &mut ParsingStateReading)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.last_mut() {
                return Some((section, state));
            }
        }
        None
    }

    fn get_latest_state(&self) -> Option<(ftd::p1::Section, ParsingStateReading)> {
        if let Some((section, state)) = self.state.last() {
            if let Some(state) = state.last() {
                return Some((section.to_owned(), state.to_owned()));
            }
        }
        None
    }

    fn remove_latest_section(&mut self) -> ftd::p1::Result<Option<ftd::p1::Section>> {
        if let Some((section, state)) = self.state.last() {
            if !state.is_empty() {
                return Err(ftd::p1::Error::ParseError {
                    message: format!("`{}` section state is not yet empty", section.name),
                    doc_id: self.doc_id.to_string(),
                    line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                });
            }
        }
        Ok(self.state.pop().map(|v| v.0))
    }

    fn remove_latest_state(&mut self) -> Option<(&mut ftd::p1::Section, ParsingStateReading)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.pop() {
                return Some((section, state));
            }
        }
        None
    }
}

pub fn parse(content: &str, doc_id: &str) -> ftd::p1::Result<Vec<ftd::p1::Section>> {
    parse_with_line_number(content, doc_id, 0)
}

pub fn parse_with_line_number(
    content: &str,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Vec<ftd::p1::Section>> {
    let mut state = State {
        content: content.to_string(),
        doc_id: doc_id.to_string(),
        line_number: if line_number > 0 {
            -(line_number as i32)
        } else {
            0
        },
        sections: Default::default(),
        state: Default::default(),
    };
    state.next()?;
    Ok(state.sections)
}

fn colon_separated_values(
    line_number: usize,
    line: &str,
    doc_id: &str,
) -> ftd::p1::Result<(String, Option<String>)> {
    if !line.contains(':') {
        return Err(ftd::p1::Error::ParseError {
            message: format!(": is missing in: {}", line),
            // TODO: context should be a few lines before and after the input
            doc_id: doc_id.to_string(),
            line_number,
        });
    }

    let mut parts = line.splitn(2, ':');
    let name = parts.next().unwrap().trim().to_string();

    let caption = match parts.next() {
        Some(c) if c.trim().is_empty() => None,
        Some(c) => Some(c.trim().to_string()),
        None => None,
    };

    Ok((name, caption))
}

fn get_name_and_kind(name_with_kind: &str) -> (String, Option<String>) {
    if let Some((kind, name)) = name_with_kind.rsplit_once(' ') {
        return (name.to_string(), Some(kind.to_string()));
    }

    (name_with_kind.to_string(), None)
}

fn get_name_kind_and_condition(name_with_kind: &str) -> (String, Option<String>, Option<String>) {
    let (name_with_kind, condition) = if let Some((name_with_kind, condition)) =
        name_with_kind.split_once(ftd::p1::utils::INLINE_IF)
    {
        (name_with_kind.to_string(), Some(condition.to_string()))
    } else {
        (name_with_kind.to_string(), None)
    };
    if let Some((kind, name)) = name_with_kind.rsplit_once(' ') {
        return (name.to_string(), Some(kind.to_string()), condition);
    }

    (name_with_kind, None, condition)
}

fn clean_line(line: &str) -> String {
    if line.starts_with("\\;;") || line.starts_with("\\-- ") {
        return line[1..].to_string();
    }
    line.to_string()
}

fn valid_line(line: &str) -> bool {
    !line.starts_with(";;")
}

fn is_caption(s: &str) -> bool {
    s.eq("caption")
}

fn is_body(s: &str) -> bool {
    s.eq("body")
}

fn is_end(s: &str) -> bool {
    s.eq("end")
}

fn new_line_split(s: &str) -> (String, String) {
    if let Some((start_line, rest_lines)) = s.trim().split_once('\n') {
        (start_line.to_string(), rest_lines.to_string())
    } else {
        (s.to_string(), "".to_string())
    }
}

fn content_index(content: &str, line_number: Option<usize>) -> String {
    use itertools::Itertools;

    let new_line_content = content.split('\n');
    let content = new_line_content.collect_vec();
    match line_number {
        Some(line_number) if content.len() > line_number => content[line_number..].join("\n"),
        _ => "".to_string(),
    }
}

pub(crate) fn get_block_header_condition(
    content: &mut String,
    line_number: &mut i32,
    doc_id: &str,
) -> ftd::p1::Result<Option<String>> {
    let mut condition = None;
    let mut new_line_number = None;
    for (line_number, line) in content.split('\n').enumerate() {
        if !valid_line(line) {
            continue;
        }
        let line = clean_line(line);
        if let Ok((name_with_kind, caption)) =
            colon_separated_values(line_number, line.as_str(), doc_id)
        {
            if name_with_kind.eq(ftd::p1::utils::IF) {
                condition = caption;
                new_line_number = Some(line_number + 1);
            }
        }
        break;
    }

    if let Some(new_line_number) = new_line_number {
        *content = content_index(content.as_str(), Some(new_line_number));
        *line_number += new_line_number as i32;
    }

    Ok(condition)
}
