// src/main.rs

mod mega_sena_crawler;
mod loto_facil_crawler;

use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {

    println!("--- INICIANDO O PROJETO projLoto ----");

    // Agendar a execução da função executar a cada 10 segundos
    loop {
        mega_sena_crawler::executar().await; // Chama a função executar
        time::sleep(Duration::from_secs(10)).await; // Espera 60 segundos
        loto_facil_crawler::executar().await; // Chama a função executar
        time::sleep(Duration::from_secs(60)).await; // Espera 60 segundos
    }
}
