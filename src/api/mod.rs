use axum::{
    Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

mod create;
mod delete;
mod kill;
mod list;

use crate::state::StatePtr;

#[derive(Serialize, Deserialize, Debug)]
pub struct VmInput {
    pub vm_name: String,
}

pub fn router(state_ptr: StatePtr) -> Router {
    Router::new()
        .route(
            "/",
            get(list::handler)
                .post(create::handler)
                .delete(delete::handler),
        )
        .route("/kill", post(kill::handler))
        .with_state(state_ptr)
}

#[cfg(test)]
mod tests {
    use super::VmInput;

    #[test]
    fn vm_input_uses_stable_json_field_name() {
        let input = VmInput {
            vm_name: "demo".to_owned(),
        };

        assert_eq!(
            serde_json::to_string(&input).unwrap(),
            r#"{"vm_name":"demo"}"#
        );
    }
}
