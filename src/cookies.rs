use std::any::type_name;
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
    use biscotti::{time::{Date, Duration, Month, OffsetDateTime, Time}, Expiration, RequestCookie, ResponseCookie};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use crate::{ReadConfig, WriteConfig, Attributes, Cookie, Storage, SameSite};

    // Cookie types
    pub struct TypeA; 
    pub struct TypeB;
    pub struct TypeC;
    pub struct TypeD;

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    pub struct GetType {
        name: String,
    }

    // Cookie type impl
    impl TypeA {
        const NAME: &'static str = "type_a";
    }
    impl TypeB {
        const NAME: &'static str = "type_b";
    }
    impl TypeC {
        const NAME: &'static str = "type_c";
    }
    impl TypeD {
        const NAME: &'static str = "type_d";
    }
    
    // read and write for type a 
    impl WriteConfig for TypeA {
        type I = GetType;

        const COOKIE_NAME: &'static str = Self::NAME;
    }
    impl ReadConfig for TypeA {
        type G = GetType;

        const COOKIE_NAME: &'static str = Self::NAME;
    }

    // read and write for type b
    impl WriteConfig for TypeB {
        type I = (String, i32);

        const COOKIE_NAME: &'static str = Self::NAME;

        fn serialize(values: Self::I) -> serde_json::Value {
            json!({
                "name": format!("{} is {}", values.0, values.1)
            })            
        }
    }
    impl ReadConfig for TypeB {
        type G = GetType;
        const COOKIE_NAME: &'static str = Self::NAME;
    }

    // read and write for type c
    impl WriteConfig for TypeC {
        type I = GetType;

        const COOKIE_NAME: &'static str = Self::NAME;

        fn attributes<'c>() -> Attributes<'c> {
            let date = Date::from_calendar_date(2024, Month::January,1).unwrap();
            let time = Time::from_hms(0,0,0).unwrap();
            let permanent = OffsetDateTime::new_utc(date, time);

            Attributes::new()
            .path("/some-path")
            .domain("..example.com")
            .same_site(SameSite::Lax)
            .secure(true)
            .http_only(true)
            .partitioned(true)
            .expires(Expiration::from(permanent))
            .max_age(Duration::hours(10))
        }
    }
    impl ReadConfig for TypeC {
        type G = GetType;
        const COOKIE_NAME: &'static str = Self::NAME;
    }

    // read and write for type d
    impl WriteConfig for TypeD {
        type I = GetType;

        const COOKIE_NAME: &'static str = Self::NAME;

        fn attributes<'c>() -> Attributes<'c> {
            Attributes::new().permanent(true)
        }
    }
    impl ReadConfig for TypeD {
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

        let cookie = Cookie::<TypeA>::new(&storage);

        let typed_request_value = cookie.get_all();

        assert_eq!(typed_request_value.is_ok(), true);
        assert_eq!(typed_request_value, Ok(get_type_values));
    }
    #[test]
    fn insert_cookie() {
        // Set up
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_a", r#"{ "name": "some value" }"#);
        let outgoing_cookie_id = outgoing_cookie.id().set_path("/");
        let get_type_value = GetType {name: "some value ".to_string()};

        let cookie = Cookie::<TypeA>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(response_cookie.unwrap().name_value(), ("type_a", r#"{"name":"some value "}"#));
    }
    #[test]
    fn insert_cookie_with_custom_serialize_impl() {
        // Set up
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_b", r#"{ "name": "some value is 32" }"#);
        let outgoing_cookie_id = outgoing_cookie.id().set_path("/");
        let get_type_value = ("some value".to_string(), 32);

        let cookie = Cookie::<TypeB>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(response_cookie.unwrap().name_value(), ("type_b", r#"{"name":"some value is 32"}"#));
    }
    #[test]
    fn insert_cookie_with_custom_attributes() {
        // Set up
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_c", r#"{ "name": "some value" }"#);
        let outgoing_cookie_id = outgoing_cookie.id().set_path("/some-path").set_domain("..example.com");
        let get_type_value = GetType {name: "some value".to_string()};

        // expiration cookie set up 
        let date = Date::from_calendar_date(2024, Month::January,1).unwrap();
        let time = Time::from_hms(0,0,0).unwrap();
        let expiration = OffsetDateTime::new_utc(date, time);

        let cookie = Cookie::<TypeC>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(response_cookie.unwrap().name_value(), ("type_c", r#"{"name":"some value"}"#));
        assert_eq!(response_cookie.unwrap().path(), Some("/some-path"));
        assert_eq!(response_cookie.unwrap().domain(), Some(".example.com"));
        assert_eq!(response_cookie.unwrap().same_site(), Some(SameSite::Lax));
        assert_eq!(response_cookie.unwrap().http_only(), Some(true));
        assert_eq!(response_cookie.unwrap().secure(), Some(true));
        assert_eq!(response_cookie.unwrap().partitioned(), Some(true));
        assert_eq!(response_cookie.unwrap().expires(), Some(Expiration::from(expiration)));
        assert_eq!(response_cookie.unwrap().max_age(), Some(Duration::hours(10)));

    }
    #[test]
    fn insert_cookie_with_permanent() {
        // Set up
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_d", r#"{ "name": "some value" }"#);
        let outgoing_cookie_id = outgoing_cookie.id();
        let get_type_value = GetType {name: "some value".to_string()};

        let cookie = Cookie::<TypeD>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(response_cookie.unwrap().name_value(), ("type_d", r#"{"name":"some value"}"#));
        assert_eq!(response_cookie.unwrap().max_age(), Some(Duration::days(20 * 365)));
    }
    #[test]
    fn remove_cookie() {
        // Set up
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_b", r#"{ "name": "some value is 32" }"#);
        let outgoing_cookie_id = outgoing_cookie.id().set_path("/");

        // removal cookie set up 
        let date = Date::from_calendar_date(1970, Month::January,1).unwrap();
        let time = Time::from_hms(0,0,0).unwrap();
        let removal_date = OffsetDateTime::new_utc(date, time);

        let cookie = Cookie::<TypeB>::new(&storage);

        cookie.remove();

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(response_cookie.unwrap().name_value(), ("type_b", ""));
        assert_eq!(response_cookie.unwrap().expires().unwrap(), Expiration::from(removal_date));
    }

}