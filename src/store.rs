use crate::types::*;
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// In-memory Student Information System store. Every student-scoped operation
/// appends to a FERPA access log. Sample data is entirely fictitious.
pub struct StudentStore {
    students: Mutex<HashMap<String, Student>>,
    courses: Mutex<HashMap<String, Course>>,
    sections: Mutex<HashMap<String, Section>>,
    enrollments: Mutex<Vec<Enrollment>>,
    assignments: Mutex<HashMap<String, Assignment>>,
    grades: Mutex<Vec<Grade>>,
    attendance: Mutex<Vec<AttendanceRecord>>,
    cases: Mutex<HashMap<String, SupportCase>>,
    guardians: Mutex<HashMap<String, Guardian>>,
    comms: Mutex<Vec<Communication>>,
    access_log: Mutex<Vec<AccessEntry>>,
    seq: Mutex<u64>,
}

impl Default for StudentStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StudentStore {
    pub fn new() -> Self {
        let s = Self {
            students: Mutex::new(HashMap::new()),
            courses: Mutex::new(HashMap::new()),
            sections: Mutex::new(HashMap::new()),
            enrollments: Mutex::new(Vec::new()),
            assignments: Mutex::new(HashMap::new()),
            grades: Mutex::new(Vec::new()),
            attendance: Mutex::new(Vec::new()),
            cases: Mutex::new(HashMap::new()),
            guardians: Mutex::new(HashMap::new()),
            comms: Mutex::new(Vec::new()),
            access_log: Mutex::new(Vec::new()),
            seq: Mutex::new(1000),
        };
        s.seed();
        s
    }

    fn next(&self, prefix: &str) -> String {
        let mut n = self.seq.lock().unwrap();
        *n += 1;
        format!("{prefix}-{}", *n)
    }

    pub fn log_access(&self, actor: &str, student_id: &str, action: &str, detail: impl Into<String>) {
        self.access_log.lock().unwrap().push(AccessEntry { at: Utc::now(), actor: actor.to_string(), student_id: student_id.to_string(), action: action.to_string(), detail: detail.into() });
    }

    pub fn access_log_for(&self, student_id: &str) -> Vec<AccessEntry> {
        self.access_log.lock().unwrap().iter().filter(|e| e.student_id == student_id).cloned().collect()
    }

    pub fn student_exists(&self, id: &str) -> bool {
        self.students.lock().unwrap().contains_key(id)
    }

    // ── students ────────────────────────────────────────────────────────────

    pub fn get_student(&self, actor: &str, id: &str) -> Option<Student> {
        let s = self.students.lock().unwrap().get(id).cloned();
        if s.is_some() { self.log_access(actor, id, "read_student", "demographics"); }
        s
    }

    pub fn find_students(&self, actor: &str, sis_id: Option<&str>, name: Option<&str>, grade_level: Option<&str>) -> Vec<Student> {
        let name_l = name.map(|n| n.to_lowercase());
        let out: Vec<Student> = self
            .students
            .lock()
            .unwrap()
            .values()
            .filter(|s| sis_id.is_none_or(|x| s.sis_id.eq_ignore_ascii_case(x))
                && name_l.as_ref().is_none_or(|n| s.family_name.to_lowercase().contains(n) || s.given_name.to_lowercase().contains(n))
                && grade_level.is_none_or(|g| s.grade_level == g))
            .cloned()
            .collect();
        for s in &out { self.log_access(actor, &s.id, "search_student", "matched"); }
        out
    }

    pub fn set_student_status(&self, id: &str, status: EnrollmentStatus, actor: &str) -> Result<Student, String> {
        let mut students = self.students.lock().unwrap();
        let s = students.get_mut(id).ok_or_else(|| format!("Student not found: {id}"))?;
        s.status = status;
        s.updated_at = Utc::now();
        self.log_access(actor, id, "set_status", format!("{status:?}"));
        Ok(s.clone())
    }

    pub fn add_accommodation(&self, id: &str, kind: &str, description: &str, actor: &str) -> Result<Accommodation, String> {
        let mut students = self.students.lock().unwrap();
        let s = students.get_mut(id).ok_or_else(|| format!("Student not found: {id}"))?;
        let a = Accommodation { id: format!("ACC-{}", &Uuid::new_v4().simple().to_string()[..8]), kind: kind.to_string(), description: description.to_string(), active: true };
        s.accommodations.push(a.clone());
        s.updated_at = Utc::now();
        self.log_access(actor, id, "add_accommodation", kind.to_string());
        Ok(a)
    }

    // ── courses / sections ──────────────────────────────────────────────────

    pub fn create_course(&self, code: &str, title: &str, credits: f64, subject: &str) -> Course {
        let c = Course { id: self.next("CRS"), code: code.to_string(), title: title.to_string(), credits, subject: subject.to_string() };
        self.courses.lock().unwrap().insert(c.id.clone(), c.clone());
        c
    }

    pub fn create_section(&self, course_id: &str, term: &str, instructor: &str, grade_weights: serde_json::Value) -> Result<Section, String> {
        if !self.courses.lock().unwrap().contains_key(course_id) {
            return Err(format!("Course not found: {course_id}"));
        }
        let s = Section { id: self.next("SEC"), course_id: course_id.to_string(), term: term.to_string(), instructor: instructor.to_string(), grade_weights, created_at: Utc::now() };
        self.sections.lock().unwrap().insert(s.id.clone(), s.clone());
        Ok(s)
    }

    pub fn get_section(&self, id: &str) -> Option<Section> {
        self.sections.lock().unwrap().get(id).cloned()
    }

    pub fn list_courses(&self) -> Vec<Course> {
        let mut v: Vec<Course> = self.courses.lock().unwrap().values().cloned().collect();
        v.sort_by(|a, b| a.code.cmp(&b.code));
        v
    }

    // ── enrollments ─────────────────────────────────────────────────────────

    pub fn enroll(&self, student_id: &str, section_id: &str, actor: &str) -> Result<Enrollment, String> {
        if !self.student_exists(student_id) { return Err(format!("Student not found: {student_id}")); }
        if self.get_section(section_id).is_none() { return Err(format!("Section not found: {section_id}")); }
        if self.enrollments.lock().unwrap().iter().any(|e| e.student_id == student_id && e.section_id == section_id && e.status == SectionEnrollmentStatus::Enrolled) {
            return Err("Already enrolled in this section".into());
        }
        let e = Enrollment { id: self.next("ENR"), student_id: student_id.to_string(), section_id: section_id.to_string(), status: SectionEnrollmentStatus::Enrolled, final_grade: None, enrolled_at: Utc::now() };
        self.enrollments.lock().unwrap().push(e.clone());
        self.log_access(actor, student_id, "enroll", section_id.to_string());
        Ok(e)
    }

    pub fn enrollments_for(&self, actor: &str, student_id: &str) -> Vec<Enrollment> {
        self.log_access(actor, student_id, "read_enrollments", "list");
        self.enrollments.lock().unwrap().iter().filter(|e| e.student_id == student_id).cloned().collect()
    }

    // ── assignments / grades ────────────────────────────────────────────────

    pub fn create_assignment(&self, section_id: &str, name: &str, category: &str, points_possible: f64, due_date: Option<NaiveDate>) -> Result<Assignment, String> {
        if self.get_section(section_id).is_none() { return Err(format!("Section not found: {section_id}")); }
        let a = Assignment { id: self.next("ASG"), section_id: section_id.to_string(), name: name.to_string(), category: category.to_string(), points_possible, due_date };
        self.assignments.lock().unwrap().insert(a.id.clone(), a.clone());
        Ok(a)
    }

    pub fn get_assignment(&self, id: &str) -> Option<Assignment> {
        self.assignments.lock().unwrap().get(id).cloned()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_grade(&self, assignment_id: &str, student_id: &str, points_earned: f64, rubric: Vec<RubricCriterion>, feedback: Option<String>, graded_by: &str) -> Result<Grade, String> {
        let asg = self.get_assignment(assignment_id).ok_or_else(|| format!("Assignment not found: {assignment_id}"))?;
        if !self.student_exists(student_id) { return Err(format!("Student not found: {student_id}")); }
        // If a rubric is supplied, points_earned is the rubric sum.
        let earned = if !rubric.is_empty() {
            rubric.iter().map(|c| c.awarded.unwrap_or(0.0)).sum()
        } else {
            points_earned
        };
        let g = Grade {
            id: self.next("GRD"), assignment_id: assignment_id.to_string(), student_id: student_id.to_string(),
            points_earned: earned, points_possible: asg.points_possible, rubric, feedback,
            graded_by: graded_by.to_string(), graded_at: Utc::now(),
        };
        // Replace any existing grade for this assignment+student.
        let mut grades = self.grades.lock().unwrap();
        grades.retain(|x| !(x.assignment_id == assignment_id && x.student_id == student_id));
        grades.push(g.clone());
        drop(grades);
        self.log_access(graded_by, student_id, "record_grade", assignment_id.to_string());
        Ok(g)
    }

    pub fn grades_for_student(&self, actor: &str, student_id: &str) -> Vec<Grade> {
        self.log_access(actor, student_id, "read_grades", "list");
        self.grades.lock().unwrap().iter().filter(|g| g.student_id == student_id).cloned().collect()
    }

    /// Compute a weighted course grade for a student in a section.
    pub fn gradebook(&self, actor: &str, student_id: &str, section_id: &str) -> Option<serde_json::Value> {
        let section = self.get_section(section_id)?;
        self.log_access(actor, student_id, "read_gradebook", section_id.to_string());
        let assignments = self.assignments.lock().unwrap();
        let grades = self.grades.lock().unwrap();
        // category -> (earned_pct_sum, count)
        let mut cat: HashMap<String, (f64, u32)> = HashMap::new();
        let mut items = Vec::new();
        for (aid, asg) in assignments.iter().filter(|(_, a)| a.section_id == section_id) {
            if let Some(g) = grades.iter().find(|g| &g.assignment_id == aid && g.student_id == student_id) {
                let pct = if g.points_possible > 0.0 { g.points_earned / g.points_possible } else { 0.0 };
                let e = cat.entry(asg.category.clone()).or_insert((0.0, 0));
                e.0 += pct; e.1 += 1;
                items.push(serde_json::json!({"assignment": asg.name, "category": asg.category, "earned": g.points_earned, "possible": g.points_possible, "pct": (pct*100.0).round()}));
            }
        }
        // Weighted overall using section weights (fallback: equal weight per graded category).
        let weights = section.grade_weights.as_object();
        let mut overall = 0.0;
        let mut wsum = 0.0;
        for (c, (sum, n)) in &cat {
            let avg = if *n > 0 { sum / *n as f64 } else { 0.0 };
            let w = weights.and_then(|o| o.get(c)).and_then(|v| v.as_f64()).unwrap_or(1.0);
            overall += avg * w; wsum += w;
        }
        let pct = if wsum > 0.0 { overall / wsum } else { 0.0 };
        Some(serde_json::json!({
            "student_id": student_id, "section_id": section_id,
            "overall_pct": (pct*1000.0).round()/10.0,
            "letter": letter_grade(pct),
            "graded_items": items.len(),
            "items": items,
        }))
    }

    // ── attendance ──────────────────────────────────────────────────────────

    pub fn record_attendance(&self, student_id: &str, section_id: Option<String>, date: NaiveDate, code: AttendanceCode, note: Option<String>, actor: &str) -> Result<AttendanceRecord, String> {
        if !self.student_exists(student_id) { return Err(format!("Student not found: {student_id}")); }
        let r = AttendanceRecord { id: self.next("ATT"), student_id: student_id.to_string(), section_id, date, code, note };
        self.attendance.lock().unwrap().push(r.clone());
        self.log_access(actor, student_id, "record_attendance", format!("{code:?}"));
        Ok(r)
    }

    /// Attendance summary + rate for a student.
    pub fn attendance_summary(&self, actor: &str, student_id: &str) -> serde_json::Value {
        self.log_access(actor, student_id, "read_attendance", "summary");
        let recs: Vec<AttendanceRecord> = self.attendance.lock().unwrap().iter().filter(|r| r.student_id == student_id).cloned().collect();
        let total = recs.len();
        let present = recs.iter().filter(|r| matches!(r.code, AttendanceCode::Present | AttendanceCode::Excused)).count();
        let absent = recs.iter().filter(|r| r.code == AttendanceCode::Absent).count();
        let tardy = recs.iter().filter(|r| r.code == AttendanceCode::Tardy).count();
        let rate = if total > 0 { (present as f64 / total as f64 * 1000.0).round()/10.0 } else { 100.0 };
        serde_json::json!({"student_id": student_id, "records": total, "present": present, "absent": absent, "tardy": tardy, "attendance_rate": rate})
    }

    // ── support ──────────────────────────────────────────────────────────────

    pub fn open_case(&self, student_id: &str, category: &str, summary: &str, assigned_to: Option<String>, created_by: &str) -> Result<SupportCase, String> {
        if !self.student_exists(student_id) { return Err(format!("Student not found: {student_id}")); }
        let id = self.next("SUP");
        let c = SupportCase { id: id.clone(), student_id: student_id.to_string(), category: category.to_string(), summary: summary.to_string(), status: SupportStatus::Open, assigned_to, notes: Vec::new(), created_by: created_by.to_string(), created_at: Utc::now(), resolved_at: None };
        self.cases.lock().unwrap().insert(id.clone(), c.clone());
        self.log_access(created_by, student_id, "open_support_case", category.to_string());
        Ok(c)
    }

    pub fn get_case(&self, id: &str) -> Option<SupportCase> {
        self.cases.lock().unwrap().get(id).cloned()
    }

    pub fn add_case_note(&self, case_id: &str, author: &str, body: &str) -> Result<CaseNote, String> {
        let mut cases = self.cases.lock().unwrap();
        let c = cases.get_mut(case_id).ok_or_else(|| format!("Case not found: {case_id}"))?;
        let n = CaseNote { id: format!("CN-{}", &Uuid::new_v4().simple().to_string()[..8]), author: author.to_string(), body: body.to_string(), created_at: Utc::now() };
        c.notes.push(n.clone());
        let sid = c.student_id.clone();
        drop(cases);
        self.log_access(author, &sid, "case_note", case_id.to_string());
        Ok(n)
    }

    pub fn set_case_status(&self, case_id: &str, status: SupportStatus, actor: &str) -> Result<SupportCase, String> {
        let mut cases = self.cases.lock().unwrap();
        let c = cases.get_mut(case_id).ok_or_else(|| format!("Case not found: {case_id}"))?;
        c.status = status;
        if matches!(status, SupportStatus::Resolved | SupportStatus::Closed) { c.resolved_at = Some(Utc::now()); }
        let out = c.clone();
        self.log_access(actor, &out.student_id, "case_status", format!("{status:?}"));
        Ok(out)
    }

    pub fn cases_for(&self, actor: &str, student_id: &str) -> Vec<SupportCase> {
        self.log_access(actor, student_id, "read_support_cases", "list");
        let mut v: Vec<SupportCase> = self.cases.lock().unwrap().values().filter(|c| c.student_id == student_id).cloned().collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v
    }

    // ── guardians & comms ─────────────────────────────────────────────────────

    pub fn add_guardian(&self, student_id: &str, name: &str, relationship: &str, contact_ref: &str, consent_on_file: bool, actor: &str) -> Result<Guardian, String> {
        if !self.student_exists(student_id) { return Err(format!("Student not found: {student_id}")); }
        let g = Guardian { id: self.next("GRD"), student_id: student_id.to_string(), name: name.to_string(), relationship: relationship.to_string(), contact_ref: contact_ref.to_string(), consent_on_file };
        self.guardians.lock().unwrap().insert(g.id.clone(), g.clone());
        self.log_access(actor, student_id, "add_guardian", relationship.to_string());
        Ok(g)
    }

    pub fn guardians_for(&self, actor: &str, student_id: &str) -> Vec<Guardian> {
        self.log_access(actor, student_id, "read_guardians", "list");
        self.guardians.lock().unwrap().values().filter(|g| g.student_id == student_id).cloned().collect()
    }

    /// Send a communication to a guardian. FERPA-gated: the guardian must have
    /// consent on file and belong to the student.
    pub fn send_communication(&self, student_id: &str, guardian_id: &str, channel: &str, subject: &str, body: &str, sent_by: &str) -> Result<Communication, String> {
        let g = self.guardians.lock().unwrap().get(guardian_id).cloned().ok_or_else(|| format!("Guardian not found: {guardian_id}"))?;
        if g.student_id != student_id {
            return Err("Guardian is not associated with this student".into());
        }
        if !g.consent_on_file {
            return Err("Cannot send: guardian does not have FERPA consent on file".into());
        }
        let c = Communication { id: self.next("COM"), student_id: student_id.to_string(), guardian_id: guardian_id.to_string(), channel: channel.to_string(), subject: subject.to_string(), body: body.to_string(), sent_by: sent_by.to_string(), sent_at: Utc::now() };
        self.comms.lock().unwrap().push(c.clone());
        self.log_access(sent_by, student_id, "send_communication", format!("guardian {guardian_id}"));
        Ok(c)
    }

    pub fn communications_for(&self, actor: &str, student_id: &str) -> Vec<Communication> {
        self.log_access(actor, student_id, "read_communications", "list");
        let mut v: Vec<Communication> = self.comms.lock().unwrap().iter().filter(|c| c.student_id == student_id).cloned().collect();
        v.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
        v
    }

    // ── transcript / analytics ───────────────────────────────────────────────

    /// Transcript: completed enrollments with final grades + cumulative GPA.
    pub fn transcript(&self, actor: &str, student_id: &str) -> Option<serde_json::Value> {
        if !self.student_exists(student_id) { return None; }
        self.log_access(actor, student_id, "read_transcript", "full");
        let enrs = self.enrollments.lock().unwrap();
        let sections = self.sections.lock().unwrap();
        let courses = self.courses.lock().unwrap();
        let mut entries = Vec::new();
        let mut qpts = 0.0; let mut credits = 0.0;
        for e in enrs.iter().filter(|e| e.student_id == student_id && e.status == SectionEnrollmentStatus::Completed) {
            if let Some(sec) = sections.get(&e.section_id) {
                if let Some(crs) = courses.get(&sec.course_id) {
                    let gp = grade_points(e.final_grade.as_deref());
                    if let Some(g) = gp { qpts += g * crs.credits; credits += crs.credits; }
                    entries.push(serde_json::json!({"course": crs.code, "title": crs.title, "term": sec.term, "credits": crs.credits, "grade": e.final_grade}));
                }
            }
        }
        let gpa = if credits > 0.0 { (qpts / credits * 100.0).round()/100.0 } else { 0.0 };
        Some(serde_json::json!({"student_id": student_id, "completed_courses": entries.len(), "credits": credits, "gpa": gpa, "entries": entries}))
    }

    /// Learning-analytics standing: risk derived from attendance + current grades.
    pub fn analytics(&self, actor: &str, student_id: &str) -> Option<serde_json::Value> {
        if !self.student_exists(student_id) { return None; }
        self.log_access(actor, student_id, "read_analytics", "risk");
        let att = self.attendance_summary(actor, student_id);
        let rate = att["attendance_rate"].as_f64().unwrap_or(100.0);
        let grades = self.grades.lock().unwrap();
        let my: Vec<&Grade> = grades.iter().filter(|g| g.student_id == student_id && g.points_possible > 0.0).collect();
        let avg = if my.is_empty() { 100.0 } else { my.iter().map(|g| g.points_earned / g.points_possible).sum::<f64>() / my.len() as f64 * 100.0 };
        let avg = (avg*10.0).round()/10.0;
        let risk = if avg < 60.0 || rate < 80.0 { "high" } else if avg < 70.0 || rate < 90.0 { "medium" } else { "low" };
        let mut factors = Vec::new();
        if avg < 70.0 { factors.push("low_grade_average"); }
        if rate < 90.0 { factors.push("attendance_concern"); }
        Some(serde_json::json!({
            "student_id": student_id, "grade_average": avg, "attendance_rate": rate,
            "risk_level": risk, "risk_factors": factors,
            "recommendation": match risk { "high" => "open a support case and notify guardians", "medium" => "monitor and offer tutoring", _ => "on track" },
        }))
    }

    // ── seed ────────────────────────────────────────────────────────────

    fn seed(&self) {
        let now = Utc::now();
        let s1 = Student { id: "STU-1001".into(), sis_id: "S44821".into(), family_name: "Mwangi".into(), given_name: "Aisha".into(), grade_level: "10".into(), program: Some("STEM".into()), status: EnrollmentStatus::Active, date_of_birth: NaiveDate::from_ymd_opt(2010, 4, 12), accommodations: vec![], enrolled_on: NaiveDate::from_ymd_opt(2024, 8, 20), created_at: now, updated_at: now };
        let s2 = Student { id: "STU-1002".into(), sis_id: "S90233".into(), family_name: "Lopez".into(), given_name: "Diego".into(), grade_level: "10".into(), program: None, status: EnrollmentStatus::Active, date_of_birth: NaiveDate::from_ymd_opt(2010, 9, 2), accommodations: vec![Accommodation { id: "ACC-seed".into(), kind: "504".into(), description: "Extended time on tests".into(), active: true }], enrolled_on: NaiveDate::from_ymd_opt(2024, 8, 20), created_at: now, updated_at: now };
        { let mut st = self.students.lock().unwrap(); st.insert(s1.id.clone(), s1.clone()); st.insert(s2.id.clone(), s2.clone()); }

        let crs = self.create_course("ALG2", "Algebra II", 1.0, "Mathematics");
        let sec = self.create_section(&crs.id, "2026-Spring", "Ms. Carter", serde_json::json!({"homework":0.3,"exams":0.7})).unwrap();
        let _ = self.enroll(&s1.id, &sec.id, "registrar");
        let hw = self.create_assignment(&sec.id, "HW 1", "homework", 20.0, None).unwrap();
        let ex = self.create_assignment(&sec.id, "Midterm", "exams", 100.0, None).unwrap();
        let _ = self.record_grade(&hw.id, &s1.id, 18.0, vec![], None, "Ms. Carter");
        let _ = self.record_grade(&ex.id, &s1.id, 82.0, vec![], None, "Ms. Carter");
        let _ = self.record_attendance(&s1.id, Some(sec.id.clone()), NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(), AttendanceCode::Present, None, "system");

        let _ = self.add_guardian(&s1.id, "Grace Mwangi", "mother", "ref:grace", true, "registrar");
    }
}

/// Letter grade from a 0–1 fraction.
fn letter_grade(pct: f64) -> &'static str {
    let p = pct * 100.0;
    if p >= 90.0 { "A" } else if p >= 80.0 { "B" } else if p >= 70.0 { "C" } else if p >= 60.0 { "D" } else { "F" }
}

/// 4.0-scale grade points from a letter grade.
fn grade_points(letter: Option<&str>) -> Option<f64> {
    match letter?.trim().to_uppercase().chars().next()? {
        'A' => Some(4.0), 'B' => Some(3.0), 'C' => Some(2.0), 'D' => Some(1.0), 'F' => Some(0.0), _ => None,
    }
}
