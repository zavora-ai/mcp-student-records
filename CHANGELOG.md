# Changelog

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
