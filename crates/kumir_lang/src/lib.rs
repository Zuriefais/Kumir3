#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Number(i32),
    Operator(String),
    Delimiter(char),
    String(String),
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let chars = input.chars().collect();
        let mut lexer = Lexer {
            input: chars,
            position: 0,
            current_char: None,
        };
        lexer.advance();
        lexer
    }

    pub fn advance(&mut self) {
        self.current_char = if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        };
        self.position += 1;
    }

    pub fn skip_whitespace(&mut self) {
        while self.current_char.map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }
    }

    pub fn skip_string(&mut self) {
        while let Some(c) = self.current_char {
            if c == '\n' {
                // Предполагаем, что комментарий заканчивается на новой строке
                self.advance();
                break;
            }
            self.advance();
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(Token::Eof),
            Some(c) => {
                if '|' == c {
                    self.skip_string();
                    self.next_token()
                } else if c.is_alphabetic() || c == '_' {
                    let word = self.collect_word();
                    if ["алг", "нач", "кон", "цел", "вещ"].contains(&word.as_str()) {
                        Ok(Token::Keyword(word))
                    } else {
                        Ok(Token::Identifier(word))
                    }
                } else if c.is_digit(10) {
                    Ok(Token::Number(self.collect_number()))
                } else if "+-*/=><".contains(c) {
                    let op = c.to_string();
                    self.advance();
                    Ok(Token::Operator(op))
                } else if c == ':' {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Ok(Token::Operator(":=".to_string()))
                    } else {
                        Ok(Token::Operator(":".to_string()))
                    }
                } else if ".,;()".contains(c) {
                    let delim = c;
                    self.advance();
                    Ok(Token::Delimiter(delim))
                } else if '"' == c {
                    self.advance();
                    Ok(Token::String(self.collect_string()))
                } else {
                    Err(format!("Неизвестный символ: {}", c))
                }
            }
        }
    }

    pub fn collect_string(&mut self) -> String {
        let mut str = String::new();
        while self
            .current_char
            .map_or(false, |c| c.is_alphanumeric() && c != '"')
        {
            str.push(self.current_char.unwrap());
            self.advance();
        }
        str
    }

    pub fn collect_word(&mut self) -> String {
        let mut word = String::new();
        while self
            .current_char
            .map_or(false, |c| c.is_alphanumeric() || c == '_')
        {
            word.push(self.current_char.unwrap());
            self.advance();
        }
        word
    }

    pub fn collect_number(&mut self) -> i32 {
        let mut num = String::new();
        while self.current_char.map_or(false, |c| c.is_digit(10)) {
            num.push(self.current_char.unwrap());
            self.advance();
        }
        num.parse().unwrap_or(0)
    }
}
