use rand::Rng;

pub fn poison_pill() -> Vec<u8> {
    let mut rng = rand::rng();
    let len = rng.random_range(16..256);
    let mut bytes = vec![0u8; len];
    rng.fill(&mut bytes[..]);
    if len > 4 {
        bytes[0] = 0xFF;
        bytes[1] = 0xFE;
    }
    bytes
}

pub fn schema_breaking_json() -> Vec<u8> {
    let mut rng = rand::rng();
    let variant = rng.random_range(0..4);
    let json = match variant {
        0 => r#"{"__corrupted":true,"value":null,"ts":-1}"#,
        1 => r#"{"key":12345,"payload":["not","a","valid","schema"]}"#,
        2 => r#"{"error":"chaos","nested":{"broken":true,"data":"0xDEADBEEF"}}"#,
        _ => r#"{"type":"INVALID","version":-999,"fields":{}}"#,
    };
    json.as_bytes().to_vec()
}
