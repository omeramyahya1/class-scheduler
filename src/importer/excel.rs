use calamine::{Reader, Xlsx, open_workbook, Data, Range, DataType};
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use super::*;

pub fn parse_excel<P: AsRef<Path>>(path: P) -> Result<ImportedData> {
    let mut workbook: Xlsx<_> = open_workbook(path).context("Failed to open excel workbook")?;
    
    let teachers = parse_teachers_sheet(&mut workbook)?;
    let classrooms = parse_classrooms_sheet(&mut workbook)?;
    let subjects = parse_subjects_sheet(&mut workbook)?;
    let progress = parse_progress_sheet(&mut workbook)?;

    Ok(ImportedData::Excel {
        teachers,
        classrooms,
        subjects,
        progress,
    })
}

fn get_sheet_range(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>, name: &str) -> Result<Range<Data>> {
    let sheet_name = workbook.sheet_names().iter()
        .find(|s| s.to_lowercase() == name.to_lowercase())
        .cloned()
        .ok_or_else(|| anyhow!("Sheet '{}' not found in workbook", name))?;
    
    workbook.worksheet_range(&sheet_name)
        .map_err(|e| anyhow!("Error reading worksheet '{}': {:?}", sheet_name, e))
}

fn find_header(headers: &[Data], aliases: &[&str]) -> Result<usize> {
    for (i, cell) in headers.iter().enumerate() {
        let val = cell.to_string().trim().to_lowercase().replace(' ', "_").replace('-', "_");
        for alias in aliases {
            if val == *alias {
                return Ok(i);
            }
        }
    }
    Err(anyhow!("Could not find column matching any alias in {:?}", aliases))
}

fn get_string_val(cell: &Data) -> Result<String> {
    match cell {
        Data::String(s) => Ok(s.clone()),
        Data::Int(i) => Ok(i.to_string()),
        Data::Float(f) => Ok(f.to_string()),
        Data::Bool(b) => Ok(b.to_string()),
        _ => Err(anyhow!("Expected string/numeric cell, found {:?}", cell)),
    }
}

fn get_int_val(cell: &Data) -> Result<i32> {
    match cell {
        Data::Int(i) => Ok(*i as i32),
        Data::Float(f) => Ok(*f as i32),
        Data::String(s) => s.parse::<i32>().map_err(|e| anyhow!("Failed to parse integer from '{}': {}", s, e)),
        Data::Bool(b) => Ok(if *b { 1 } else { 0 }),
        _ => Err(anyhow!("Expected integer/numeric cell, found {:?}", cell)),
    }
}

fn get_float_val(cell: &Data) -> Result<f64> {
    match cell {
        Data::Float(f) => Ok(*f),
        Data::Int(i) => Ok(*i as f64),
        Data::String(s) => s.parse::<f64>().map_err(|e| anyhow!("Failed to parse float from '{}': {}", s, e)),
        _ => Err(anyhow!("Expected float/numeric cell, found {:?}", cell)),
    }
}

fn parse_teachers_sheet(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Vec<TeacherImport>> {
    let range = match get_sheet_range(workbook, "Teachers") {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };
    
    let mut iter = range.rows();
    let headers = match iter.next() {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let name_idx = find_header(headers, &["name"])?;
    let daily_idx = find_header(headers, &["max_daily_periods", "daily_periods", "max_daily"])?;
    let weekly_idx = find_header(headers, &["max_weekly_periods", "weekly_periods", "max_weekly"])?;
    let qual_idx = find_header(headers, &["qualified_subjects", "subjects", "qualification"])?;

    let mut teachers = Vec::new();
    for row in iter {
        if row.is_empty() || row.iter().all(|c| c.is_empty()) {
            continue;
        }
        let name = get_string_val(&row[name_idx])?;
        let max_daily_periods = get_int_val(&row[daily_idx])?;
        let max_weekly_periods = get_int_val(&row[weekly_idx])?;
        let qualified_raw = get_string_val(&row[qual_idx]).unwrap_or_default();
        let qualified_subjects = qualified_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        teachers.push(TeacherImport {
            name,
            max_daily_periods,
            max_weekly_periods,
            qualified_subjects,
        });
    }

    Ok(teachers)
}

fn parse_classrooms_sheet(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Vec<ClassroomImport>> {
    let range = match get_sheet_range(workbook, "Classrooms") {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };
    
    let mut iter = range.rows();
    let headers = match iter.next() {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let name_idx = find_header(headers, &["name", "classroom", "class"])?;
    let grade_idx = find_header(headers, &["grade_level", "grade", "level"])?;

    let mut classrooms = Vec::new();
    for row in iter {
        if row.is_empty() || row.iter().all(|c| c.is_empty()) {
            continue;
        }
        let name = get_string_val(&row[name_idx])?;
        let grade_level = get_int_val(&row[grade_idx])?;
        classrooms.push(ClassroomImport { name, grade_level });
    }
    Ok(classrooms)
}

fn parse_subjects_sheet(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Vec<SubjectImport>> {
    let range = match get_sheet_range(workbook, "Subjects") {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };
    
    let mut iter = range.rows();
    let headers = match iter.next() {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let name_idx = find_header(headers, &["name", "subject"])?;
    let base_idx = find_header(headers, &["base_periods_per_week", "periods_per_week", "base_periods", "periods"])?;

    let mut subjects = Vec::new();
    for row in iter {
        if row.is_empty() || row.iter().all(|c| c.is_empty()) {
            continue;
        }
        let name = get_string_val(&row[name_idx])?;
        let base_periods_per_week = get_int_val(&row[base_idx])?;
        subjects.push(SubjectImport { name, base_periods_per_week });
    }
    Ok(subjects)
}

fn parse_progress_sheet(workbook: &mut Xlsx<std::io::BufReader<std::fs::File>>) -> Result<Vec<AcademicProgressImport>> {
    let range = match get_sheet_range(workbook, "Progress")
        .or_else(|_| get_sheet_range(workbook, "AcademicProgress"))
        .or_else(|_| get_sheet_range(workbook, "Academic Progress")) 
    {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };
    
    let mut iter = range.rows();
    let headers = match iter.next() {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    let class_idx = find_header(headers, &["classroom_name", "classroom", "class"])?;
    let sub_idx = find_header(headers, &["subject_name", "subject"])?;
    let cur_idx = find_header(headers, &["current_progress_ratio", "current_progress", "current", "progress"])?;
    let exp_idx = find_header(headers, &["expected_progress_ratio", "expected_progress", "expected"])?;

    let mut progress = Vec::new();
    for row in iter {
        if row.is_empty() || row.iter().all(|c| c.is_empty()) {
            continue;
        }
        let classroom_name = get_string_val(&row[class_idx])?;
        let subject_name = get_string_val(&row[sub_idx])?;
        let current_progress_ratio = get_float_val(&row[cur_idx])?;
        let expected_progress_ratio = get_float_val(&row[exp_idx])?;
        progress.push(AcademicProgressImport {
            classroom_name,
            subject_name,
            current_progress_ratio,
            expected_progress_ratio,
        });
    }
    Ok(progress)
}
