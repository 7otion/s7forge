//! Valve's KeyValues text format, as written by the Steam client in `.vdf`
//! and `.acf` files: nested `"key" "value"` and `"key" { … }` pairs.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Table(Table),
}

pub type Table = Vec<(String, Value)>;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub offset: usize,
    pub message: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}

enum Token {
    Text(String),
    Open,
    Close,
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

/// Parses a whole document into its top-level pairs.
pub fn parse(text: &str) -> Result<Table, ParseError> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        at: 0,
    };
    let mut pairs = Vec::new();
    while let Some(token) = parser.token()? {
        let Token::Text(key) = token else {
            return Err(parser.error("expected a key"));
        };
        pairs.push((key, parser.value()?));
    }
    Ok(pairs)
}

/// The nested table stored under `key`, if any. Keys compare case-insensitively,
/// as Steam itself treats them.
pub fn table<'a>(pairs: &'a [(String, Value)], key: &str) -> Option<&'a Table> {
    match lookup(pairs, key)? {
        Value::Table(table) => Some(table),
        Value::Text(_) => None,
    }
}

/// The text stored under `key`, if any.
pub fn text<'a>(pairs: &'a [(String, Value)], key: &str) -> Option<&'a str> {
    match lookup(pairs, key)? {
        Value::Text(text) => Some(text),
        Value::Table(_) => None,
    }
}

/// The text stored under `key`, parsed as an unsigned number.
pub fn number(pairs: &[(String, Value)], key: &str) -> Option<u64> {
    text(pairs, key)?.parse().ok()
}

fn lookup<'a>(pairs: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    pairs
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

impl Parser<'_> {
    fn value(&mut self) -> Result<Value, ParseError> {
        match self.token()? {
            Some(Token::Text(text)) => Ok(Value::Text(text)),
            Some(Token::Open) => {
                let mut pairs = Vec::new();
                loop {
                    match self.token()? {
                        Some(Token::Close) => return Ok(Value::Table(pairs)),
                        Some(Token::Text(key)) => pairs.push((key, self.value()?)),
                        Some(Token::Open) => return Err(self.error("expected a key")),
                        None => return Err(self.error("unclosed table")),
                    }
                }
            }
            Some(Token::Close) => Err(self.error("unexpected '}'")),
            None => Err(self.error("expected a value")),
        }
    }

    fn token(&mut self) -> Result<Option<Token>, ParseError> {
        loop {
            match self.bytes.get(self.at) {
                None => return Ok(None),
                Some(b' ' | b'\t' | b'\r' | b'\n') => self.at += 1,
                Some(b'/') if self.bytes.get(self.at + 1) == Some(&b'/') => {
                    while !matches!(self.bytes.get(self.at), None | Some(b'\n')) {
                        self.at += 1;
                    }
                }
                Some(b'{') => {
                    self.at += 1;
                    return Ok(Some(Token::Open));
                }
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Some(Token::Close));
                }
                Some(b'"') => return self.quoted().map(|text| Some(Token::Text(text))),
                Some(_) => return Ok(Some(Token::Text(self.bare()))),
            }
        }
    }

    fn quoted(&mut self) -> Result<String, ParseError> {
        self.at += 1;
        let mut bytes = Vec::new();
        loop {
            match self.bytes.get(self.at) {
                None => return Err(self.error("unterminated string")),
                Some(b'"') => {
                    self.at += 1;
                    return Ok(String::from_utf8_lossy(&bytes).into_owned());
                }
                Some(b'\\') => {
                    let Some(&escaped) = self.bytes.get(self.at + 1) else {
                        return Err(self.error("unterminated escape"));
                    };
                    bytes.push(match escaped {
                        b'n' => b'\n',
                        b't' => b'\t',
                        b'r' => b'\r',
                        other => other,
                    });
                    self.at += 2;
                }
                Some(&byte) => {
                    bytes.push(byte);
                    self.at += 1;
                }
            }
        }
    }

    fn bare(&mut self) -> String {
        let start = self.at;
        while !matches!(
            self.bytes.get(self.at),
            None | Some(b' ' | b'\t' | b'\r' | b'\n' | b'{' | b'}' | b'"')
        ) {
            self.at += 1;
        }
        String::from_utf8_lossy(&self.bytes[start..self.at]).into_owned()
    }

    fn error(&self, message: &'static str) -> ParseError {
        ParseError {
            offset: self.at,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
"AppWorkshop"
{
	"appid"		"1142710"
	// a comment
	"WorkshopItemsInstalled"
	{
		"2789844937"
		{
			"size"		"125919"
			"timeupdated"		"1701529643"
		}
	}
	"path"		"C:\\steam\\with \"quotes\""
	bare	value
}
"#;

    #[test]
    fn parses_nested_tables_and_escapes() {
        let root = parse(SAMPLE).unwrap();
        let app = table(&root, "appworkshop").unwrap();
        assert_eq!(number(app, "appid"), Some(1142710));
        let installed = table(app, "WorkshopItemsInstalled").unwrap();
        let item = table(installed, "2789844937").unwrap();
        assert_eq!(number(item, "size"), Some(125919));
        assert_eq!(text(app, "path"), Some(r#"C:\steam\with "quotes""#));
        assert_eq!(text(app, "bare"), Some("value"));
    }

    #[test]
    fn missing_keys_are_none() {
        let root = parse(SAMPLE).unwrap();
        let app = table(&root, "AppWorkshop").unwrap();
        assert_eq!(text(app, "nope"), None);
        assert_eq!(table(app, "appid"), None);
        assert_eq!(number(app, "path"), None);
    }

    #[test]
    fn truncated_file_is_an_error() {
        let truncated = &SAMPLE[..SAMPLE.len() / 2];
        let error = parse(truncated).unwrap_err();
        assert!(matches!(
            error.message,
            "unclosed table" | "unterminated string" | "expected a value"
        ));
    }

    #[test]
    fn stray_close_is_an_error() {
        assert_eq!(parse("}").unwrap_err().message, "expected a key");
        assert_eq!(parse("\"k\" }").unwrap_err().message, "unexpected '}'");
    }
}
