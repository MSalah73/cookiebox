use bakery::{Attributes, Cookie, CookieMiddleware, Processor, ProcessorConfig, ReadConfig, SameSite, Storage, WriteConfig};
use actix_web::{
    dev::Payload, test, web, App, FromRequest, HttpMessage, HttpRequest, Responder
};
use std::future::{ready, Ready};

pub struct TypeA;
impl TypeA {
    const NAME: &'static str = "Type A";
}
impl ReadConfig for TypeA {
    type G = String;
    const COOKIE_NAME: &'static str = Self::NAME; 
}
impl WriteConfig for TypeA {
    type I = String;
    const COOKIE_NAME: &'static str = Self::NAME; 
    fn attributes<'c>() -> bakery::Attributes<'c> {
        Attributes::new().same_site(SameSite::Lax).http_only(true)
    }
}

pub struct CookieCollection<'c>(Cookie<'c,TypeA>);

impl FromRequest for CookieCollection<'static>{
    type Error = Box<dyn std::error::Error>;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Data is cheep to clone since it uses arc
        let a = req.extensions();
        assert!(a.get::<Storage>().is_some());
        match req.extensions().get::<Storage>() {
            Some(storage) => ready(Ok(CookieCollection(Cookie::<TypeA>::new(&storage)))),
            None => ready(Err("Processor not found in app data".into())),
        }
    }
}
async fn register_cookie(cookie: CookieCollection<'_>) -> impl Responder {
    cookie.0.insert("id".to_string());
    "Logged in"
}
async fn get_cookie(cookie: CookieCollection<'_>) -> impl Responder {
    let cookie = cookie.0.get().unwrap();
    assert_eq!(cookie, "id");
    "Logged out"
}
async fn get_all_cookie(cookie: CookieCollection<'_>) -> impl Responder {
    let cookie = cookie.0.get_all().unwrap();
    assert_eq!(cookie, vec!["id", "id2"]);
    "Logged out"
}
async fn remove_cookie(cookie: CookieCollection<'_>) -> impl Responder {
    cookie.0.remove();
    "Logged out"
}

#[actix_web::test]
async fn cookie_storage() -> std::io::Result<()> {
    let processor: Processor = ProcessorConfig::default().into();
    let app = test::init_service(
        App::new()
            .wrap(
                CookieMiddleware::new(processor.clone())
            )
            .route("/register", web::post().to(register_cookie))
            .route("/get", web::post().to(get_cookie))
            .route("/get-all", web::post().to(get_all_cookie))
            .route("/remove", web::post().to(remove_cookie)),
    )
    .await;

    // registering cookies to the browser
    let request = test::TestRequest::post().uri("/register").to_request();
    let response = test::call_service(&app, request).await;
    let cookie_header = response.headers().get(actix_web::http::header::SET_COOKIE)
    .unwrap()
    .to_str()
    .unwrap();

    assert_eq!(cookie_header, "Type%20A=%22id%22; HttpOnly; SameSite=Lax");

    // getting back cookies from the browser
    let cookie_header = "Type%20A=%22id%22";
    let request = test::TestRequest::post().insert_header((actix_web::http::header::COOKIE, cookie_header)).uri("/get").to_request();
    let response = test::call_service(&app, request).await;

    // getting back a list of cookies  with same name from the browser
    let cookie_header = "Type%20A=%22id%22; Type%20A=%22id2%22;";
    let request = test::TestRequest::post().insert_header((actix_web::http::header::COOKIE, cookie_header)).uri("/get-all").to_request();
    let response = test::call_service(&app, request).await;

    // remove cookies from the user browser
    let request = test::TestRequest::post().uri("/remove").to_request();
    let response = test::call_service(&app, request).await;
    let cookie_header = response.headers().get(actix_web::http::header::SET_COOKIE)
    .unwrap()
    .to_str()
    .unwrap();

    assert_eq!(cookie_header, "Type%20A=; Expires=Thu, 01 Jan 1970 00:00:00 GMT");

    Ok(())
}
