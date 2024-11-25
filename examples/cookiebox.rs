use actix_web::{get, App, HttpMessage, HttpResponse, HttpServer};
use cookiebox::cookiebox_macros::{cookie, FromRequest};
use cookiebox::cookies::{Cookie, CookieName, IncomingConfig, OutgoingConfig};
use cookiebox::{
    config::{CryptoAlgorithm, CryptoRule},
    CookieMiddleware, Key, Processor, ProcessorConfig,
};
use cookiebox::{Attributes, SameSite};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up the proccesor for the middleware
    let mut cookie_config = ProcessorConfig::default();

    // Set up the rules encrypted cookies
    let crypto_rule = CryptoRule {
        cookie_names: vec!["__cookie-b".to_string()],
        algorithm: CryptoAlgorithm::Encryption,
        key: Key::generate(),
        fallbacks: vec![],
    };

    cookie_config.crypto_rules.push(crypto_rule);

    let processor: Processor = cookie_config.into();

    HttpServer::new(move || {
        App::new()
            // The middleware handles the extraction and traformation the cookies from the request handler
            .wrap(CookieMiddleware::new(processor.clone()))
            .service(get_cookie_a)
            .service(get_cookie_b)
            .service(add_cookie_b)
            .service(update_cookie_b)
            .service(remove_cookie_b)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Data Types
#[derive(Serialize, Deserialize, Debug)]
pub struct CookieData {
    pub data: String,
}

//Define cookies
#[cookie(name = "__cookie-a")]
pub struct CookieA;
#[cookie(name = "__cookie-b")]
pub struct CookieB;

// Cookie type configuration
//
// Cookie A
// This generic type parameter would give Cookie type get and get_all
impl IncomingConfig for CookieA {
    type Get = String;
}
// Cookie B
// This generic type parameter would give Cookie type get, get_all, insert, and remove.
impl IncomingConfig for CookieB {
    type Get = CookieData;
}
impl OutgoingConfig for CookieB {
    type Insert = (String, i32);

    // Customize serialization method
    fn serialize(values: Self::Insert) -> serde_json::Value {
        json!({
            "data": format!("Name: {} - Age: {}", values.0, values.1)
        })
    }
    // Configure attributes for cookie
    fn attributes<'c>() -> Attributes<'c> {
        Attributes::new().same_site(SameSite::Lax).http_only(true)
    }
}
// Implement FromRequest for CookieCollection
#[derive(FromRequest)]
pub struct CookieCollection<'c> {
    cookie_a: Cookie<'c, CookieA>,
    cookie_b: Cookie<'c, CookieB>,
}

#[get("add_cookie_b")]
async fn add_cookie_b(cookies_collection: CookieCollection<'_>) -> HttpResponse {
    cookies_collection
        .cookie_b
        .insert(("Scarlet".to_string(), 27));

    HttpResponse::Ok().body("Encrypted cookie added")
}

#[get("get_cookie_b")]
async fn get_cookie_b(cookies_collection: CookieCollection<'_>) -> HttpResponse {
    // This returns a Ok(CookieData) if found, otherwise Err(CookieBoxError)
    let data = cookies_collection
        .cookie_b
        .get()
        .map_err(|e| eprint!("Unable to get cookie data - {e}"));

    HttpResponse::Ok().json(data)
}

#[get("update_cookie_b")]
async fn update_cookie_b(cookies_collection: CookieCollection<'_>) -> HttpResponse {
    // This returns a Ok(CookieData) if found, otherwise Err(CookieBoxError)
    let old_data = cookies_collection
        .cookie_b
        .get()
        .map_err(|e| eprint!("Unable to get cookie data - {e}"));

    // Since the path, domain, and name are the same, this would replace the current data with the below
    cookies_collection
        .cookie_b
        .insert(("Jason".to_string(), 22));

    HttpResponse::Ok().body(format!(
        "old data: {:?} - Go to get_cookie_b to check the new value",
        old_data
    ))
}

#[get("remove_cookie_b")]
async fn remove_cookie_b(cookies_collection: CookieCollection<'_>) -> HttpResponse {
    cookies_collection.cookie_b.remove();

    HttpResponse::Ok().body("__cookie-b removed")
}
// Add a new cookie in the browser with the value `%22STRING%22` and set the attributes to defualt values to get
// this cookie - Check the Attribute::default
#[get("get_cookie_a")]
async fn get_cookie_a(cookies_collection: CookieCollection<'_>) -> HttpResponse {
    // This returns a Ok(String) if found, otherwise Err(CookieBoxError)
    let data = cookies_collection
        .cookie_a
        .get()
        .map_err(|e| eprint!("Unable to get cookie data - {e}"));

    HttpResponse::Ok().body(format!("{:?}", data))
}
