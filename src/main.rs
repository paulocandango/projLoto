mod mega_sena_crawler;
mod loto_facil_crawler;
mod power_ball_crawler;
mod china_welfare_crawler;
mod dinamico_crawler;
mod setup;
mod bet;

use std::env;
use tokio::time::{self, Duration};
use actix_web::{web, App, HttpServer, Responder};
use tera::{Tera, Context};
use actix_files as fs;
use dotenvy::dotenv;
use mysql_async::Opts;
// Para servir arquivos estáticos

#[actix_web::main]
async fn main() {
    // 1. Imprime um log quando começa a executar
    println!("--- INICIANDO A EXECUÇÃO DA MAIN ----");

    // Carrega as variáveis do arquivo .env
    dotenv().ok();

    // Lê a variável MYSQL_URL do ambiente
    let url = env::var("MYSQL_URL").expect("MYSQL_URL não encontrada");
    println!("Conectando ao banco de dados em: {}", url.as_str());

    // 2. Agenda a execução da função `update_crawlers` a cada 5 segundos
    let mut intervalo = time::interval(Duration::from_secs(1*60));

    // 3. Iniciar o servidor HTTP antes do loop
    println!("--- INICIANDO SERVIDOR HTTP ---");
    actix_web::rt::spawn(start_server()); // Executa o servidor em segundo plano

    // Fim da main
    println!("--- FIM DA MAIN ----");


    loop {
        // Aguarda o próximo "tick" do intervalo
        intervalo.tick().await;

        // 3. Executa a função `update_crawlers` a cada 5 segundos
        update_crawlers().await;
    }

}

// Função que imprime um log quando é executada
async fn update_crawlers() {
    println!("--- Executando update_crawlers ----");
    //mega_sena_crawler::executar().await;
    //loto_facil_crawler::executar().await;
    //power_ball_crawler::executar().await;
    //china_welfare_crawler::executar().await;
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
            .route("/", web::get().to(index))
            .route("/setup", web::get().to(setup::setup))
            .route("/create", web::post().to(setup::create_lottery))
            .route("/createsetup", web::get().to(setup::create_setup))
            .route("/delete", web::post().to(setup::delete_lottery)) // Nova rota de exclusão
            .route("/bet", web::get().to(bet::bet))
            .route("/placeBet", web::post().to(bet::place_bet))
            .route("/validatePayment", web::get().to(bet::validate_payment))
    })
        .bind("127.0.0.1:8080")?
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
