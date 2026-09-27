mod error;
pub mod limits;
mod protocol;
mod subject;
mod work;

pub use error::ApiError;
pub use protocol::{Answer, AnswerOption, DiscoveryCategory, Protocol, Question};
pub use subject::{SubjectRef, FIELDS};
pub use work::{
    ClaimOutcome, ClaimPosition, ClassificationReceipt, ClassificationSubmission, DiscoveryFlag,
    ReviewAssignment, ReviewReceipt, ReviewSubmission, Task, Vote,
};

pub fn build_version(canister: &str) -> String {
    format!("{canister} {}", env!("CARGO_PKG_VERSION"))
}
