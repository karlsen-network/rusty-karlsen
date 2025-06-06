use crate::script_builder;
use thiserror::Error;
use wasm_bindgen::{JsError, JsValue};
use workflow_wasm::jserror::JsErrorData;

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    JsValue(JsErrorData),

    #[error(transparent)]
    Wasm(#[from] workflow_wasm::error::Error),

    #[error(transparent)]
    ScriptBuilder(#[from] script_builder::ScriptBuilderError),

    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    SerdeWasmBindgen(JsErrorData),

    #[error(transparent)]
    NetworkType(#[from] karlsen_consensus_core::network::NetworkTypeError),

    #[error("Error converting property `{0}`: {1}")]
    Convert(&'static str, String),

    #[error("Error processing JSON: {0}")]
    SerdeJson(String),
}

impl Error {
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }

    pub fn convert<S: std::fmt::Display>(prop: &'static str, msg: S) -> Self {
        Self::Convert(prop, msg.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Custom(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Custom(err.to_string())
    }
}

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        match value {
            Error::JsValue(js_error_data) => js_error_data.into(),
            _ => JsValue::from(value.to_string()),
        }
    }
}

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        Self::JsValue(err.into())
    }
}

impl From<JsError> for Error {
    fn from(err: JsError) -> Self {
        Self::JsValue(err.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err.to_string())
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        Self::SerdeWasmBindgen(JsValue::from(err).into())
    }
}
