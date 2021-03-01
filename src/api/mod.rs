use hyper::{Body, Method, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};

mod create;
mod delete;
mod kill;
mod list;

use crate::state::{StatePtr, Vm};

#[derive(Serialize, Deserialize, Debug)]
pub struct VmInput {
    pub vm_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VmListOuput {
    vms: Vec<Vm>,
}

pub async fn router(
    req: Request<Body>,
    state_ptr: StatePtr,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => create::handler(req, state_ptr).await,
        (&Method::DELETE, "/") => delete::handler(req, state_ptr).await,
        (&Method::POST, "/kill") => kill::handler(req, state_ptr).await,
        (&Method::GET, "/") => list::handler(state_ptr).await,
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn build_response(status: StatusCode, body: String) -> hyper::Response<hyper::Body> {
    match Response::builder()
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .status(status)
        .body(Body::from(body))
    {
        Ok(r) => r,
        Err(e) => Response::builder()
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!(
                "Error forming response [{}]",
                e.to_string()
            )))
            .unwrap_or_default(),
    }
}
