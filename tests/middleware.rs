use actix_web::{test, web, App, HttpMessage, HttpResponse};
use cookiebox::cookiebox_macros::{cookie, FromRequest};
use cookiebox::cookies::{Cookie, CookieName, IncomingConfig, OutgoingConfig};
use cookiebox::{Attributes, CookieMiddleware, Processor, ProcessorConfig, SameSite};

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

#[derive(FromRequest)]
pub struct CookieCollection<'c>(Cookie<'c, TypeA>);

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
async fn remove_cookie(cookie: CookieCollection<'_>) -> HttpResponse {
    cookie.0.remove();
    HttpResponse::Ok().finish()
}

#[actix_web::test]
async fn cookie_middleware_tests() -> std::io::Result<()> {
    let processor: Processor = ProcessorConfig::default().into();
    let app = test::init_service(
        App::new()
            .wrap(CookieMiddleware::new(processor.clone()))
            .route("/register", web::post().to(register_cookie))
            .route("/get", web::post().to(get_cookie))
            .route("/get-all", web::post().to(get_all_cookie))
            .route("/remove", web::post().to(remove_cookie)),
    )
    .await;

    // registering cookies to the browser
    let request = test::TestRequest::post().uri("/register").to_request();
    let response = test::call_service(&app, request).await;
    let cookie_header = response
        .headers()
        .get(actix_web::http::header::SET_COOKIE)
        .expect("Cookie header not found")
        .to_str()
        .expect("Unable to stringify cookie header");

    assert_eq!(cookie_header, "Type%20A=%22id%22; HttpOnly; SameSite=Lax");

    // getting back cookies from the browser
    let cookie_header = "Type%20A=%22id%22";
    let request = test::TestRequest::post()
        .insert_header((actix_web::http::header::COOKIE, cookie_header))
        .uri("/get")
        .to_request();
    let response = test::call_service(&app, request).await;
    let body_str: String = test::read_body_json(response).await;

    assert_eq!(body_str, "id");

    // getting back a list of cookies  with same name from the browser
    let cookie_header = "Type%20A=%22id%22; Type%20A=%22id2%22;";
    let request = test::TestRequest::post()
        .insert_header((actix_web::http::header::COOKIE, cookie_header))
        .uri("/get-all")
        .to_request();
    let response = test::call_service(&app, request).await;
    let body_vec: Vec<String> = test::read_body_json(response).await;

    assert_eq!(body_vec, vec!["id", "id2"]);

    // remove cookies from the user browser
    let request = test::TestRequest::post().uri("/remove").to_request();
    let response = test::call_service(&app, request).await;
    let cookie_header = response
        .headers()
        .get(actix_web::http::header::SET_COOKIE)
        .expect("Cookie header not found")
        .to_str()
        .expect("Unable to stringify cookie header");

    assert_eq!(
        cookie_header,
        "Type%20A=; Expires=Thu, 01 Jan 1970 00:00:00 GMT"
    );

    Ok(())
}
