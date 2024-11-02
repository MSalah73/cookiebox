use serde::{Deserialize, Serialize};
use serde_json::json;
use bakery::{ReadConfig, WriteConfig, Attributes, Cookie, Storage, SameSite};



//
//
// TEST
//
//
pub struct TypeA; 
pub struct TypeB;
pub struct TypeC;

#[derive(Deserialize, Serialize, Debug)]
pub struct GetType {
    name: String,
}
impl TypeA {
    const NAME: &'static str = "Predefined Static Name A";
}
impl TypeB {
    const NAME: &'static str = "Predefined Static Name B";
}
impl TypeC {
    const NAME: &'static str = "Predefined Static Name C";
}
impl WriteConfig for TypeA {
    type I = GetType;

    const COOKIE_NAME: &'static str = Self::NAME;
    
    fn serialize(value: Self::I) -> serde_json::Value {
        json!(value)
    }
    fn attributes<'c>() -> Attributes<'c> {
        Attributes::new()
            .path("NEW PATH")
            .secure(false)
            .same_site(SameSite::Lax)
    }
}
impl WriteConfig for TypeB {
    type I = String;

    const COOKIE_NAME: &'static str = Self::NAME;

    fn attributes<'c>() -> Attributes<'c> {
        Attributes::default().path("")
    }
}
impl ReadConfig for TypeA {
    type G = GetType;
    const COOKIE_NAME: &'static str = Self::NAME;
}

impl ReadConfig for TypeC {
    type G = String;
    const COOKIE_NAME: &'static str = Self::NAME;
}
fn main() {
    let storage = Storage::new();
    let cookieA = Cookie::<TypeA>::new(&storage);
    let mut cookieB = Cookie::<TypeB>::new(&storage);
    let cookieC = Cookie::<TypeC>::new(&storage);
    cookieB.set_path("/custom-path");

    cookieA.insert(GetType{ name: "GetTypeStruct - member name's value".to_string()});
    cookieB.insert("Value for cookie b".to_string());
    let a_data = cookieA.get().map_err(|e| print!("{}\n",e));
    cookieB.insert("Value for cookie b - the second time".to_string());
    print!("{:?}",a_data);
}
