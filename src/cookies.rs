use std::any::type_name;
use std::borrow::Cow;
use biscotti::{RemovalCookie, ResponseCookie};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;
use crate::attributes::{Attributes, AttributesSetter};
use crate::storage::Storage;

#[derive(Error,Debug, PartialEq)]
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

    pub fn get_all(&self) -> Result<Vec<T::G>, BakeryError>
    {
        let data = &self
            .storage
            .request_storage
            .borrow();

        let data = data.get_all(T::COOKIE_NAME)
            .ok_or(BakeryError::NotFound(T::COOKIE_NAME.to_string()))?;

        let mut result = Vec::new();

        for value in data.values(){
            let data  = serde_json::from_str(value)
                .map_err(|_| BakeryError::Deserialization(value.to_string(), type_name::<T::G>().to_string()))?;
            result.push(data);
        }

        Ok(result)
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
        let attributes = match &self.attributes {
            Some(attributes) => attributes,
            None => &T::attributes()
        };

        let removal_cookie = RemovalCookie::new(T::COOKIE_NAME);

        let removal_cookie = removal_cookie.set_attributes(attributes);

        self.storage.response_storage.borrow_mut().insert(removal_cookie); 
    }
    // Some time you might want to set dynamic path and domain
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

#[cfg(test)]
mod tests {
    use biscotti::{RequestCookie, ResponseCookie};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use crate::{ReadConfig, WriteConfig, Attributes, Cookie, Storage, SameSite};

    pub struct TypeA; 
    pub struct TypeB;
    pub struct TypeC;

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    pub struct GetType {
        name: String,
    }
    impl TypeA {
        const NAME: &'static str = "type_a";
    }
    impl TypeB {
        const NAME: &'static str = "type_b";
    }
    impl TypeC {
        const NAME: &'static str = "type_c";
    }
    impl WriteConfig for TypeA {
        type I = GetType;

        const COOKIE_NAME: &'static str = Self::NAME;
    }
    impl ReadConfig for TypeA {
        type G = GetType;
        const COOKIE_NAME: &'static str = Self::NAME;
    }

    #[test]
    fn get() {
        // Set up
        let storage = Storage::new();
        let incoming_cookie = RequestCookie::new("type_a", r#"{ "name": "some value" }"#);
        let get_type_value = GetType {name: "some value".to_string()};

        storage.request_storage.borrow_mut().append(incoming_cookie);
        let cookie_a = Cookie::<TypeA>::new(&storage);

        let typed_request_value = cookie_a.get();
        assert_eq!(typed_request_value.is_ok(), true);
        assert_eq!(typed_request_value, Ok(get_type_value));
    }
    #[test]
    fn get_all() {
        // Set up
        let storage = Storage::new();
        let incoming_cookie_a = RequestCookie::new("type_a", r#"{ "name": "some value 1" }"#);
        let incoming_cookie_b = RequestCookie::new("type_a", r#"{ "name": "some value 2" }"#);
        let get_type_values = vec![GetType {name: "some value 1".to_string()}, GetType {name: "some value 2".to_string()}];

        storage.request_storage.borrow_mut().append(incoming_cookie_a);
        storage.request_storage.borrow_mut().append(incoming_cookie_b);
        let cookie_a = Cookie::<TypeA>::new(&storage);

        let typed_request_value = cookie_a.get_all();
        assert_eq!(typed_request_value.is_ok(), true);
        assert_eq!(typed_request_value, Ok(get_type_values));
    }
    #[test]
    fn insert_cookie() {
        // Set up
        let storage = Storage::new();
        let outgoing_cookie_a = ResponseCookie::new("type_a", r#"{ "name": "some value" }"#);
        let outgoing_cookie_id_a = outgoing_cookie_a.id().set_path("/");
        let get_type_value = GetType {name: "some value ".to_string()};

        let cookie_a = Cookie::<TypeA>::new(&storage);

        cookie_a.insert(get_type_value);
        let binding = storage.response_storage.borrow();
        let cookie = binding.get(outgoing_cookie_id_a);

        assert_eq!(cookie.is_some(), true);
        assert_eq!(cookie.unwrap().name_value(), ("type_a", r#"{"name":"some value "}"#));
    }
}