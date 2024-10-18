use std::{cell::RefCell, rc::Rc};

use biscotti::{RequestCookies, ResponseCookies};

// Would be possible to use none here - initialize them when needed
// Response - when first insert
// Request Empty if there are no response cookie - handle error saying no cookie data was found
#[derive(Clone)]
pub struct Storage<'s>{
    pub request_storage: Rc<RefCell<RequestCookies<'s>>>,
    pub response_storage: Rc<RefCell<ResponseCookies<'s>>>,
}
impl Storage<'_> {
    pub fn new() -> Self {
        Storage {
            request_storage: Rc::new(RefCell::new(RequestCookies::new())),
            response_storage: Rc::new(RefCell::new(ResponseCookies::new())),
        }
    }
}