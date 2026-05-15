use apache_avro::types::Value;
use apache_avro::Schema;
use siege::schema_registry::SchemaId;
use siege::SiegeError;

pub struct AvroSerializer {
    schema: Schema,
    header: [u8; 5],
}

impl AvroSerializer {
    pub fn new(schema: Schema, schema_id: SchemaId) -> Self {
        let mut header = [0u8; 5];
        header[0] = 0x00;
        header[1..5].copy_from_slice(&schema_id.0.to_be_bytes());
        Self { schema, header }
    }

    pub fn serialize(&self, value: Value) -> Result<Vec<u8>, SiegeError> {
        let mut buf = Vec::with_capacity(128);
        buf.extend_from_slice(&self.header);

        let resolved = value
            .resolve(&self.schema)
            .map_err(|e| SiegeError::Seed(format!("avro resolve: {e}")))?;

        apache_avro::to_avro_datum(&self.schema, resolved)
            .map(|datum| {
                buf.extend_from_slice(&datum);
                buf
            })
            .map_err(|e| SiegeError::Seed(format!("avro encode: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema() -> Schema {
        Schema::parse_str(r#"{
            "type": "record",
            "name": "Test",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "age", "type": "int"}
            ]
        }"#).expect("test schema should be valid")
    }

    #[test]
    fn serialize_prepends_confluent_header() {
        let schema = test_schema();
        let serializer = AvroSerializer::new(schema, SchemaId(42));

        let value = Value::Record(vec![
            ("name".into(), Value::String("test".into())),
            ("age".into(), Value::Int(25)),
        ]);

        let bytes = serializer.serialize(value).expect("serialize should succeed");

        assert_eq!(bytes[0], 0x00, "magic byte");
        assert_eq!(&bytes[1..5], &42_i32.to_be_bytes(), "schema id");
        assert!(bytes.len() > 5, "should have avro payload after header");
    }

    #[test]
    fn serialize_produces_valid_avro() {
        let schema = test_schema();
        let serializer = AvroSerializer::new(schema.clone(), SchemaId(1));

        let value = Value::Record(vec![
            ("name".into(), Value::String("alice".into())),
            ("age".into(), Value::Int(30)),
        ]);

        let bytes = serializer.serialize(value).expect("serialize should succeed");
        let avro_payload = &bytes[5..];

        let decoded = apache_avro::from_avro_datum(&schema, &mut avro_payload.as_ref(), None)
            .expect("datum should decode");

        match decoded {
            Value::Record(fields) => {
                assert_eq!(fields[0].1, Value::String("alice".into()));
                assert_eq!(fields[1].1, Value::Int(30));
            }
            other => panic!("expected record, got {other:?}"),
        }
    }
}
