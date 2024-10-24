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
        sleep(Duration::from_secs(5)).await;

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

        //----------------------------------------------------------------------------------------------------------------------
        // CONSULTANDO OS GANHADORES
        //----------------------------------------------------------------------------------------------------------------------













        // Consulta os registros na tabela Bet correspondentes à URL atual
        println!("--- Buscando apostas para essa URL: {} ---", &url);

        let bet_sql = r#"
    SELECT b.*
    FROM Bet b
    JOIN Lottery l ON b.id_lottery = l.id_lottery
    WHERE l.results_url = ?
"#;

        let bets: Vec<Row> = conn.exec(bet_sql, (url.clone(),)).await?;

        // Verifica se a consulta retornou algum registro
        if bets.is_empty() {
            println!("Nenhuma aposta encontrada para a URL: {}", url);
        } else {
            // Imprime cada linha encontrada na tabela Bet
            for bet in bets {
                let id_bet: i64 = bet.get("id_bet").unwrap_or(0);
                let wallet: String = bet.get("wallet").unwrap_or_default();
                let numbers: String = bet.get("numbers").unwrap_or_default();
                let checking_id: String = bet.get("checking_id").unwrap_or_default();

                println!("--- Aposta Encontrada ---");
                println!("ID da Aposta: {}", id_bet);
                println!("Carteira: {}", wallet);
                println!("Números: {}", numbers);
                println!("Checking ID: {}", checking_id);
                println!("--------------------------");
            }
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
