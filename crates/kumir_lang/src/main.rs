use kumir_lang::lexer::{Lexer, Token};

fn main() {
    env_logger::init();
    let code = include_str!("test.kum");
    let mut lexer = Lexer::new(code);
    let mut tokens = vec![];
    loop {
        match lexer.next_token() {
            Ok(Token::Eof) => break,
            Ok(token) => {
                println!("{:?}", token);
                tokens.push(token);
            }
            Err(e) => println!("Ошибка: {}", e),
        }
    }
}
