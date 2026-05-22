//! Mirrors upstream `mindustry.logic.LParser`.

use std::{collections::BTreeMap, fmt, marker::PhantomData};

pub const LOGIC_PARSER_MAX_TOKENS: usize = 16;
pub const LOGIC_PARSER_MAX_JUMPS: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicParseError {
    pub message: String,
}

impl LogicParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LogicParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid code. {}", self.message)
    }
}

impl std::error::Error for LogicParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicStatementKind {
    Label {
        name: String,
        line: usize,
    },
    Instruction {
        tokens: Vec<String>,
        line: usize,
        jump_label: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicParserOutput {
    pub statements: Vec<LogicStatementKind>,
    pub jump_locations: BTreeMap<String, usize>,
}

pub fn parse_logic_statements(text: &str) -> Result<LogicParserOutput, LogicParseError> {
    LogicParser::new(text).parse()
}

struct LogicParser<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    statements: Vec<LogicStatementKind>,
    jump_locations: BTreeMap<String, usize>,
    _marker: PhantomData<&'a str>,
}

impl<'a> LogicParser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            chars: text.chars().collect(),
            pos: 0,
            line: 0,
            statements: Vec::new(),
            jump_locations: BTreeMap::new(),
            _marker: PhantomData,
        }
    }

    fn parse(mut self) -> Result<LogicParserOutput, LogicParseError> {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                '\n' | ';' | ' ' => self.pos += 1,
                '\r' => self.pos = (self.pos + 2).min(self.chars.len()),
                _ => self.statement()?,
            }
        }

        Ok(LogicParserOutput {
            statements: self.statements,
            jump_locations: self.jump_locations,
        })
    }

    fn statement(&mut self) -> Result<(), LogicParseError> {
        let mut expect_next = false;
        let mut tokens = Vec::new();

        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if tokens.len() >= LOGIC_PARSER_MAX_TOKENS {
                return Err(LogicParseError::new(format!(
                    "Line too long; may only contain {} tokens",
                    LOGIC_PARSER_MAX_TOKENS
                )));
            }
            if c == '\n' || c == ';' {
                break;
            }
            if expect_next && c != ' ' && c != '#' && c != '\t' {
                return Err(LogicParseError::new("Expected space after string/token."));
            }

            expect_next = false;
            if c == '#' {
                self.comment();
                break;
            } else if c == '"' {
                tokens.push(self.string()?);
                expect_next = true;
            } else if c != ' ' && c != '\t' {
                tokens.push(self.token());
                expect_next = true;
            } else {
                self.pos += 1;
            }
        }

        if !tokens.is_empty() {
            check_logic_tokens(&mut tokens);
            if tokens.len() == 1 && tokens[0].ends_with(':') {
                if self.jump_locations.len() >= LOGIC_PARSER_MAX_JUMPS {
                    return Err(LogicParseError::new(format!(
                        "Too many jump locations. Max jumps: {}",
                        LOGIC_PARSER_MAX_JUMPS
                    )));
                }
                let label = tokens[0][..tokens[0].len() - 1].to_string();
                self.jump_locations.insert(label.clone(), self.line);
                self.statements.push(LogicStatementKind::Label {
                    name: label,
                    line: self.line,
                });
            } else {
                let mut jump_label = None;
                if tokens[0] == "jump" && tokens.len() > 1 && !can_parse_i32(&tokens[1]) {
                    jump_label = Some(tokens[1].clone());
                    tokens[1] = "-1".to_string();
                }

                for token in tokens.iter_mut().skip(1) {
                    if token == "@configure" {
                        *token = "@config".to_string();
                    }
                    if token == "configure" {
                        *token = "config".to_string();
                    }
                }

                self.statements.push(LogicStatementKind::Instruction {
                    tokens,
                    line: self.line,
                    jump_label,
                });
                self.line += 1;
            }
        }
        Ok(())
    }

    fn comment(&mut self) {
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            if c == '\n' {
                break;
            }
        }
    }

    fn string(&mut self) -> Result<String, LogicParseError> {
        let from = self.pos;
        let mut utflen = 0usize;

        self.pos += 1;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c == '\n' {
                return Err(LogicParseError::new(
                    "Missing closing quote \" before end of line.",
                ));
            } else if c == '"' {
                break;
            }
            utflen += java_modified_utf_char_len(c);
            self.pos += 1;
        }

        if self.pos >= self.chars.len() || self.chars[self.pos] != '"' {
            return Err(LogicParseError::new(
                "Missing closing quote \" before end of file.",
            ));
        }
        if utflen > 65535 {
            return Err(LogicParseError::new("String value too long."));
        }

        self.pos += 1;
        Ok(self.chars[from..self.pos].iter().collect())
    }

    fn token(&mut self) -> String {
        let from = self.pos;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c == '\n' || c == ' ' || c == '#' || c == '\t' || c == ';' {
                break;
            }
            self.pos += 1;
        }
        self.chars[from..self.pos].iter().collect()
    }
}

pub fn check_logic_tokens(tokens: &mut [String]) {
    if tokens.first().is_some_and(|token| token == "op") && tokens.len() > 1 {
        if tokens[1] == "atan2" {
            tokens[1] = "angle".to_string();
        } else if tokens[1] == "dst" {
            tokens[1] = "len".to_string();
        }
    }
}

fn can_parse_i32(value: &str) -> bool {
    value.parse::<i32>().is_ok()
}

fn java_modified_utf_char_len(c: char) -> usize {
    let code = c as u32;
    if code != 0 && code <= 0x7f {
        1
    } else if code <= 0x7ff {
        2
    } else if code <= 0xffff {
        3
    } else {
        // Java source parser works with UTF-16 chars. A supplementary Unicode scalar
        // is two surrogate chars, each encoded as three bytes by modified UTF-8.
        6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_tokenizes_comments_strings_labels_and_legacy_names_like_java() {
        let parsed = parse_logic_statements(
            "\n# comment\nstart:\nop atan2 result x y;op dst d x y\njump start equal a b\nset c @configure\nset d configure\nprint \"hello world\" # trailing",
        )
        .unwrap();

        assert_eq!(parsed.jump_locations.get("start"), Some(&0));
        assert_eq!(
            parsed.statements[0],
            LogicStatementKind::Label {
                name: "start".into(),
                line: 0
            }
        );
        assert_eq!(
            parsed.statements[1],
            LogicStatementKind::Instruction {
                tokens: vec!["op", "angle", "result", "x", "y"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                line: 0,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[2],
            LogicStatementKind::Instruction {
                tokens: vec!["op", "len", "d", "x", "y"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                line: 1,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[3],
            LogicStatementKind::Instruction {
                tokens: vec!["jump", "-1", "equal", "a", "b"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                line: 2,
                jump_label: Some("start".into())
            }
        );
        assert_eq!(
            parsed.statements[4],
            LogicStatementKind::Instruction {
                tokens: vec!["set", "c", "@config"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                line: 3,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[5],
            LogicStatementKind::Instruction {
                tokens: vec!["set", "d", "config"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                line: 4,
                jump_label: None
            }
        );
        assert_eq!(
            parsed.statements[6],
            LogicStatementKind::Instruction {
                tokens: vec!["print", "\"hello world\""]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                line: 5,
                jump_label: None
            }
        );
    }

    #[test]
    fn parser_reports_java_style_string_and_token_errors() {
        assert!(parse_logic_statements("set a \"unterminated")
            .unwrap_err()
            .message
            .contains("before end of file"));
        assert!(parse_logic_statements("set a \"unterminated\n")
            .unwrap_err()
            .message
            .contains("before end of line"));
        assert!(parse_logic_statements("set a \"ok\"next")
            .unwrap_err()
            .message
            .contains("Expected space"));

        let too_long = format!("print \"{}\"", "a".repeat(65_536));
        assert!(parse_logic_statements(&too_long)
            .unwrap_err()
            .message
            .contains("String value too long"));

        let many_tokens = "set a b c d e f g h i j k l m n o p";
        assert!(parse_logic_statements(many_tokens)
            .unwrap_err()
            .message
            .contains("Line too long"));
    }
}
