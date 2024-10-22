mod mega_sena_crawler;
mod loto_facil_crawler;
mod power_ball_crawler;
mod china_welfare_crawler;

use tokio::time::{self, Duration};
use actix_web::{web, App, HttpServer, Responder};
use tera::{Tera, Context};
use actix_files as fs; // Para servir arquivos estáticos

#[actix_web::main]
async fn main() {
    // 1. Imprime um log quando começa a executar
    println!("--- INICIANDO A EXECUÇÃO DA MAIN ----");

    // 2. Agenda a execução da função `update_crawlers` a cada 5 segundos
    let mut intervalo = time::interval(Duration::from_secs(10*60));

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
            .route("/setup", web::get().to(setup))
            .route("/bet", web::get().to(bet))
            .route("/placeBet", web::post().to(place_bet))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}


// Controller para a rota /bet
async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    let nome_pessoa = "Paulo".to_string(); // Define a variável com o nome
    context.insert("nome_pessoa", &nome_pessoa); // Insere a variável no contexto

    // Renderiza o template usando Tera
    let rendered = tmpl.render("index.html", &context).unwrap();
    // Retorna o HTML renderizado como resposta com o cabeçalho correto
    actix_web::HttpResponse::Ok()
        .content_type("text/html") // Define o tipo de conteúdo como HTML
        .body(rendered) // Adiciona o corpo da resposta
}

// Controller para a rota /bet
async fn bet(tmpl: web::Data<Tera>) -> impl Responder {

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
async fn place_bet(tmpl: web::Data<Tera>, form: web::Form<BetForm>) -> impl Responder {
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


#[derive(serde::Deserialize)]
struct BetForm {
    lottery: String,
    wallet: String,
    numbers: String,
}


// Controller para a rota /bet
async fn setup(tmpl: web::Data<Tera>) -> impl Responder {
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


fn parse_numbers(numbers_str: &str) -> Vec<i32> {
    numbers_str
        .split(',') // Divide a string pelos separadores de vírgula
        .filter_map(|s| s.trim().parse::<i32>().ok()) // Remove espaços e faz o parsing para i32
        .collect() // Coleta os resultados em um vetor
}