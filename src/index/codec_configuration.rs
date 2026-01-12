use std::borrow::Cow;

use serde::{Deserialize, Serialize};

const STANDARD_CODEC_ID: &str = "tantivy-default";

/// A Codec configuration is just a serializable object.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CodecConfiguration {
    codec_id: Cow<'static, str>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    props: serde_json::Value,
}

impl CodecConfiguration {
    /// Returns true if the codec is the standard codec.
    pub fn is_standard(&self) -> bool {
        self.codec_id == STANDARD_CODEC_ID && self.props.is_null()
    }
}

impl Default for CodecConfiguration {
    fn default() -> Self {
        CodecConfiguration {
            codec_id: Cow::Borrowed(STANDARD_CODEC_ID),
            props: serde_json::Value::Null,
        }
    }
}
