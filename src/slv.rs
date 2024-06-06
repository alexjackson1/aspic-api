use aspic::ICCMA23Serialize;
use crustabri::{
    aa::{AAFramework, ArgumentSet},
    solvers::PreferredSemanticsSolver,
};
use rocket::{response::status::BadRequest, serde::json::Json};

use crate::{bld::_build, Specification};

fn _solve() {
    let labels = vec!["a", "b", "c"];
    let arguments = ArgumentSet::new_with_labels(&labels);
    let mut framework = AAFramework::new_with_argument_set(arguments);
    framework.new_attack(&labels[0], &labels[1]);
}

#[post("/solve", data = "<specification>")]
pub fn solve(specification: Json<Specification>) -> Result<String, BadRequest<String>> {
    let spec = specification.into_inner();
    let (framework, theory) = _build(spec)?;
    Ok(framework.to_iccma23())
}
