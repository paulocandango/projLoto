// src/mega_sena_crawler.rs

use std::process::Command;
use std::time::Duration;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio; // Importa o módulo tokio
use tokio::time::sleep;
use scraper::{Html, Selector}; // Importa o scraper

pub async fn executar() {

    println!("[CRAWLER] --- CAIXA ECONOMICA FEDERAL - MEGASENA ---");

    // Inicia o geckodriver como um subprocesso
    let mut geckodriver = Command::new("resource/geckodriver.exe")
        .arg("--port")
        .arg("4444") // Define a porta como 4444
        .spawn()
        .expect("Falha ao iniciar o geckodriver");

    // Configura as capacidades do Firefox, incluindo o caminho para o executável
    let mut caps = DesiredCapabilities::firefox();
    caps.set_firefox_binary("C:\\Program Files\\Mozilla Firefox\\firefox.exe");

    // Configura o driver do Firefox
    let driver = WebDriver::new("http://127.0.0.1:4444", caps).await.unwrap();

    // Abre a página da Mega-Sena
    driver.get("http://loterias.caixa.gov.br/Paginas/Mega-Sena.aspx").await.unwrap();

    // Espera um pouco para garantir que a página carregue completamente
    sleep(Duration::from_secs(10)).await; // Espera 10 segundos

    // Obtém o HTML da página
    let html = driver.page_source().await.unwrap();

    // Usa o scraper para analisar o HTML
    let document = Html::parse_document(&html);

    // Identificando o Crawler
    println!("--- CAIXA ECONOMICA FEDERAL - MEGASENA - Identificando o concurso e os números sorteados ---");

    // Recuperando o ID do concurso
    let concurso_selector = Selector::parse("h2 > span.ng-binding").unwrap();
    if let Some(resultado) = document.select(&concurso_selector).next() {
        // Captura o texto do concurso
        let concurso_texto = resultado.inner_html(); // ou use resultado.text() para pegar apenas o texto sem HTML
        println!("{}", concurso_texto);
    } else {
        println!("Resultado não encontrado.");
    }

    // Recuperando os NÚMEROS sorteados
    let numero_selector = Selector::parse("div.item-powerball").unwrap();

    for element in document.select(&numero_selector) {
        // Captura apenas o texto dos elementos, sem HTML
        let numero_texto = element.text().collect::<String>().trim().to_string();

        // Imprime cada número sorteado
        println!("{}", numero_texto);
    }

    // Fecha o driver
    driver.quit().await.unwrap();

    // Encerra o geckodriver
    let _ = geckodriver.kill();
}
