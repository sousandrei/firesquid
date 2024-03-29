use serde::{Deserialize, Serialize};
use warp::Filter;

mod create;
mod delete;
mod kill;
mod list;

use crate::state::StatePtr;

#[derive(Serialize, Deserialize, Debug)]
pub struct VmInput {
    pub vm_name: String,
}

fn with_state(
    state_ptr: StatePtr,
) -> impl Filter<Extract = (StatePtr,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state_ptr.clone())
}

pub fn router(
    state_ptr: StatePtr,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    // 404
    let not_found = warp::path::end().map(|| "Hello, World at root!");

    let route_create = warp::path::end()
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state_ptr.clone()))
        .and_then(create::handler);

    let route_delete = warp::path::end()
        .and(warp::delete())
        .and(warp::body::json())
        .and(with_state(state_ptr.clone()))
        .and_then(delete::handler);

    let route_kill = warp::path!("kill")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state_ptr.clone()))
        .and_then(kill::handler);

    let route_list = warp::path::end()
        .and(warp::get())
        .and(with_state(state_ptr))
        .and_then(list::handler);

    // routes
    let routes = route_create
        .or(route_delete)
        .or(route_kill)
        .or(route_list)
        .or(not_found);

    routes.with(warp::log("firesquid::api"))
}
