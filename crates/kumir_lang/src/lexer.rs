use log::info;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    Int(i32),
    Float(f32),
    Operator(Operator),
    Delimiter(Delimiter),
    String(String),
    Char(char),
    Bool(bool),
    Eof,
}

impl Token {
    pub fn as_operator(&self) -> Option<Operator> {
        match self {
            Token::Operator(op) => Some(*op),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    /// Addition (+)
    Plus,
    /// Subtraction (-)
    Minus,
    /// Multiplication (*)
    Multiply,
    /// Division (/)
    Divide,
    /// Implication (>=)
    GreaterOrEqual,
    /// Greater than (>)
    Greater,
    /// Less than (<)
    Less,
    /// Less than or equal to (<=)
    LessOrEqual,
    /// Equal to (=)
    Equal,
    /// Not equal to (<>)
    NotEqual,
    /// Equal to (==)
    EqualBool,
    /// Colon (:)
    Colon,
    /// Assignment (:=)
    Assignment,
}

impl Operator {
    pub fn precedence(&self) -> i32 {
        match self {
            Operator::Multiply | Operator::Divide => 3,
            Operator::Plus | Operator::Minus => 2,
            Operator::Greater
            | Operator::Less
            | Operator::GreaterOrEqual
            | Operator::LessOrEqual => 1,
            Operator::EqualBool => 0,
            _ => -1,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Delimiter {
    /// .
    Period,
    /// ,
    Comma,
    /// ;
    Semicolon,
    /// (
    ParenthesisOpen,
    /// )
    ParenthesisClose,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operation {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    Alg,
    Start,
    Stop,
    TypeDef(TypeDefinition),
    Condition(Condition),
    Loop(Loop),
    Range(Range),
    IO(IO),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Condition {
    ///если
    If,
    ///всё
    EndCondition,
    ///то
    Then,
    ///иначе
    Else,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Loop {
    ///нц
    Start,
    ///кц
    End,
    ///кц_при
    EndIf,
    ///пока
    While,
    ///выход
    Break,
    ///раз
    Times,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Range {
    /// для
    For,
    /// от
    From,
    /// до
    To,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IO {
    ///ввод
    Input,
    ///нс
    ChangeLine,
    ///вывод
    Output,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
            "если" => Keyword::Condition(Condition::If),
            "все" => Keyword::Condition(Condition::EndCondition),
            "то" => Keyword::Condition(Condition::Then),
            "иначе" => Keyword::Condition(Condition::Else),
            "нц" => Keyword::Loop(Loop::Start),
            "кц_при" => Keyword::Loop(Loop::EndIf),
            "кц" => Keyword::Loop(Loop::End),
            "пока" => Keyword::Loop(Loop::While),
            "раз" => Keyword::Loop(Loop::Times),
            "для" => Keyword::Range(Range::For),
            "от" => Keyword::Range(Range::From),
            "до" => Keyword::Range(Range::To),
            "ввод" => Keyword::IO(IO::Input),
            "нс" => Keyword::IO(IO::ChangeLine),
            "вывод" => Keyword::IO(IO::Output),
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
                    if [
                        "алг",
                        "нач",
                        "кон",
                        "цел",
                        "вещ",
                        "лог",
                        "сим",
                        "лит",
                        "если",
                        "все",
                        "то",
                        "иначе",
                        "нц",
                        "кц_при",
                        "кц",
                        "пока",
                        "для",
                        "от",
                        "до",
                        "ввод",
                        "вывод",
                        "нс",
                        "раз",
                    ]
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
                } else if "+-*/=><:".contains(c) {
                    let operator = self.collect_operator(&c);
                    self.advance();
                    operator
                } else if ".,;()".contains(c) {
                    let delim: Delimiter = match c {
                        '.' => Delimiter::Period,
                        ',' => Delimiter::Comma,
                        ';' => Delimiter::Semicolon,
                        '(' => Delimiter::ParenthesisOpen,
                        ')' => Delimiter::ParenthesisClose,
                        _ => panic!("error"),
                    };

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

    pub fn collect_operator(&mut self, c: &char) -> Result<Token, String> {
        let next = self.input.get(self.position);
        info!(
            "Collecting operator: {:?}, {:?}, {:?}",
            self.input.get(self.position - 1),
            c,
            next
        );

        match (c, next) {
            ('<', Some('>')) => {
                self.advance();
                Ok(Token::Operator(Operator::NotEqual))
            }
            ('>', Some('=')) => {
                self.advance();
                Ok(Token::Operator(Operator::GreaterOrEqual))
            }
            ('<', Some('=')) => {
                self.advance();
                Ok(Token::Operator(Operator::LessOrEqual))
            }
            ('=', Some('=')) => {
                self.advance();
                Ok(Token::Operator(Operator::EqualBool))
            }
            (':', Some('=')) => {
                self.advance();
                Ok(Token::Operator(Operator::Assignment))
            }
            ('=', _) => Ok(Token::Operator(Operator::Equal)),
            (':', _) => Ok(Token::Operator(Operator::Colon)),
            ('>', _) => Ok(Token::Operator(Operator::Greater)),
            ('<', _) => Ok(Token::Operator(Operator::Less)),
            ('+', _) => Ok(Token::Operator(Operator::Plus)),
            ('-', _) => Ok(Token::Operator(Operator::Minus)),
            ('*', _) => Ok(Token::Operator(Operator::Multiply)),
            ('/', _) => Ok(Token::Operator(Operator::Divide)),
            (_, _) => Err(format!(
                "Couldn't collect operator from: {:?}, {:?}",
                c, next
            )),
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
        while self.current_char.map_or(false, |c| c != '"') {
            str.push(self.current_char.unwrap());
            self.advance();
        }
        self.advance();
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
