use std::any::type_name;
use biscotti::{RemovalCookie, ResponseCookie};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;
use crate::attributes::{Attributes, AttributesSetter};
use crate::storage::Storage;
use bakery_macros::cookie;

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

impl<T: IncomingConfig> Cookie<'_, T> {
    pub fn get(&self) -> Result<T::Get, BakeryError>
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
            .map_err(|_| BakeryError::Deserialization(data.value().to_string(), type_name::<T::Get>().to_string()))?;
        Ok(data)
    }

    pub fn get_all(&self) -> Result<Vec<T::Get>, BakeryError>
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
                .map_err(|_| BakeryError::Deserialization(value.to_string(), type_name::<T::Get>().to_string()))?;
            result.push(data);
        }

        Ok(result)
    }
}
impl<'c,T: OutgoingConfig> Cookie<'c, T> {
    pub fn insert(&self, value: T::Insert)
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
pub trait OutgoingConfig: CookieName {
    type Insert: Serialize; 

    fn serialize(values: Self::Insert) -> Value {
        json!(values)
    }
    
    fn attributes<'c>() -> Attributes<'c> {
        Attributes::default()
    }
}

pub trait IncomingConfig: CookieName{
    type Get: DeserializeOwned; 
}

pub trait CookieName {
    const COOKIE_NAME: &'static str;
}

#[cfg(test)]
mod tests {
    use biscotti::{time::{Date, Duration, Month, OffsetDateTime, Time}, Expiration, RequestCookie, ResponseCookie};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use crate::{IncomingConfig, OutgoingConfig, Attributes, Cookie, Storage, SameSite};
    use bakery_macros::cookie;
    use crate::cookies::CookieName;
    
    // Cookie types
    #[cookie(name = "type_a")]
    pub struct TypeA; 
    #[cookie(name = "type_b")]
    pub struct TypeB;
    #[cookie(name = "type_c")]
    pub struct TypeC;
    #[cookie(name = "type_d")]
    pub struct TypeD;

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    pub struct GetType {
        name: String,
    }

    // read and write for type a 
    impl OutgoingConfig for TypeA {
        type Insert = GetType;
    }
    impl IncomingConfig for TypeA {
        type Get = GetType;
    }

    // read and write for type b
    impl OutgoingConfig for TypeB {
        type Insert = (String, i32);

        fn serialize(values: Self::Insert) -> serde_json::Value {
            json!({
                "name": format!("{} is {}", values.0, values.1)
            })            
        }
    }
    impl IncomingConfig for TypeB {
        type Get = GetType;
    }

    // read and write for type c
    impl OutgoingConfig for TypeC {
        type Insert = GetType;

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
    impl IncomingConfig for TypeC {
        type Get = GetType;
    }

    // read and write for type d
    impl OutgoingConfig for TypeD {
        type Insert = GetType;

        fn attributes<'c>() -> Attributes<'c> {
            Attributes::new().permanent(true)
        }
    }
    impl IncomingConfig for TypeD {
        type Get = GetType;
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