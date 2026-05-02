mod lexer;

use lexer::Lexer;

fn main() {
    let source_code = r#"
        let x = 42,
            y = "Hola HULK",
            greet = x @@ y
        in print(greet);
    "#;

    println!(">>> Código de entrada:");
    println!("{}", source_code);
    println!("--------------------------------------------------");
    println!(">>> Salida de Tokens generados:");

    let mut lexer = Lexer::new(source_code);
    
    // Consumo del iterador perezoso del lexer
    while let Some(token) = lexer.next() {
        println!("{:?}", token);
    }
}
