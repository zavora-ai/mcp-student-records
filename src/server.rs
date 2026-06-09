use crate::store::StudentStore;
use crate::types::*;
use adk_mcp_sdk::{HealthCheck, HealthStatus};
use chrono::NaiveDate;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

fn dactor() -> String { "agent".into() }
fn date(s: &Option<String>) -> Option<NaiveDate> { s.as_ref().and_then(|x| NaiveDate::parse_from_str(x, "%Y-%m-%d").ok()) }

// Every student-scoped tool carries `actor` for FERPA access logging.

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindStudentsInput { pub actor: String, pub sis_id: Option<String>, pub name: Option<String>, pub grade_level: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StudentScopeInput { pub actor: String, pub student_id: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetStatusInput { pub actor: String, pub student_id: String, pub status: EnrollmentStatus }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddAccommodationInput { pub actor: String, pub student_id: String, pub kind: String, pub description: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateCourseInput { pub code: String, pub title: String, #[serde(default = "dcredits")] pub credits: f64, #[serde(default)] pub subject: String }
fn dcredits() -> f64 { 1.0 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSectionInput { pub course_id: String, pub term: String, pub instructor: String, #[serde(default = "dweights")] pub grade_weights: serde_json::Value }
fn dweights() -> serde_json::Value { serde_json::json!({}) }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EnrollInput { pub actor: String, pub student_id: String, pub section_id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateAssignmentInput { pub section_id: String, pub name: String, #[serde(default = "dcat")] pub category: String, pub points_possible: f64, pub due_date: Option<String> }
fn dcat() -> String { "general".into() }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecordGradeInput { pub assignment_id: String, pub student_id: String, #[serde(default)] pub points_earned: f64, #[serde(default)] pub rubric: Vec<RubricCriterion>, pub feedback: Option<String>, #[serde(default = "dactor")] pub graded_by: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GradebookInput { pub actor: String, pub student_id: String, pub section_id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecordAttendanceInput { pub actor: String, pub student_id: String, pub section_id: Option<String>, pub date: String, pub code: AttendanceCode, pub note: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenCaseInput { pub student_id: String, #[serde(default = "dcat")] pub category: String, pub summary: String, pub assigned_to: Option<String>, #[serde(default = "dactor")] pub created_by: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CaseNoteInput { pub case_id: String, pub author: String, pub body: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CaseStatusInput { pub case_id: String, pub status: SupportStatus, #[serde(default = "dactor")] pub actor: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CaseIdInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddGuardianInput { pub actor: String, pub student_id: String, pub name: String, pub relationship: String, pub contact_ref: String, #[serde(default)] pub consent_on_file: bool }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendCommInput { pub student_id: String, pub guardian_id: String, #[serde(default = "dchannel")] pub channel: String, pub subject: String, pub body: String, #[serde(default = "dactor")] pub sent_by: String }
fn dchannel() -> String { "email".into() }

// v1.1 inputs
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddCompetencyInput { pub code: String, #[serde(default)] pub subject: String, pub description: String, #[serde(default)] pub prerequisites: Vec<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListCompetenciesInput { pub subject: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MasteryEvidenceInput { pub student_id: String, pub competency_id: String, pub score: f64, #[serde(default = "dactor")] pub actor: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MasteryScopeInput { pub actor: String, pub student_id: String, pub subject: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddGradReqInput { pub program: String, pub name: String, pub subject: String, pub required_credits: f64 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProgramInput { pub program: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PlaceHoldInput { pub student_id: String, #[serde(default = "dhold")] pub kind: String, pub reason: String, #[serde(default = "dactor")] pub placed_by: String }
fn dhold() -> String { "registration".into() }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HoldIdInput { pub hold_id: String, #[serde(default = "dactor")] pub actor: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HoldsScopeInput { pub actor: String, pub student_id: String, #[serde(default)] pub active_only: bool }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RaiseFlagInput { pub student_id: String, #[serde(default = "dcat")] pub category: String, #[serde(default = "dsev")] pub severity: String, pub detail: String, #[serde(default = "dactor")] pub raised_by: String }
fn dsev() -> String { "medium".into() }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlagStatusInput { pub flag_id: String, pub status: FlagStatus, #[serde(default = "dactor")] pub actor: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FlagsScopeInput { pub actor: String, pub student_id: String, #[serde(default)] pub open_only: bool }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StudentActorInput { pub student_id: String, #[serde(default = "dactor")] pub actor: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AssignInterventionInput { pub student_id: String, pub tier: u8, pub focus: String, pub strategy: String, pub assigned_to: Option<String>, #[serde(default = "dactor")] pub actor: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InterventionIdInput { pub intervention_id: String, #[serde(default = "dactor")] pub actor: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InterventionsScopeInput { pub actor: String, pub student_id: String, #[serde(default)] pub active_only: bool }

// ── server ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct StudentServer { pub store: Arc<StudentStore> }

#[tool_router(server_handler)]
impl StudentServer {
    // ── students ────────────────────────────────────────────────────────────

    #[tool(description = "Find students by SIS id, name, or grade level. PHI/FERPA read — logged to the access log.")]
    fn find_students(&self, Parameters(i): Parameters<FindStudentsInput>) -> String {
        let ss = self.store.find_students(&i.actor, i.sis_id.as_deref(), i.name.as_deref(), i.grade_level.as_deref());
        let out: Vec<serde_json::Value> = ss.iter().map(|s| serde_json::json!({"id": s.id, "sis_id": s.sis_id, "name": format!("{}, {}", s.family_name, s.given_name), "grade_level": s.grade_level, "status": s.status})).collect();
        serde_json::to_string_pretty(&serde_json::json!({"count": out.len(), "students": out})).unwrap()
    }

    #[tool(description = "Get a student's record (demographics, accommodations). FERPA read — logged.")]
    fn get_student(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        match self.store.get_student(&i.actor, &i.student_id) { Some(s) => serde_json::to_string_pretty(&s).unwrap(), None => format!("Student not found: {}", i.student_id) }
    }

    #[tool(description = "Set a student's enrollment status. Gated. FERPA-logged.")]
    fn set_student_status(&self, Parameters(i): Parameters<SetStatusInput>) -> String {
        match self.store.set_student_status(&i.student_id, i.status, &i.actor) {
            Ok(s) => serde_json::to_string_pretty(&serde_json::json!({"id": s.id, "status": s.status})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Add an accommodation (IEP/504/ELL) to a student. Gated. FERPA-logged.")]
    fn add_accommodation(&self, Parameters(i): Parameters<AddAccommodationInput>) -> String {
        match self.store.add_accommodation(&i.student_id, &i.kind, &i.description, &i.actor) {
            Ok(a) => serde_json::to_string_pretty(&serde_json::json!({"accommodation_id": a.id, "kind": a.kind})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Get the FERPA access log (accounting of disclosures) for a student.")]
    fn get_access_log(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        self.store.log_access(&i.actor, &i.student_id, "read_access_log", "accounting");
        let log = self.store.access_log_for(&i.student_id);
        serde_json::to_string_pretty(&serde_json::json!({"student_id": i.student_id, "entries": log.len(), "access_log": log})).unwrap()
    }

    // ── courses / sections / enrollment ──────────────────────────────────────

    #[tool(description = "Create a course in the catalog.")]
    fn create_course(&self, Parameters(i): Parameters<CreateCourseInput>) -> String {
        let c = self.store.create_course(&i.code, &i.title, i.credits, &i.subject);
        serde_json::to_string_pretty(&serde_json::json!({"course_id": c.id, "code": c.code})).unwrap()
    }

    #[tool(description = "List catalog courses.")]
    fn list_courses(&self) -> String {
        let c = self.store.list_courses();
        serde_json::to_string_pretty(&serde_json::json!({"count": c.len(), "courses": c})).unwrap()
    }

    #[tool(description = "Create a course section (offering) for a term, with grade-category weights e.g. {\"homework\":0.3,\"exams\":0.7}.")]
    fn create_section(&self, Parameters(i): Parameters<CreateSectionInput>) -> String {
        match self.store.create_section(&i.course_id, &i.term, &i.instructor, i.grade_weights) {
            Ok(s) => serde_json::to_string_pretty(&serde_json::json!({"section_id": s.id, "term": s.term})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Enroll a student in a section. Gated. FERPA-logged.")]
    fn enroll(&self, Parameters(i): Parameters<EnrollInput>) -> String {
        match self.store.enroll(&i.student_id, &i.section_id, &i.actor) {
            Ok(e) => serde_json::to_string_pretty(&serde_json::json!({"enrollment_id": e.id, "status": e.status})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List a student's enrollments. FERPA read — logged.")]
    fn get_enrollments(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        let e = self.store.enrollments_for(&i.actor, &i.student_id);
        serde_json::to_string_pretty(&serde_json::json!({"count": e.len(), "enrollments": e})).unwrap()
    }

    // ── grades / rubric ──────────────────────────────────────────────────────

    #[tool(description = "Create an assignment in a section (category maps to the section's grade weights).")]
    fn create_assignment(&self, Parameters(i): Parameters<CreateAssignmentInput>) -> String {
        match self.store.create_assignment(&i.section_id, &i.name, &i.category, i.points_possible, date(&i.due_date)) {
            Ok(a) => serde_json::to_string_pretty(&serde_json::json!({"assignment_id": a.id, "name": a.name})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Record a grade for a student on an assignment. If a rubric is given, the score is the sum of awarded criterion points (rubric grading). Gated. FERPA-logged.")]
    fn record_grade(&self, Parameters(i): Parameters<RecordGradeInput>) -> String {
        match self.store.record_grade(&i.assignment_id, &i.student_id, i.points_earned, i.rubric, i.feedback, &i.graded_by) {
            Ok(g) => serde_json::to_string_pretty(&serde_json::json!({"grade_id": g.id, "points_earned": g.points_earned, "points_possible": g.points_possible})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Get all grades for a student. FERPA read — logged.")]
    fn get_grades(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        let g = self.store.grades_for_student(&i.actor, &i.student_id);
        serde_json::to_string_pretty(&serde_json::json!({"count": g.len(), "grades": g})).unwrap()
    }

    #[tool(description = "Compute a student's weighted course grade (gradebook) for a section. FERPA read — logged.")]
    fn gradebook(&self, Parameters(i): Parameters<GradebookInput>) -> String {
        match self.store.gradebook(&i.actor, &i.student_id, &i.section_id) {
            Some(v) => serde_json::to_string_pretty(&v).unwrap(), None => format!("Section not found: {}", i.section_id) }
    }

    // ── attendance ──────────────────────────────────────────────────────────

    #[tool(description = "Record an attendance entry (present/absent/tardy/excused) for a student. Gated. FERPA-logged.")]
    fn record_attendance(&self, Parameters(i): Parameters<RecordAttendanceInput>) -> String {
        let Some(d) = date(&Some(i.date.clone())) else { return "Error: date must be YYYY-MM-DD".into() };
        match self.store.record_attendance(&i.student_id, i.section_id, d, i.code, i.note, &i.actor) {
            Ok(r) => serde_json::to_string_pretty(&serde_json::json!({"attendance_id": r.id, "code": r.code})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Get a student's attendance summary and rate. FERPA read — logged.")]
    fn attendance_summary(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        serde_json::to_string_pretty(&self.store.attendance_summary(&i.actor, &i.student_id)).unwrap()
    }

    // ── support ──────────────────────────────────────────────────────────────

    #[tool(description = "Open a student support case (academic/attendance/behavioral/wellbeing). FERPA-logged.")]
    fn open_support_case(&self, Parameters(i): Parameters<OpenCaseInput>) -> String {
        match self.store.open_case(&i.student_id, &i.category, &i.summary, i.assigned_to, &i.created_by) {
            Ok(c) => serde_json::to_string_pretty(&serde_json::json!({"case_id": c.id, "status": c.status})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Get a support case with its notes.")]
    fn get_support_case(&self, Parameters(i): Parameters<CaseIdInput>) -> String {
        match self.store.get_case(&i.id) { Some(c) => serde_json::to_string_pretty(&c).unwrap(), None => format!("Case not found: {}", i.id) }
    }

    #[tool(description = "Add a note to a support case. FERPA-logged.")]
    fn add_case_note(&self, Parameters(i): Parameters<CaseNoteInput>) -> String {
        match self.store.add_case_note(&i.case_id, &i.author, &i.body) {
            Ok(n) => serde_json::to_string_pretty(&serde_json::json!({"note_id": n.id, "added": true})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Set a support case status (open/in_progress/resolved/closed). Gated.")]
    fn set_case_status(&self, Parameters(i): Parameters<CaseStatusInput>) -> String {
        match self.store.set_case_status(&i.case_id, i.status, &i.actor) {
            Ok(c) => serde_json::to_string_pretty(&serde_json::json!({"id": c.id, "status": c.status})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List a student's support cases. FERPA read — logged.")]
    fn get_support_cases(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        let c = self.store.cases_for(&i.actor, &i.student_id);
        serde_json::to_string_pretty(&serde_json::json!({"count": c.len(), "cases": c})).unwrap()
    }

    // ── guardians & comms ─────────────────────────────────────────────────────

    #[tool(description = "Add a guardian to a student. Set consent_on_file=true to authorize record disclosures (FERPA). Gated.")]
    fn add_guardian(&self, Parameters(i): Parameters<AddGuardianInput>) -> String {
        match self.store.add_guardian(&i.student_id, &i.name, &i.relationship, &i.contact_ref, i.consent_on_file, &i.actor) {
            Ok(g) => serde_json::to_string_pretty(&serde_json::json!({"guardian_id": g.id, "consent_on_file": g.consent_on_file})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List a student's guardians. FERPA read — logged.")]
    fn get_guardians(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        let g = self.store.guardians_for(&i.actor, &i.student_id);
        serde_json::to_string_pretty(&serde_json::json!({"count": g.len(), "guardians": g})).unwrap()
    }

    #[tool(description = "Send a communication to a guardian. FERPA-gated: the guardian must belong to the student and have consent on file. Gated; requires approval in production.")]
    fn send_communication(&self, Parameters(i): Parameters<SendCommInput>) -> String {
        match self.store.send_communication(&i.student_id, &i.guardian_id, &i.channel, &i.subject, &i.body, &i.sent_by) {
            Ok(c) => serde_json::to_string_pretty(&serde_json::json!({"communication_id": c.id, "guardian_id": c.guardian_id})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List communications sent about a student. FERPA read — logged.")]
    fn get_communications(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        let c = self.store.communications_for(&i.actor, &i.student_id);
        serde_json::to_string_pretty(&serde_json::json!({"count": c.len(), "communications": c})).unwrap()
    }

    // ── transcript / analytics ───────────────────────────────────────────────

    #[tool(description = "Get a student's transcript: completed courses, credits, and cumulative GPA. FERPA read — logged.")]
    fn get_transcript(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        match self.store.transcript(&i.actor, &i.student_id) {
            Some(v) => serde_json::to_string_pretty(&v).unwrap(), None => format!("Student not found: {}", i.student_id) }
    }

    #[tool(description = "Learning-analytics standing: grade average, attendance rate, risk level, and recommendation. FERPA read — logged.")]
    fn analytics(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        match self.store.analytics(&i.actor, &i.student_id) {
            Some(v) => serde_json::to_string_pretty(&v).unwrap(), None => format!("Student not found: {}", i.student_id) }
    }

    // ═══════════════════════════════════════════════════════════════════
    // v1.1 — competency / mastery (Adaptive Tutor)
    // ═══════════════════════════════════════════════════════════════════

    #[tool(description = "Add a learning competency/standard (with optional prerequisite competency ids for sequencing).")]
    fn add_competency(&self, Parameters(i): Parameters<AddCompetencyInput>) -> String {
        let c = self.store.add_competency(&i.code, &i.subject, &i.description, i.prerequisites);
        serde_json::to_string_pretty(&serde_json::json!({"competency_id": c.id, "code": c.code})).unwrap()
    }

    #[tool(description = "List competencies, optionally by subject.")]
    fn list_competencies(&self, Parameters(i): Parameters<ListCompetenciesInput>) -> String {
        let c = self.store.list_competencies(i.subject.as_deref());
        serde_json::to_string_pretty(&serde_json::json!({"count": c.len(), "competencies": c})).unwrap()
    }

    #[tool(description = "Record a mastery evidence observation (0–1 score) for a student on a competency; updates the rolling mastery level. Gated. FERPA-logged.")]
    fn record_mastery_evidence(&self, Parameters(i): Parameters<MasteryEvidenceInput>) -> String {
        match self.store.record_mastery_evidence(&i.student_id, &i.competency_id, i.score, &i.actor) {
            Ok(m) => serde_json::to_string_pretty(&serde_json::json!({"competency_id": m.competency_id, "level": m.level, "score": m.score, "evidence": m.evidence_count})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Get a student's competency mastery (level + score per competency), optionally by subject. FERPA read — logged.")]
    fn get_mastery(&self, Parameters(i): Parameters<MasteryScopeInput>) -> String {
        let m = self.store.mastery_for(&i.actor, &i.student_id, i.subject.as_deref());
        serde_json::to_string_pretty(&serde_json::json!({"student_id": i.student_id, "count": m.len(), "mastery": m})).unwrap()
    }

    #[tool(description = "Adaptive learning path: competencies the student is ready to learn next (not yet proficient, prerequisites met) and those blocked on prerequisites. FERPA read — logged.")]
    fn learning_path(&self, Parameters(i): Parameters<MasteryScopeInput>) -> String {
        serde_json::to_string_pretty(&self.store.learning_path(&i.actor, &i.student_id, i.subject.as_deref())).unwrap()
    }

    // ═══════════════════════════════════════════════════════════════════
    // v1.1 — degree audit / standing / holds (Academic Policy Advisor)
    // ═══════════════════════════════════════════════════════════════════

    #[tool(description = "Add a graduation/program requirement (credits in a subject area).")]
    fn add_grad_requirement(&self, Parameters(i): Parameters<AddGradReqInput>) -> String {
        let r = self.store.add_grad_requirement(&i.program, &i.name, &i.subject, i.required_credits);
        serde_json::to_string_pretty(&serde_json::json!({"requirement_id": r.id, "program": r.program, "subject": r.subject})).unwrap()
    }

    #[tool(description = "List graduation requirements for a program.")]
    fn list_grad_requirements(&self, Parameters(i): Parameters<ProgramInput>) -> String {
        let r = self.store.list_grad_requirements(&i.program);
        serde_json::to_string_pretty(&serde_json::json!({"count": r.len(), "requirements": r})).unwrap()
    }

    #[tool(description = "Degree audit: a student's earned vs. required credits per subject for their program, with progress % and on-track flag. FERPA read — logged.")]
    fn degree_audit(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        match self.store.degree_audit(&i.actor, &i.student_id) {
            Some(v) => serde_json::to_string_pretty(&v).unwrap(), None => format!("Student not found: {}", i.student_id) }
    }

    #[tool(description = "Academic standing (honor_roll/good_standing/academic_warning/academic_probation) derived from GPA. FERPA read — logged.")]
    fn academic_standing(&self, Parameters(i): Parameters<StudentScopeInput>) -> String {
        match self.store.academic_standing(&i.actor, &i.student_id) {
            Some(v) => serde_json::to_string_pretty(&v).unwrap(), None => format!("Student not found: {}", i.student_id) }
    }

    #[tool(description = "Place a hold on a student account (registration/transcript/financial). Gated. FERPA-logged.")]
    fn place_hold(&self, Parameters(i): Parameters<PlaceHoldInput>) -> String {
        match self.store.place_hold(&i.student_id, &i.kind, &i.reason, &i.placed_by) {
            Ok(h) => serde_json::to_string_pretty(&serde_json::json!({"hold_id": h.id, "kind": h.kind, "active": h.active})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Release a hold. Gated. FERPA-logged.")]
    fn release_hold(&self, Parameters(i): Parameters<HoldIdInput>) -> String {
        match self.store.release_hold(&i.hold_id, &i.actor) {
            Ok(h) => serde_json::to_string_pretty(&serde_json::json!({"hold_id": h.id, "active": h.active, "released_at": h.released_at})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List holds on a student (set active_only=true for current holds). FERPA read — logged.")]
    fn get_holds(&self, Parameters(i): Parameters<HoldsScopeInput>) -> String {
        let h = self.store.holds_for(&i.actor, &i.student_id, i.active_only);
        serde_json::to_string_pretty(&serde_json::json!({"count": h.len(), "holds": h})).unwrap()
    }

    // ═══════════════════════════════════════════════════════════════════
    // v1.1 — early warning / interventions (Student Support / Analytics)
    // ═══════════════════════════════════════════════════════════════════

    #[tool(description = "Raise an early-warning flag on a student (category/severity/detail). FERPA-logged.")]
    fn raise_flag(&self, Parameters(i): Parameters<RaiseFlagInput>) -> String {
        match self.store.raise_flag(&i.student_id, &i.category, &i.severity, &i.detail, &i.raised_by) {
            Ok(f) => serde_json::to_string_pretty(&serde_json::json!({"flag_id": f.id, "category": f.category, "severity": f.severity, "status": f.status})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "Set an early-warning flag's status (open/acknowledged/resolved/dismissed).")]
    fn set_flag_status(&self, Parameters(i): Parameters<FlagStatusInput>) -> String {
        match self.store.set_flag_status(&i.flag_id, i.status, &i.actor) {
            Ok(f) => serde_json::to_string_pretty(&serde_json::json!({"flag_id": f.id, "status": f.status})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List early-warning flags on a student (set open_only=true for active alerts). FERPA read — logged.")]
    fn get_flags(&self, Parameters(i): Parameters<FlagsScopeInput>) -> String {
        let f = self.store.flags_for(&i.actor, &i.student_id, i.open_only);
        serde_json::to_string_pretty(&serde_json::json!({"count": f.len(), "flags": f})).unwrap()
    }

    #[tool(description = "Auto-evaluate early-warning signals (grades, attendance, mastery) and raise flags past threshold. Returns the flags raised. FERPA-logged.")]
    fn evaluate_early_warning(&self, Parameters(i): Parameters<StudentActorInput>) -> String {
        match self.store.evaluate_early_warning(&i.student_id, &i.actor) {
            Ok(flags) => serde_json::to_string_pretty(&serde_json::json!({"student_id": i.student_id, "flags_raised": flags.len(), "flags": flags.iter().map(|f| serde_json::json!({"id": f.id, "category": f.category, "severity": f.severity, "detail": f.detail})).collect::<Vec<_>>()})).unwrap(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Assign an MTSS/RTI intervention (tier 1–3) with a focus and strategy. Gated. FERPA-logged.")]
    fn assign_intervention(&self, Parameters(i): Parameters<AssignInterventionInput>) -> String {
        match self.store.assign_intervention(&i.student_id, i.tier, &i.focus, &i.strategy, i.assigned_to, &i.actor) {
            Ok(iv) => serde_json::to_string_pretty(&serde_json::json!({"intervention_id": iv.id, "tier": iv.tier, "focus": iv.focus})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "End an active intervention. FERPA-logged.")]
    fn end_intervention(&self, Parameters(i): Parameters<InterventionIdInput>) -> String {
        match self.store.end_intervention(&i.intervention_id, &i.actor) {
            Ok(iv) => serde_json::to_string_pretty(&serde_json::json!({"intervention_id": iv.id, "active": iv.active, "ended_at": iv.ended_at})).unwrap(), Err(e) => format!("Error: {e}") }
    }

    #[tool(description = "List a student's interventions (set active_only=true for current ones). FERPA read — logged.")]
    fn get_interventions(&self, Parameters(i): Parameters<InterventionsScopeInput>) -> String {
        let iv = self.store.interventions_for(&i.actor, &i.student_id, i.active_only);
        serde_json::to_string_pretty(&serde_json::json!({"count": iv.len(), "interventions": iv})).unwrap()
    }
}

#[async_trait::async_trait]
impl HealthCheck for StudentServer {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus { healthy: true, message: Some("operational".into()), latency_ms: Some(1) }
    }
}
