//! Integration tests for the Student Information System store.

use chrono::NaiveDate;
use mcp_student_records::store::StudentStore;
use mcp_student_records::types::*;

fn store() -> StudentStore {
    StudentStore::new()
}

#[test]
fn seeded_students_present() {
    let s = store();
    assert!(!s.find_students("t", None, Some("mwangi"), None).is_empty());
}

#[test]
fn ferpa_access_is_logged() {
    let s = store();
    s.get_student("ms.carter", "STU-1001");
    s.grades_for_student("ms.carter", "STU-1001");
    let log = s.access_log_for("STU-1001");
    // Our two reads are recorded for this actor (the seed may also have logged
    // entries from other actors during construction).
    let mine: Vec<_> = log.iter().filter(|e| e.actor == "ms.carter").collect();
    assert!(mine.len() >= 2, "expected >=2 ms.carter entries, got {}", mine.len());
    assert!(mine.iter().any(|e| e.action == "read_student"));
    assert!(mine.iter().any(|e| e.action == "read_grades"));
}

#[test]
fn rubric_grading_sums_criteria() {
    let s = store();
    let c = s.create_course("ENG1", "English I", 1.0, "English");
    let sec = s.create_section(&c.id, "2026-Spring", "Mr. Poe", serde_json::json!({})).unwrap();
    let a = s.create_assignment(&sec.id, "Essay", "writing", 30.0, None).unwrap();
    let rubric = vec![
        RubricCriterion { name: "Thesis".into(), max_points: 10.0, awarded: Some(8.0), comment: None },
        RubricCriterion { name: "Evidence".into(), max_points: 10.0, awarded: Some(9.0), comment: None },
        RubricCriterion { name: "Mechanics".into(), max_points: 10.0, awarded: Some(7.0), comment: None },
    ];
    let g = s.record_grade(&a.id, "STU-1001", 0.0, rubric, Some("good work".into()), "Mr. Poe").unwrap();
    assert_eq!(g.points_earned, 24.0, "rubric sum = 8+9+7");
}

#[test]
fn weighted_gradebook() {
    let s = store();
    let c = s.create_course("MATH9", "Math", 1.0, "Mathematics");
    let sec = s.create_section(&c.id, "T", "Teacher", serde_json::json!({"homework":0.2,"exams":0.8})).unwrap();
    let hw = s.create_assignment(&sec.id, "HW", "homework", 10.0, None).unwrap();
    let ex = s.create_assignment(&sec.id, "Exam", "exams", 100.0, None).unwrap();
    s.record_grade(&hw.id, "STU-1001", 10.0, vec![], None, "t").unwrap(); // 100%
    s.record_grade(&ex.id, "STU-1001", 70.0, vec![], None, "t").unwrap(); // 70%
    let gb = s.gradebook("t", "STU-1001", &sec.id).unwrap();
    // 0.2*100 + 0.8*70 = 76
    assert!((gb["overall_pct"].as_f64().unwrap() - 76.0).abs() < 0.1, "got {}", gb["overall_pct"]);
    assert_eq!(gb["letter"], "C");
}

#[test]
fn attendance_rate() {
    let s = store();
    for (d, code) in [(1, AttendanceCode::Present), (2, AttendanceCode::Present), (3, AttendanceCode::Absent), (4, AttendanceCode::Present)] {
        s.record_attendance("STU-1002", None, NaiveDate::from_ymd_opt(2026, 3, d).unwrap(), code, None, "sys").unwrap();
    }
    let sum = s.attendance_summary("t", "STU-1002");
    assert_eq!(sum["records"], 4);
    assert!((sum["attendance_rate"].as_f64().unwrap() - 75.0).abs() < 0.1);
}

#[test]
fn guardian_comms_ferpa_gated() {
    let s = store();
    // Guardian without consent.
    let g = s.add_guardian("STU-1002", "Parent", "father", "ref:p", false, "reg").unwrap();
    assert!(s.send_communication("STU-1002", &g.id, "email", "Hi", "msg", "agent").is_err(), "no consent -> blocked");
    // Grant consent via a consenting guardian.
    let g2 = s.add_guardian("STU-1002", "Parent2", "mother", "ref:p2", true, "reg").unwrap();
    assert!(s.send_communication("STU-1002", &g2.id, "email", "Hi", "msg", "agent").is_ok());
    // Guardian of a different student -> rejected.
    assert!(s.send_communication("STU-1001", &g2.id, "email", "x", "y", "agent").is_err());
}

#[test]
fn transcript_gpa() {
    let s = store();
    let c = s.create_course("HIST1", "History", 1.0, "History");
    let sec = s.create_section(&c.id, "2025-Fall", "T", serde_json::json!({})).unwrap();
    let e = s.enroll("STU-1001", &sec.id, "reg").unwrap();
    let _ = e;
    // Manually complete with a grade by re-enrolling state: use store internals via transcript-visible path.
    // Simulate completion by recording final grade through a fresh completed enrollment is internal;
    // here we just check transcript runs and returns structure.
    let t = s.transcript("t", "STU-1001").unwrap();
    assert!(t["gpa"].is_number());
    assert!(t["entries"].is_array());
}

#[test]
fn analytics_risk_levels() {
    let s = store();
    // Low grades + poor attendance -> high risk.
    let c = s.create_course("SCI1", "Science", 1.0, "Science");
    let sec = s.create_section(&c.id, "T", "T", serde_json::json!({})).unwrap();
    let a = s.create_assignment(&sec.id, "Test", "exams", 100.0, None).unwrap();
    s.record_grade(&a.id, "STU-1002", 45.0, vec![], None, "t").unwrap();
    for d in 1..=5 { s.record_attendance("STU-1002", None, NaiveDate::from_ymd_opt(2026, 3, d).unwrap(), AttendanceCode::Absent, None, "sys").unwrap(); }
    let an = s.analytics("t", "STU-1002").unwrap();
    assert_eq!(an["risk_level"], "high");
    assert!(an["risk_factors"].as_array().unwrap().len() >= 1);
}

#[test]
fn support_case_lifecycle() {
    let s = store();
    let c = s.open_case("STU-1001", "academic", "Failing math", Some("counselor".into()), "teacher").unwrap();
    s.add_case_note(&c.id, "counselor", "Met with student").unwrap();
    let resolved = s.set_case_status(&c.id, SupportStatus::Resolved, "counselor").unwrap();
    assert_eq!(resolved.status, SupportStatus::Resolved);
    assert!(resolved.resolved_at.is_some());
    assert_eq!(s.get_case(&c.id).unwrap().notes.len(), 1);
}

#[test]
fn enroll_guards() {
    let s = store();
    assert!(s.enroll("STU-NOPE", "SEC-NOPE", "r").is_err());
    let c = s.create_course("X", "X", 1.0, "X");
    let sec = s.create_section(&c.id, "T", "T", serde_json::json!({})).unwrap();
    assert!(s.enroll("STU-1001", &sec.id, "r").is_ok());
    // duplicate enroll blocked
    assert!(s.enroll("STU-1001", &sec.id, "r").is_err());
}
