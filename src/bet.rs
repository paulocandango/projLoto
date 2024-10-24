use actix_web::{web, Responder, HttpResponse, Error};
use tera::{Tera, Context};
use crate::{parse_numbers, BetForm};
use reqwest::Client;
use serde_json::{from_str, Value};
use base64::encode;
use std::io::Cursor;
use image::{Luma, ImageBuffer, DynamicImage, ImageFormat};
use qrcode::QrCode;
use num_format::{Locale, ToFormattedString};
use serde::Deserialize;
use std::error::Error as StdError;
use mysql_async::{prelude::*, Pool, Row};

// Controller para a rota /bet
pub async fn bet(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    // Conexão com o banco de dados
    let url = "mysql://root:123456@localhost/loto";
    let pool = Pool::new(url);
    let mut conn = match pool.get_conn().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Erro ao conectar ao banco: {:?}", e);
            return HttpResponse::InternalServerError().body("Erro na conexão com o banco");
        }
    };

    // Executa a consulta para obter as loterias
    let sql = "SELECT lottery_name FROM Lottery WHERE is_dinamic = 1";
    let lotteries: Vec<Row> = match conn.exec(sql, ()).await {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Erro ao executar a consulta: {:?}", e);
            return HttpResponse::InternalServerError().body("Erro na consulta ao banco");
        }
    };

    // Extrai os nomes das loterias e adiciona ao contexto
    let lottery_names: Vec<String> = lotteries
        .into_iter()
        .filter_map(|row| row.get("lottery_name"))
        .collect();
    context.insert("lotteries", &lottery_names);

    // Renderiza o template usando Tera
    let rendered = match tmpl.render("bet.html", &context) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Erro ao renderizar template: {:?}", e);
            return HttpResponse::InternalServerError().body("Erro ao renderizar a página");
        }
    };

    // Retorna o HTML renderizado como resposta
    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}


// --------------------------------------------------------------------------------------------------------------------------------------------

// Controller para a rota /placeBet
pub async fn place_bet(tmpl: web::Data<Tera>, form: web::Form<BetForm>) -> impl Responder {
    println!("--- Registrando Aposta ---");
    println!("Loteria: {}", form.lottery);
    println!("Carteira Bitcoin: {}", form.wallet);
    println!("Números escolhidos: {:?}", form.numbers);

    // RECUPERA O BALANÇO DA CARTEIRA
    let formatted_balance = get_wallet_details().await.unwrap_or_else(|e| {
        println!("Erro ao obter saldo: {}", e);
        "0".to_string()
    })
        .parse::<u64>() // Converte para u64 para formatação
        .unwrap_or(0)
        .to_formatted_string(&Locale::en);

    // GERA A FATURA LIGHTNING
    let invoice_response = match create_invoice(100, "Aposta LotteryBTC").await {
        Ok(response) => response,
        Err(e) => {
            println!("Erro ao criar fatura: {}", e);
            return actix_web::HttpResponse::InternalServerError().body("Erro ao gerar fatura");
        }
    };

    // Recebe os atributos do response
    let payment_request = invoice_response.get("payment_request").and_then(Value::as_str).unwrap_or("").to_string();
    let checking_id = invoice_response.get("checking_id").and_then(Value::as_str).unwrap_or("").to_string();

    // Gera o QR Code com a fatura Lightning
    let qr_code_base64 = generate_qr_code_base64(&payment_request);

    // Cria o contexto para o template HTML
    let mut context = Context::new();
    context.insert("lottery", &form.lottery);
    context.insert("wallet", &form.wallet);
    context.insert("numbers", &form.numbers);
    context.insert("formatted_balance", &formatted_balance);
    context.insert("qr_code_base64", &qr_code_base64);
    context.insert("qrcode", &payment_request);
    context.insert("checking_id", &checking_id);

    // Renderiza o template HTML com o contexto
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
async fn create_invoice(amount: i64, memo: &str) -> Result<Value, reqwest::Error> {
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

    println!("Invoice Response: {:?}", response); // Log para depuração

    Ok(response) // Retorna o JSON inteiro
}

//------------------------------------------------------------------------------------------
pub fn generate_qr_code_base64(data: &str) -> String {
    // Cria o QR Code a partir do dado fornecido
    let code = QrCode::new(data).unwrap();

    // Obtém a largura do QR Code
    let width = code.width();
    let data = code.to_vec(); // Cada elemento é um bool: true = preto, false = branco

    // Define o fator de escala para ampliar a imagem (8x maior)
    let scale = 4;

    // Cria um buffer de imagem com o novo tamanho (escala aplicada)
    let mut image = ImageBuffer::new((width * scale) as u32, (width * scale) as u32);

    // Preenche a imagem com os pixels do QR Code aplicando a escala
    for x in 0..width {
        for y in 0..width {
            let idx = y * width + x; // Calcula o índice na matriz linear
            let color = if data[idx] {
                Luma([0]) // Preto
            } else {
                Luma([255]) // Branco
            };

            // Preenche os pixels ampliados
            for dx in 0..scale {
                for dy in 0..scale {
                    image.put_pixel((x * scale + dx) as u32, (y * scale + dy) as u32, color);
                }
            }
        }
    }

    // Converte a imagem para DynamicImage para salvar como PNG
    let dynamic_image = DynamicImage::ImageLuma8(image);

    // Cria um buffer para armazenar a imagem PNG
    let mut buffer = Cursor::new(Vec::new());

    // Salva a imagem no formato PNG no buffer
    dynamic_image.write_to(&mut buffer, ImageFormat::Png).unwrap();

    // Converte o conteúdo do buffer em uma string Base64
    let base64_string = encode(buffer.get_ref());

    // Retorna a imagem como uma data URI para exibição em HTML
    format!("data:image/png;base64,{}", base64_string)
}

//--------------------------------------------------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PaymentStatus {
    paid: bool,
}

#[derive(Deserialize)]
pub struct PaymentQuery {
    pub checking_id: String,
    pub lottery: String,
    pub wallet: String,
    pub numbers: String,
}

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

pub async fn create_new_bet(
    checking_id: &str,
    lottery: &str,
    wallet: &str,
    numbers: &str,
) -> Result<(), Box<dyn StdError>> {
    println!("Iniciando processo de criação de nova aposta.");
    println!("checking_id: {}", checking_id);
    println!("Loteria: {}", lottery);
    println!("Carteira: {}", wallet);
    println!("Números: {}", numbers);

    let url = "mysql://root:123456@localhost/loto";
    let pool = Pool::new(url);
    let mut conn = pool.get_conn().await?;

    let sql = "SELECT * FROM Lottery WHERE lottery_name LIKE ? LIMIT 1";
    let result: Option<Row> = conn.exec_first(sql, (format!("%{}%", lottery),)).await?;

    if let Some(row) = result {
        let id_lottery: i64 = row.get("id_lottery").unwrap_or(0);
        let lottery_name: String = row.get("lottery_name").unwrap_or_default();

        println!("Loteria encontrada: ID = {}, Nome = {}", id_lottery, lottery_name);

        let insert_sql = r#"
            INSERT INTO Bet (id_lottery, wallet, numbers, checking_id)
            VALUES (?, ?, ?, ?)
        "#;

        conn.exec_drop(insert_sql, (id_lottery, wallet, numbers, checking_id)).await?;
        println!("Aposta criada com sucesso!");
    } else {
        println!("Nenhuma loteria encontrada para: {}", lottery);
    }

    Ok(())
}