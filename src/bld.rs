use aspic::{StructuredAF, SystemDescription, Theory};
use rocket::{response::status::BadRequest, serde::json::Json};
use serde::Serialize;

use crate::{Specification, _validate, val::PartialSpecification};

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
            aspic::nom::Err::Error(e) | aspic::nom::Err::Failure(e) => BadRequest(e.to_string()),
            aspic::nom::Err::Incomplete(_) => BadRequest("Incomplete input".to_string()),
        })
    }
}

impl TryFrom<PartialSpecification> for SystemDescription {
    type Error = BadRequest<String>;

    fn try_from(spec: PartialSpecification) -> Result<Self, Self::Error> {
        SystemDescription::parse(
            spec.axioms.as_deref().unwrap_or(""),
            spec.premises.as_deref().unwrap_or(""),
            spec.knowledge_preferences.as_deref().unwrap_or(""),
            spec.inference_rules.as_deref().unwrap_or(""),
            spec.rule_preferences.as_deref().unwrap_or(""),
            spec.contraries.as_deref().unwrap_or(""),
        )
        .map_err(|e| match e {
            aspic::nom::Err::Error(e) | aspic::nom::Err::Failure(e) => BadRequest(e.to_string()),
            aspic::nom::Err::Incomplete(_) => BadRequest("Incomplete input".to_string()),
        })
    }
}

#[derive(Serialize)]
pub struct BuildResponse {
    theory: Theory,
    framework: StructuredAF,
}

pub fn _build(specification: Specification) -> Result<(StructuredAF, Theory), BadRequest<String>> {
    let spec = specification.into();
    let description = _validate(&spec)
        .is_empty()
        .then(|| SystemDescription::try_from(spec))
        .ok_or_else(|| BadRequest("Invalid input".to_string()))??;

    let theory = Theory::try_from(description).map_err(|e| BadRequest(format!("{:?}", e)))?;
    let mut framework = theory
        .generate_arguments()
        .map_err(|e| BadRequest(format!("{:?}", e)))?;
    theory
        .calculate_attack(&mut framework)
        .map_err(|e| BadRequest(format!("{:?}", e)))?;

    Ok((framework, theory))
}

#[post("/build", data = "<specification>")]
pub fn build(
    specification: Json<Specification>,
) -> Result<Json<BuildResponse>, BadRequest<String>> {
    let spec = specification.into_inner();
    let (framework, theory) = _build(spec)?;
    Ok(Json(BuildResponse { framework, theory }))
}
