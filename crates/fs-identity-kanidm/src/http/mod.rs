//! The HTTP seam: what the client needs from a transport, and the real one.
//!
//! Pattern: Facade (declarations).

mod reqwest_seam;
mod seam;
mod token;

pub use reqwest_seam::ReqwestSeam;
pub use seam::{HttpSeam, Method, Request, Response};
pub use token::Token;
