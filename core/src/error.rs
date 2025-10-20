use thiserror::Error;
use wasm_bindgen::JsValue;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("JavaScript error: {0}")]
    JsError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Relay error: {0}")]
    RelayError(String),
    
    #[error("Signer error: {0}")]
    SignerError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("{0}")]
    Other(String),
}

impl From<JsValue> for CoreError {
    fn from(value: JsValue) -> Self {
        if let Some(s) = value.as_string() {
            CoreError::JsError(s)
        } else {
            CoreError::JsError(format!("{:?}", value))
        }
    }
}

impl From<CoreError> for JsValue {
    fn from(error: CoreError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        CoreError::ParseError(error.to_string())
    }
}

impl From<rexie::Error> for CoreError {
    fn from(error: rexie::Error) -> Self {
        CoreError::StorageError(error.to_string())
    }
}

impl From<serde_wasm_bindgen::Error> for CoreError {
    fn from(error: serde_wasm_bindgen::Error) -> Self {
        CoreError::ParseError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;

