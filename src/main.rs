mod mega_sena_crawler;
mod loto_facil_crawler;

use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    // 1. Imprime um log quando começa a executar
    println!("--- INICIANDO A EXECUÇÃO DA MAIN ----");

    // 2. Agenda a execução da função `update_crawlers` a cada 5 segundos
    let mut intervalo = time::interval(Duration::from_secs(5*60));

    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");
    println!("--- FIM DA MAIN ----");


    loop {
        // Aguarda o próximo "tick" do intervalo
        intervalo.tick().await;

        // 3. Executa a função `update_crawlers` a cada 5 segundos
        update_crawlers().await;
    }

}

async fn start_crawlers() {

}

// Função que imprime um log quando é executada
async fn update_crawlers() {
    println!("--- Executando update_crawlers ----");
    mega_sena_crawler::executar().await;
    loto_facil_crawler::executar().await; // Chama a função executar
}
