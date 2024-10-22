use actix_web::{web, Responder};
use tera::{Tera, Context};

// Controller para a rota /setup
pub async fn setup(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    let nome_pessoa = "Paulo".to_string(); // Define a variável com o nome
    context.insert("nome_pessoa", &nome_pessoa); // Insere a variável no contexto

    // Renderiza o template usando Tera
    let rendered = tmpl.render("setup.html", &context).unwrap();

    // Retorna o HTML renderizado como resposta com o cabeçalho correto
    actix_web::HttpResponse::Ok()
        .content_type("text/html") // Define o tipo de conteúdo como HTML
        .body(rendered) // Adiciona o corpo da resposta
}
