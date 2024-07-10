use crate::paths::{MediaType, Parameter, RequestBody, Response, Responses};
use crate::reference_or::ReferenceOr;
use crate::security::SecurityScheme;
use crate::ApiErrorComponent;
use crate::Schema;

use schemars::schema::{ArrayValidation, InstanceType, SchemaObject, SingleOrVec};
use std::collections::BTreeMap;

pub trait ApiComponent {
  fn content_type() -> String {
    "application/json".to_string()
  }

  fn required() -> bool {
    true
  }

  /// Contains children schemas for this operation
  /// Each child can also contain child schemas
  fn child_schemas() -> Vec<(String, ReferenceOr<Schema>)>;

  fn raw_schema() -> Option<ReferenceOr<Schema>> {
    None
  }

  fn schema() -> Option<(String, ReferenceOr<Schema>)>;

  fn securities() -> BTreeMap<String, SecurityScheme> {
    Default::default()
  }

  fn security_requirement_name() -> Option<String> {
    None
  }

  fn request_body() -> Option<RequestBody> {
    Self::schema().map(|(name, _)| RequestBody {
      content: BTreeMap::from_iter(vec![(
        Self::content_type(),
        MediaType {
          schema: Some(ReferenceOr::Reference {
            _ref: format!("#/components/schemas/{}", name),
          }),
          ..Default::default()
        },
      )]),
      required: Some(Self::required()),
      ..Default::default()
    })
  }

  fn error_responses() -> Vec<(String, Response)> {
    vec![]
  }

  fn error_schemas() -> BTreeMap<String, (String, ReferenceOr<Schema>)> {
    BTreeMap::default()
  }

  fn responses(_content_type: Option<String>) -> Option<Responses> {
    None
  }

  fn parameters() -> Vec<Parameter> {
    vec![]
  }
}

impl<T> ApiComponent for Option<T>
where
  T: ApiComponent,
{
  fn required() -> bool {
    false
  }

  fn child_schemas() -> Vec<(String, ReferenceOr<Schema>)> {
    T::child_schemas()
  }

  fn raw_schema() -> Option<ReferenceOr<Schema>> {
    T::raw_schema()
  }

  fn schema() -> Option<(String, ReferenceOr<Schema>)> {
    T::schema()
  }

  fn securities() -> BTreeMap<String, SecurityScheme> {
    T::securities()
  }

  fn security_requirement_name() -> Option<String> {
    T::security_requirement_name()
  }
}

impl<T> ApiComponent for Vec<T>
where
  T: ApiComponent,
{
  fn required() -> bool {
    true
  }

  fn child_schemas() -> Vec<(String, ReferenceOr<Schema>)> {
    let mut schemas = T::schema()
      .into_iter()
      .collect::<Vec<(String, ReferenceOr<Schema>)>>();
    schemas.append(&mut T::child_schemas());
    schemas
  }

  fn raw_schema() -> Option<ReferenceOr<Schema>> {
    T::raw_schema()
  }

  fn schema() -> Option<(String, ReferenceOr<Schema>)> {
    T::schema().map(|(name, schema)| {
      let _ref = match schema {
        ReferenceOr::Reference { _ref } => _ref,
        ReferenceOr::Object(_) => format!("#/components/schemas/{}", name),
      };

      (
        name,
        ReferenceOr::Object(Schema::Object(SchemaObject {
          instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Array))),
          array: Some(Box::new(ArrayValidation {
            items: Some(Schema::new_ref(_ref).into()),
            ..Default::default()
          })),
          ..Default::default()
        })),
      )
    })
  }
}

impl<T, E> ApiComponent for Result<T, E>
where
  T: ApiComponent,
  E: ApiErrorComponent,
{
  fn required() -> bool {
    T::required()
  }

  fn child_schemas() -> Vec<(String, ReferenceOr<Schema>)> {
    T::child_schemas()
  }

  fn raw_schema() -> Option<ReferenceOr<Schema>> {
    T::raw_schema()
  }

  fn schema() -> Option<(String, ReferenceOr<Schema>)> {
    T::schema()
  }

  // We expect error to be present only for response part
  fn error_responses() -> Vec<(String, Response)> {
    E::error_responses()
  }

  // We expect error to be present only for response part
  fn error_schemas() -> BTreeMap<String, (String, ReferenceOr<Schema>)> {
    E::schemas_by_status_code()
  }

  fn responses(content_type: Option<String>) -> Option<Responses> {
    T::responses(content_type)
  }
}

#[cfg(test)]
mod test {
  use crate::reference_or::ReferenceOr;
  use crate::ApiComponent;
  use assert_json_diff::assert_json_eq;
  use schemars::schema::{InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec};
  use schemars::{Map, Set};
  use serde_json::json;

  #[test]
  #[allow(dead_code)]
  fn api_component_schema_vec() {
    struct TestChild {
      surname: String,
    }

    impl ApiComponent for TestChild {
      fn child_schemas() -> Vec<(String, ReferenceOr<Schema>)> {
        vec![]
      }

      fn schema() -> Option<(String, ReferenceOr<Schema>)> {
        Some((
          "TestChild".to_string(),
          ReferenceOr::Object(Schema::Object(SchemaObject {
            object: Some(Box::new(ObjectValidation {
              required: Set::from_iter(vec!["surname".to_string()]),
              properties: Map::from_iter(vec![(
                "surname".to_string(),
                Schema::Object(SchemaObject {
                  instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                  ..Default::default()
                }),
              )]),
              ..Default::default()
            })),
            ..Default::default()
          })),
        ))
      }
    }

    struct Test {
      name: String,
      surname: TestChild,
    }

    impl ApiComponent for Test {
      fn child_schemas() -> Vec<(String, ReferenceOr<Schema>)> {
        <TestChild as ApiComponent>::schema().into_iter().collect()
      }

      fn schema() -> Option<(String, ReferenceOr<Schema>)> {
        Some((
          "Test".to_string(),
          ReferenceOr::Object(Schema::Object(SchemaObject {
            object: Some(Box::new(ObjectValidation {
              required: Set::from_iter(vec!["name".to_string(), "surname".to_string()]),
              properties: Map::from_iter(vec![
                (
                  "name".to_string(),
                  Schema::Object(SchemaObject {
                    instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                    ..Default::default()
                  }),
                ),
                (
                  "surname".to_string(),
                  Schema::new_ref("#/components/schemas/TestChild".to_string()),
                ),
              ]),
              ..Default::default()
            })),
            ..Default::default()
          })),
        ))
      }
    }

    let schema = <Vec<Test> as ApiComponent>::schema();
    assert!(schema.is_some());

    let json =
      serde_json::to_value(schema.expect("Missing schema").1).expect("Unable to serialize as Json");
    assert_json_eq!(
      json,
      json!({
        "type": "array",
        "items": {
          "$ref": "#/components/schemas/Test"
        }
      })
    );

    let child_schema = <Vec<Test> as ApiComponent>::child_schemas();
    assert_eq!(child_schema.len(), 2);
    let first_child_schema = child_schema.first().cloned();
    assert!(first_child_schema.is_some());

    let json = serde_json::to_value(first_child_schema.expect("Missing child schema").1)
      .expect("Unable to serialize as Json");
    assert_json_eq!(
      json,
      json!({
        "properties": {
          "name": {
            "type": "string"
          },
          "surname": {
            "$ref": "#/components/schemas/TestChild"
          }
        },
        "required": [
          "name",
          "surname"
        ]
      })
    );

    let last_child_schema = child_schema.last().cloned();
    assert!(last_child_schema.is_some());

    let json = serde_json::to_value(last_child_schema.expect("Missing child schema").1)
      .expect("Unable to serialize as Json");
    assert_json_eq!(
      json,
      json!( {
        "properties": {
          "surname": {
            "type": "string"
          }
        },
        "required": [
          "surname"
        ]
      })
    );
  }
}
