use crate::ApiComponent;

macro_rules! simple_modifier {
  ($($ty:ty),*) => {
    $(
      impl ApiComponent for $ty {
        fn child_schemas() -> Vec<(String, $crate::reference_or::ReferenceOr<$crate::Schema>)> {
          vec![]
        }
        fn raw_schema() -> Option<$crate::reference_or::ReferenceOr<$crate::Schema>> {
          let gen = schemars::gen::SchemaSettings::openapi3().into_generator();

          let schema: $crate::reference_or::ReferenceOr<$crate::Schema> =
            $crate::Schema::Object(gen.into_root_schema_for::<$ty>().schema).into();
          Some(schema)
        }
        fn schema() -> Option<(String, $crate::reference_or::ReferenceOr<$crate::Schema>)> {
          None
        }
      }
    )*
  };
}

simple_modifier!(
  char, String, bool, f32, f64, i8, i16, i32, u8, u16, u32, i64, i128, isize, u64, u128, usize
);

#[cfg(feature = "chrono")]
simple_modifier!(chrono::NaiveDate, chrono::NaiveTime, chrono::NaiveDateTime);
#[cfg(feature = "rust_decimal")]
simple_modifier!(rust_decimal::Decimal);
#[cfg(feature = "uuid")]
simple_modifier!(uuid::Uuid);
#[cfg(feature = "url")]
simple_modifier!(url::Url);

#[cfg(feature = "chrono")]
impl<T: chrono::TimeZone> ApiComponent for chrono::DateTime<T> {
  fn child_schemas() -> Vec<(String, crate::reference_or::ReferenceOr<crate::Schema>)> {
    vec![]
  }

  fn raw_schema() -> Option<crate::reference_or::ReferenceOr<crate::Schema>> {
    let gen = schemars::gen::SchemaSettings::openapi3().into_generator();

    let schema: crate::reference_or::ReferenceOr<crate::Schema> =
      crate::Schema::Object(gen.into_root_schema_for::<chrono::DateTime<T>>().schema).into();
    Some(schema)
  }

  fn schema() -> Option<(String, crate::reference_or::ReferenceOr<crate::Schema>)> {
    None
  }
}
