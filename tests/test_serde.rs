#![cfg(feature = "serde")]
use koicore::command::{Command, CompositeValue, Parameter, Value};
use serde_json;

#[test]
fn test_command_serialization() {
    let cmd = Command::new(
        "character",
        vec![
            Parameter::from("Alice"),
            Parameter::from("Hello, world!"),
            Parameter::Composite(
                "action".to_string(),
                CompositeValue::Single(Value::String("walk".to_string())),
            ),
        ],
    );

    let json = serde_json::to_string(&cmd).unwrap();

    // Verify JSON structure (order of keys in params might vary if it was a map, but params is a List)
    // Parameter::Basic(Alice) -> "Alice"
    // Parameter::Basic("Hello, world!") -> "Hello, world!"
    // Parameter::Composite("action", Single("walk")) -> {"action": "walk"}
    let expected_json =
        r#"{"name":"character","params":["Alice","Hello, world!",{"action":"walk"}]}"#;
    assert_eq!(json, expected_json);

    let decoded: Command = serde_json::from_str(&json).unwrap();
    assert_eq!(cmd, decoded);
}

#[test]
fn test_complex_command_serialization() {
    let cmd = Command::new(
        "action",
        vec![
            Parameter::Composite(
                "list".to_string(),
                CompositeValue::List(vec![
                    Value::Int(1),
                    Value::Float(2.5),
                    Value::Bool(true),
                    Value::String("test".to_string()),
                ]),
            ),
            Parameter::Composite(
                "dict".to_string(),
                CompositeValue::Dict(vec![
                    ("key1".to_string(), Value::Int(10)),
                    ("key2".to_string(), Value::String("value".to_string())),
                ]),
            ),
        ],
    );

    let json = serde_json::to_string(&cmd).unwrap();
    println!("JSON: {}", json);

    // {"list": [1, 2.5, true, "test"]}
    // {"dict": {"key1": 10, "key2": "value"}}
    // Note: Dict iteration order is preserved from the Vec<(String, Value)> in serialization implementation?
    // CompositeValue::Dict stores Vec<(String, Value)>. My manual impl iterates over this Vec.
    // So order IS preserved.
    let expected_json = r#"{"name":"action","params":[{"list":[1,2.5,true,"test"]},{"dict":{"key1":10,"key2":"value"}}]}"#;
    assert_eq!(json, expected_json);

    let decoded: Command = serde_json::from_str(&json).unwrap();
    assert_eq!(cmd, decoded);
}

#[test]
fn test_text_command_serialization() {
    let cmd = Command::new_text("Hello, world!");
    let json = serde_json::to_string(&cmd).unwrap();
    assert_eq!(json, r#"{"name":"@text","params":["Hello, world!"]}"#);
    let decoded: Command = serde_json::from_str(&json).unwrap();
    assert_eq!(cmd, decoded);
}

#[test]
fn test_annotation_command_serialization() {
    let cmd = Command::new_annotation("This is an annotation");
    let json = serde_json::to_string(&cmd).unwrap();
    let decoded: Command = serde_json::from_str(&json).unwrap();
    assert_eq!(cmd, decoded);
}
