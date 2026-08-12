pub mod schema;
pub mod models;
pub mod importer;
pub mod db;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ==========================================
// Strongly Typed IDs & Basic Enums
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeacherId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClassroomId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubjectId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    Sunday, // Standard Middle Eastern school week starts Sunday
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeSlot {
    pub day: DayOfWeek,
    pub period: u8, // Period 1, 2, 3...
}

// ==========================================
// Primary Data Models
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: SubjectId,
    pub name: String,
    /// Default required periods per week for a normal class
    pub base_periods_per_week: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
    pub id: TeacherId,
    pub name: String,
    /// Subjects this teacher is qualified to teach
    pub qualified_subjects: HashSet<SubjectId>,
    /// Preferred periods (Soft Constraint)
    pub preferences: Vec<TimeSlotPreference>,
    /// Max periods this teacher can teach per day/week
    pub max_daily_periods: u8,
    pub max_weekly_periods: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classroom {
    pub id: ClassroomId,
    /// e.g. "Grade 10 - Section A"
    pub name: String,
    pub grade_level: u8,
}

// ==========================================
// Academic Progress & Dynamic Scheduling
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicProgress {
    pub classroom_id: ClassroomId,
    pub subject_id: SubjectId,
    /// Current curriculum progress ratio (e.g. 0.75 = 75% complete)
    pub current_progress_ratio: f32,
    /// Targeted ratio by the current date
    pub expected_progress_ratio: f32,
}

impl AcademicProgress {
    /// Calculate adjusted weekly periods based on whether the class is behind schedule
    pub fn calculate_required_periods(&self, base_periods: u8) -> u8 {
        if self.current_progress_ratio < self.expected_progress_ratio {
            let lag_factor = self.expected_progress_ratio - self.current_progress_ratio;
            // Boost periods if falling significantly behind (e.g., +1 or +2 extra slots/week)
            base_periods + (lag_factor * 4.0).round() as u8
        } else {
            base_periods
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalendarEvent {
    NationalHoliday(NaiveDate),
    EmergencyBreak {
        start: NaiveDate,
        end: NaiveDate,
        reason: String,
    },
}

// ==========================================
// Constraints (Hard & Soft)
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlotPreference {
    pub slot: TimeSlot,
    /// Positive values = Preferred, Negative values = Disliked/Unavailable
    pub preference_score: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardConstraint {
    /// A teacher cannot teach two classes at the same slot
    TeacherNoOverlap,
    /// A classroom can only have one teacher at a time
    ClassroomNoOverlap,
    /// Fixed periods (e.g. Sunday Period 1 is always Morning Assembly)
    FixedSlot {
        classroom_id: ClassroomId,
        subject_id: SubjectId,
        slot: TimeSlot,
    },
    /// Teacher cannot teach more than N consecutive periods
    MaxConsecutivePeriods {
        teacher_id: TeacherId,
        max_periods: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SoftConstraint {
    /// Honor teacher preferred/disliked time slots
    TeacherPreference { teacher_id: TeacherId, weight: u32 },
    /// Minimize gaps/free periods in a teacher's middle of the day
    MinimizeTeacherGaps { teacher_id: TeacherId, weight: u32 },
    /// Spread subject evenly throughout the week (don't stack Math 3 times on Sunday)
    DistributeSubjectEvenly {
        classroom_id: ClassroomId,
        subject_id: SubjectId,
        weight: u32,
    },
}

// ==========================================
// Solution / Timetable Output Structure
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleAssignment {
    pub classroom_id: ClassroomId,
    pub subject_id: SubjectId,
    pub teacher_id: TeacherId,
    pub slot: TimeSlot,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Timetable {
    pub assignments: Vec<ScheduleAssignment>,
}

impl Timetable {
    /// Quick lookup helper to check if a teacher is busy at a given slot
    pub fn get_teacher_assignment(
        &self,
        teacher_id: TeacherId,
        slot: TimeSlot,
    ) -> Option<&ScheduleAssignment> {
        self.assignments
            .iter()
            .find(|a| a.teacher_id == teacher_id && a.slot == slot)
    }
}

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "classes_scheduler")]
#[command(about = "Classes Scheduler CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Import configuration from excel or csv file
    Import {
        /// Path to the input file
        #[arg(short, long)]
        file: String,

        /// Format of the input file (excel or csv)
        #[arg(short, long, value_enum)]
        format: Format,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Format {
    Excel,
    Csv,
}

use diesel::Connection;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();

    match args.command {
        Commands::Import { file, format } => {
            println!("Importing from {} with format {:?}", file, format);
            
            // 1. Parse file
            let imported_data = match format {
                Format::Excel => importer::excel::parse_excel(&file)?,
                Format::Csv => importer::csv_parser::parse_csv(&file)?,
            };

            // 2. Connect to database
            let database_url = std::env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable must be set"))?;
            
            let mut conn = diesel::pg::PgConnection::establish(&database_url)
                .map_err(|e| anyhow::anyhow!("Error connecting to database {}: {:?}", database_url, e))?;

            // 3. Bulk import
            db::repo::bulk_import(&mut conn, imported_data)?;
            println!("Successfully imported configuration into database.");
        }
    }

    Ok(())
}
