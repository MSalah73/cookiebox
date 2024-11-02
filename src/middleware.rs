use actix_web::{
    dev::{forward_ready, ResponseHead, Service, ServiceRequest, ServiceResponse, Transform}, http::header::{HeaderValue, SET_COOKIE}, HttpMessage
};
use anyhow::anyhow;
use biscotti::{errors::ProcessIncomingError, Processor, RequestCookie};
use std::{future::Future, pin::Pin, rc::Rc};
use actix_utils::future::{ready, Ready};

use crate::Storage;

pub struct CookieMiddleware {
    processor: Rc<Processor>,
}

impl CookieMiddleware {
    pub fn new(processor: Processor) -> Self {
        Self {
            processor: Rc::new(processor),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CookieMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = InnerCookieMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerCookieMiddleware {
            service: Rc::new(service),
            processor: Rc::clone(&self.processor),
        }))
    }
}

pub fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub struct InnerCookieMiddleware<S> {
    service: Rc<S>,
    processor: Rc<Processor>,
}

impl<S, B> Service<ServiceRequest> for InnerCookieMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    // We only gonna use LocalBoxFuture from futures-rs
    // no need to import a whole dependency where can just the equivalent here
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);
    
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let processor = Rc::clone(&self.processor);
        let storage =  Storage::new();

        Box::pin(async move {
            
            //clone storage is cheap, since we are only coping the rc
            extract_cookies(&req, &processor, storage.clone()).map_err(e500)?;

            req.extensions_mut().insert(storage.clone());

            let mut response = service.call(req).await?;

            process_response_cookies(response.response_mut().head_mut(), &processor, storage.clone()).map_err(e500)?;

            Ok(response)
        })
    }
}

// Currently, the parse header methods and process_incoming method does not support returning a RequestCookie with owned
// name and value only borrowed. for the time being, I have reconstructed the parse header method to do just that until proper
// support in added to the biscotti crate.  
fn extract_cookies(req: &ServiceRequest, processor: &Processor, storage: Storage) -> Result<(), anyhow::Error> 
{
    let cookie_header = req.headers().get(actix_web::http::header::COOKIE);

    let cookie_header = match cookie_header {
        Some(header) => header.to_str().map_err(|e| {
            anyhow!("Invalid cookie header encoding: {}", e)
        })?,
        None => return Ok(())
    };

    for cookie in cookie_header.split(';') {
        if cookie.chars().all(char::is_whitespace) {
            continue;
        }

        let (name, value) = match cookie.split_once('=') {
            Some((name, value)) => (name.trim(), value.trim()),
            None => {
                return Err(anyhow!("Expected a name-value pair, but no `=` was found in `{}`", cookie.to_string()));
            }
        };

        if name.is_empty() {
            return Err(anyhow!("The name of a cookie cannot be empty, but found an empty name with `{}` as value", value.to_string()));
        }

        let cookie = match processor.process_incoming(name, value) {
            Ok(c) => c,
            Err(e) => {
                let t = match e {
                    ProcessIncomingError::Crypto(e) => "an encrypted",
                    ProcessIncomingError::Decoding(e) => "a singed",
                    _ => unreachable!()
                };
                return Err(anyhow!("Failed to process `{}` as {t} request cookie", name));
            }
        };

        let cookie = RequestCookie::new(cookie.name().to_owned(), cookie.value().to_owned());
        storage.request_storage.borrow_mut().append(cookie);
    }

    Ok(())
}
fn process_response_cookies(
    response: &mut ResponseHead,
    processor: &Processor,
    storage: Storage
) -> Result<(), anyhow::Error> {
    let response_storage = storage.response_storage.take();
    for cookie in response_storage.header_values(&processor) {
        let cookie = HeaderValue::from_str(&cookie).map_err(|e| {
            anyhow!("Failed to attached cookies to outgoing response: {}", e)
        })?;
        response.headers_mut().append(SET_COOKIE, cookie);
    }

    Ok(())
}
