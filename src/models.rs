use crate::schema::*;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = teachers)]
pub struct TeacherDb {
    pub id: i32,
    pub name: String,
    pub max_daily_periods: i32,
    pub max_weekly_periods: i32,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = teachers)]
pub struct NewTeacherDb {
    pub name: String,
    pub max_daily_periods: i32,
    pub max_weekly_periods: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = classrooms)]
pub struct ClassroomDb {
    pub id: i32,
    pub name: String,
    pub grade_level: i32,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = classrooms)]
pub struct NewClassroomDb {
    pub name: String,
    pub grade_level: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = subjects)]
pub struct SubjectDb {
    pub id: i32,
    pub name: String,
    pub base_periods_per_week: i32,
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = subjects)]
pub struct NewSubjectDb {
    pub name: String,
    pub base_periods_per_week: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = teacher_qualifications)]
pub struct TeacherQualificationDb {
    pub teacher_id: i32,
    pub subject_id: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = academic_progress)]
pub struct AcademicProgressDb {
    pub classroom_id: i32,
    pub subject_id: i32,
    pub current_progress_ratio: f64,
    pub expected_progress_ratio: f64,
}
