# Class Scheduler CLI

`class_scheduler` is a high-performance command-line tool written in Rust designed to solve Constraint Satisfaction Problems (CSP) for Middle Eastern school timetabling.

---

## 1. Core Domain Context & Invariants

Middle Eastern school scheduling has unique organizational rules:
- **Stationary Classrooms:** Students remain in a single, fixed classroom cohort (e.g. Grade 10 - Section A) for the entire academic year. Room allocation is static; scheduling revolves entirely around allocating **teachers** to **classrooms** for specific **time slots**.
- **Academic Week:** Sunday through Thursday (with optional Saturday).
- **Strong Typing:** The system uses distinct wrapper types (`TeacherId`, `ClassroomId`, `SubjectId`) rather than raw integers to prevent entity assignment mistakes.

---

## 2. Directory Structure

```
class_scheduler/
├── .agents/              # Workspace Agent Customizations
│   ├── AGENT.md          # Developer role, principles, style guidelines
│   ├── artifacts/        # Reports, summaries, and diagrams
│   │   ├── comprehension_report.md
│   │   └── db_importer_setup.md
│   ├── rules/
│   │   └── rust_guidelines.md # Enforces Cargo, Rust features + context sync
│   ├── skills/
│   │   └── SKILL.md      # Commands, Excel parsing & constraint details
│   ├── specs/
│   │   └── domain.md     # Domain entities, constraint & DB schemas
│   └── wiki/
│       └── memory.md     # Project state & workspace memory
├── migrations/           # Diesel ORM PostgreSQL migrations
│   └── 2026-08-12-000000_init/
│       ├── up.sql        # Database tables setup
│       └── down.sql      # Database teardown
├── src/                  # Rust source code
│   ├── db/
│   │   ├── mod.rs        # DB module declarations
│   │   └── repo.rs       # Transactional bulk-upsert repo logic
│   ├── importer/
│   │   ├── mod.rs        # Ingestion submodules & DTO declarations
│   │   ├── excel.rs      # calamine Excel sheet parser
│   │   └── csv_parser.rs # Header-detecting CSV entity parser
│   ├── schema.rs         # Diesel-generated schema macros
│   ├── models.rs         # Diesel ORM model mappings
│   └── main.rs           # Core domain models, CLI entry point & clap setup
├── Cargo.toml            # Dependencies and features config
├── docker-compose.yml    # PostgreSQL container configuration
├── .env                  # Local environment database credentials
└── .env.example          # Environment template
```

---

## 3. Database Layer & Containerization

### Docker Setup
A containerized PostgreSQL 16 database service is defined in `docker-compose.yml`:
- **User:** `scheduler_user`
- **Password:** `scheduler_password`
- **Database:** `classes_scheduler`
- **Port:** `5432`

To start the database container, run:
```bash
docker compose up -d
```

### Diesel Schema Definition
The database layer maps core domain entities into five relational tables:
- **`teachers`**: Holds teachers and their daily/weekly workload limits.
- **`classrooms`**: Cohorts/Sections with unique names and grade levels.
- **`subjects`**: Courses taught with unique names and base hours per week.
- **`teacher_qualifications`**: Joint table mapping teachers to subjects they are qualified to teach.
- **`academic_progress`**: Tracks curriculum pacing ratios per classroom-subject.

---

## 4. Ingestion Engine (`importer`)

The system handles both workbook spreadsheets and single entity table files:
- **Excel Ingest (`src/importer/excel.rs`):** Parses multi-sheet Excel workbooks (`Teachers`, `Classrooms`, `Subjects`, `Progress` sheets) using the `calamine` crate. Columns are dynamically resolved using header alias search.
- **CSV Ingest (`src/importer/csv_parser.rs`):** Inspects the headers of a `.csv` file and maps it automatically to the correct entity DTO (Teachers, Classrooms, Subjects, or Progress).
- **Transactional Repository (`src/db/repo.rs`):** Inserts bulk-ingested data within a single database `transaction` block to guarantee atomicity, executing upserts (`on_conflict`) where rows overlap.

---

## 5. Command-Line Interface (CLI)

The CLI is powered by the `clap` crate.

### Ingestion CLI Subcommand
To import school configurations from an Excel workbook:
```bash
cargo run -- import --file path/to/input.xlsx --format excel
```

To import individual CSV entities:
```bash
cargo run -- import --file path/to/teachers.csv --format csv
```
