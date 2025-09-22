// AW Filesystem Snapshots Daemon
//
// This crate implements a privileged daemon for handling filesystem snapshot operations
// with proper privilege escalation using sudo.

pub mod types;
pub mod operations;
pub mod server;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{encode_length_prefixed_json, decode_length_prefixed_json};
    use crate::types::{Request, Response};

    #[test]
    fn test_length_prefixed_json_encoding() {
        let request = Request::ping();
        let encoded = encode_length_prefixed_json(&request).unwrap();
        let decoded: Request = decode_length_prefixed_json(&encoded).unwrap();
        assert_eq!(request.command, decoded.command);
    }

    #[test]
    fn test_ping_response() {
        let request = Request::ping();
        let response = operations::process_request(request);
        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_invalid_command() {
        let request = Request {
            command: "invalid".to_string(),
            filesystem: None,
            snapshot: None,
            clone: None,
            source: None,
            target: None,
            destination: None,
        };
        let response = operations::process_request(request);
        assert!(!response.success);
        assert!(response.error.is_some());
        assert!(response.error.unwrap().contains("Unknown command"));
    }

    #[test]
    fn test_missing_parameters() {
        // Test clone without snapshot parameter
        let request = Request {
            command: "clone".to_string(),
            filesystem: Some("zfs".to_string()),
            snapshot: None,
            clone: Some("test".to_string()),
            source: None,
            target: None,
            destination: None,
        };
        let response = operations::process_request(request);
        assert!(!response.success);
        assert!(response.error.unwrap().contains("Missing snapshot parameter"));
    }
}