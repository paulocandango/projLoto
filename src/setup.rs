use std::collections::HashMap;
use actix_web::{web, Responder, HttpResponse};
use tera::{Tera, Context};
use mysql_async::{Pool, prelude::*};

// Controller para a rota /setup
pub async fn setup(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    // Conectando ao banco de dados MySQL e buscando os dados da tabela Lottery
    match fetch_lottery_data().await {
        Ok(results) => {
            context.insert("lottery_data", &results);
        }
        Err(e) => {
            context.insert("db_error", &e.to_string());
        }
    }

    // Renderiza o template usando Tera
    let rendered = tmpl.render("setup.html", &context).unwrap();

    // Retorna o HTML renderizado como resposta
    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

// Função para buscar os dados da tabela
async fn fetch_lottery_data() -> Result<Vec<(String, String, String, String)>, Box<dyn std::error::Error>> {
    let url = "mysql://root:123456@localhost/loto";
    let pool = Pool::new(url);
    let mut conn = pool.get_conn().await?;
    let result: Vec<(String, String, String, String)> = conn
        .query("SELECT lottery_name, results_url, contest_selector, numbers_selector FROM Lottery")
        .await?;
    Ok(result)
}

// Controller para a rota /delete (exclusão)
pub async fn delete_lottery(form: web::Form<HashMap<String, String>>) -> impl Responder {
    let lottery_name = form.get("lottery_name").unwrap().to_string();

    // Executa a exclusão do registro no banco
    match delete_lottery_by_name(&lottery_name).await {
        Ok(_) => HttpResponse::SeeOther().header("Location", "/setup").finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro ao excluir: {}", e)),
    }
}

// Função que executa o DELETE no banco de dados
async fn delete_lottery_by_name(lottery_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = "mysql://root:123456@localhost/loto";
    let pool = Pool::new(url);
    let mut conn = pool.get_conn().await?;

    // Executa o DELETE
    conn.exec_drop("DELETE FROM Lottery WHERE lottery_name = :name", params! { "name" => lottery_name })
        .await?;

    Ok(())
}


// Rota para renderizar o formulário de criação
pub async fn create_setup(tmpl: web::Data<Tera>) -> impl Responder {
    let context = Context::new();

    // Renderiza a página do formulário para criar novo registro
    let rendered = tmpl.render("createsetup.html", &context).unwrap();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

// Rota para processar o formulário de criação e inserir o registro no banco
pub async fn create_lottery(form: web::Form<HashMap<String, String>>) -> impl Responder {
    let lottery_name = form.get("lottery_name").unwrap().to_string();
    let results_url = form.get("results_url").unwrap().to_string();
    let contest_selector = form.get("contest_selector").unwrap().to_string();
    let numbers_selector = form.get("numbers_selector").unwrap().to_string();

    // Insere o registro no banco de dados
    match insert_lottery(&lottery_name, &results_url, &contest_selector, &numbers_selector).await {
        Ok(_) => HttpResponse::SeeOther().header("Location", "/setup").finish(),
        Err(e) => HttpResponse::InternalServerError().body(format!("Erro ao inserir: {}", e)),
    }
}

// Função que insere o novo registro no banco
async fn insert_lottery(
    lottery_name: &str,
    results_url: &str,
    contest_selector: &str,
    numbers_selector: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "mysql://root:123456@localhost/loto";
    let pool = Pool::new(url);
    let mut conn = pool.get_conn().await?;

    // Executa o INSERT
    conn.exec_drop(
        r"INSERT INTO Lottery (lottery_name, results_url, contest_selector, numbers_selector)
        VALUES (:name, :url, :contest, :numbers)",
        params! {
            "name" => lottery_name,
            "url" => results_url,
            "contest" => contest_selector,
            "numbers" => numbers_selector
        }
    ).await?;

    Ok(())
}
