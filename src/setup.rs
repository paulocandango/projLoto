use actix_web::{web, Responder};
use tera::{Tera, Context};
use mysql_async::{Pool, prelude::*};

// Controller para a rota /setup
pub async fn setup(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    // Conectando ao banco de dados MySQL e buscando os dados da tabela Lottery
    match fetch_lottery_data().await {
        Ok(results) => {
            // Adiciona os resultados no contexto para o template
            context.insert("lottery_data", &results);
        }
        Err(e) => {
            // Em caso de erro, insere a mensagem de erro no contexto para o template
            context.insert("db_error", &e.to_string());
        }
    }

    // Renderiza o template usando Tera
    let rendered = tmpl.render("setup.html", &context).unwrap();

    // Retorna o HTML renderizado como resposta
    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

async fn fetch_lottery_data() -> Result<Vec<(String, String, String, String)>, Box<dyn std::error::Error>> {
    // URL de conexão ao MySQL
    let url = "mysql://root:123456@localhost/loto";

    // Cria o pool de conexões
    let pool = Pool::new(url);

    // Estabelece uma conexão
    let mut conn = pool.get_conn().await?;

    // Executa a consulta na tabela Lottery
    let result: Vec<(String, String, String, String)> = conn
        .query("SELECT lottery_name, results_url, contest_selector, numbers_selector FROM Lottery")
        .await?;

    // Retorna o resultado da consulta
    Ok(result)
}
