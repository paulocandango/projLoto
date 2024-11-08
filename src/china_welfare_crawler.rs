// src/china_welfare_crawler.rs

use std::error::Error;
use std::process::Command;
use std::time::Duration;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio; // Importa o módulo tokio
use tokio::time::sleep;
use scraper::{Html, Selector}; // Importa o scraper

pub async fn executar() {

    println!("[CRAWLER] --- CHINA WELFARE LOTERRY - Loteria de Bem-Estar da China ---");

    let result = async {

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
        driver.get("http://www.cwl.gov.cn/ygkj/wqkjgg/ssq/").await.unwrap();

        // Espera um pouco para garantir que a página carregue completamente
        sleep(Duration::from_secs(10)).await; // Espera 10 segundos

        // Obtém o HTML da página
        let html = driver.page_source().await.unwrap();

        // Usa o scraper para analisar o HTML
        let document = Html::parse_document(&html);

        // Identificando o Crawler
        println!("--- CCHINA WELFARE LOTERRY - Loteria do bem-estar - Identificando o concurso ---");

        // Recuperando o ID do concurso
        let data_sorteio_selector = Selector::parse("table.ssq_table > tbody > tr:first-child > td:nth-child(2)").unwrap();

        if let Some(data_sorteio_element) = document.select(&data_sorteio_selector).next() {
            let data_sorteio_texto = data_sorteio_element.inner_html();
            println!("Data do sorteio: {}", data_sorteio_texto);
        } else {
            println!("Data do sorteio não encontrada.");
        }

        // Recuperando os ELEMENTOS sorteados
        println!("--- CCHINA WELFARE LOTERRY - Loteria do bem-estar - Identificando os elementos sorteados ---");
        // Seletor para capturar a terceira td da primeira tr
        let td_selector = Selector::parse("table.ssq_table > tbody > tr:first-child > td:nth-child(3)").unwrap();

        // Encontrando a terceira TD da primeira TR
        if let Some(td_element) = document.select(&td_selector).next() {
            // Selecionando os divs que contém os números
            let numero_selector = Selector::parse("div.qiu-item").unwrap();

            // Iterando sobre cada div que contém um número
            for numero_element in td_element.select(&numero_selector) {
                // Capturando o número de dentro da tag <font>
                if let Some(font_element) = numero_element.text().next() {
                    // Exibindo o número
                    println!("{}", font_element.trim());
                }
            }
        }

        // Fecha o driver
        driver.quit().await.unwrap();

        // Encerra o geckodriver
        let _ = geckodriver.kill();
        Ok::<(), Box<dyn Error>>(())
    }.await;
    match result {
        Ok(_) => {
            println!("[CRAWLER] --- CHINA WELFARE LOTERRY --- EXECUTADO COM SUCESSO!");
        }
        Err(e) => {
            eprintln!("[CRAWLER] --- CHINA WELFARE LOTERRY --- Erro: {}", e);
        }
    }
}
