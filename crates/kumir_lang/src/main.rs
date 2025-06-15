use kumir_lang::{Lexer, Token};

fn main() {
    let code = include_str!("test.kum");
    let mut lexer = Lexer::new(code);
    loop {
        match lexer.next_token() {
            Ok(Token::Eof) => break,
            Ok(token) => println!("{:?}", token),
            Err(e) => println!("Ошибка: {}", e),
        }
    }
}
