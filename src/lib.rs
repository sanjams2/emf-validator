use std::str::FromStr;
use jsonschema::{Draft, JSONSchema};
use lazy_static::lazy_static;
use serde_json::Value;
use wasm_bindgen::prelude::*;

const SCHEMA_BYTES: &[u8] = include_bytes!("../emf_schema.json");

lazy_static!{
    static ref SCHEMA_VAL: Value = serde_json::from_slice(SCHEMA_BYTES)
        .expect("json parse failed");
    static ref SCHEMA: JSONSchema = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&SCHEMA_VAL)
        .expect("A valid schema");
}

#[wasm_bindgen]
pub fn validate(json: &str) -> Result<(), String> {
    if json.len() < 2 {
        return Err("Invalid size".to_string());
    }
    let json: Value = Value::from_str(json).map_err(|e| e.to_string())?;
    SCHEMA.validate(&json)
        .map_err(|e|
            e.map(|ie| {
                let path = ie.instance_path.to_string();
                let path = if path.len() > 0 { path } else { "/".to_string() };
                format!("{} [Path: {}]", ie.to_string(), path)
            }).collect::<Vec<String>>().join(", ")
        )
}

#[cfg(test)]
mod tests {
    use crate::validate;

    #[test]
    fn it_works() {
        let json = r#"{
            "_aws":{
                "Timestamp": 1666917984381,
                "CloudWatchMetrics": [
                    {
                        "Namespace": "MyAppMetricNamespace",
                        "Dimensions": [
                            []
                        ],
                        "Metrics": [
                            {
                                "Name": "page2view"
                            }
                        ]
                    }
                ]
            },
            "page2view": 1,
            "UserAgent": "Go-http-client/1.1",
            "RequestURI": "HEAD /metrics/v1/page2view?pageversion=1.0.2 HTTP/1.1",
            "pageversion": "1.0.2"
        }"#;
        validate(json).expect("wrong json");
    }

    #[test]
    fn it_fails() {
        let json = r#"{
            "_aws":{
                "Timestap": 1666917984381,
                "CloudWatchMetrics": [
                    {
                        "Namespace": "MyAppMetricNamespace",
                        "Dimesions": [
                            []
                        ],
                        "Metris": [
                            {
                                "Name": "page2view"
                            }
                        ]
                    }
                ]
            }
        }"#;
        let res = validate(json);
        match res {
            Err(e) => {
                assert!(e.contains("\"Dimensions\" is a required property"), "Error should contain: \"Dimensions\" is a required property");
                assert!(e.contains("\"Metrics\" is a required property"), "Error should contain: \"Metrics\" is a required property");
            },
            Ok(_) => assert!(false, "validation should fail")
        }
    }
}
