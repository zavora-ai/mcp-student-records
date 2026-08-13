# Changelog

## [1.2.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [1.1.0] - 2026-06-09

### Added — closing gaps that under-served two agents
- **Competency / mastery (unblocks Adaptive Tutor):** learning standards with prerequisites,
  mastery records updated from evidence (rolling score → level), and an adaptive `learning_path`
  that recommends competencies ready to learn (prerequisites met) vs. blocked
  — `add_competency`, `list_competencies`, `record_mastery_evidence`, `get_mastery`, `learning_path`
- **Degree audit & academic standing (unblocks Academic Policy Advisor):** graduation requirements,
  a `degree_audit` (earned vs. required credits per subject, progress %, on-track flag),
  GPA-derived `academic_standing`, and registration/transcript/financial **holds**
  — `add_grad_requirement`, `list_grad_requirements`, `degree_audit`, `academic_standing`,
  `place_hold`, `release_hold`, `get_holds`
- **Early warning & interventions (strengthens Student Support / Learning Analytics):**
  early-warning flags, an `evaluate_early_warning` that auto-raises flags from grades/attendance/mastery,
  and MTSS/RTI **intervention tiers** (1–3)
  — `raise_flag`, `set_flag_status`, `get_flags`, `evaluate_early_warning`,
  `assign_intervention`, `end_intervention`, `get_interventions`
- 19 new tools (now 46 total); all additive — v1.0 API unchanged. New writes are FERPA-logged and gated as appropriate.
- 9 new integration tests (24 total).

## [1.0.0] - 2026-06-09

### Added
- 27 MCP tools for a full Student Information System: 5 students + 5 courses/enrollment + 4 grades + 2 attendance + 5 support + 4 guardians/comms + 2 records/analytics
- Students with demographics, enrollment status, and accommodations (IEP/504/ELL)
- Courses & sections (with grade-category weights), enrollment with duplicate guard
- Grades with rubric scoring (criterion sums) and a weighted gradebook → letter grade
- Attendance records with computed rate
- Support cases with notes and status workflow
- Guardians with FERPA consent; communications gated on consent + association
- Transcript with cumulative GPA (4.0 scale); learning-analytics risk from grades + attendance
- **FERPA access logging** on every student-scoped operation (accounting of disclosures)
- Attributable access (actor required); send_communication classed external_write
- Modeled as a general SIS platform; education agents (academic policy, adaptive tutor,
  learning analytics, parent communication, rubric grading, student support) are clients of it
- Gated FERPA-sensitive writes (status, accommodation, enroll, grade, attendance, guardian, communication)
- `adk-mcp-sdk` HealthCheck + validated `mcp-server.toml` manifest
- 14 tests (10 store + 4 manifest); verified end-to-end over MCP stdio
