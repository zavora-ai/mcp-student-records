//! v1.1 integration tests: competency/mastery, degree audit & standing, early warning.

use mcp_student_records::store::StudentStore;
use mcp_student_records::types::*;

fn store() -> StudentStore {
    StudentStore::new()
}

#[test]
fn mastery_rolling_average_and_level() {
    let s = store();
    let c = s.add_competency("X.1", "Mathematics", "Test comp", vec![]);
    s.record_mastery_evidence("STU-1001", &c.id, 0.6, "t").unwrap();
    let m = s.record_mastery_evidence("STU-1001", &c.id, 0.8, "t").unwrap();
    // rolling avg of 0.6 and 0.8 = 0.7 -> Developing
    assert!((m.score - 0.7).abs() < 0.001);
    assert_eq!(m.level, MasteryLevel::Developing);
    assert_eq!(m.evidence_count, 2);
}

#[test]
fn learning_path_respects_prerequisites() {
    let s = store();
    // Seeded: linear (proficient ~0.85) -> quadratic (developing) -> functions (not started).
    let path = s.learning_path("t", "STU-1001", Some("Mathematics"));
    let ready: Vec<String> = path["recommended_next"].as_array().unwrap().iter().map(|x| x["code"].as_str().unwrap().to_string()).collect();
    let blocked: Vec<String> = path["blocked_on_prerequisites"].as_array().unwrap().iter().map(|x| x["code"].as_str().unwrap().to_string()).collect();
    // Quadratic is ready (its prereq linear is proficient); functions is blocked (quadratic not proficient).
    assert!(ready.iter().any(|c| c.contains("REI.B.4")), "quadratic should be ready: {ready:?}");
    assert!(blocked.iter().any(|c| c.contains("IF.A.1")), "functions should be blocked: {blocked:?}");
}

#[test]
fn degree_audit_computes_remaining() {
    let s = store();
    let audit = s.degree_audit("t", "STU-1001").unwrap();
    assert_eq!(audit["program"], "STEM");
    // No completed courses seeded with passing finals -> 0 earned, not on track.
    assert_eq!(audit["on_track_to_graduate"], false);
    let reqs = audit["requirements"].as_array().unwrap();
    assert!(reqs.iter().any(|r| r["subject"] == "Mathematics" && r["required"] == 4.0));
}

#[test]
fn academic_standing_from_gpa() {
    let s = store();
    let st = s.academic_standing("t", "STU-1001").unwrap();
    assert!(st["standing"].is_string());
    assert!(st["gpa"].is_number());
}

#[test]
fn holds_lifecycle() {
    let s = store();
    let h = s.place_hold("STU-1002", "registration", "unpaid fees", "bursar").unwrap();
    assert!(h.active);
    assert_eq!(s.holds_for("t", "STU-1002", true).len(), 1);
    let r = s.release_hold(&h.id, "bursar").unwrap();
    assert!(!r.active && r.released_at.is_some());
    assert_eq!(s.holds_for("t", "STU-1002", true).len(), 0, "no active holds after release");
}

#[test]
fn early_warning_raises_flags() {
    let s = store();
    // Make STU-1002 struggling: low grade + absences.
    let c = s.create_course("SCI2", "Science", 1.0, "Science");
    let sec = s.create_section(&c.id, "T", "T", serde_json::json!({})).unwrap();
    let a = s.create_assignment(&sec.id, "Quiz", "exams", 100.0, None).unwrap();
    s.record_grade(&a.id, "STU-1002", 40.0, vec![], None, "t").unwrap();
    for d in 1..=5 { s.record_attendance("STU-1002", None, chrono::NaiveDate::from_ymd_opt(2026,3,d).unwrap(), AttendanceCode::Absent, None, "sys").unwrap(); }
    let raised = s.evaluate_early_warning("STU-1002", "early-warning").unwrap();
    let cats: Vec<&str> = raised.iter().map(|f| f.category.as_str()).collect();
    assert!(cats.contains(&"grades"));
    assert!(cats.contains(&"attendance"));
    // flags are queryable
    assert!(s.flags_for("t", "STU-1002", true).len() >= 2);
}

#[test]
fn intervention_lifecycle() {
    let s = store();
    let iv = s.assign_intervention("STU-1001", 2, "math", "Small-group tutoring 3x/week", Some("specialist".into()), "counselor").unwrap();
    assert_eq!(iv.tier, 2);
    assert!(iv.active);
    let ended = s.end_intervention(&iv.id, "counselor").unwrap();
    assert!(!ended.active && ended.ended_at.is_some());
    assert_eq!(s.interventions_for("t", "STU-1001", true).len(), 0);
}

#[test]
fn tier_clamped() {
    let s = store();
    let iv = s.assign_intervention("STU-1001", 9, "reading", "x", None, "c").unwrap();
    assert_eq!(iv.tier, 3, "tier clamped to 1..=3");
}

#[test]
fn mastery_evidence_validates_refs() {
    let s = store();
    assert!(s.record_mastery_evidence("STU-NOPE", "CMP-x", 0.5, "t").is_err());
    let c = s.add_competency("Y.1", "Math", "d", vec![]);
    assert!(s.record_mastery_evidence("STU-NOPE", &c.id, 0.5, "t").is_err());
}
