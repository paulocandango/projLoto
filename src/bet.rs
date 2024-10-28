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

    let formatted_balance = "1000".to_string();

    let VALOR_FIXO_APOSTA = 1000 * 1000;
    let invoice_response = create_invoice(VALOR_FIXO_APOSTA, "Aposta LotteryBTC").await.unwrap();
    println!("invoice_response {:?}", invoice_response);

    // Acessando "request" corretamente dentro de "data" -> "invoice"
    let payment_request = invoice_response
        .get("data")
        .and_then(|data| data.get("invoice"))
        .and_then(|invoice| invoice.get("request"))
        .and_then(|request| request.as_str())
        .unwrap_or("");

    println!("payment_request {}", payment_request);

    // Acessando o "id" dentro de "data"
    let checking_id = invoice_response
        .get("data")
        .and_then(|data| data.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("");

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


// Função para criar uma fatura Lightning
async fn create_invoice(amount: i64, memo: &str) -> Result<Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = "https://api.zebedee.io/v0/charges";
    let api_key = "xfWlrZeeNzk0JButS3LEG57k5FLTiBIq";

    let params = serde_json::json!({
        "amount": amount.to_string(),  // Zebedee espera string para o amount
        "description": memo
    });

    let response = client
        .post(url)
        .header("apikey", api_key) // Zebedee usa 'apikey' como cabeçalho
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

    // Endpoint da Zebedee para verificar o status do charge
    let api_url = format!("https://api.zebedee.io/v0/charges/{}", checking_id);
    let api_key = "xfWlrZeeNzk0JButS3LEG57k5FLTiBIq";

    let client = reqwest::Client::new();
    match client
        .get(&api_url)
        .header("apikey", api_key) // Cabeçalho de autenticação
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Erro ao ler corpo".to_string());



            println!("Status: {}, Corpo: {}", status, body);




            if status.is_success() {
                if let Ok(json) = from_str::<Value>(&body) {
                    // Verifica se o status do pagamento é "completed" no campo correto
                    if json
                        .get("data")
                        .and_then(|data| data.get("status"))
                        .and_then(Value::as_str)
                        == Some("completed")
                    {
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
