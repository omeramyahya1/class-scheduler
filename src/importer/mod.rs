pub mod excel;
pub mod csv_parser;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherImport {
    pub name: String,
    pub max_daily_periods: i32,
    pub max_weekly_periods: i32,
    pub qualified_subjects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassroomImport {
    pub name: String,
    pub grade_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectImport {
    pub name: String,
    pub base_periods_per_week: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicProgressImport {
    pub classroom_name: String,
    pub subject_name: String,
    pub current_progress_ratio: f64,
    pub expected_progress_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportedData {
    Excel {
        teachers: Vec<TeacherImport>,
        classrooms: Vec<ClassroomImport>,
        subjects: Vec<SubjectImport>,
        progress: Vec<AcademicProgressImport>,
    },
    Teachers(Vec<TeacherImport>),
    Classrooms(Vec<ClassroomImport>),
    Subjects(Vec<SubjectImport>),
    Progress(Vec<AcademicProgressImport>),
}
