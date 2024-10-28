mod mega_sena_crawler;
mod loto_facil_crawler;
mod power_ball_crawler;
mod china_welfare_crawler;
mod dinamico_crawler;
mod setup;
mod bet;

use std::{env, io};
use tokio::time::{self, Duration};
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use tera::{Tera, Context};
use actix_files as fs;
use dotenvy::dotenv;

use std::process::Command;
use mysql_async::Opts;
// Para servir arquivos estáticos

// Handler para a rota `/index2`
async fn index2_handler() -> impl Responder {
    HttpResponse::Ok().body("Você foi redirecionado para /index2")
}

// Handler para a rota `/`
async fn index_handler() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/index2"))
        .finish()
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Carrega as variáveis de ambiente do arquivo .env, se existir
    dotenv().ok();

    // Obtém a porta da variável de ambiente ou usa a 8080 como padrão
    let port = env::var("PORT").unwrap_or_else(|_| "80".to_string());
    let address = format!("0.0.0.0:{}", port);
    println!("Iniciando servidor na porta {}", port);

    // Cria uma instância de Tera para carregar os templates
    let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

    // Inicia o servidor Actix Web
    HttpServer::new(move || {
        // Usa `move` para mover `tera` para dentro da closure
        App::new()
            .data(tera.clone()) // Compartilha o Tera com o App
            .service(fs::Files::new("/static", "./static").show_files_listing()) // Serve os arquivos estáticos
            .route("/", web::get().to(index)) // Rota para "/"
            .route("/setup", web::get().to(setup::setup))
            .route("/createsetup", web::get().to(setup::create_setup))
            .route("/create", web::post().to(setup::create_lottery))
            .route("/delete", web::post().to(setup::delete_lottery))
    })
        .bind(address)?
        .run()
        .await
}



async fn start_mysql_service() -> Result<(), io::Error> {
    println!("Iniciando serviço MySQL...");

    // Tenta executar o comando 'net start mysql80'
    let status = Command::new("cmd")
        .args(["/C", "net start mysql80"])
        .status()?;

    if status.success() {
        println!("Serviço MySQL iniciado com sucesso.");
    } else {
        eprintln!("Erro: O serviço MySQL não pôde ser iniciado.");
        println!(" LEIA O ARQUIVO LEIA-ME.TXT");
        println!(" READ FILE READ-ME.TXT");
    }

    // Retorna Ok independentemente do sucesso ou falha do comando
    Ok(())
}

// Função que imprime um log quando é executada
async fn update_crawlers() {
    println!("--- CRAWLERS DISABLED - DISABLED CRAWLERS ----");
    println!("--- VISIT http://localhost:8080/ ----");
    println!("--- VISIT http://localhost:8080/setup ----");
    println!("--- EXECUTANDO CRAWLERS - CRAWLERS HABILITADOS ----");
    mega_sena_crawler::executar().await;
    loto_facil_crawler::executar().await;
    power_ball_crawler::executar().await;
    china_welfare_crawler::executar().await;
    dinamico_crawler::executar().await;
}

// Função para inicializar o servidor HTTP
async fn start_server() -> std::io::Result<()> {
    // Cria uma instância de Tera para carregar os templates
    let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

    // Inicia o servidor Actix Web
    HttpServer::new(move || {
        App::new()
            .data(tera.clone()) // Compartilha o Tera com o App
            .service(fs::Files::new("/static", "./static").show_files_listing()) // Serve os arquivos estáticos
            //.route("/", web::get().to(index))
            //.route("/setup", web::get().to(setup::setup))
            //.route("/create", web::post().to(setup::create_lottery))
            //.route("/createsetup", web::get().to(setup::create_setup))
            //.route("/delete", web::post().to(setup::delete_lottery)) // Nova rota de exclusão
            //.route("/bet", web::get().to(bet::bet))
            //.route("/placeBet", web::post().to(bet::place_bet))
            //.route("/validatePayment", web::get().to(bet::validate_payment))
    })
        .bind("0.0.0.0:80")?
        .run()
        .await
}


// Controller para a rota /
async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    // Renderiza o template usando Tera
    let rendered = tmpl.render("index.html", &context).unwrap();
    // Retorna o HTML renderizado como resposta com o cabeçalho correto
    actix_web::HttpResponse::Ok()
        .content_type("text/html") // Define o tipo de conteúdo como HTML
        .body(rendered) // Adiciona o corpo da resposta
}




#[derive(serde::Deserialize)]
struct BetForm {
    lottery: String,
    wallet: String,
    numbers: String,
}


fn parse_numbers(numbers_str: &str) -> Vec<i32> {
    numbers_str
        .split(',') // Divide a string pelos separadores de vírgula
        .filter_map(|s| s.trim().parse::<i32>().ok()) // Remove espaços e faz o parsing para i32
        .collect() // Coleta os resultados em um vetor
}
