//! cookiebox's core functionality  
use crate::attributes::{Attributes, AttributesSetter};
use crate::storage::Storage;
use biscotti::{RemovalCookie, ResponseCookie};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::any::type_name;
use thiserror::Error;

/// The error returned by [IncomingConfig] get methods
#[derive(Error, Debug, PartialEq)]
pub enum CookieBoxError {
    #[error("`{0}` does not exist")]
    NotFound(String),
    #[error("Failed to deserialize `{0}` to type `{1}`")]
    Deserialization(String, String),
}

/// Base struct for cookie generic types
pub struct Cookie<'c, T> {
    storage: Storage<'c>,
    attributes: Option<Attributes<'c>>,
    _marker: std::marker::PhantomData<T>,
}

impl<'c, T> Cookie<'c, T> {
    /// Create a cookie instance for any generic type parameter
    pub fn new(storage: &Storage<'c>) -> Self {
        Cookie {
            storage: storage.clone(),
            attributes: None,
            _marker: std::marker::PhantomData,
        }
    }
}
/// Provide methods to `get` data from a cookie instance for any generic type parameter that implements [IncomingConfig]
impl<T: IncomingConfig> Cookie<'_, T> {
    /// Retrieves the date from the [Storage] request collection using the cookie name specified by [CookieName].
    ///
    /// The date is returned as the associated type defined by the `Get` from type [CookieName].
    /// # Example
    /// ```no_run
    /// use cookiebox::cookiebox_macros::cookie;
    /// use cookiebox::cookies::{Cookie, CookieName, IncomingConfig};
    /// use actix_web::HttpResponse;
    /// 
    /// // Set up generic cookie type
    /// #[cookie(name = "my-cookie")]
    /// pub struct MyCookie;
    /// 
    /// impl IncomingConfig for MyCookie {
    ///     type Get = String;
    /// }
    ///  
    /// // Assume implementation of `FromRequest` to create Cookie instances 
    /// pub struct CookieCollection<'c>(Cookie<'c, MyCookie>);
    /// 
    /// async fn get_cookie(cookie: CookieCollection<'_>) -> HttpResponse {
    ///     cookie.0.get();
    ///     HttpResponse::Ok().finish()
    /// }
    /// ```
    pub fn get(&self) -> Result<T::Get, CookieBoxError> {
        let data = &self
            .storage
            .request_storage
            .borrow()
            .get(T::COOKIE_NAME)
            .ok_or(CookieBoxError::NotFound(T::COOKIE_NAME.to_string()))?;
        // Does it panic?  use clone
        // Closure - Implement response error to remove it - i think
        let data = serde_json::from_str(data.value()).map_err(|_| {
            CookieBoxError::Deserialization(
                data.value().to_string(),
                type_name::<T::Get>().to_string(),
            )
        })?;
        Ok(data)
    }

    /// Retrieves a list of data items from the [Storage] request collection with the same name using the cookie name specified by [CookieName].
    ///
    /// Each item in the list is of the associated type `Get` type from [CookieName].
    /// 
    /// # Example
    /// ```no_run
    /// use cookiebox::cookiebox_macros::cookie;
    /// use cookiebox::cookies::{Cookie, CookieName, IncomingConfig};
    /// use actix_web::HttpResponse;
    /// 
    /// // Set up generic cookie type
    /// #[cookie(name = "my-cookie")]
    /// pub struct MyCookie;
    /// 
    /// impl IncomingConfig for MyCookie {
    ///     type Get = String;
    /// }
    ///  
    /// // Assume implementation of `FromRequest` to create Cookie instances 
    /// pub struct CookieCollection<'c>(Cookie<'c, MyCookie>);
    /// 
    /// async fn get_all_cookies(cookie: CookieCollection<'_>) -> HttpResponse {
    ///     // return a Vec of set type
    ///     cookie.0.get_all();
    ///     HttpResponse::Ok().finish()
    /// }
    /// ```
    pub fn get_all(&self) -> Result<Vec<T::Get>, CookieBoxError> {
        let data = &self.storage.request_storage.borrow();

        let data = data
            .get_all(T::COOKIE_NAME)
            .ok_or(CookieBoxError::NotFound(T::COOKIE_NAME.to_string()))?;

        let mut result = Vec::new();

        for value in data.values() {
            let data = serde_json::from_str(value).map_err(|_| {
                CookieBoxError::Deserialization(value.to_string(), type_name::<T::Get>().to_string())
            })?;
            result.push(data);
        }

        Ok(result)
    }
}

/// Provide methods to `insert` and `remove` from a cookie instance for any generic type parameter that implements [OutgoingConfig]
impl<'c, T: OutgoingConfig> Cookie<'c, T> {
    /// Add a cookie to the [Storage] response collection which later attached to an HTTP response using the `Set-Cookie` header.
    ///
    /// # Example
    /// ```no_run
    /// use cookiebox::cookiebox_macros::cookie;
    /// use cookiebox::cookies::{Cookie, CookieName, OutgoingConfig};
    /// use actix_web::HttpResponse;
    /// 
    /// // Set up generic cookie type
    /// #[cookie(name = "my-cookie")]
    /// pub struct MyCookie;
    /// 
    /// impl OutgoingConfig for MyCookie {
    ///     type Insert = String;
    /// }
    ///  
    /// // Assume implementation of `FromRequest` to create Cookie instances 
    /// pub struct CookieCollection<'c>(Cookie<'c, MyCookie>);
    /// 
    /// async fn insert_cookie(cookie: CookieCollection<'_>) -> HttpResponse {
    ///     // The cookie removal id determined by name path and domain
    ///     cookie.0.insert("cookie value".to_string());
    ///     HttpResponse::Ok().finish()
    /// }
    /// ```
    pub fn insert(&self, value: T::Insert) {
        let data = T::serialize(value);

        let response_cookie = ResponseCookie::new(T::COOKIE_NAME, data.to_string());

        let attributes = match &self.attributes {
            Some(attributes) => attributes,
            None => &T::attributes(),
        };

        let response_cookie = response_cookie.set_attributes(attributes);

        self.storage
            .response_storage
            .borrow_mut()
            .insert(response_cookie);
    }
    /// Add a removal cookie to the [Storage] response collection which later attached to an HTTP response using the `Set-Cookie` header.
    ///
    /// Cookie removal is determined by name, path, and domain
    /// 
    /// # Example
    /// ```no_run
    /// use cookiebox::cookiebox_macros::cookie;
    /// use cookiebox::cookies::{Cookie, CookieName, OutgoingConfig};
    /// use actix_web::HttpResponse;
    /// 
    /// // Set up generic cookie type
    /// #[cookie(name = "my-cookie")]
    /// pub struct MyCookie;
    ///
    /// impl OutgoingConfig for MyCookie {
    ///     type Insert = String;
    /// }
    ///  
    /// // Assume implementation of `FromRequest` to create Cookie instances 
    /// pub struct CookieCollection<'c>(Cookie<'c, MyCookie>);
    /// 
    /// async fn remove_cookie(cookie: CookieCollection<'_>) -> HttpResponse {
    ///     // The cookie removal determined by name, path, and domain
    ///     cookie.0.remove();
    ///     HttpResponse::Ok().finish()
    /// }
    /// ```
    pub fn remove(&self) {
        let attributes = match &self.attributes {
            Some(attributes) => attributes,
            None => &T::attributes(),
        };

        let removal_cookie = RemovalCookie::new(T::COOKIE_NAME);

        let removal_cookie = removal_cookie.set_attributes(attributes);

        self.storage
            .response_storage
            .borrow_mut()
            .insert(removal_cookie);
    }
}

/// Provide internal customization for `insert` and `remove` methods in [Cookie].
///
/// The `insert` and `remove` will be available for types used as generic parameters for `Cookie`.
/// ```no_run
/// use cookiebox::cookiebox_macros::cookie;
/// use cookiebox::cookies::{CookieName, OutgoingConfig};
/// 
/// // Define a generic cookie type struct
/// #[cookie(name = "__my-cookie")]
/// pub struct MyCookie;
///
/// impl OutgoingConfig for MyCookie {
///    // Configure the insert type
///    type Insert = String;
///    
///    // The default serialization is used here, if further customization implement the `serialize` method.
///    
///    // The default attributes is used here which consists of http-only: true, SameSite: Lax, and
///    // path: "/"
/// }
/// ```
pub trait OutgoingConfig: CookieName {
    /// The serialization type when inserting a cookie to storage
    type Insert: Serialize;

    /// Provides default serialization for a cookie. This can can be overwriting
    fn serialize(values: Self::Insert) -> Value {
        json!(values)
    }

    /// Provides preset attributes for a cookie. This can can be overwriting
    fn attributes<'c>() -> Attributes<'c> {
        Attributes::default()
    }
}

/// Provide internal customization for `get` and `get_all` methods in [Cookie].
///
/// The `get` and `get_all` will be available for types used as generic parameters for `Cookie`.
/// ```no_run
/// use cookiebox::cookiebox_macros::cookie;
/// use cookiebox::cookies::{CookieName, IncomingConfig};
/// // Define a generic cookie type struct
/// #[cookie(name = "__my-cookie")]
/// pub struct MyCookie;
///
/// impl IncomingConfig for MyCookie {
///     // Configure the get return type
///     type Get = String;
/// }
/// ```
pub trait IncomingConfig: CookieName {
    /// The deserialization type when getting a cookie from storage
    type Get: DeserializeOwned;
}

/// This is the base implementation of a cookie type
///
/// This is either implemented manually or with macro `#[Cookie(name = "...")]`
pub trait CookieName {
    const COOKIE_NAME: &'static str;
}

#[cfg(test)]
mod tests {
    use crate::cookiebox_macros::cookie;
    use crate::cookies::{Cookie, CookieName, IncomingConfig, OutgoingConfig};
    use crate::time::{Date, Duration, Month, OffsetDateTime, Time};
    use crate::{Attributes, Expiration, SameSite, Storage};
    use biscotti::{RequestCookie, ResponseCookie};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

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
            let date = Date::from_calendar_date(2024, Month::January, 1).unwrap();
            let time = Time::from_hms(0, 0, 0).unwrap();
            let date = OffsetDateTime::new_utc(date, time);

            Attributes::new()
                .path("/some-path")
                .domain("..example.com")
                .same_site(SameSite::Lax)
                .secure(true)
                .http_only(true)
                .partitioned(true)
                .expires(Expiration::from(date))
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
        // Initialize storage
        let storage = Storage::new();
        let incoming_cookie = RequestCookie::new("type_a", r#"{ "name": "some value" }"#);
        let get_type_value = GetType {
            name: "some value".to_string(),
        };

        storage.request_storage.borrow_mut().append(incoming_cookie);

        // Use generic type parameter to create a cookie instance
        let cookie = Cookie::<TypeA>::new(&storage);

        let typed_request_value = cookie.get();

        assert_eq!(typed_request_value.is_ok(), true);
        assert_eq!(typed_request_value, Ok(get_type_value));
    }
    #[test]
    fn get_all() {
        // Set up
        // Initialize storage
        let storage = Storage::new();
        let incoming_cookie_a = RequestCookie::new("type_a", r#"{ "name": "some value 1" }"#);
        let incoming_cookie_b = RequestCookie::new("type_a", r#"{ "name": "some value 2" }"#);
        let get_type_values = vec![
            GetType {
                name: "some value 1".to_string(),
            },
            GetType {
                name: "some value 2".to_string(),
            },
        ];

        storage
            .request_storage
            .borrow_mut()
            .append(incoming_cookie_a);
        storage
            .request_storage
            .borrow_mut()
            .append(incoming_cookie_b);

        // Use generic type parameter to create a cookie instance
        let cookie = Cookie::<TypeA>::new(&storage);

        let typed_request_value = cookie.get_all();

        assert_eq!(typed_request_value.is_ok(), true);
        assert_eq!(typed_request_value, Ok(get_type_values));
    }
    #[test]
    fn insert_cookie() {
        // Set up
        // Initialize storage
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_a", r#"{ "name": "some value" }"#);
        // The id determined by name path and domain
        let outgoing_cookie_id = outgoing_cookie.id().set_path("/");
        let get_type_value = GetType {
            name: "some value ".to_string(),
        };

        // Use generic type parameter to create a cookie instance
        let cookie = Cookie::<TypeA>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(
            response_cookie.unwrap().name_value(),
            ("type_a", r#"{"name":"some value "}"#)
        );
    }
    #[test]
    fn insert_cookie_with_custom_serialize_impl() {
        // Set up
        // Initialize storage
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_b", r#"{ "name": "some value is 32" }"#);
        // The id determined by name path and domain
        let outgoing_cookie_id = outgoing_cookie.id().set_path("/");
        let get_type_value = ("some value".to_string(), 32);

        // Use generic type parameter to create a cookie instance
        let cookie = Cookie::<TypeB>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(
            response_cookie.unwrap().name_value(),
            ("type_b", r#"{"name":"some value is 32"}"#)
        );
    }
    #[test]
    fn insert_cookie_with_custom_attributes() {
        // Set up
        // Initialize storage
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_c", r#"{ "name": "some value" }"#);
        // The id determined by name path and domain
        let outgoing_cookie_id = outgoing_cookie
            .id()
            .set_path("/some-path")
            .set_domain("..example.com");
        let get_type_value = GetType {
            name: "some value".to_string(),
        };

        // expiration cookie set up
        let date = Date::from_calendar_date(2024, Month::January, 1).unwrap();
        let time = Time::from_hms(0, 0, 0).unwrap();
        let expiration = OffsetDateTime::new_utc(date, time);

        // Use generic type parameter to create a cookie instance
        let cookie = Cookie::<TypeC>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(
            response_cookie.unwrap().name_value(),
            ("type_c", r#"{"name":"some value"}"#)
        );
        assert_eq!(response_cookie.unwrap().path(), Some("/some-path"));
        assert_eq!(response_cookie.unwrap().domain(), Some(".example.com"));
        assert_eq!(response_cookie.unwrap().same_site(), Some(SameSite::Lax));
        assert_eq!(response_cookie.unwrap().http_only(), Some(true));
        assert_eq!(response_cookie.unwrap().secure(), Some(true));
        assert_eq!(response_cookie.unwrap().partitioned(), Some(true));
        assert_eq!(
            response_cookie.unwrap().expires(),
            Some(Expiration::from(expiration))
        );
        assert_eq!(
            response_cookie.unwrap().max_age(),
            Some(Duration::hours(10))
        );
    }
    #[test]
    fn insert_cookie_with_permanent() {
        // Set up
        // Initialize storage
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_d", r#"{ "name": "some value" }"#);
        // The id determined by name path and domain
        let outgoing_cookie_id = outgoing_cookie.id();
        let get_type_value = GetType {
            name: "some value".to_string(),
        };

        // Use generic type parameter to create a cookie instance
        let cookie = Cookie::<TypeD>::new(&storage);

        cookie.insert(get_type_value);

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(
            response_cookie.unwrap().name_value(),
            ("type_d", r#"{"name":"some value"}"#)
        );
        assert_eq!(
            response_cookie.unwrap().max_age(),
            Some(Duration::days(20 * 365))
        );
    }
    #[test]
    fn remove_cookie() {
        // Set up
        // Initialize storage
        let storage = Storage::new();
        let outgoing_cookie = ResponseCookie::new("type_b", r#"{ "name": "some value is 32" }"#);
        // The id determined by name path and domain
        let outgoing_cookie_id = outgoing_cookie.id().set_path("/");

        // removal cookie set up
        let date = Date::from_calendar_date(1970, Month::January, 1).unwrap();
        let time = Time::from_hms(0, 0, 0).unwrap();
        let removal_date = OffsetDateTime::new_utc(date, time);

        // Use generic type parameter to create a cookie instance
        let cookie = Cookie::<TypeB>::new(&storage);

        cookie.remove();

        let binding = storage.response_storage.borrow();
        let response_cookie = binding.get(outgoing_cookie_id);

        assert_eq!(response_cookie.is_some(), true);
        assert_eq!(response_cookie.unwrap().name_value(), ("type_b", ""));
        assert_eq!(
            response_cookie.unwrap().expires().unwrap(),
            Expiration::from(removal_date)
        );
    }
}
