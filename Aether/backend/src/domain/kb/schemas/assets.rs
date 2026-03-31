use crate::domain::kb::registry::{BlockSchema, SchemaError};
use serde_json::Value;

fn require_string_field<'a>(
    payload: &'a Value,
    field: &str,
    schema_name: &str,
) -> Result<&'a str, SchemaError> {
    payload.get(field).and_then(|v| v.as_str()).ok_or_else(|| {
        SchemaError::ValidationFailed(format!("Missing '{}' in {}", field, schema_name))
    })
}

fn asset_filename(payload: &Value) -> &str {
    payload
        .get("display_name")
        .and_then(|v| v.as_str())
        .or_else(|| payload.get("original_filename").and_then(|v| v.as_str()))
        .unwrap_or("")
}

pub struct ImageAssetSchema;

impl BlockSchema for ImageAssetSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        require_string_field(payload, "file_path", "image_asset")?;
        let mime_type = require_string_field(payload, "mime_type", "image_asset")?;

        if !mime_type.starts_with("image/") {
            return Err(SchemaError::ValidationFailed(format!(
                "Invalid image mime_type: {}",
                mime_type,
            )));
        }

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        // Search by alt_text if available
        let alt = payload
            .get("alt_text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        // Maybe filename from path?
        let path = payload
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let filename = asset_filename(payload);
        format!("{} {} {}", alt, filename, path).trim().to_string()
    }
}

pub struct PdfAssetSchema;

impl BlockSchema for PdfAssetSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        require_string_field(payload, "file_path", "pdf_asset")?;
        let mime_type = require_string_field(payload, "mime_type", "pdf_asset")?;
        let extension = payload
            .pointer("/metadata/extension")
            .and_then(|v| v.as_str());

        if mime_type != "application/pdf" && extension != Some("pdf") {
            return Err(SchemaError::ValidationFailed(format!(
                "Invalid pdf asset signature: mime_type={}, extension={}",
                mime_type,
                extension.unwrap_or(""),
            )));
        }

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        let filename = asset_filename(payload);
        let path = payload
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let hash = payload.get("hash").and_then(|v| v.as_str()).unwrap_or("");
        format!("{} {} {}", filename, path, hash).trim().to_string()
    }
}

pub struct FileAssetSchema;

impl BlockSchema for FileAssetSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        require_string_field(payload, "file_path", "file_asset")?;
        require_string_field(payload, "mime_type", "file_asset")?;
        let filename = asset_filename(payload);
        if filename.is_empty() {
            return Err(SchemaError::ValidationFailed(
                "Missing display name in file_asset".into(),
            ));
        }

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        let filename = asset_filename(payload);
        let mime_type = payload
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let extension = payload
            .pointer("/metadata/extension")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        format!("{} {} {}", filename, mime_type, extension)
            .trim()
            .to_string()
    }
}

pub struct IpAssetSchema;

impl BlockSchema for IpAssetSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Required: address
        let address = require_string_field(payload, "address", "ip_asset")?;

        // Basic IP validation (rudimentary check, mostly relying on frontend)
        // In production, we'd parse with std::net::IpAddr
        if address.parse::<std::net::IpAddr>().is_err() {
            return Err(SchemaError::ValidationFailed(format!(
                "Invalid IP address format: {}",
                address
            )));
        }

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        let address = payload
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tags = payload
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        format!("{} {}", address, tags).trim().to_string()
    }
}

pub struct CredentialStubSchema;

impl BlockSchema for CredentialStubSchema {
    fn validate(&self, payload: &Value) -> Result<(), SchemaError> {
        // Required: service, key_id
        require_string_field(payload, "service", "credential_stub")?;
        require_string_field(payload, "key_id", "credential_stub")?;

        Ok(())
    }

    fn to_searchable_text(&self, payload: &Value) -> String {
        // Only index service and partial Key ID. Do NOT index vault paths usually.
        let service = payload
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let key_id = payload.get("key_id").and_then(|v| v.as_str()).unwrap_or("");

        format!("{} {}", service, key_id).trim().to_string()
    }
}
