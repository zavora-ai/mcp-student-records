# Student Records MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-student-records.svg)](https://crates.io/crates/mcp-student-records)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

A full Student Information System (SIS) for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) education agents. 46 MCP tools covering students, courses & enrollment, grades & rubric scoring, attendance, support cases, guardians & communications, transcripts, and learning analytics — with **FERPA access logging on every student-scoped operation**.

## A platform, not a point solution

This is modeled as a general SIS (à la Banner / PowerSchool / Workday Student), so a wide range of education agents are simply clients: Academic Policy Advisor, Adaptive Tutor, Learning Analytics, Parent Communication, Rubric Grading Assistant, and Student Support — plus anything else that needs to read or update the student record under proper governance.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-student-records/main/docs/architecture.svg" alt="Student Records MCP Architecture" width="750"/>
</p>

## Capabilities

- **Students** — demographics, enrollment status, accommodations (IEP/504/ELL)
- **Courses & sections** — catalog + term offerings with grade-category weights
- **Enrollment** — student↔section with status
- **Grades & rubrics** — assignment grades, **rubric scoring** (criterion sums), and a **weighted gradebook** → letter grade
- **Attendance** — records + computed rate
- **Support** — intervention cases with notes and status
- **Guardians & communications** — guardian links with consent; messaging
- **Transcript** — completed-course history + cumulative GPA (4.0 scale)
- **Learning analytics** — risk standing derived from grade average + attendance

## FERPA posture

- **Every student-scoped read or write is logged** to a per-student access log (accounting of disclosures); `get_access_log` returns it.
- **Attributable access** — every student-scoped tool requires an `actor`.
- **Guardian communications are consent-gated** — `send_communication` refuses unless the guardian belongs to the student and has consent on file; it's classed `external_write` and gated.
- **Minimal PII** — guardians carry a contact *reference*, not raw contact details; sample data is fictitious.

## Tools (27)

### Students (5)
`find_students` · `get_student` · `set_student_status` (gated) · `add_accommodation` (gated) · `get_access_log`

### Courses, Sections & Enrollment (5)
`create_course` · `list_courses` · `create_section` · `enroll` (gated) · `get_enrollments`

### Grades & Gradebook (4)
`create_assignment` · `record_grade` (gated; rubric-aware) · `get_grades` · `gradebook`

### Attendance (2)
`record_attendance` (gated) · `attendance_summary`

### Support (5)
`open_support_case` · `get_support_case` · `add_case_note` · `set_case_status` · `get_support_cases`

### Guardians & Communications (4)
`add_guardian` (gated) · `get_guardians` · `send_communication` (FERPA-gated, external) · `get_communications`

### Records & Analytics (2)
`get_transcript` · `analytics`

### Competency & Mastery — Adaptive Tutor (5)
`add_competency` · `list_competencies` · `record_mastery_evidence` (gated) · `get_mastery` · `learning_path`

Mastery is tracked per learning standard (with prerequisites). `learning_path` recommends the competencies a student is **ready to learn next** — not yet proficient, prerequisites met — and what's blocked, giving an Adaptive Tutor real mastery data instead of letter grades.

### Degree Audit & Standing — Academic Policy Advisor (7)
`add_grad_requirement` · `list_grad_requirements` · `degree_audit` · `academic_standing` · `place_hold` (gated) · `release_hold` (gated) · `get_holds`

`degree_audit` reports earned-vs-required credits per subject for the student's program (progress %, on-track flag); `academic_standing` derives honor-roll/probation from GPA; holds gate registration/transcript/financial actions.

### Early Warning & Interventions — Student Support / Learning Analytics (7)
`raise_flag` · `set_flag_status` · `get_flags` · `evaluate_early_warning` · `assign_intervention` (gated) · `end_intervention` · `get_interventions`

`evaluate_early_warning` auto-raises flags from grades, attendance, and mastery; interventions follow MTSS/RTI tiers (1–3).

## Example

```
> find_students(actor: "ms.carter", name: "mwangi")        → STU-1001 Mwangi, Aisha
> gradebook(actor: "ms.carter", student_id: "STU-1001", section_id: "SEC-…")
  → overall 84.4% = B   (HW 90% × 0.3 + Midterm 82% × 0.7)

> analytics(actor: "counselor", student_id: "STU-1001")
  → grade_average 86 · attendance_rate 100 · risk_level "low"

> send_communication(student_id: "STU-1001", guardian_id: "GRD-…", subject: "Progress", body: "…", sent_by: "parent.agent")
  → COM-1010   (allowed: guardian has consent on file)

> get_access_log(actor: "audit", student_id: "STU-1001")
  → 13 entries — full accounting of disclosures
```

Rubric grading:
```
> record_grade(assignment_id: "ASG-…", student_id: "STU-1001",
    rubric: [{name:"Thesis",max_points:10,awarded:8}, {name:"Evidence",max_points:10,awarded:9}])
  → points_earned 17 (sum of awarded criteria)
```

## Installation

### 1. Build

```bash
git clone https://github.com/zavora-ai/mcp-student-records
cd mcp-student-records
cargo build --release
```

### 2. Add to your MCP client

**Claude Desktop / Kiro / Cursor / Windsurf:**
```json
{
  "mcpServers": {
    "student-records": {
      "command": "/path/to/mcp-student-records"
    }
  }
}
```

### 3. Use it

```
> find_students(actor: "advisor", grade_level: "10")
> get_transcript(actor: "registrar", student_id: "STU-1001")
```

## Data Handling

- **FERPA by construction** — access logging, attributable access, and consent-gated guardian communications are built in.
- **This is an integration scaffold, not a certified SIS.** In production, back it with your real SIS of record and enforce auth, RBAC (role-appropriate fields), and encryption; bind `actor` to an authenticated identity.
- **No real student PII** ships in the crate; sample data is fictitious.

## MCP Server Manifest

```toml
server_id = "mcp_student_records"
display_name = "Student Records (SIS)"
version = "1.1.0"
domain = "education"
risk_level = "high"
writes_allowed = "gated"
transports = ["stdio"]
```

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and approval gates
- **Structured tracing** — `RUST_LOG` env-filter for observability

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.94.1 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.

## MCP 2026-07-28 rollout (P3 regulated)

This server uses `rmcp` 3.1.2 and `adk-mcp-sdk` 0.2 with a minimum supported
Rust version of **1.94.1**. It accepts stateless MCP 2026 requests with
per-request protocol, client identity, and capability metadata while retaining
the legacy MCP 2025-11-25 initialize flow for ordinary tools.

- **Tasks:** None; this server's operations are short-lived and execute directly.
- **MRTR approvals:** `set_student_status`, `add_accommodation`, `create_course`, `create_section`, `enroll`, `create_assignment`, `record_grade`, `record_attendance`, `open_support_case`, `add_case_note`, `set_case_status`, `add_guardian`, `send_communication`, `add_competency`, `record_mastery_evidence`, `add_grad_requirement`, `place_hold`, `release_hold`, `raise_flag`, `set_flag_status`, `evaluate_early_warning`, `assign_intervention`, `end_intervention`
- **Discovery and routing:** rmcp serves on-demand discovery and validates the
  per-request protocol envelope; HTTP deployments can route with `Mcp-Method`
  and `Mcp-Name`. The packaged binary currently uses stdio.
- **Caching:** `tools/list` returns a public `ttlMs` of 60,000 for MCP 2026;
  rmcp omits the cache fields for legacy clients.
- **Deprecated extensions:** this server does not add new Roots, Sampling, or
  dynamic client-registration dependencies.

Protected tools require `MCP_REQUEST_STATE_KEY` with at least 32 high-entropy
bytes. All replicas must share that key so sealed approval state can resume on
another instance. Approval state is bound to the client identity, tool, and
arguments and expires after two minutes. Missing identity, invalid state,
rejection, or legacy protocol use fails closed. Task records are process-local
for the current stdio runtime; use a durable task store before deploying the
server behind scale-to-zero HTTP infrastructure.
