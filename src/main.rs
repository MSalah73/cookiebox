
use bakery_macros::cookie;
use bakery::cookies::CookieName;

fn main () {
    //#[cookie(name = "ABC", private)]
    #[cookie(name = "ABC")]
    struct A{}
    print!("From main -- {}", A::COOKIE_NAME)
}