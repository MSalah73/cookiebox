use std::any::type_name;
use std::borrow::Cow;
use biscotti::{RemovalCookie, ResponseCookie};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;
use crate::attributes::{Attributes, AttributesSetter};
use crate::storage::Storage;

#[derive(Error,Debug)]
pub enum BakeryError{
    #[error("`{0}` does not exist")]
    NotFound(String),
    #[error("Failed to deserialize `{0}` to type `{1}`")]
    Deserialization(String, String),
}

pub struct Cookie<'c, T> {
    storage: Storage<'c>,
    attributes: Option<Attributes<'c>>,
    _marker: std::marker::PhantomData<T>,
}
impl<'c, T> Cookie<'c, T>{
    pub fn new(
        storage: &Storage<'c>
    ) -> Self {
        Cookie {
            storage: storage.clone(),
            attributes: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: ReadConfig> Cookie<'_, T> {
    pub fn get(&self) -> Result<T::G, BakeryError>
    {
        let data = &self
            .storage
            .request_storage
            .borrow()
            .get(T::COOKIE_NAME)
            .ok_or(BakeryError::NotFound(T::COOKIE_NAME.to_string()))?;
        // Does it panic?  use clone
        // Closure - Implement response error to remove it - i think
        let data = serde_json::from_str(data.value())
            .map_err(|_| BakeryError::Deserialization(data.value().to_string(), type_name::<T::G>().to_string()))?;
        Ok(data)
    }
}
impl<'c,T: WriteConfig> Cookie<'c, T> {
    pub fn insert(&self, value: T::I)
    {
        let data = T::serialize(value);
    
        let response_cookie = ResponseCookie::new(T::COOKIE_NAME, data.to_string());

        let attributes = match &self.attributes {
            Some(attributes) => attributes,
            None => &T::attributes()
        };

        let response_cookie = response_cookie.set_attributes(&attributes);

        self.storage.response_storage.borrow_mut().insert(response_cookie); 
    }
    pub fn remove(&self) {
        let response_cookie = RemovalCookie::new(T::COOKIE_NAME);
        self.storage.response_storage.borrow_mut().insert(response_cookie); 
    }
    pub fn set_path<P: Into<Cow<'c, str>>>(&mut self, path: P) {
        let attributes = match self.attributes.take() {
            Some(attributes) => attributes.path(path),
            None => T::attributes()
        };
        self.attributes = Some(attributes);
    }
    pub fn set_domain<D: Into<Cow<'c, str>>>(&mut self, domain: D) {
        let attributes = match self.attributes.take() {
            Some(attributes) => attributes.domain(domain),
            None => T::attributes()
        };
        self.attributes = Some(attributes);
    }
}

// Does name really need to be static - can it only live as long as the request?
pub trait WriteConfig {
    type I: Serialize; 

    const COOKIE_NAME: &'static str;

    fn serialize(values: Self::I) -> Value {
        json!(values)
    }
    
    fn attributes<'c>() -> Attributes<'c> {
        Attributes::default()
    }
}

pub trait ReadConfig {
    type G: DeserializeOwned; 

    const COOKIE_NAME: &'static str;
}
