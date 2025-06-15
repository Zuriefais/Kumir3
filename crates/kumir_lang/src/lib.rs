#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    Int(i32),
    Float(f32),
    Operator(String),
    Delimiter(char),
    String(String),
    Char(char),
    Bool(bool),
    Eof,
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Alg,
    Start,
    Stop,
    TypeDef(TypeDefinition),
}

#[derive(Debug, PartialEq)]
pub enum TypeDefinition {
    Int,
    Bool,
    Float,
    String,
    Char,
}

impl From<&str> for Keyword {
    fn from(value: &str) -> Self {
        match value {
            "алг" => Keyword::Alg,
            "нач" => Keyword::Start,
            "кон" => Keyword::Stop,
            "цел" => Keyword::TypeDef(TypeDefinition::Int),
            "вещ" => Keyword::TypeDef(TypeDefinition::Float),
            "лог" => Keyword::TypeDef(TypeDefinition::Bool),
            "сим" => Keyword::TypeDef(TypeDefinition::Char),
            "лит" => Keyword::TypeDef(TypeDefinition::String),
            _ => {
                panic!("Error")
            }
        }
    }
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
                    if ["алг", "нач", "кон", "цел", "вещ", "лог", "сим", "лит"]
                        .contains(&word.as_str())
                    {
                        Ok(Token::Keyword(Keyword::from(word.as_str())))
                    } else if ["да", "нет"].contains(&word.as_str()) {
                        match word.as_str() {
                            "да" => Ok(Token::Bool(true)),
                            "нет" => Ok(Token::Bool(false)),
                            _ => Err("Ошибка".to_string()),
                        }
                    } else {
                        Ok(Token::Identifier(word))
                    }
                } else if c.is_digit(10) {
                    Ok(match self.collect_number() {
                        Number::Float(f) => Token::Float(f),
                        Number::Int(i) => Token::Int(i),
                    })
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
                } else if '\'' == c {
                    self.advance();
                    let char_token = self.collect_char();
                    self.advance();
                    char_token
                } else {
                    Err(format!("Неизвестный символ: {}", c))
                }
            }
        }
    }

    pub fn collect_char(&mut self) -> Result<Token, String> {
        let char = self.current_char.unwrap();
        self.advance();
        if self.current_char.unwrap() == '\'' {
            Ok(Token::Char(char))
        } else {
            Err("символ не может быть длиннее 1".to_string())
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

    pub fn collect_number(&mut self) -> Number {
        let mut num = String::new();
        let mut is_float = false;
        while self
            .current_char
            .map_or(false, |c| c.is_digit(10) || c == '.')
        {
            if self.current_char.unwrap() == '.' {
                is_float = true
            }
            num.push(self.current_char.unwrap());
            self.advance();
        }
        if is_float {
            Number::Float(num.parse().unwrap_or(0.0))
        } else {
            Number::Int(num.parse().unwrap_or(0))
        }
    }
}

pub enum Number {
    Float(f32),
    Int(i32),
}
