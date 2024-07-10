use indexmap::IndexMap;

use crate::{
  paths::{Operation, OperationType, PathItem},
  Components,
};

pub trait DefinitionHolder {
  fn path(&self) -> &str;
  fn operations(&mut self) -> IndexMap<OperationType, Operation>;
  fn components(&mut self) -> Vec<Components>;
  fn update_path_items(&mut self, path_op_map: &mut IndexMap<String, PathItem>) {
    let ops = self.operations();
    if !ops.is_empty() {
      let op_map = path_op_map.entry(self.path().into()).or_default();
      op_map.operations.extend(ops);
    }
  }
}
