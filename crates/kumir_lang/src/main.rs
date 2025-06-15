use kumir_lang::{Lexer, Token};

fn main() {
    let code = "алг
    нач
      вещ число
      цел целое, сотые
      лит запись
      число := 3.14
      тест(целое, сотые, запись, число)
    кон

    алг тест (рез цел m, n, лит т, арг вещ y)
    нач
      вещ r
      m := int(y)
      r := (y - m)*100
      n := int(r)
      т := вещ_в_лит(y)
    кон";
    let mut lexer = Lexer::new(code);
    loop {
        match lexer.next_token() {
            Ok(Token::Eof) => break,
            Ok(token) => println!("{:?}", token),
            Err(e) => println!("Ошибка: {}", e),
        }
    }
}
