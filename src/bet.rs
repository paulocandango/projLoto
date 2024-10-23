use actix_web::{web, Responder};
use tera::{Tera, Context};
use crate::{parse_numbers, BetForm};
use reqwest::Client;
use serde_json::Value;
use base64::encode;
use std::io::Cursor;
use image::{Luma, ImageBuffer, DynamicImage, ImageFormat};
use qrcode::QrCode;

// Controller para a rota /bet
pub async fn bet(tmpl: web::Data<Tera>) -> impl Responder {

    let mut context = Context::new();

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

    // Gera a fatura Lightning
    let payment_request = match create_invoice(1000, "Aposta LotteryBTC").await {
        Ok(request) => request,
        Err(e) => {
            println!("Erro ao criar fatura: {}", e);
            return actix_web::HttpResponse::InternalServerError().body("Erro ao gerar fatura");
        }
    };

    // Gera o QR Code com a fatura Lightning
    let qr_code_base64 = generate_qr_code_base64(&payment_request);

    // Cria o contexto para o template HTML
    let mut context = Context::new();
    context.insert("lottery", &form.lottery);
    context.insert("wallet", &form.wallet);
    context.insert("numbers", &form.numbers);
    context.insert("qr_code_base64", &qr_code_base64);
    context.insert("qrcode", &payment_request);
    context.insert("formatted_balance", &formatted_balance);

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

//------------------------------------------------------------------------------------------
pub fn generate_qr_code_base64(data: &str) -> String {
    // Cria o QR Code a partir do dado fornecido
    let code = QrCode::new(data).unwrap();

    // Obtém a matriz de módulos do QR Code (como uma slice de bools)
    let width = code.width();
    let data = code.to_vec(); // Cada elemento é um bool: true = preto, false = branco

    // Cria um buffer de imagem (escala de cinza)
    let mut image = ImageBuffer::new(width as u32, width as u32);

    // Preenche a imagem com os pixels do QR Code
    for x in 0..width {
        for y in 0..width {
            let idx = y * width + x; // Calcula o índice correto na matriz linear
            let color = if data[idx] {
                Luma([0]) // Preto
            } else {
                Luma([255]) // Branco
            };
            image.put_pixel(x as u32, y as u32, color);
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