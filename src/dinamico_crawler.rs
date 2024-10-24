use std::process::Command;
use std::time::Duration;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio; // Importa o módulo tokio
use tokio::time::sleep;
use scraper::{Html, Selector}; // Importa o scraper
use mysql_async::{prelude::*, Pool, Row};
use std::error::Error;

pub async fn executar() -> Result<(), Box<dyn Error>> {
    println!("[CRAWLER] --- DINÂMICO ---");

    // Inicia o geckodriver como um subprocesso
    let mut geckodriver = Command::new("resource/geckodriver.exe")
        .arg("--port")
        .arg("4444")
        .spawn()
        .expect("Falha ao iniciar o geckodriver");

    // Configura as capacidades do Firefox, incluindo o caminho para o executável
    let mut caps = DesiredCapabilities::firefox();
    caps.set_firefox_binary("C:\\Program Files\\Mozilla Firefox\\firefox.exe");

    // Inicia o WebDriver
    let driver = WebDriver::new("http://127.0.0.1:4444", caps).await?;

    // Conexão com o banco de dados
    let url = "mysql://root:123456@localhost/loto";
    let pool = Pool::new(url);
    let mut conn = pool.get_conn().await?;

    // Consulta os sites dinâmicos e seus seletores
    let sql = "SELECT results_url, contest_selector, numbers_selector FROM Lottery WHERE is_dinamic = 1";
    let results: Vec<Row> = conn.exec(sql, ()).await?;

    // Itera sobre cada resultado da consulta
    for row in results {
        // Extrai os campos da linha
        let url: String = row.get("results_url").unwrap_or_default();
        let contest_selector: String = row.get("contest_selector").unwrap_or_default();
        let numbers_selector: String = row.get("numbers_selector").unwrap_or_default();

        println!("[CRAWLER] Acessando URL: {}", url);

        // Acessa a URL fornecida
        driver.get(&url).await?;

        // Aguarda para garantir o carregamento completo da página
        sleep(Duration::from_secs(10)).await;

        // Extrai o HTML da página
        let html = driver.page_source().await?;
        let document = Html::parse_document(&html);

        println!("--- DINÂMICO CRAWLER - Identificando o concurso e os números sorteados ---");

        // Recupera o ID do concurso usando o seletor fornecido
        if let Ok(concurso_sel) = Selector::parse(&contest_selector) {
            if let Some(resultado) = document.select(&concurso_sel).next() {
                let concurso_texto = resultado.inner_html();
                println!("Concurso: {}", concurso_texto);
            } else {
                println!("Concurso não encontrado.");
            }
        } else {
            println!("Erro ao parsear o seletor de concurso: {}", contest_selector);
        }

        // Recupera os números sorteados usando o seletor fornecido
        if let Ok(elementos_sel) = Selector::parse(&numbers_selector) {
            if let Some(resultado) = document.select(&elementos_sel).next() {
                let elementos_texto = resultado.inner_html();
                println!("Elementos: {}", elementos_texto);
            } else {
                println!("Elementos não encontrados.");
            }
        } else {
            println!("Erro ao parsear o seletor de elementos: {}", numbers_selector);
        }

        // Aguarda antes de continuar para evitar sobrecarga
        sleep(Duration::from_secs(10)).await;
    }

    // Encerra o WebDriver
    driver.quit().await?;

    // Encerra o geckodriver
    let _ = geckodriver.kill();

    Ok(())
}
