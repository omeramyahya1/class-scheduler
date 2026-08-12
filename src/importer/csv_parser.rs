use std::path::Path;
use anyhow::{Result, Context, anyhow};
use csv::ReaderBuilder;
use super::*;

pub fn parse_csv<P: AsRef<Path>>(path: P) -> Result<ImportedData> {
    let file = std::fs::File::open(path).context("Failed to open CSV file")?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let headers = rdr.headers().context("Failed to parse CSV headers")?;
    let headers_normalized: Vec<String> = headers.iter()
        .map(|h| h.trim().to_lowercase().replace(' ', "_").replace('-', "_"))
        .collect();

    // Check which entity matches these headers
    if headers_normalized.iter().any(|h| h == "max_daily_periods" || h == "max_weekly_periods" || h == "qualified_subjects") {
        let mut teachers = Vec::new();
        let name_idx = find_header_csv(&headers_normalized, &["name"])?;
        let daily_idx = find_header_csv(&headers_normalized, &["max_daily_periods", "daily_periods", "max_daily"])?;
        let weekly_idx = find_header_csv(&headers_normalized, &["max_weekly_periods", "weekly_periods", "max_weekly"])?;
        let qual_idx = find_header_csv(&headers_normalized, &["qualified_subjects", "subjects", "qualification"])?;

        for result in rdr.records() {
            let record = result.context("Failed to read CSV record")?;
            let name = record.get(name_idx).unwrap_or_default().trim().to_string();
            let max_daily_periods = record.get(daily_idx).unwrap_or_default().parse::<i32>().unwrap_or(0);
            let max_weekly_periods = record.get(weekly_idx).unwrap_or_default().parse::<i32>().unwrap_or(0);
            let qualified_raw = record.get(qual_idx).unwrap_or_default().trim();
            let qualified_subjects = if qualified_raw.is_empty() {
                Vec::new()
            } else {
                qualified_raw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };

            teachers.push(TeacherImport {
                name,
                max_daily_periods,
                max_weekly_periods,
                qualified_subjects,
            });
        }
        Ok(ImportedData::Teachers(teachers))
    } else if headers_normalized.iter().any(|h| h == "grade_level" || h == "classroom" || h == "class") {
        let mut classrooms = Vec::new();
        let name_idx = find_header_csv(&headers_normalized, &["name", "classroom", "class"])?;
        let grade_idx = find_header_csv(&headers_normalized, &["grade_level", "grade", "level"])?;

        for result in rdr.records() {
            let record = result.context("Failed to read CSV record")?;
            let name = record.get(name_idx).unwrap_or_default().trim().to_string();
            let grade_level = record.get(grade_idx).unwrap_or_default().parse::<i32>().unwrap_or(0);
            classrooms.push(ClassroomImport { name, grade_level });
        }
        Ok(ImportedData::Classrooms(classrooms))
    } else if headers_normalized.iter().any(|h| h == "base_periods_per_week" || h == "periods_per_week" || h == "base_periods") {
        let mut subjects = Vec::new();
        let name_idx = find_header_csv(&headers_normalized, &["name", "subject"])?;
        let base_idx = find_header_csv(&headers_normalized, &["base_periods_per_week", "periods_per_week", "base_periods", "periods"])?;

        for result in rdr.records() {
            let record = result.context("Failed to read CSV record")?;
            let name = record.get(name_idx).unwrap_or_default().trim().to_string();
            let base_periods_per_week = record.get(base_idx).unwrap_or_default().parse::<i32>().unwrap_or(0);
            subjects.push(SubjectImport { name, base_periods_per_week });
        }
        Ok(ImportedData::Subjects(subjects))
    } else if headers_normalized.iter().any(|h| h == "current_progress_ratio" || h == "expected_progress_ratio" || h == "current_progress") {
        let mut progress = Vec::new();
        let class_idx = find_header_csv(&headers_normalized, &["classroom_name", "classroom", "class"])?;
        let sub_idx = find_header_csv(&headers_normalized, &["subject_name", "subject"])?;
        let cur_idx = find_header_csv(&headers_normalized, &["current_progress_ratio", "current_progress", "current", "progress"])?;
        let exp_idx = find_header_csv(&headers_normalized, &["expected_progress_ratio", "expected_progress", "expected"])?;

        for result in rdr.records() {
            let record = result.context("Failed to read CSV record")?;
            let classroom_name = record.get(class_idx).unwrap_or_default().trim().to_string();
            let subject_name = record.get(sub_idx).unwrap_or_default().trim().to_string();
            let current_progress_ratio = record.get(cur_idx).unwrap_or_default().parse::<f64>().unwrap_or(0.0);
            let expected_progress_ratio = record.get(exp_idx).unwrap_or_default().parse::<f64>().unwrap_or(0.0);
            progress.push(AcademicProgressImport {
                classroom_name,
                subject_name,
                current_progress_ratio,
                expected_progress_ratio,
            });
        }
        Ok(ImportedData::Progress(progress))
    } else {
        Err(anyhow!("Could not detect CSV layout based on headers: {:?}", headers))
    }
}

fn find_header_csv(headers: &[String], aliases: &[&str]) -> Result<usize> {
    for (i, val) in headers.iter().enumerate() {
        for alias in aliases {
            if val == alias {
                return Ok(i);
            }
        }
    }
    Err(anyhow!("Could not find column matching any alias in {:?}", aliases))
}
