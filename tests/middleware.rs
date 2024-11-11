use bakery::{Attributes, Cookie, CookieMiddleware, Processor, ProcessorConfig, IncomingConfig, SameSite, Storage, OutgoingConfig};
use actix_web::{
    dev::Payload, test, web, App, FromRequest, HttpMessage, HttpRequest, HttpResponse
};
use std::future::{ready, Ready};
use bakery_macros::cookie;
use bakery::cookies::CookieName;
#[cookie(name = "Type A")]
pub struct TypeA;

impl IncomingConfig for TypeA {
    type Get = String;
}
impl OutgoingConfig for TypeA {
    type Insert = String;

    fn attributes<'c>() -> Attributes<'c> {
        Attributes::new().same_site(SameSite::Lax).http_only(true)
    }
}

pub struct CookieCollection<'c>(Cookie<'c,TypeA>);

impl FromRequest for CookieCollection<'static>{
    type Error = Box<dyn std::error::Error>;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let a = req.extensions();
        assert!(a.get::<Storage>().is_some());
        match req.extensions().get::<Storage>() {
            Some(storage) => ready(Ok(CookieCollection(Cookie::<TypeA>::new(&storage)))),
            None => ready(Err("Processor not found in app data".into())),
        }
    }
}
async fn register_cookie(cookie: CookieCollection<'_>) -> HttpResponse {
    cookie.0.insert("id".to_string());
    HttpResponse::Ok().finish()
}
async fn get_cookie(cookie: CookieCollection<'_>) -> HttpResponse {
    let cookie = cookie.0.get().expect("Unable to get cookie");
    HttpResponse::Ok().json(cookie)
}
async fn get_all_cookie(cookie: CookieCollection<'_>) -> HttpResponse {
    let cookie = cookie.0.get_all().expect("Unable to get cookies");
    HttpResponse::Ok().json(cookie)
}
async fn remove_cookie(cookie: CookieCollection<'_>) -> HttpResponse{
    cookie.0.remove();
    HttpResponse::Ok().finish()
}

#[actix_web::test]
async fn cookie_middleware_tests() -> std::io::Result<()> {
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
    .expect("Cookie header not found")
    .to_str()
    .expect("Unable to stringify cookie header");

    assert_eq!(cookie_header, "Type%20A=%22id%22; HttpOnly; SameSite=Lax");

    // getting back cookies from the browser
    let cookie_header = "Type%20A=%22id%22";
    let request = test::TestRequest::post().insert_header((actix_web::http::header::COOKIE, cookie_header)).uri("/get").to_request();
    let response = test::call_service(&app, request).await;
    let body_str: String = test::read_body_json(response).await;

    assert_eq!(body_str, "id");

    // getting back a list of cookies  with same name from the browser
    let cookie_header = "Type%20A=%22id%22; Type%20A=%22id2%22;";
    let request = test::TestRequest::post().insert_header((actix_web::http::header::COOKIE, cookie_header)).uri("/get-all").to_request();
    let response = test::call_service(&app, request).await;
    let body_vec: Vec<String>= test::read_body_json(response).await;

    assert_eq!(body_vec, vec!["id", "id2"]);

    // remove cookies from the user browser
    let request = test::TestRequest::post().uri("/remove").to_request();
    let response = test::call_service(&app, request).await;
    let cookie_header = response.headers().get(actix_web::http::header::SET_COOKIE)
    .expect("Cookie header not found")
    .to_str()
    .expect("Unable to stringify cookie header");

    assert_eq!(cookie_header, "Type%20A=; Expires=Thu, 01 Jan 1970 00:00:00 GMT");

    Ok(())
}
