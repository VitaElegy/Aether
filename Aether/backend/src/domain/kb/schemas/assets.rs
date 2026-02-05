use crate::domain::kb::registry::{BlockSchema, SchemaError};
use serde_json::Value;

pub struct ImageAssetSchema;

impl BlockSchema for ImageAssetSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Required: file_path
        if payload.get("file_path").and_then(|v| v.as_str()).is_none() {
            return Err(SchemaError::ValidationFailed("Missing 'file_path' in image_asset".into()));
        }
        // Required: mime_type
        if payload.get("mime_type").and_then(|v| v.as_str()).is_none() {
            return Err(SchemaError::ValidationFailed("Missing 'mime_type' in image_asset".into()));
        }
        
        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        // Search by alt_text if available
        let alt = payload.get("alt_text").and_then(|v| v.as_str()).unwrap_or("");
        // Maybe filename from path?
        let path = payload.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        format!("{} {}", alt, path).trim().to_string()
    }
}

pub struct IpAssetSchema;

impl BlockSchema for IpAssetSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Required: address
        let address = payload.get("address").and_then(|v| v.as_str());
        if address.is_none() {
            return Err(SchemaError::ValidationFailed("Missing 'address' in ip_asset".into()));
        }

        // Basic IP validation (rudimentary check, mostly relying on frontend)
        // In production, we'd parse with std::net::IpAddr
        let addr_str = address.unwrap();
        if addr_str.parse::<std::net::IpAddr>().is_err() {
             return Err(SchemaError::ValidationFailed(format!("Invalid IP address format: {}", addr_str)));
        }

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        let address = payload.get("address").and_then(|v| v.as_str()).unwrap_or("");
        let tags = payload.get("tags").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
            
        format!("{} {}", address, tags).trim().to_string()
    }
}

pub struct CredentialStubSchema;

impl BlockSchema for CredentialStubSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Required: service, key_id
        if payload.get("service").and_then(|v| v.as_str()).is_none() {
             return Err(SchemaError::ValidationFailed("Missing 'service' in credential_stub".into()));
        }
        if payload.get("key_id").and_then(|v| v.as_str()).is_none() {
             return Err(SchemaError::ValidationFailed("Missing 'key_id' in credential_stub".into()));
        }
        
        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        // Only index service and partial Key ID. Do NOT index vault paths usually.
        let service = payload.get("service").and_then(|v| v.as_str()).unwrap_or("");
        let key_id = payload.get("key_id").and_then(|v| v.as_str()).unwrap_or("");
        
        format!("{} {}", service, key_id).trim().to_string()
    }
}
