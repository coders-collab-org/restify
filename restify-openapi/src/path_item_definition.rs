use crate::{paths::Operation, Components};

pub trait PathItemDefinition {
  fn is_visible() -> bool {
    true
  }

  fn operation() -> Operation {
    Default::default()
  }

  fn components() -> Vec<Components> {
    Default::default()
  }
}
