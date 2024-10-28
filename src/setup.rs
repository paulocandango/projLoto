use actix_web::{web, Responder, HttpResponse};
use tera::{Tera, Context};
use tokio_postgres::{Error, Client, Config};
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;

// Função para estabelecer a conexão com PostgreSQL via TLS
async fn establish_connection() -> Result<Client, Error> {
    // Configura o TLS Connector
    let tls_connector = TlsConnector::builder()
        .build()
        .expect("Falha ao construir TlsConnector.");
    let tls = MakeTlsConnector::new(tls_connector);

    // Configura os parâmetros de conexão
    let mut config = Config::new();
    config.host("dpg-csfce008fa8c739toahg-a.oregon-postgres.render.com");
    config.port(5432);
    config.user("lotouser");
    config.password("msvW0N3SdsLh12rbJRcONRTYWTBTqIHY");
    config.dbname("loto");
    config.ssl_mode(tokio_postgres::config::SslMode::Require);

    let (client, connection) = config.connect(tls).await?;

    // Inicia a conexão em uma tarefa separada
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Erro na conexão: {}", e);
        }
    });

    Ok(client)
}

// Controller para a rota /setup
pub async fn setup(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();

    match fetch_lottery_data().await {
        Ok(results) => {
            context.insert("lottery_data", &results);
        }
        Err(err) => {
            context.insert("error", &format!("Erro ao buscar dados: {}", err));
        }
    }

    let rendered = tmpl.render("setup.html", &context);
    match rendered {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(err) => HttpResponse::InternalServerError().body(format!("Erro ao renderizar template: {}", err)),
    }
}

// Função para buscar dados da tabela Lottery
async fn fetch_lottery_data() -> Result<Vec<(i32, String, String, String, String)>, Error> {
    match establish_connection().await {
        Ok(client) => {
            match client
                .query(
                    "SELECT id_lottery, lottery_name, results_url, contest_selector, numbers_selector FROM lottery",
                    &[],
                )
                .await
            {
                Ok(rows) => {
                    let results: Vec<(i32, String, String, String, String)> = rows
                        .into_iter()
                        .map(|row| {
                            let record = (
                                row.get(0),
                                row.get(1),
                                row.get(2),
                                row.get(3),
                                row.get(4),
                            );
                            // Imprime cada registro recuperado
                            println!("Registro recuperado: {:?}", record);
                            record
                        })
                        .collect();
                    Ok(results)
                }
                Err(query_error) => {
                    // Imprime mensagem de erro ao executar a consulta
                    println!("Erro na consulta ao banco de dados: {:?}", query_error);
                    Err(query_error)
                }
            }
        }
        Err(connection_error) => {
            // Imprime mensagem de erro ao estabelecer a conexão
            println!("Erro ao conectar ao banco de dados: {:?}", connection_error);
            Err(connection_error)
        }
    }
}
