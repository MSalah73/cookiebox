
[![crates.io](https://img.shields.io/crates/v/cookiebox?label=latest)](https://crates.io/crates/cookiebox)
[![build status](https://github.com/Msalah73/cookiebox/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/Msalah73/cookiebox/actions/workflows/ci.yml/)
[![Documentation](https://docs.rs/cookiebox/badge.svg?version=latest)](https://docs.rs/cookiebox/latest)
![Apache 2.0 or MIT licensed](https://img.shields.io/crates/l/cookiebox)
[![dependency status](https://deps.rs/repo/github/Msalah73/cookiebox/status.svg)](https://deps.rs/repo/github/Msalah73/cookiebox)

# Cookiebox

<!-- cargo-rdme start -->

A type safe cookie management crate for the Actix Web framework.

Cookiebox provides a type safe and flexible approach to managing cookies in Actix web based applications.
It allows you to define, configure, and manage cookies with minimal boilerplate.

## Features
- This crate uses [biscotti](https://docs.rs/biscotti/latest/biscotti/) under the hood, which inherit most of it's features.
- Offers the ability to configure settings on a per cookie basis.
- Enforces type definitions for deserializing cookie values upon retrieval.
- Allows customization of both the data type and data serialization.
- Provides a straightforward and type safe interface for managing cookies.

## Usage
To start using the cookiebox in your web application, you must register [CookieMiddleware] in your App.
```rust
use actix_web::{web, App, HttpServer, HttpResponse, Error};
use cookiebox::{Processor, ProcessorConfig, CookieMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   // Start by creating a `Processor` from the `ProcessorConfig`
   // This decides which cookie needs to decrypted or verified.
   let processor: Processor = ProcessorConfig::default().into();
  
   HttpServer::new(move ||
           App::new()
           // Add cookie middleware
           .wrap(CookieMiddleware::new(processor.clone()))
           .default_service(web::to(|| HttpResponse::Ok())))
       .bind(("127.0.0.1", 8080))?
       .run()
       .await
}
```
Define your desired cookie types with customizable configurations.
```rust
use actix_web::HttpMessage;
use cookiebox::cookiebox_macros::{cookie, FromRequest};
use cookiebox::cookies::{Cookie, CookieName, IncomingConfig, OutgoingConfig};
use cookiebox::{Attributes, SameSite};
use serde_json::json;

// Define a cookie type
#[cookie(name = "__my-cookie")]
pub struct MyCookie;

// IncomingConfig gives the cookie type get and get_all cookies with similar name
// You may opt out if don't want read cookie data
impl IncomingConfig for MyCookie {
   // Configure the get return to any custom type
   type Get = String;
}
// OutgoingConfig gives the cookie type insert and remove cookies
// You may opt out if don't want insert or remove a cookie
impl OutgoingConfig for MyCookie {
   // Configure the insert to any custom type
   type Insert = (String, i32);
   
   // In most cases, the default serialization should be sufficient. However, if needed,
   // you can customize the way the cookie value is serialized by implementing this method.
   fn serialize(values: Self::Insert) -> serde_json::Value {
       json!(
            format!("String: {} - i32: {}", values.0, values.1)
       )
   }
   
   // Set the appropriate attribute for the cookie, check `Attributes` for more details
   fn attributes<'c>() -> Attributes<'c> {
       Attributes::new().same_site(SameSite::Lax).http_only(false)
   }
}
// Once defined, you need to add these cookies in a collection struct and use derive macro to implement FromRequest
// Note: The macro only allows struct with either a single unnamed field or multiple named fields
#[derive(FromRequest)]
pub struct CookieCollection<'c>(Cookie<'c, MyCookie>);

```
Now, your cookies can be accessed in request handlers by using `CookieCollection` as a parameter.

If you would like to see an example, click [here](https://github.com/MSalah73/cookiebox/tree/master/examples).

<!-- cargo-rdme end -->