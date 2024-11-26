use biscotti::{time::Duration, Expiration};
use biscotti::{RemovalCookie, ResponseCookie, SameSite};
use std::borrow::Cow;

/// Simple builder for cookie attributes
///
/// [Attributes] acts as a facade to [ResponseCookie](https://docs.rs/biscotti/latest/biscotti/struct.ResponseCookie.html) and [RemovalCoolie](https://docs.rs/biscotti/latest/biscotti/struct.RemovalCookie.html)
///
/// ```no_run
/// use cookiebox::cookiebox_macros::cookie;
/// use cookiebox::cookies::{CookieName, OutgoingConfig};
/// use cookiebox::{Attributes, SameSite, Expiration};
/// use cookiebox::time::{Date, Duration, Month, OffsetDateTime, Time};
///
/// #[cookie(name = "my-cookie")]
/// pub struct MyCookie;
///
/// impl OutgoingConfig for MyCookie {
///     type Insert = String;
///
///     fn attributes<'c>() -> Attributes<'c> {
///         let date = Date::from_calendar_date(2024, Month::January, 1).unwrap();
///         let time = Time::from_hms(0, 0, 0).unwrap();
///         let date = OffsetDateTime::new_utc(date, time);
///
///         Attributes::new()
///             .path("/some-path")
///             // the leading dot is stripped
///             .domain("..example.com")
///             .same_site(SameSite::Lax)
///             .secure(true)
///             .http_only(true)
///             .partitioned(true)
///             .expires(Expiration::from(date))
///             // max_age take precedence over expires
///             .max_age(Duration::hours(10))
///             // This sets max_age and expires to 20 years in the future
///             .permanent(true)
///     }
/// }
/// ```
pub struct Attributes<'c> {
    path: Option<Cow<'c, str>>,
    domain: Option<Cow<'c, str>>,
    secure: Option<bool>,
    http_only: Option<bool>,
    partitioned: Option<bool>,
    same_site: Option<SameSite>,
    max_age: Option<Duration>,
    expires: Option<Expiration>,
    permanent: bool,
}
impl<'c> Attributes<'c> {
    /// Create a new [Attributes] instance
    pub fn new() -> Self {
        Attributes {
            path: None,
            http_only: None,
            same_site: None,
            domain: None,
            secure: None,
            partitioned: None,
            max_age: None,
            expires: None,
            permanent: false,
        }
    }
    /// Sets the `path` of `self` to `path`
    #[inline]
    pub fn path<T: Into<Cow<'c, str>>>(mut self, path: T) -> Self {
        self.path = Some(path.into());
        self
    }
    /// Sets the `domain` of `self` to `domain`
    ///
    /// **Note**: if the Domain starts with a leading `.`, the leading `.` is stripped.
    #[inline]
    pub fn domain<T: Into<Cow<'c, str>>>(mut self, domain: T) -> Self {
        self.domain = Some(domain.into());
        self
    }
    /// Sets the `secure` of `self` to `value`
    #[inline]
    pub fn secure<T: Into<Option<bool>>>(mut self, value: T) -> Self {
        self.secure = value.into();
        self
    }
    /// Sets the `http_only` of `self` to `value`
    #[inline]
    pub fn http_only<T: Into<Option<bool>>>(mut self, value: T) -> Self {
        self.http_only = value.into();
        self
    }
    /// Sets the `same_site` of `self` to `value`
    ///
    /// **Note**: If `SameSite` attribute is set to `None`, the `Secure` flag will be set automatically , unless explicitly set to `false`.
    pub fn same_site<T: Into<Option<SameSite>>>(mut self, value: T) -> Self {
        self.same_site = value.into();
        self
    }
    /// Sets the `max_age` of `self` to `value`
    #[inline]
    pub fn max_age<T: Into<Option<Duration>>>(mut self, value: T) -> Self {
        self.max_age = value.into();
        self
    }
    /// Sets the `expires` of `self` to `value`
    #[inline]
    pub fn expires<T: Into<Option<Expiration>>>(mut self, value: T) -> Self {
        self.expires = value.into();
        self
    }
    /// Sets the `partitioned` of `self` to `value`
    ///
    /// **Note**: Partitioned cookies require the `Secure` attribute. If not set explicitly, the browser will automatically set it to `true`.

    #[inline]
    pub fn partitioned<T: Into<Option<bool>>>(mut self, value: T) -> Self {
        self.partitioned = value.into();
        self
    }
    /// Sets the `permanent` of `self` to `value`
    #[inline]
    pub fn permanent(mut self, value: bool) -> Self {
        self.permanent = value;
        self
    }
}
/// Create [Attributes] with default values - `path: "/"`,  `SameSite: Lax`, and `http_only: true`
impl Default for Attributes<'_> {
    fn default() -> Self {
        Attributes {
            path: Some("/".into()),
            http_only: Some(true),
            same_site: Some(SameSite::Lax),
            domain: None,
            secure: None,
            partitioned: None,
            max_age: None,
            expires: None,
            permanent: false,
        }
    }
}

pub(crate) trait AttributesSetter<'c> {
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

        if attributes.permanent {
            self = self.make_permanent()
        } else {
            self = self.set_max_age(attributes.max_age);

            if let Some(expires) = attributes.expires {
                self = self.set_expires(expires)
            }
        }

        self.set_secure(attributes.secure)
            .set_http_only(attributes.http_only)
            .set_same_site(attributes.same_site)
            .set_partitioned(attributes.partitioned)
    }
}

impl<'c> AttributesSetter<'c> for RemovalCookie<'c> {
    fn set_attributes(mut self, attributes: &Attributes<'c>) -> Self {
        if let Some(path) = &attributes.path {
            self = self.set_path(path.clone())
        }
        if let Some(domain) = &attributes.domain {
            self = self.set_domain(domain.clone())
        }
        self
    }
}
