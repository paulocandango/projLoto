use actix_web::{web, Responder, HttpResponse, Error};
use tera::{Tera, Context};
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::error::Error as StdError;
use tokio_postgres::{Client, Row};
use crate::BetForm;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;

// Função para estabelecer conexão com o PostgreSQL
async fn establish_connection() -> Result<Client, tokio_postgres::Error> {
    let tls_connector = TlsConnector::builder().build().expect("Falha ao construir TLS");
    let tls = MakeTlsConnector::new(tls_connector);

    let mut config = tokio_postgres::Config::new();
    config.host("dpg-csfce008fa8c739toahg-a.oregon-postgres.render.com");
    config.port(5432);
    config.user("lotouser");
    config.password("msvW0N3SdsLh12rbJRcONRTYWTBTqIHY");
    config.dbname("loto");
    config.ssl_mode(tokio_postgres::config::SslMode::Require);

    let (client, connection) = config.connect(tls).await?;
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
            let query = "SELECT lottery_name FROM lottery WHERE is_dinamic = TRUE";
            match client.query(query, &[]).await {
                Ok(rows) => {
                    let lotteries: Vec<String> = rows
                        .into_iter()
                        .map(|row| row.get(0))
                        .collect();
                    context.insert("lotteries", &lotteries);
                }
                Err(e) => {
                    eprintln!("Erro ao executar a consulta: {:?}", e);
                    return HttpResponse::InternalServerError()
                        .body("Erro na consulta ao banco");
                }
            }
        }
        Err(e) => {
            eprintln!("Erro ao conectar ao banco: {:?}", e);
            return HttpResponse::InternalServerError()
                .body("Erro na conexão com o banco");
        }
    }

    let rendered = match tmpl.render("bet.html", &context) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("Erro ao renderizar template: {:?}", e);
            return HttpResponse::InternalServerError().body("Erro ao renderizar a página");
        }
    };

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

// Função para criar uma nova aposta
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

    let client = establish_connection().await?;

    let query = "SELECT id_lottery, lottery_name FROM lottery WHERE lottery_name LIKE $1 LIMIT 1";
    let rows = client.query(query, &[&format!("%{}%", lottery)]).await?;

    if let Some(row) = rows.get(0) {
        let id_lottery: i32 = row.get(0);
        let lottery_name: String = row.get(1);

        println!("Loteria encontrada: ID = {}, Nome = {}", id_lottery, lottery_name);

        let insert_query = "
            INSERT INTO bet (id_lottery, wallet, numbers, checking_id)
            VALUES ($1, $2, $3, $4)
        ";
        client
            .execute(insert_query, &[&id_lottery, &wallet, &numbers, &checking_id])
            .await?;

        println!("Aposta criada com sucesso!");
    } else {
        println!("Nenhuma loteria encontrada para: {}", lottery);
    }

    Ok(())
}
