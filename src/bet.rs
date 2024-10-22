use actix_web::{web, Responder};
use tera::{Tera, Context};
use crate::{parse_numbers, BetForm};

// Controller para a rota /bet
pub async fn bet(tmpl: web::Data<Tera>) -> impl Responder {

    let mut context = Context::new();

    let nome_pessoa = "Paulo".to_string(); // Define a variável com o nome
    context.insert("nome_pessoa", &nome_pessoa); // Insere a variável no contexto

    // Renderiza o template usando Tera
    let rendered = tmpl.render("bet.html", &context).unwrap();
    // Retorna o HTML renderizado como resposta com o cabeçalho correto
    actix_web::HttpResponse::Ok()
        .content_type("text/html") // Define o tipo de conteúdo como HTML
        .body(rendered) // Adiciona o corpo da resposta
}

// Controller para a rota /placeBet
pub async fn place_bet(tmpl: web::Data<Tera>, form: web::Form<BetForm>) -> impl Responder {
    println!("--- Registrando Aposta ---");
    println!("Loteria: {}", form.lottery);
    println!("Carteira Bitcoin: {}", form.wallet);
    println!("Números escolhidos: {:?}", parse_numbers(&form.numbers));

    // Criando contexto para a página placebet.html
    let mut context = Context::new();
    context.insert("lottery", &form.lottery);
    context.insert("wallet", &form.wallet);
    context.insert("numbers", &form.numbers);

    // Renderiza o template usando Tera
    let rendered = tmpl.render("placebet.html", &context).unwrap();

    // Retorna o HTML renderizado como resposta com o cabeçalho correto
    actix_web::HttpResponse::Ok()
        .content_type("text/html") // Define o tipo de conteúdo como HTML
        .body(rendered) // Adiciona o corpo da resposta
}
