#[derive(Debug)]
pub struct Scanner<'input> {
    input: &'input str,
    chars: std::iter::Peekable<std::str::CharIndices<'input>>,
    /// index is byte position in the input
    pub index: usize,
    fuel: fastn_p1::Fuel,
    pub output: fastn_p1::ParseOutput,
}

pub struct Index<'input> {
    index: usize,
    chars: std::iter::Peekable<std::str::CharIndices<'input>>,
}

impl<'input> Scanner<'input> {
    pub fn new(input: &str, fuel: fastn_p1::Fuel) -> Scanner {
        assert!(input.len() < 10_000_000); // can't parse > 10MB file
        Scanner {
            input,
            chars: input.char_indices().peekable(),
            fuel,
            index: 0,
            output: fastn_p1::ParseOutput::default(),
        }
    }

    fn span(&self, start: usize) -> fastn_p1::Span {
        fastn_p1::Span {
            start,
            end: self.index,
        }
    }

    pub fn take_while<F: Fn(char) -> bool>(&mut self, f: F) -> Option<fastn_p1::Span> {
        let start = self.index;
        while let Some(c) = self.peek() {
            if !f(c) {
                break;
            }
            self.pop();
        }

        if self.index == start {
            return None;
        }

        Some(self.span(start))
    }

    pub fn index(&self) -> Index<'input> {
        Index {
            index: self.index,
            chars: self.chars.clone(),
        }
    }

    pub fn reset(&mut self, index: Index<'input>) {
        self.index = index.index;
        self.chars = index.chars;
    }

    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|v| v.1)
    }

    pub fn pop(&mut self) -> Option<char> {
        let (idx, c) = self.chars.next()?;
        // self.index = idx;
        // Update the index by the byte length of the character
        self.index = idx + c.len_utf8();
        Some(c)
    }

    pub fn take(&mut self, t: char) -> bool {
        if self.peek() == Some(t) {
            self.pop();
            true
        } else {
            false
        }
    }

    pub fn skip_spaces(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' {
                self.pop();
                continue;
            }
            break;
        }
    }

    pub fn take_till_char_or_end_of_line(&mut self, t: char) -> Option<fastn_p1::Span> {
        self.take_while(|c| c != t && c != '\n')
    }

    #[cfg(test)]
    pub fn remaining(&self) -> &str {
        let char_remaining = self.chars.clone().map(|c| c.1).collect::<String>();
        let str_remaining = &self.input[self.index..];

        assert_eq!(
            char_remaining, str_remaining,
            "Character-based and byte-based remaining text do not match"
        );

        str_remaining
    }

    pub fn one_of(&mut self, choices: &[&'static str]) -> Option<&'static str> {
        #[allow(clippy::manual_find)]
        // clippy wants us to use this:
        //
        // ```rs
        // choices
        //     .iter()
        //     .find(|&choice| self.token(choice).is_some())
        //     .copied();
        // ```
        //
        // but this is clearer:
        for choice in choices {
            if self.token(choice).is_some() {
                return Some(choice);
            }
        }
        None
    }

    // returns the span from current position to the end of token
    pub fn token(&mut self, t: &'static str) -> Option<fastn_p1::Span> {
        let start = self.index();
        for char in t.chars() {
            assert!(char.is_ascii()); // we are assuming this is ascii string
            if self.peek() != Some(char) {
                self.reset(start);
                return None;
            }
            self.pop();
        }

        Some(self.span(start.index))
    }
}
