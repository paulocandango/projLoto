use actix_web::{web, HttpResponse, Responder, Error};
use serde_json::{from_str, json, Value};
use tera::{Context, Tera};
use tokio_postgres::{Client, NoTls, Row};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use std::io::Cursor;
use base64::encode;
use image::{ImageBuffer, DynamicImage, ImageFormat, Luma};
use qrcode::QrCode;
use num_format::{Locale, ToFormattedString};
use std::error::Error as StdError;
use std::env;
use serde::Deserialize;
use crate::BetForm;


// Função para estabelecer conexão com PostgreSQL com TLS
async fn establish_connection() -> Result<Client, tokio_postgres::Error> {
    let tls_connector = TlsConnector::builder().build().expect("Falha ao construir TLS");
    let tls = MakeTlsConnector::new(tls_connector);

    let (client, connection) = tokio_postgres::Config::new()
        .host("dpg-csfce008fa8c739toahg-a.oregon-postgres.render.com")
        .port(5432)
        .user("lotouser")
        .password("msvW0N3SdsLh12rbJRcONRTYWTBTqIHY")
        .dbname("loto")
        .ssl_mode(tokio_postgres::config::SslMode::Require)
        .connect(tls)
        .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro na conexão: {}", e);
        }
    });

    Ok(client)
}

// Controller para a rota /bet
pub async fn bet(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    match establish_connection().await {
        Ok(client) => {
            let query = "SELECT lottery_name FROM lottery WHERE is_dinamic = true";
            let rows = client.query(query, &[]).await.unwrap_or_default();
            let lotteries: Vec<String> = rows.into_iter().map(|row| row.get(0)).collect();
            context.insert("lotteries", &lotteries);
        }
        Err(e) => {
            eprintln!("Erro ao conectar ao banco: {:?}", e);
            return HttpResponse::InternalServerError().body("Erro na conexão com o banco");
        }
    }

    let rendered = tmpl.render("bet.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

// Controller para a rota /placeBet
pub async fn place_bet(tmpl: web::Data<Tera>, form: web::Form<BetForm>) -> impl Responder {
    let formatted_balance = get_wallet_details().await.unwrap_or("0".to_string())
        .parse::<u64>().unwrap_or(0).to_formatted_string(&Locale::en);

    let invoice_response = create_invoice(100, "Aposta LotteryBTC").await.unwrap();
    println!("invoice_response {:?}", invoice_response);

    let payment_request = invoice_response.get("payment_request").unwrap().as_str().unwrap();
    print!("payment_request {} ", payment_request);

    let checking_id = invoice_response.get("checking_id").unwrap().as_str().unwrap();
    println!("checking_id {}", checking_id);

    let qr_code_base64 = generate_qr_code_base64(payment_request);
    println!("qr_code_base64 {:?}", qr_code_base64);

    let mut context = Context::new();
    context.insert("lottery", &form.lottery);
    context.insert("wallet", &form.wallet);
    context.insert("numbers", &form.numbers);
    context.insert("formatted_balance", &formatted_balance);
    context.insert("qr_code_base64", &qr_code_base64);
    context.insert("qrcode", payment_request);
    context.insert("checking_id", checking_id);

    let rendered = tmpl.render("placebet.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

// Função para obter o saldo da carteira
async fn get_wallet_details() -> Result<String, Box<dyn StdError>> {
    // Lógica para obter os detalhes da carteira
    Ok("1000".to_string())  // Exemplo de saldo
}

// Função para criar uma fatura Lightning
async fn create_invoice(amount: i64, memo: &str) -> Result<Value, reqwest::Error> {
    let client =  reqwest::Client::new();
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

    println!("Invoice Response: {:?}", response); // Log para depuração

    Ok(response) // Retorna o JSON inteiro
}

// Função para gerar QR Code em Base64
pub fn generate_qr_code_base64(data: &str) -> String {
    let code = QrCode::new(data).unwrap();
    let width = code.width();
    let data = code.to_vec();
    let scale = 4;

    let mut image = ImageBuffer::new((width * scale) as u32, (width * scale) as u32);
    for x in 0..width {
        for y in 0..width {
            let idx = y * width + x;
            let color = if data[idx] { Luma([0]) } else { Luma([255]) };
            for dx in 0..scale {
                for dy in 0..scale {
                    image.put_pixel((x * scale + dx) as u32, (y * scale + dy) as u32, color);
                }
            }
        }
    }

    let dynamic_image = DynamicImage::ImageLuma8(image);
    let mut buffer = Cursor::new(Vec::new());
    dynamic_image.write_to(&mut buffer, ImageFormat::Png).unwrap();
    let base64_string = encode(buffer.get_ref());
    format!("data:image/png;base64,{}", base64_string)
}


#[derive(Deserialize)]
pub struct PaymentQuery {
    pub checking_id: String,
    pub lottery: String,
    pub wallet: String,
    pub numbers: String,
}

// Função para validar pagamento
pub async fn validate_payment(query: web::Query<PaymentQuery>) -> Result<HttpResponse, Box<dyn StdError>> {
    let PaymentQuery { checking_id, lottery, wallet, numbers } = query.into_inner();

    println!("Recebendo dados da requisição:");
    println!("checking_id: {}", checking_id);
    println!("Loteria: {}", lottery);
    println!("Carteira: {}", wallet);
    println!("Números: {}", numbers);

    let api_url = format!("https://demo.lnbits.com/api/v1/payments/{}", checking_id);
    let api_key = "4b63979273164f77ab6df8c7fd68e5ae";

    let client = reqwest::Client::new();
    match client.get(&api_url).header("X-Api-Key", api_key).send().await {
        Ok(mut response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Erro ao ler corpo".to_string());

            println!("Status: {}, Corpo: {}", status, body);

            if status.is_success() {
                if let Ok(json) = from_str::<Value>(&body) {
                    if json.get("paid").and_then(Value::as_bool) == Some(true) {
                        if let Err(e) = create_new_bet(&checking_id, &lottery, &wallet, &numbers).await {
                            eprintln!("Erro ao criar aposta: {}", e);
                        }
                        return Ok(HttpResponse::Ok().body("Pagamento Confirmado"));
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Erro na requisição: {:?}", err);
        }
    }

    Ok(HttpResponse::Ok().body("Aguardando confirmação do pagamento..."))
}

// Função para criar uma nova aposta no PostgreSQL
pub async fn create_new_bet(
    checking_id: &str,
    lottery: &str,
    wallet: &str,
    numbers: &str,
) -> Result<(), Box<dyn StdError>> {
    let client = establish_connection().await?;
    let query = "SELECT id_lottery FROM lottery WHERE lottery_name = $1 LIMIT 1";
    let rows = client.query(query, &[&lottery]).await?;

    if let Some(row) = rows.get(0) {
        let id_lottery: i32 = row.get(0);
        let insert_query = "INSERT INTO bet (id_lottery, wallet, numbers, checking_id) VALUES ($1, $2, $3, $4)";
        client.execute(insert_query, &[&id_lottery, &wallet, &numbers, &checking_id]).await?;
    } else {
        println!("Nenhuma loteria encontrada para: {}", lottery);
    }

    Ok(())
}
