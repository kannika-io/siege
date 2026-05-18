use apache_avro::Schema;
use apache_avro::types::Value;
use fake::Fake;
use fake::RngExt;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use fake::rand::rngs::StdRng;

use siege::{SeedError, SiegeError};

pub fn generate_record(schema: &Schema, rng: &mut StdRng) -> Result<Value, SiegeError> {
    generate_value(schema, None, rng)
}

fn generate_value(
    schema: &Schema,
    field_name: Option<&str>,
    rng: &mut StdRng,
) -> Result<Value, SiegeError> {
    use fake::faker::address::en::StreetName;
    use fake::faker::internet::en::SafeEmail;

    match schema {
        Schema::String => {
            let s: String = match field_name {
                Some("name" | "ranger_name" | "captain" | "advisor" | "maester") => {
                    Name().fake_with_rng(rng)
                }
                Some("email") => SafeEmail().fake_with_rng(rng),
                Some("address" | "location" | "destination") => {
                    StreetName().fake_with_rng(rng)
                }
                _ => Sentence(3..6).fake_with_rng(rng),
            };
            Ok(Value::String(s))
        }
        Schema::Int => Ok(Value::Int(rng.random_range(1..1000))),
        Schema::Long => Ok(Value::Long(rng.random_range(1..100_000))),
        Schema::Float => Ok(Value::Float(rng.random_range(0.0..100.0))),
        Schema::Double => Ok(Value::Double(rng.random_range(0.0..100.0))),
        Schema::Boolean => Ok(Value::Boolean(rng.random_bool(0.5))),
        Schema::Record(record_schema) => {
            let fields: Result<Vec<(String, Value)>, SiegeError> = record_schema
                .fields
                .iter()
                .map(|field| {
                    let val = generate_value(&field.schema, Some(&field.name), rng)?;
                    Ok((field.name.clone(), val))
                })
                .collect();
            Ok(Value::Record(fields?))
        }
        Schema::Enum(enum_schema) => {
            let idx = rng.random_range(0..enum_schema.symbols.len() as u32);
            Ok(Value::Enum(idx, enum_schema.symbols[idx as usize].clone()))
        }
        Schema::Array(inner) => {
            let len = rng.random_range(1..5);
            let items: Result<Vec<Value>, SiegeError> = (0..len)
                .map(|_| generate_value(&inner.items, None, rng))
                .collect();
            Ok(Value::Array(items?))
        }
        Schema::Union(union_schema) => {
            let variants: Vec<_> = union_schema
                .variants()
                .iter()
                .filter(|v| !matches!(v, Schema::Null))
                .collect();
            if variants.is_empty() {
                Ok(Value::Null)
            } else {
                let idx = rng.random_range(0..variants.len());
                generate_value(variants[idx], field_name, rng)
            }
        }
        Schema::Null => Ok(Value::Null),
        Schema::Bytes => Ok(Value::Bytes(vec![rng.random(), rng.random(), rng.random()])),
        _ => Err(SiegeError::Seed(SeedError::Failed(format!("unsupported schema type: {schema:?}")))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::rand::SeedableRng;

    fn test_schema() -> Schema {
        Schema::parse_str(r#"{
            "type": "record",
            "name": "Test",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "age", "type": "int"},
                {"name": "active", "type": "boolean"},
                {"name": "score", "type": "double"}
            ]
        }"#).expect("test schema should be valid")
    }

    #[test]
    fn generates_record_with_all_fields() {
        let schema = test_schema();
        let mut rng = StdRng::seed_from_u64(42);
        let value = generate_record(&schema, &mut rng).expect("should generate");

        match value {
            Value::Record(fields) => {
                assert_eq!(fields.len(), 4);
                assert!(matches!(fields[0].1, Value::String(_)));
                assert!(matches!(fields[1].1, Value::Int(_)));
                assert!(matches!(fields[2].1, Value::Boolean(_)));
                assert!(matches!(fields[3].1, Value::Double(_)));
            }
            other => panic!("expected record, got {other:?}"),
        }
    }

    #[test]
    fn deterministic_with_same_seed() {
        let schema = test_schema();
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        let v1 = generate_record(&schema, &mut rng1).expect("should generate");
        let v2 = generate_record(&schema, &mut rng2).expect("should generate");

        assert_eq!(format!("{v1:?}"), format!("{v2:?}"));
    }

    #[test]
    fn different_seeds_produce_different_data() {
        let schema = test_schema();
        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(99);

        let v1 = generate_record(&schema, &mut rng1).expect("should generate");
        let v2 = generate_record(&schema, &mut rng2).expect("should generate");

        assert_ne!(format!("{v1:?}"), format!("{v2:?}"));
    }

    #[test]
    fn name_field_gets_realistic_name() {
        let schema = Schema::parse_str(r#"{
            "type": "record",
            "name": "Test",
            "fields": [{"name": "name", "type": "string"}]
        }"#).expect("test schema should be valid");

        let mut rng = StdRng::seed_from_u64(42);
        let value = generate_record(&schema, &mut rng).expect("should generate");

        match value {
            Value::Record(fields) => {
                match &fields[0].1 {
                    Value::String(s) => {
                        assert!(s.contains(' '), "name field should contain a space (first + last name), got: {s}");
                    }
                    other => panic!("expected string, got {other:?}"),
                }
            }
            other => panic!("expected record, got {other:?}"),
        }
    }
}
