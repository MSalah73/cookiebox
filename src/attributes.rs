use std::borrow::Cow;
use biscotti::{time::Duration, Expiration};
use biscotti::{SameSite, ResponseCookie};

pub struct Attributes<'c>{
    path: Option<Cow<'c,str>>,
    domain: Option<Cow<'c, str>>,
    secure: Option<bool>,
    http_only: Option<bool>, 
    partitioned: Option<bool>,
    same_site: Option<SameSite>,
    max_age: Option<Duration>,
    expires: Option<Expiration>,
}
impl<'c> Attributes<'c> {

    pub fn new() -> Self {
      Attributes {
           path: None,
           http_only: None,
           same_site: None,
           domain: None,
           secure: None,
           partitioned: None,
           max_age:None,
           expires: None,
       }
    }
    #[inline]
    pub fn path<T: Into<Cow<'c, str>>>(mut self, path: T) -> Self {
        self.path = Some(path.into());
        self
    } 
    #[inline]
    pub fn domain<T: Into<Cow<'c, str>>>(mut self, domain: T) -> Self {
        self.domain = Some(domain.into());
        self
    } 
    #[inline]
    pub fn secure<T: Into<Option<bool>>>(mut self, value: T) -> Self {
        self.secure = value.into();
        self
    } 
    #[inline]
    pub fn http_only<T: Into<Option<bool>>>(mut self, value: T) -> Self {
        self.http_only = value.into();
        self
    }
    pub fn same_site<T: Into<Option<SameSite>>>(mut self, value: T) -> Self {
        self.same_site = value.into();
        self
    }
    #[inline]
    pub fn max_age<T: Into<Option<Duration>>>(mut self, value: T) -> Self {
        self.max_age = value.into();
        self
    }
    #[inline]
    pub fn expires<T: Into<Option<Expiration>>>(mut self, value: T) -> Self {
        self.expires = value.into();
        self
    }
    #[inline]
    pub fn partitioned<T: Into<Option<bool>>>(mut self, value: T) -> Self {
        self.partitioned = value.into();
        self
    }
}
impl Default for Attributes<'_> {
   fn default() -> Self {
      Attributes {
           path: Some("/".into()),
           http_only: Some(true),
           same_site: Some(SameSite::Lax),
           domain: None,
           secure: None,
           partitioned: None,
           max_age:None,
           expires: None,
       }
   }
}

pub(crate)  trait AttributesSetter<'c> {
    fn set_attributes(self, attributes: &Attributes<'c>) -> Self;
}

impl<'c> AttributesSetter<'c> for ResponseCookie<'c> {
   fn set_attributes(mut self, attributes: &Attributes<'c>) -> Self {
        if let Some(path) = &attributes.path {
            self = self.set_path(path.clone())
        }
        if let Some(domain) = &attributes.domain {
            self = self.set_domain(domain.clone())
        }
        if let Some(expires) = attributes.expires {
            self = self.set_expires(expires)
        }
        self
            .set_secure(attributes.secure)
            .set_http_only(attributes.http_only)
            .set_same_site(attributes.same_site)
            .set_max_age(attributes.max_age)
            .set_partitioned(attributes.partitioned)
   } 
}
