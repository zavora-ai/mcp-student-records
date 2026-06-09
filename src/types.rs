use chrono::{DateTime, NaiveDate, Utc};
use rmcp::schemars;
use serde::{Deserialize, Serialize};

// ── students ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    Active,
    Inactive,
    Graduated,
    Withdrawn,
    Suspended,
}

/// A student record. PII-bearing; access is logged and minimized. Sample data
/// is fictitious.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Student {
    pub id: String,
    /// Student information number (site identifier).
    pub sis_id: String,
    pub family_name: String,
    pub given_name: String,
    pub grade_level: String,
    pub program: Option<String>,
    pub status: EnrollmentStatus,
    pub date_of_birth: Option<NaiveDate>,
    /// Education accommodations (IEP/504) flag and note.
    pub accommodations: Vec<Accommodation>,
    pub enrolled_on: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Accommodation {
    pub id: String,
    /// e.g. "IEP", "504", "ELL".
    pub kind: String,
    pub description: String,
    pub active: bool,
}

// ── courses & sections ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Course {
    pub id: String,
    pub code: String,
    pub title: String,
    pub credits: f64,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Section {
    pub id: String,
    pub course_id: String,
    pub term: String,
    pub instructor: String,
    /// Weighted assignment categories, e.g. {"homework":0.3,"exams":0.7}.
    pub grade_weights: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ── enrollments ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SectionEnrollmentStatus {
    Enrolled,
    Dropped,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Enrollment {
    pub id: String,
    pub student_id: String,
    pub section_id: String,
    pub status: SectionEnrollmentStatus,
    /// Final letter grade once completed.
    pub final_grade: Option<String>,
    pub enrolled_at: DateTime<Utc>,
}

// ── grades / rubrics ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Assignment {
    pub id: String,
    pub section_id: String,
    pub name: String,
    /// Category used by section grade weights, e.g. "homework", "exams".
    pub category: String,
    pub points_possible: f64,
    pub due_date: Option<NaiveDate>,
}

/// One criterion of a rubric.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RubricCriterion {
    pub name: String,
    pub max_points: f64,
    pub awarded: Option<f64>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Grade {
    pub id: String,
    pub assignment_id: String,
    pub student_id: String,
    pub points_earned: f64,
    pub points_possible: f64,
    /// Optional rubric breakdown.
    pub rubric: Vec<RubricCriterion>,
    pub feedback: Option<String>,
    pub graded_by: String,
    pub graded_at: DateTime<Utc>,
}

// ── attendance ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttendanceCode {
    Present,
    Absent,
    Tardy,
    Excused,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AttendanceRecord {
    pub id: String,
    pub student_id: String,
    pub section_id: Option<String>,
    pub date: NaiveDate,
    pub code: AttendanceCode,
    pub note: Option<String>,
}

// ── support / interventions ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SupportStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SupportCase {
    pub id: String,
    pub student_id: String,
    /// e.g. "academic", "attendance", "behavioral", "wellbeing".
    pub category: String,
    pub summary: String,
    pub status: SupportStatus,
    pub assigned_to: Option<String>,
    pub notes: Vec<CaseNote>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CaseNote {
    pub id: String,
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

// ── guardians & communications ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Guardian {
    pub id: String,
    pub student_id: String,
    pub name: String,
    pub relationship: String,
    /// Opaque contact reference (token), not raw contact details.
    pub contact_ref: String,
    /// FERPA: whether this guardian is authorized to receive records.
    pub consent_on_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Communication {
    pub id: String,
    pub student_id: String,
    pub guardian_id: String,
    pub channel: String,
    pub subject: String,
    pub body: String,
    pub sent_by: String,
    pub sent_at: DateTime<Utc>,
}

// ── access log (FERPA accounting) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AccessEntry {
    pub at: DateTime<Utc>,
    pub actor: String,
    pub student_id: String,
    pub action: String,
    pub detail: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// v1.1 — competency/mastery, degree audit & standing, early warning
// ═══════════════════════════════════════════════════════════════════════════

// ── competency / mastery (Adaptive Tutor) ───────────────────────────────────

/// A learning standard / competency in a framework (e.g. CCSS, NGSS, or local).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Competency {
    pub id: String,
    /// Framework code, e.g. "CCSS.MATH.HSA.REI.B.3".
    pub code: String,
    pub subject: String,
    pub description: String,
    /// Prerequisite competency ids (for learning-path sequencing).
    pub prerequisites: Vec<String>,
}

/// A student's mastery level for a competency.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MasteryLevel {
    NotStarted,
    Beginning,
    Developing,
    Proficient,
    Advanced,
}

impl MasteryLevel {
    /// 0–1 score for analytics/ordering.
    pub fn score(self) -> f64 {
        match self {
            MasteryLevel::NotStarted => 0.0,
            MasteryLevel::Beginning => 0.25,
            MasteryLevel::Developing => 0.5,
            MasteryLevel::Proficient => 0.8,
            MasteryLevel::Advanced => 1.0,
        }
    }
    pub fn is_proficient(self) -> bool {
        matches!(self, MasteryLevel::Proficient | MasteryLevel::Advanced)
    }
}

/// A student's mastery record for one competency, updated from evidence.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MasteryRecord {
    pub student_id: String,
    pub competency_id: String,
    pub level: MasteryLevel,
    /// Rolling 0–1 score from evidence.
    pub score: f64,
    /// Count of evidence observations.
    pub evidence_count: u32,
    pub updated_at: DateTime<Utc>,
}

// ── graduation / degree audit / standing / holds (Academic Policy Advisor) ───

/// A graduation/program requirement (credits in a subject area).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GradRequirement {
    pub id: String,
    /// Program this applies to (e.g. "STEM", "default").
    pub program: String,
    pub name: String,
    pub subject: String,
    pub required_credits: f64,
}

/// Academic standing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcademicStanding {
    HonorRoll,
    GoodStanding,
    AcademicWarning,
    AcademicProbation,
}

/// A registration/records hold on a student account.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Hold {
    pub id: String,
    pub student_id: String,
    /// e.g. "registration", "transcript", "financial".
    pub kind: String,
    pub reason: String,
    pub active: bool,
    pub placed_by: String,
    pub created_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
}

// ── early warning / intervention tiers (Student Support / Analytics) ─────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlagStatus {
    Open,
    Acknowledged,
    Resolved,
    Dismissed,
}

/// An early-warning flag/alert raised on a student.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Flag {
    pub id: String,
    pub student_id: String,
    /// e.g. "attendance", "grades", "behavior", "mastery", "composite".
    pub category: String,
    /// "low" | "medium" | "high".
    pub severity: String,
    pub detail: String,
    pub status: FlagStatus,
    pub raised_by: String,
    pub created_at: DateTime<Utc>,
}

/// MTSS/RTI intervention tier (1 = universal, 3 = intensive).
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Intervention {
    pub id: String,
    pub student_id: String,
    pub tier: u8,
    /// e.g. "reading", "math", "attendance", "behavior", "sel".
    pub focus: String,
    pub strategy: String,
    pub active: bool,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}
