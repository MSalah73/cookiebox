//! A strongly typed cookie management crate for the Actix Web framework.
//!
//! cookiebox provides a robust, type-safe, and flexible approach to managing cookies in Actix web based applications.
//! It allows you to define, configure, and retrieve cookies with minimal boilerplate.
//!
//! # Features
//! - This crate uses [biscotti](https://docs.rs/biscotti/latest/biscotti/) under the hood which inherit most of it's features.
//! - Supports custom configuration settings per cookie
//! - Define specific types for deserializing cookie values during retrieval
//! - Customize data type and serialization method for each cookie.
//! - A Simple and type safe interface for retrieving, adding, removing cookies.
//!
//! # Usage
//! To start using the cookiebox crate in your web application you must register [CookieMiddleware] in your App.
//! ```no_run
//!use actix_web::{web, App, HttpServer, HttpResponse, Error};
//!use cookiebox::{Processor, ProcessorConfig, CookieMiddleware};
//!
//!#[actix_web::main]
//!async fn main() -> std::io::Result<()> {
//!    // Start by creating a `Processor` from the `ProcessorConfig`
//!    // This decides which cookie needs to decrypted or verified.
//!    let processor: Processor = ProcessorConfig::default().into();
//!   
//!    HttpServer::new(move ||
//!            App::new()
//!            // Add cookie middleware
//!            .wrap(CookieMiddleware::new(processor.clone()))
//!            .default_service(web::to(|| HttpResponse::Ok())))
//!        .bind(("127.0.0.1", 8080))?
//!        .run()
//!        .await
//!}
//! ```
//! Now, define the desired typed cookies with custom configuration
//! ```no_run
//!use cookiebox::cookiebox_macros::cookie;
//!use cookiebox::cookies::{Cookie, CookieName, IncomingConfig, OutgoingConfig};
//!use cookiebox::{Attributes, SameSite};
//!use cookiebox::Storage;
//!use actix_web::{HttpRequest, FromRequest, HttpMessage, dev::Payload};
//!use actix_utils::future::{ready, Ready};
//!use serde_json::json;
//!
//!// Define you cookie type struct
//!#[cookie(name = "__my-cookie")]
//!pub struct MyCookie;
//!
//!// IncomingConfig give the cookie type get and get_all cookie with similar name
//!// You may opt out if don't want read cookie data
//!impl IncomingConfig for MyCookie {
//!    // Configure the get return to any custom type
//!    type Get = String;
//!}
//!// OutgoingConfig give the cookie type insert and remove cookie
//!// You may opt out if don't want insert or remove a cookie
//!impl OutgoingConfig for MyCookie {
//!    // Configure the insert to any custom type
//!    type Insert = (String, i32);
//!    
//!    // In most cases, the default serialization should be sufficient. However, if needed,
//!    // you can customize the way the cookie value is serialized by implementing this method.
//!    fn serialize(values: Self::Insert) -> serde_json::Value {
//!        json!(
//!             format!("String: {} - i32: {}", values.0, values.1)
//!        )
//!    }
//!    
//!    // Set the appropriate attribute for the cookie check `Attributes` for more details
//!    fn attributes<'c>() -> Attributes<'c> {
//!        Attributes::new().same_site(SameSite::Lax).http_only(false)
//!    }
//!}
//!// Add all cookies in cookie collection
//! pub struct CookieCollection<'c>(Cookie<'c, MyCookie>);
//!
//!//Once defined, your cookies can be accessed in request handlers by implementing `FromRequest` for a collection
//!//of typed cookies.
//!impl FromRequest for CookieCollection<'static> {
//!    type Error = Box<dyn std::error::Error>;
//!    type Future = Ready<Result<Self, Self::Error>>;
//!
//!    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
//!        // fetch the storage value from request extensions and add it to all your cookies
//!        match req.extensions().get::<Storage>() {
//!            Some(storage) => ready(Ok(CookieCollection(Cookie::<MyCookie>::new(&storage)))),
//!            None => ready(Err("Storage is missing".into())),
//!        }
//!    }
//! }
//! ```
mod attributes;
pub mod cookies;
mod middleware;
mod storage;

pub use attributes::Attributes;
pub use biscotti::{time, Expiration, Processor, ProcessorConfig, SameSite};
pub use cookiebox_macros;
pub use middleware::CookieMiddleware;
pub use storage::Storage;
