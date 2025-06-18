use kumir_lang::{
    ast::Parser,
    lexer::{Lexer, Token},
};
use log::info;

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
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    info!("AST generated: {:?}", ast)
}
