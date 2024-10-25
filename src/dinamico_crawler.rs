use std::env;
use std::process::Command;
use std::time::Duration;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio; // Importa o módulo tokio
use tokio::time::sleep;
use scraper::{Html, Selector}; // Importa o scraper
use mysql_async::{prelude::*, Pool, Row};
use std::error::Error;
use reqwest::Client; // Adicionando o cliente HTTP

const LN_API_KEY: &str = "1673bd51f74f41e7baeaf290be710009"; // Substitua com sua chave

const LN_API_URL: &str = "https://demo.lnbits.com/api/v1/payments"; // URL da LNBits API


pub async fn executar() -> Result<(), Box<dyn Error>> {
    println!("[CRAWLER] --- DINAMICO ---");

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
    let url = env::var("MYSQL_URL").expect("MYSQL_URL não encontrada");
    println!("url: {}", url);
    let pool = Pool::new(url.as_str());
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

        driver.get(&url).await?;
        sleep(Duration::from_secs(5)).await;

        let html = driver.page_source().await?;
        let document = Html::parse_document(&html);

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

        let bet_sql = r#"
            SELECT b.*
            FROM Bet b
            JOIN Lottery l ON b.id_lottery = l.id_lottery
            WHERE l.results_url = ?
        "#;

        let bets: Vec<Row> = conn.exec(bet_sql, (url.clone(),)).await?;

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

            if comparar_numeros(&elementos_texto, &numbers) {
                println!("Aposta Vencedora! Efetuando pagamento...");

                // Efetua o pagamento para a carteira vencedora
                match efetuar_pagamento(&wallet, 100).await {
                    Ok(_) => println!("Pagamento efetuado com sucesso para a carteira: {}", wallet),
                    Err(e) => eprintln!("Erro ao efetuar pagamento: {}", e),
                }
            } else {
                println!("Aposta não premiada.");
            }
        }

        sleep(Duration::from_secs(10)).await;
    }

    driver.quit().await?;
    let _ = geckodriver.kill();

    Ok(())
}

async fn efetuar_pagamento(wallet: &str, amount: i64) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let params = serde_json::json!({
        "out": true,
        "amount": amount,
        "memo": "Prêmio da aposta vencedora",
        "bolt11": wallet
    });

    let response = client
        .post(LN_API_URL)
        .header("X-Api-Key", LN_API_KEY)
        .json(&params)
        .send()
        .await?;

    // Captura o status antes de consumir o corpo da resposta
    let status = response.status();
    let body = response.text().await.unwrap_or_else(|_| "Erro desconhecido".to_string());

    if status.is_success() {
        println!("Pagamento efetuado com sucesso para: {}", wallet);
    } else {
        eprintln!("Erro no pagamento: {} - {}", status, body);
    }

    Ok(())
}

// Função para comparar os números sorteados com os apostados
fn comparar_numeros(sorteados: &str, apostados: &str) -> bool {
    let numeros_sorteados: Vec<&str> = sorteados.split(',').map(|s| s.trim()).collect();
    let numeros_apostados: Vec<&str> = apostados.split(',').map(|s| s.trim()).collect();
    numeros_sorteados == numeros_apostados
}
