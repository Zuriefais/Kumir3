use kumir_lang::{
    ast::Parser,
    lexer::{Lexer, Token},
};
use log::info;

fn main() {
    env_logger::init();
    let code = include_str!("test2.kum");
    let mut lexer = Lexer::new(code);
    let mut tokens = vec![];
    loop {
        match lexer.next_token() {
            Ok(Token::Eof) => break,
            Ok(token) => {
                tokens.push(token);
            }
            Err(e) => println!("Ошибка: {}", e),
        }
    }
    info!(
        "Tokens:\n{}",
        tokens
            .iter()
            .enumerate()
            .map(|(i, t)| format!("  {}: {:#?}", i, t))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    info!("AST generated: {:#?}", ast)
}
