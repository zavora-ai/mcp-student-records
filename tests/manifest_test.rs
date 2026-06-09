//! Validate mcp-server.toml parses, passes SDK validation, has the right tool
//! count, and gates the FERPA-sensitive writes.

use adk_mcp_sdk::manifest::ServerManifest;
use std::path::Path;

fn manifest() -> ServerManifest {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("mcp-server.toml");
    ServerManifest::from_file(&path).expect("manifest should parse")
}

#[test]
fn manifest_parses_and_validates() {
    let m = manifest();
    assert!(m.validate().is_empty(), "validation errors: {:?}", m.validate());
    assert_eq!(m.server_id, "mcp_student_records");
    assert_eq!(m.domain, "education");
    assert_eq!(m.tools.len(), 46, "expected 46 declared tools");
}

#[test]
fn v11_gated_writes() {
    let m = manifest();
    for name in ["record_mastery_evidence", "place_hold", "release_hold", "assign_intervention"] {
        let t = m.tools.iter().find(|t| t.name == name).unwrap_or_else(|| panic!("{name} present"));
        assert!(t.requires_approval, "{name} must require approval");
    }
}

#[test]
fn sensitive_writes_are_gated() {
    let m = manifest();
    for name in ["set_student_status", "add_accommodation", "enroll", "record_grade", "record_attendance", "add_guardian", "send_communication"] {
        let t = m.tools.iter().find(|t| t.name == name).unwrap_or_else(|| panic!("{name} present"));
        assert!(t.requires_approval, "{name} must require approval");
    }
}

#[test]
fn comms_is_external_write() {
    use adk_mcp_sdk::risk::RiskClass;
    let m = manifest();
    let c = m.tools.iter().find(|t| t.name == "send_communication").unwrap();
    assert_eq!(c.risk_class, RiskClass::ExternalWrite);
}

#[test]
fn ferpa_reads_are_read_only() {
    use adk_mcp_sdk::risk::RiskClass;
    let m = manifest();
    for name in ["get_student", "get_grades", "gradebook", "attendance_summary", "get_transcript", "analytics", "get_access_log"] {
        let t = m.tools.iter().find(|t| t.name == name).unwrap();
        assert_eq!(t.risk_class, RiskClass::ReadOnly, "{name} should be read_only");
    }
}
