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
    println!("Números escolhidos: {:?}", parse_numbers(&form.numbers));

    // Dentro da função
    let formatted_balance = match get_wallet_details().await {
        Ok(wallet_info) => {
            if let Some(balance) = wallet_info["balance"].as_i64() {
                // Usando um locale válido
                let locale = Locale::from_str("pt_BR").unwrap_or(Locale::en);
                balance.to_formatted_string(&locale)
            } else {
                "0".to_string()
            }
        }
        Err(e) => {
            println!("Erro ao consultar a carteira: {}", e);
            "Erro ao obter saldo".to_string()
        }
    };

    // Criando contexto para a página placebet.html
    let mut context = Context::new();
    context.insert("lottery", &form.lottery);
    context.insert("wallet", &form.wallet);
    context.insert("numbers", &form.numbers);
    context.insert("balance", &formatted_balance);

    // Renderiza o template usando Tera
    let rendered = tmpl.render("placebet.html", &context).unwrap();

    // Retorna o HTML renderizado como resposta com o cabeçalho correto
    actix_web::HttpResponse::Ok()
        .content_type("text/html") // Define o tipo de conteúdo como HTML
        .body(rendered) // Adiciona o corpo da resposta
}

// Função para obter detalhes da carteira via API LNBits
async fn get_wallet_details() -> Result<Value, reqwest::Error> {
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

    Ok(response)
}

