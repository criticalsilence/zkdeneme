#[macro_use] extern crate rocket;

use reqwest::Client;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::form::Form;
use rocket::response::content;
use std::env;
use dotenv::dotenv;

#[derive(Deserialize)]
struct BalanceResponse {
    result: Option<String>,
    error: Option<ErrorResponse>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    code: i32,
    message: String,
}

#[derive(FromForm)]
struct AddressForm {
    address: String,
}

#[derive(Serialize)]
struct AirdropResponse {
    message: String,
    balance: f64,
}

#[post("/check_airdrop", data = "<address_form>")]
async fn check_airdrop(address_form: Form<AddressForm>) -> Json<AirdropResponse> {
    dotenv().ok();
    let api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set");
    let api_url = format!("https://zksync-sepolia.g.alchemy.com/v2/{}", api_key);

    let client = Client::new();
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [address_form.address, "latest"],
        "id": 1
    });

    let response = client
        .post(&api_url)
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let response_text = resp.text().await.unwrap();
            println!("Response: {}", response_text);

            let balance_response: BalanceResponse = serde_json::from_str(&response_text).unwrap();

            if let Some(result) = balance_response.result {
                let balance_in_wei = u64::from_str_radix(&result[2..], 16).unwrap();
                let balance_in_eth = balance_in_wei as f64 / 1e18;

                let message = if balance_in_eth >= 0.1 {
                    format!("Airdrop'a hak kazandınız!")
                } else {
                    format!("Maalesef airdrop'a hak kazanamadınız.")
                };

                Json(AirdropResponse {
                    message,
                    balance: balance_in_eth,
                })
            } else if let Some(error) = balance_response.error {
                Json(AirdropResponse {
                    message: format!("Hata: code = {}, message = {}", error.code, error.message),
                    balance: 0.0,
                })
            } else {
                Json(AirdropResponse {
                    message: "Beklenmedik bir hata oluştu.".to_string(),
                    balance: 0.0,
                })
            }
        }
        Err(_) => Json(AirdropResponse {
            message: "API isteği başarısız oldu.".to_string(),
            balance: 0.0,
        }),
    }
}

#[get("/")]
fn index() -> content::RawHtml<&'static str> {
    content::RawHtml(r#"
        <html>
            <head>
                <title>ZkSync Testnet Airdrop Checker</title>
            </head>
            <body>
                <h1>Airdrop Checker</h1>
                <form action="/check_airdrop" method="post">
                    <label for="address">Ethereum Address:</label>
                    <input type="text" id="address" name="address">
                    <button type="submit">Check Airdrop</button>
                </form>
            </body>
        </html>
    "#)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![check_airdrop])
}
