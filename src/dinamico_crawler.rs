use std::env;
use std::process::Command;
use std::time::Duration;
use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio;
use tokio::time::sleep;
use scraper::{Html, Selector};

use reqwest::Client as HttpClient;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use serde::Deserialize;
use tokio_postgres::{Client, Config, Connection, Error, NoTls, Socket};
use serde_json::Value;


#[derive(Deserialize)]
struct LnurlResponse {
    callback: String,     // URL de callback para gerar a fatura
    maxSendable: u64,     // Máximo permitido para envio (em millisatoshis)
    minSendable: u64,     // Mínimo permitido (em millisatoshis)
    tag: String,          // Tipo de LNURL (ex.: 'payRequest')
    metadata: String,     // Metadados sobre o pagamento
}

#[derive(Deserialize)]
struct InvoiceResponse {
    pr: String,           // Fatura BOLT11
    routes: Vec<String>,  // Rotas opcionais para o pagamento (pode estar vazio)
}

const API_KEY: &str = "xfWlrZeeNzk0JButS3LEG57k5FLTiBIq";

pub async fn efetuar_pagamento_ln_adress(wallet: &str) -> Result<(), Box<dyn std::error::Error>> {

    println!("[CRAWLER] --- DINAMICO --- efetuar_pagamento_ln_adress");

    // 1. Resolver o Lightning Address
    let (username, domain) = wallet.split_once('@')
        .ok_or("Endereço Lightning inválido")?;
    let lnurl = format!("https://{}/.well-known/lnurlp/{}", domain, username);

    println!("Resolvendo LNURL: {}", lnurl);

    // 2. Requisição HTTP para resolver o LNURL
    let client = reqwest::Client::new();
    let response = client.get(&lnurl).send().await?;

    if !response.status().is_success() {
        return Err(format!("Erro ao resolver LNURL: {}", response.status()).into());
    }

    let lnurl_data: LnurlResponse = response.json().await?;
    println!("Callback URL: {}", lnurl_data.callback);

    // 3. Verificar se o valor fixo de 1000 sats é permitido
    let amount_msats = 1000 * 1000; // 1000 satoshis em millisatoshis
    if amount_msats < lnurl_data.minSendable || amount_msats > lnurl_data.maxSendable {
        return Err("Valor fora dos limites permitidos pelo endereço Lightning.".into());
    }

    // 4. Gerar fatura via callback
    let callback_url = format!("{}?amount={}", lnurl_data.callback, amount_msats);
    println!("Gerando fatura via: {}", callback_url);

    let invoice_response = client.get(&callback_url).send().await?;

    if !invoice_response.status().is_success() {
        return Err(format!("Erro ao gerar fatura: {}", invoice_response.status()).into());
    }

    let invoice_data: InvoiceResponse = invoice_response.json().await?;
    let bolt11_invoice = invoice_data.pr;
    println!("Fatura BOLT11 gerada: {}", bolt11_invoice);

    // 5. Pagar a fatura gerada
    let payment_response = client
        .post("https://api.zebedee.io/v0/ln-address/send-payment") // Endpoint da API de pagamento da Zebedee
        .header("Content-Type", "application/json")
        .header("apikey", API_KEY)
        .json(&serde_json::json!({
            "lnAddress": wallet,
            "amount": amount_msats,
            "comment": "Pagamento via Lightning Address"
        }))
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    if !payment_response.status().is_success() {
        return Err(format!("Erro ao efetuar pagamento: {}", payment_response.status()).into());
    }

    println!("Pagamento de 1000 sats realizado com sucesso para: {}", wallet);

    Ok(())
}

pub async fn executar() -> Result<(), Box<dyn std::error::Error>> {

    println!("[CRAWLER] --- DINAMICO ---");

    let result = async {


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

        // Conexão com PostgreSQL
        let (client, connection) = establish_pg_connection().await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Erro na conexão: {}", e);
            }
        });

        // Consulta os sites dinâmicos e seus seletores
        let sql = "SELECT results_url, contest_selector, numbers_selector FROM Lottery WHERE is_dinamic = true";
        let results = client.query(sql, &[]).await?;

        // Itera sobre cada resultado da consulta
        for row in results {
            let url: String = row.get("results_url");
            let contest_selector: String = row.get("contest_selector");
            let numbers_selector: String = row.get("numbers_selector");

            // Faz scraping da página
            driver.get(&url).await?;
            sleep(Duration::from_secs(10)).await;

            // Extrai e processa os dados
            let page_source = driver.source().await?;
            let fragment = Html::parse_document(&page_source);

            let contest = extract_text(&fragment, &contest_selector);
            let numbers = extract_text(&fragment, &numbers_selector);

            println!("Concurso: {}, Números: {}", contest, numbers);










            // Consulta as apostas feitas para essa loteria
            let bet_sql = r#"
                SELECT b.id_bet, b.wallet, b.numbers, b.checking_id
                FROM Bet b
                JOIN Lottery l ON b.id_lottery = l.id_lottery
                WHERE l.results_url = $1
            "#;

            let bets = client.query(bet_sql, &[&url]).await?;
            for bet in bets {
                let id_bet: i32 = bet.get("id_bet");
                let wallet: String = bet.get("wallet");
                let numbers: String = bet.get("numbers");
                let checking_id: String = bet.get("checking_id");

                println!("--- Aposta Encontrada ---");
                println!("ID da Aposta: {}", id_bet);
                println!("Carteira: {}", wallet);
                println!("Números da Aposta: {}", numbers);
                println!("Checking ID: {}", checking_id);
                println!("--------------------------");

                if comparar_numeros(&numbers, &numbers) {

                    println!("Aposta Vencedora! Efetuando pagamento...");

                    match efetuar_pagamento_ln_adress(&wallet).await {
                        Ok(_) => println!("Pagamento efetuado com sucesso para a carteira: {}", wallet),
                        Err(e) => eprintln!("Erro ao efetuar pagamento: {}", e),
                    }
                    /*match efetuar_pagamento_via_lnurl(&wallet).await {
                        Ok(_) => println!("Pagamento efetuado com sucesso para a carteira: {}", wallet),
                        Err(e) => eprintln!("Erro ao efetuar pagamento: {}", e),
                    }*/
                } else {
                    println!("Aposta não premiada.");
                }
            }

            sleep(Duration::from_secs(10)).await;
        }

        driver.quit().await?;
        let _ = geckodriver.kill();

        Ok::<(), Box<dyn std::error::Error>>(())

    }
    .await;

    match result {
        Ok(_) => {
            println!("[CRAWLER] --- DINAMICO --- EXECUTADO COM SUCESSO!");
            Ok(())
        }
        Err(e) => {
            eprintln!("[CRAWLER] --- DINAMICO --- EXECUTADO COM ERRO: {}", e);
            Ok(())
        }
    }
}
















async fn establish_pg_connection() -> Result<(Client, Connection<Socket, postgres_native_tls::TlsStream<Socket>>), Error> {

    // Configurando o conector TLS
    let tls_connector = TlsConnector::builder()
        .build()
        .expect("Falha ao construir TlsConnector.");
    let tls = MakeTlsConnector::new(tls_connector);

    // Configurando os parâmetros de conexão
    let mut config = Config::new();
    config.host("dpg-csfce008fa8c739toahg-a.oregon-postgres.render.com");
    config.port(5432);
    config.user("lotouser");
    config.password("msvW0N3SdsLh12rbJRcONRTYWTBTqIHY");
    config.dbname("loto");
    config.ssl_mode(tokio_postgres::config::SslMode::Require); // Força uso de SSL/TLS

    // Estabelecendo a conexão
    let (client, connection) = config.connect(tls).await?;
    Ok((client, connection))
}

fn extract_text(fragment: &Html, selector_str: &str) -> String {
    let selector = Selector::parse(selector_str).unwrap();
    fragment
        .select(&selector)
        .next()
        .map(|element| element.inner_html())
        .unwrap_or_default()
}


async fn efetuar_pagamento(wallet: &str) -> Result<(), Box<dyn std::error::Error>> {

    let VALOR_FIXO_PREMIO: i64 = 1000*1000;

    let client = reqwest::Client::new(); // Cliente HTTP
    let url = "https://api.zebedee.io/v0/payments";
    let api_key = "xfWlrZeeNzk0JButS3LEG57k5FLTiBIq";

    // Construindo os parâmetros para o payout
    let params = serde_json::json!({
        "amount": VALOR_FIXO_PREMIO.to_string(), // Quantidade em msats como string
        "description": "Prêmio da aposta vencedora",
        "invoice": wallet // Invoice BOLT11 do destinatário
    });

    // Enviando a requisição POST para a Zebedee
    let response = client
        .post(url)
        .header("apikey", api_key)
        .json(&params)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await.unwrap_or_else(|_| "Erro desconhecido".to_string());

    // Verificando se o pagamento foi bem-sucedido
    if status.is_success() {
        println!("Pagamento efetuado com sucesso para: {}", wallet);
    } else {
        eprintln!("Erro no pagamento: {} - {}", status, body);
    }

    Ok(())
}


async fn efetuar_pagamento_via_lnurl(ln_identifier: &str) -> Result<(), Box<dyn std::error::Error>> {

    let client = reqwest::Client::new();

    // Cortando a string na posição do '@' e pegando a parte anterior
    let username = ln_identifier.split('@').next().unwrap_or("");
    println!("Usuário extraído: {}", username);

    // Construindo o LNURL a partir do identificador Zebedee
    let lnurl = format!("https://zbd.gg/.well-known/lnurlp/{}", username);
    println!("Resolvendo LNURL: {}", lnurl);

    // Etapa 1: Obter informações de pagamento via LNURL
    let lnurl_response = client.get(&lnurl).send().await?;
    let status = lnurl_response.status();
    let lnurl_info: Value = lnurl_response.json().await?;

    println!("LNURL Info: {}", lnurl_info);

    if !status.is_success() {
        eprintln!("Erro ao resolver LNURL: {}", status);
        return Ok(());
    }

    // Extraindo a URL de pagamento e limites
    let callback_url = lnurl_info["callback"].as_str().unwrap();
    let min_sendable = lnurl_info["minSendable"].as_i64().unwrap_or(0);
    let max_sendable = lnurl_info["maxSendable"].as_i64().unwrap_or(0);

    eprintln!("CALCULO DE VALOR MINIMO={}", min_sendable);
    eprintln!("CALCULO DE VALOR MAXIMO={}", max_sendable);

    let PREMIO_FIXO: i64=1100;

    let payment_url = format!("{}?amount={}", callback_url, PREMIO_FIXO);
    println!("Enviando pagamento para: {}", payment_url);
    println!("PREMIO_FIXO: {}", PREMIO_FIXO);

    // Enviando a requisição de pagamento
    let payment_response = client.get(&payment_url).send().await?;
    let payment_status = payment_response.status();
    let payment_body = payment_response.text().await.unwrap_or_else(|_| "Erro desconhecido".to_string());

    println!("Resposta do pagamento: Status = {}, Body = {}", payment_status, payment_body);

    if payment_status.is_success() {
        println!("Pagamento efetuado com sucesso via LNURL.");
    } else {
        eprintln!("Erro no pagamento: {} - {}", payment_status, payment_body);
    }

    Ok(())
}


// Função para comparar os números sorteados com os apostados
fn comparar_numeros(sorteados: &str, apostados: &str) -> bool {
    let numeros_sorteados: Vec<&str> = sorteados.split(',').map(|s| s.trim()).collect();
    let numeros_apostados: Vec<&str> = apostados.split(',').map(|s| s.trim()).collect();
    numeros_sorteados == numeros_apostados
}