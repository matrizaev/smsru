//! Transport layer: HTTP and wire-format details (serialization/deserialization).

mod send_sms;

pub use send_sms::{decode_send_sms_json_response, encode_send_sms_form};
