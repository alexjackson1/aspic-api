use crate::{schemars, Specification};
use rocket::{
    response::status::BadRequest,
    serde::{self, json::Json},
};
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};

use aspic::parse;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct PartialSpecification {
    pub axioms: Option<String>,
    pub premises: Option<String>,
    pub inference_rules: Option<String>,
    pub contraries: Option<String>,
    pub rule_preferences: Option<String>,
    pub knowledge_preferences: Option<String>,
}

impl PartialSpecification {
    pub fn is_empty(&self) -> bool {
        self.axioms.is_none()
            && self.premises.is_none()
            && self.inference_rules.is_none()
            && self.contraries.is_none()
            && self.rule_preferences.is_none()
            && self.knowledge_preferences.is_none()
    }
}

impl From<Specification> for PartialSpecification {
    fn from(spec: Specification) -> Self {
        PartialSpecification {
            axioms: Some(spec.axioms),
            premises: Some(spec.premises),
            inference_rules: Some(spec.inference_rules),
            contraries: Some(spec.contraries),
            rule_preferences: Some(spec.rule_preferences),
            knowledge_preferences: Some(spec.knowledge_preferences),
        }
    }
}

fn conditionally_validate<'a, T>(
    x: Option<&'a String>,
    g: impl FnOnce(&'a str) -> parse::ParsingResult<'a, T>,
) -> Option<String> {
    x.map(|y| {
        g(y).err().map(|e| match e {
            aspic::nom::Err::Error(ve) => ve.to_string(),
            aspic::nom::Err::Failure(ve) => ve.to_string(),
            aspic::nom::Err::Incomplete(_) => "Error processing formula set".to_string(),
        })
    })?
}

pub fn _validate(specification: &PartialSpecification) -> PartialSpecification {
    let mut errors = PartialSpecification {
        axioms: None,
        premises: None,
        inference_rules: None,
        contraries: None,
        rule_preferences: None,
        knowledge_preferences: None,
    };

    errors.axioms = conditionally_validate(specification.axioms.as_ref(), parse::formula_set);
    errors.premises = conditionally_validate(specification.premises.as_ref(), parse::formula_set);
    errors.inference_rules = conditionally_validate(
        specification.inference_rules.as_ref(),
        parse::inference_rules,
    );
    errors.contraries =
        conditionally_validate(specification.contraries.as_ref(), parse::contraries);
    errors.rule_preferences = conditionally_validate(
        specification.rule_preferences.as_ref(),
        parse::rule_preferences,
    );
    errors.knowledge_preferences = conditionally_validate(
        specification.knowledge_preferences.as_ref(),
        parse::knowledge_preferences,
    );

    errors
}

#[openapi]
#[post("/validate", data = "<specification>")]
pub fn validate(
    specification: Json<PartialSpecification>,
) -> Result<(), BadRequest<Json<PartialSpecification>>> {
    let spec = specification.into_inner();
    let errors = _validate(&spec);
    errors
        .is_empty()
        .then(|| ())
        .ok_or_else(|| BadRequest(Json(errors)))
}
