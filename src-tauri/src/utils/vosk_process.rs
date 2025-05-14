use log::{info, debug, warn};

#[allow(dead_code)]
pub fn vosk_result_to_string(json_string: &str) -> String {
    // Parse the JSON string
    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&json_string) {
        if let Some(text) = result["partial"].as_str() {
            let trimmed_text = text.trim();
            if trimmed_text.is_empty() {
                debug!("Vosk result is empty");
                return String::new();
            }
            info!("Vosk result: {}", trimmed_text);
            return trimmed_text.to_string();
        }
    }
    warn!("Failed to parse Vosk result: {}", json_string);
    String::new()
}
