// src/mega_sena_crawler.rs

use std::process::Command;
use std::time::Duration;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio; // Importa o módulo tokio
use tokio::time::sleep;
use scraper::{Html, Selector}; // Importa o scraper
use mysql_async::{prelude::*, Pool, Row};

pub async fn executar() -> Result<(), Box<dyn std::error::Error>> {
    println!("[CRAWLER] --- DINAMICO ---");

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

    // Conexão com o banco de dados
    let url = "mysql://root:123456@localhost/loto";
    let pool = Pool::new(url);
    let mut conn = pool.get_conn().await?;

    // Consulta no banco para obter os sites dinâmicos
    let sql = "SELECT results_url FROM Lottery WHERE is_dinamic = 1";
    let results: Vec<Row> = conn.exec(sql, ()).await?; // Recupera todas as linhas

    // Itera sobre cada resultado retornado da consulta
    for row in results {
        let url: String = row.get("results_url").unwrap_or_default();
        println!("[CRAWLER] Acessando URL: {}", url);

        // Abre a página da loteria
        driver.get(&url).await.unwrap();

        // Espera um pouco para garantir que a página carregue completamente
        sleep(Duration::from_secs(10)).await;

        // Obtém o HTML da página
        let html = driver.page_source().await.unwrap();
        let document = Html::parse_document(&html);

        // Identificando o Crawler
        println!("--- DINAMICO CRAWLER - Identificando o concurso e os números sorteados ---");

        // Recuperando o ID do concurso
        let concurso_selector = Selector::parse("#identity").unwrap();
        if let Some(resultado) = document.select(&concurso_selector).next() {
            let concurso_texto = resultado.inner_html();
            println!("Concurso: {}", concurso_texto);
        } else {
            println!("Concurso não encontrado.");
        }

        // Recuperando os ELEMENTOS sorteados
        let elementos_selector = Selector::parse("#elements").unwrap();
        if let Some(resultado) = document.select(&elementos_selector).next() {
            let elementos_texto = resultado.inner_html();
            println!("Elementos: {}", elementos_texto);
        } else {
            println!("Elementos não encontrados.");
        }

        sleep(Duration::from_secs(10)).await;
    }

    // Fecha o driver
    driver.quit().await.unwrap();

    // Encerra o geckodriver
    let _ = geckodriver.kill();

    Ok(())
}
