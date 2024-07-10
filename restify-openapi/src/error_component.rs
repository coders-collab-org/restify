use crate::paths::Response;
use crate::reference_or::ReferenceOr;
use crate::Schema;
use std::collections::BTreeMap;

pub trait ApiErrorComponent {
  fn schemas_by_status_code() -> BTreeMap<String, (String, ReferenceOr<Schema>)>;
  fn error_responses() -> Vec<(String, Response)>;
}
