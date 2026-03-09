mod endpoints;
mod configurator;
pub mod middleware;
pub mod response;

pub use configurator::configure_routes;
pub use response::{ApiResponse, Error};
