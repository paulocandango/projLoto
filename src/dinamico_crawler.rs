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

    // Configura as capacidades do Firefox
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
        let url: String = row.get("results_url").unwrap_or_default();
        let contest_selector: String = row.get("contest_selector").unwrap_or_default();
        let numbers_selector: String = row.get("numbers_selector").unwrap_or_default();

        println!("[CRAWLER] Acessando URL: {}", url);

        // Acessa a URL fornecida
        driver.get(&url).await?;
        sleep(Duration::from_secs(5)).await; // Aguarda o carregamento

        // Extrai o HTML da página e analisa
        let html = driver.page_source().await?;
        let document = Html::parse_document(&html);

        // Identifica o concurso
        if let Ok(concurso_sel) = Selector::parse(&contest_selector) {
            if let Some(resultado) = document.select(&concurso_sel).next() {
                let concurso_texto = resultado.inner_html();
                println!("Concurso: {}", concurso_texto);
            } else {
                println!("Concurso não encontrado.");
            }
        }

        // Recupera e formata os números sorteados
        let elementos_texto = if let Ok(elementos_sel) = Selector::parse(&numbers_selector) {
            if let Some(resultado) = document.select(&elementos_sel).next() {
                resultado.inner_html()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        println!("Números sorteados: {}", elementos_texto);

        // Consulta as apostas relacionadas à URL atual
        let bet_sql = r#"
            SELECT b.*
            FROM Bet b
            JOIN Lottery l ON b.id_lottery = l.id_lottery
            WHERE l.results_url = ?
        "#;

        let bets: Vec<Row> = conn.exec(bet_sql, (url.clone(),)).await?;

        // Verifica e compara os números das apostas
        for bet in bets {
            let id_bet: i64 = bet.get("id_bet").unwrap_or(0);
            let wallet: String = bet.get("wallet").unwrap_or_default();
            let numbers: String = bet.get("numbers").unwrap_or_default();
            let checking_id: String = bet.get("checking_id").unwrap_or_default();

            println!("--- Aposta Encontrada ---");
            println!("ID da Aposta: {}", id_bet);
            println!("Carteira: {}", wallet);
            println!("Números da Aposta: {}", numbers);
            println!("Checking ID: {}", checking_id);
            println!("--------------------------");

            // Comparação entre os números sorteados e os apostados
            if comparar_numeros(&elementos_texto, &numbers) {
                println!("!!! Aposta Vencedora Encontrada !!! ID da Aposta: {}", id_bet);
            } else {
                println!("Aposta não premiada.");
            }
        }

        sleep(Duration::from_secs(10)).await; // Aguarda para evitar sobrecarga
    }

    // Encerra o WebDriver e o geckodriver
    driver.quit().await?;
    let _ = geckodriver.kill();

    Ok(())
}

// Função para comparar os números sorteados com os apostados
fn comparar_numeros(sorteados: &str, apostados: &str) -> bool {
    let numeros_sorteados: Vec<&str> = sorteados.split(',').map(|s| s.trim()).collect();
    let numeros_apostados: Vec<&str> = apostados.split(',').map(|s| s.trim()).collect();
    numeros_sorteados == numeros_apostados
}
