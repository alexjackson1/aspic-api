#[macro_use]
extern crate rocket;

use aspic::parse::rule_preferences;
use aspic::{ICCMA23Serialize, StructuredAF, Theory};
use rocket::response::status::BadRequest;
use rocket::serde::{json::Json, Deserialize};
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{openapi, openapi_get_routes, swagger_ui::*};
use serde::Serialize;

use aspic::nom::Err;

use aspic::{
    parse::{contraries, formula_set, inference_rules},
    SystemDescription,
};

#[derive(Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct ValidationRequest {
    axioms: Option<String>,
    premises: Option<String>,
    inference_rules: Option<String>,
    contraries: Option<String>,
    rule_preferences: Option<String>,
    knowledge_preferences: Option<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct ValidationErrors {
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    axioms: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    premises: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inference_rules: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contraries: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferences: Option<String>,
}

fn _validate(data: &Json<ValidationRequest>) -> ValidationErrors {
    let mut errors = ValidationErrors {
        valid: true,
        axioms: None,
        premises: None,
        inference_rules: None,
        contraries: None,
        preferences: None,
    };

    if let Some(axioms) = &data.axioms {
        if formula_set(axioms).is_err() {
            errors.valid = false;
            errors.axioms = Some("Invalid formula set".to_string());
        }
    }

    if let Some(premises) = &data.premises {
        if formula_set(premises).is_err() {
            errors.valid = false;
            errors.premises = Some("Invalid formula set".to_string());
        }
    }

    if let Some(ir) = &data.inference_rules {
        if inference_rules(ir).is_err() {
            errors.valid = false;
            errors.inference_rules = Some("Invalid inference rules".to_string());
        }
    }

    if let Some(c) = &data.contraries {
        if contraries(c).is_err() {
            errors.valid = false;
            errors.contraries = Some("Invalid contraries".to_string());
        }
    }

    if let Some(rp) = &data.rule_preferences {
        if rule_preferences(&rp).is_err() {
            errors.valid = false;
            errors.preferences = Some("Invalid rule preferences".to_string());
        }
    }

    if let Some(kp) = &data.knowledge_preferences {
        if rule_preferences(&kp).is_err() {
            errors.valid = false;
            errors.preferences = Some("Invalid knowledge preferences".to_string());
        }
    }

    errors
}

#[openapi]
#[post("/validate", data = "<validate_payload>")]
fn validate(validate_payload: Json<ValidationRequest>) -> Json<ValidationErrors> {
    Json(_validate(&validate_payload))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct BuildTheoryResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theory: Option<aspic::Theory>,
}

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

impl TryFrom<Specification> for SystemDescription {
    type Error = BadRequest<String>;

    fn try_from(spec: Specification) -> Result<Self, Self::Error> {
        SystemDescription::parse(
            &spec.axioms,
            &spec.premises,
            &spec.knowledge_preferences,
            &spec.inference_rules,
            &spec.rule_preferences,
            &spec.contraries,
        )
        .map_err(|e| match e {
            Err::Error(e) | Err::Failure(e) => {
                use aspic::nom::error::VerboseErrorKind::*;
                let msg = e
                    .errors
                    .iter()
                    .map(|(s, err_kind)| match err_kind {
                        Context(e) => format!("{}: {}", s, e),
                        Nom(e) => format!("{}: {:?}", s, e),
                        _ => format!("{}: {:?}", s, err_kind),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                BadRequest(msg)
            }
            Err::Incomplete(_) => BadRequest("Incomplete input".to_string()),
        })
    }
}

#[post("/build", data = "<spec>")]
fn build(spec: Json<Specification>) -> Result<Json<Theory>, BadRequest<String>> {
    let description = SystemDescription::try_from(spec.0)?;
    match Theory::try_from(description) {
        Ok(theory) => Ok(Json(theory)),
        Err(e) => Err(BadRequest(format!("{:?}", e))),
    }
}

#[derive(Serialize)]
struct GenerateResponse {
    framework: StructuredAF,
    theory: Theory,
}

#[post("/generate", data = "<spec>")]
fn generate(spec: Json<Specification>) -> Result<Json<GenerateResponse>, BadRequest<String>> {
    let description = SystemDescription::try_from(spec.0)?;
    let theory = match Theory::try_from(description) {
        Ok(theory) => Ok(theory),
        Err(e) => Err(BadRequest(format!("{:?}", e))),
    }?;
    let mut framework = theory
        .generate_arguments()
        .map_err(|e| BadRequest(format!("{:?}", e)))?;
    theory
        .calculate_attack(&mut framework)
        .map_err(|e| BadRequest(format!("{:?}", e)))?;

    Ok(Json(GenerateResponse { framework, theory }))
}

#[post("/solve", data = "<spec>")]
fn solve(spec: Json<Specification>) -> Result<String, BadRequest<String>> {
    let description = SystemDescription::try_from(spec.0)?;
    let theory = match Theory::try_from(description) {
        Ok(theory) => Ok(theory),
        Err(e) => Err(BadRequest(format!("{:?}", e))),
    }?;
    let mut framework = theory
        .generate_arguments()
        .map_err(|e| BadRequest(format!("{:?}", e)))?;
    theory
        .calculate_attack(&mut framework)
        .map_err(|e| BadRequest(format!("{:?}", e)))?;

    Ok(framework.to_iccma23())
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
        .mount("/", routes![validate, build, generate, solve])
}
