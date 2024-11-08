mod mega_sena_crawler;
mod loto_facil_crawler;
mod power_ball_crawler;
mod china_welfare_crawler;
mod dinamico_crawler;
mod setup;
mod bet;
use tokio::time::{self, Duration};

use std::{env, io};
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use tera::{Tera, Context};
use actix_files as fs;
use dotenvy::dotenv;

// Handler para a rota `/`
async fn index_handler() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/index2"))
        .finish()
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let mut intervalo = time::interval(Duration::from_secs(5*60));

    // LOOP PARA EXECUÇÃO DO CRAWLER - CASO QUEIRA RODAR O SERVIDOR WEB TERÁ QUE COMENTAR


    /*loop {
        // Aguarda o próximo "tick" do intervalo
        intervalo.tick().await;

        // 3. Executa a função `update_crawlers` a cada x segundos
        update_crawlers().await;
    }*/


    println!("INICIANDO A VERSAO DA BRANCH render QUE NAO TEM OS CRAWLERS - CRIADA PARA PUBLICACAO NO SITE RENDER.COM");
    println!("OS CROWLERS ESTAO NA BRANCH local ");
    println!("OS CROWLERS ESTAO NA BRANCH local ");
    println!("OS CROWLERS ESTAO NA BRANCH local ");
    println!("OS CROWLERS ESTAO NA BRANCH local ");
    println!("OS CROWLERS ESTAO NA BRANCH local ");

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
            .route("/bet", web::get().to(bet::bet))
            .route("/placeBet", web::post().to(bet::place_bet))
            .route("/validatePayment", web::get().to(bet::validate_payment))
    })
        .bind(address)?
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

async fn update_crawlers() {
    println!("--- EXECUTANDO CRAWLERS - CRAWLERS HABILITADOS ----");

    println!("--- YOU MUST HAVE FIREFOX INSTALLED ----");
    println!("--- MANDATORY IN C:\\Program Files\\Mozilla Firefox\\firefox.exe ----");


    mega_sena_crawler::executar().await;
    loto_facil_crawler::executar().await;
    power_ball_crawler::executar().await;
    china_welfare_crawler::executar().await;
    dinamico_crawler::executar().await;
}