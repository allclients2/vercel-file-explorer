use warp::reject::Reject;
use std::fmt;

#[derive(Debug)]
pub struct CustomRejection {
    pub message: String,
}

impl fmt::Display for CustomRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Reject for CustomRejection {}