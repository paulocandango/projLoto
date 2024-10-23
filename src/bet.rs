use actix_web::{web, Responder};
use num_format::{Locale, ToFormattedString};
use std::str::FromStr;
use tera::{Tera, Context};
use crate::{parse_numbers, BetForm};
use reqwest::Client;
use serde_json::Value;

// Controller para a rota /bet
pub async fn bet(tmpl: web::Data<Tera>) -> impl Responder {

    let mut context = Context::new();

    let nome_pessoa = "Paulo".to_string(); // Define a variável com o nome
    context.insert("nome_pessoa", &nome_pessoa); // Insere a variável no contexto

    // Renderiza o template usando Tera
    let rendered = tmpl.render("bet.html", &context).unwrap();
    // Retorna o HTML renderizado como resposta com o cabeçalho correto
    actix_web::HttpResponse::Ok()
        .content_type("text/html") // Define o tipo de conteúdo como HTML
        .body(rendered) // Adiciona o corpo da resposta
}


// --------------------------------------------------------------------------------------------------------------------------------------------

// Controller para a rota /placeBet
pub async fn place_bet(tmpl: web::Data<Tera>, form: web::Form<BetForm>) -> impl Responder {
    println!("--- Registrando Aposta ---");
    println!("Loteria: {}", form.lottery);
    println!("Carteira Bitcoin: {}", form.wallet);
    println!("Números escolhidos: {:?}", form.numbers);

    let formatted_balance = get_wallet_details().await.unwrap_or_else(|e| {
        println!("Erro ao obter saldo: {}", e);
        "0".to_string()
    });

    let qr_code = match create_invoice(1000, "Aposta LotteryBTC").await {
        Ok(qr) => qr,
        Err(e) => {
            println!("Erro ao criar fatura: {}", e);
            String::from("Erro ao gerar QR Code")
        }
    };

    let mut context = Context::new();
    context.insert("lottery", &form.lottery);
    context.insert("wallet", &form.wallet);
    context.insert("numbers", &form.numbers);
    context.insert("balance", &formatted_balance);
    context.insert("qrcode", &qr_code); // Inserindo o QR Code no contexto

    let rendered = tmpl.render("placebet.html", &context).unwrap();

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

// Função para obter detalhes da carteira
async fn get_wallet_details() -> Result<String, reqwest::Error> {
    let client = Client::new();
    let url = "https://demo.lnbits.com/api/v1/wallet";
    let api_key = "4b63979273164f77ab6df8c7fd68e5ae";

    let response = client
        .get(url)
        .header("X-Api-Key", api_key)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response["balance"].as_i64().unwrap_or(0).to_string())
}

// Função para criar uma fatura e retornar o QR Code
async fn create_invoice(amount: i64, memo: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let url = "https://demo.lnbits.com/api/v1/payments";
    let api_key = "4b63979273164f77ab6df8c7fd68e5ae";

    let params = serde_json::json!({
        "out": false,
        "amount": amount,
        "memo": memo
    });

    let response = client
        .post(url)
        .header("X-Api-Key", api_key)
        .json(&params)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(response["payment_request"].as_str().unwrap_or("").to_string())
}

