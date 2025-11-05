use base64::Engine; // for encode
use frontend::utils::auth::decode_token;

// Since frontend is a bin crate, we manually test decode_token logic by recreating payload part.
// We'll craft a JWT-like string with a base64url encoded payload and dummy header/signature.

fn make_token(payload_json: &serde_json::Value) -> String {
    let header = serde_json::json!({"alg":"HS256","typ":"JWT"});
    let header_b64 = base64_url(serde_json::to_string(&header).unwrap().as_bytes());
    let payload_b64 = base64_url(payload_json.to_string().as_bytes());
    // signature part can be anything for our decoding test
    format!("{}.{}.sig", header_b64, payload_b64)
}

fn base64_url(data: &[u8]) -> String {
    let s = base64::engine::general_purpose::STANDARD.encode(data);
    s.replace('+', "-").replace('/', "_").trim_end_matches('=').to_string()
}

#[test]
fn decode_token_extracts_claims() {
    let payload = serde_json::json!({
        "sub": "42",
        "email": "user@example.com",
        "name": "Alice",
        "roles": ["Admin", "Homeowner"],
        "exp": 999999
    });
    let token = make_token(&payload);
    let claims = decode_token(&token).expect("decode ok");
    assert_eq!(claims.sub, "42");
    assert_eq!(claims.name, "Alice");
    assert_eq!(claims.roles.len(), 2);
}

#[test]
fn decode_token_rejects_invalid_format() {
    assert!(decode_token("not-a-jwt").is_none());
}
