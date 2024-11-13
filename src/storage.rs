//! Holds a collection of both request and response cookies
use std::{cell::RefCell, rc::Rc};

use biscotti::{RequestCookies, ResponseCookies};

#[derive(Clone)]
pub struct Storage<'s> {
    pub(crate) request_storage: Rc<RefCell<RequestCookies<'s>>>,
    pub(crate) response_storage: Rc<RefCell<ResponseCookies<'s>>>,
}
impl Storage<'_> {
    pub(crate) fn new() -> Self {
        Storage {
            request_storage: Rc::new(RefCell::new(RequestCookies::new())),
            response_storage: Rc::new(RefCell::new(ResponseCookies::new())),
        }
    }
}
