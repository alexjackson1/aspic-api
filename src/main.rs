#[macro_use]
extern crate rocket;

use rocket::serde::Deserialize;
use rocket_okapi::okapi::schemars;
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi_get_routes, swagger_ui::*};

mod bld;
mod slv;
mod val;

use bld::build;
use slv::solve;
use val::{_validate, okapi_add_operation_for_validate_, validate};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Specification {
    axioms: String,
    premises: String,
    inference_rules: String,
    contraries: String,
    rule_preferences: String,
    knowledge_preferences: String,
}

fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/v1/openapi.json".to_string(),
        urls: vec![UrlObject::new("v1", "/v1/openapi.json")],
        ..Default::default()
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/v1", openapi_get_routes![validate])
        .mount("/docs", make_swagger_ui(&get_docs()))
        .mount("/", routes![validate, build, solve])
}
