use crate::models::*;
use crate::schema::*;
use crate::importer::ImportedData;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use anyhow::Result;

pub fn bulk_import(conn: &mut PgConnection, data: ImportedData) -> Result<()> {
    conn.transaction::<_, anyhow::Error, _>(|conn| {
        match data {
            ImportedData::Excel { teachers, classrooms, subjects, progress } => {
                // 1. Insert subjects
                let mut subject_name_to_id = std::collections::HashMap::new();
                for sub in subjects {
                    let inserted_sub: SubjectDb = diesel::insert_into(subjects::table)
                        .values((
                            subjects::name.eq(&sub.name),
                            subjects::base_periods_per_week.eq(sub.base_periods_per_week),
                        ))
                        .on_conflict(subjects::name)
                        .do_update()
                        .set(subjects::base_periods_per_week.eq(sub.base_periods_per_week))
                        .get_result(conn)?;
                    subject_name_to_id.insert(sub.name.clone(), inserted_sub.id);
                }

                // 2. Insert classrooms
                let mut classroom_name_to_id = std::collections::HashMap::new();
                for cls in classrooms {
                    let inserted_cls: ClassroomDb = diesel::insert_into(classrooms::table)
                        .values((
                            classrooms::name.eq(&cls.name),
                            classrooms::grade_level.eq(cls.grade_level),
                        ))
                        .on_conflict(classrooms::name)
                        .do_update()
                        .set(classrooms::grade_level.eq(cls.grade_level))
                        .get_result(conn)?;
                    classroom_name_to_id.insert(cls.name.clone(), inserted_cls.id);
                }

                // 3. Insert teachers & qualifications
                for t in teachers {
                    let existing_teacher: Option<TeacherDb> = teachers::table
                        .filter(teachers::name.eq(&t.name))
                        .first(conn)
                        .optional()?;

                    let teacher_id = match existing_teacher {
                        Some(ext) => {
                            diesel::update(teachers::table.filter(teachers::id.eq(ext.id)))
                                .set((
                                    teachers::max_daily_periods.eq(t.max_daily_periods),
                                    teachers::max_weekly_periods.eq(t.max_weekly_periods),
                                ))
                                .get_result::<TeacherDb>(conn)?
                                .id
                        }
                        None => {
                            diesel::insert_into(teachers::table)
                                .values((
                                    teachers::name.eq(&t.name),
                                    teachers::max_daily_periods.eq(t.max_daily_periods),
                                    teachers::max_weekly_periods.eq(t.max_weekly_periods),
                                ))
                                .get_result::<TeacherDb>(conn)?
                                .id
                        }
                    };

                    for sub_name in &t.qualified_subjects {
                        let sub_id = if let Some(&id) = subject_name_to_id.get(sub_name) {
                            id
                        } else {
                            let inserted_sub: SubjectDb = diesel::insert_into(subjects::table)
                                .values((
                                    subjects::name.eq(sub_name),
                                    subjects::base_periods_per_week.eq(5),
                                ))
                                .on_conflict(subjects::name)
                                .do_update()
                                .set(subjects::name.eq(sub_name))
                                .get_result(conn)?;
                            subject_name_to_id.insert(sub_name.clone(), inserted_sub.id);
                            inserted_sub.id
                        };

                        diesel::insert_into(teacher_qualifications::table)
                            .values((
                                teacher_qualifications::teacher_id.eq(teacher_id),
                                teacher_qualifications::subject_id.eq(sub_id),
                            ))
                            .on_conflict((teacher_qualifications::teacher_id, teacher_qualifications::subject_id))
                            .do_nothing()
                            .execute(conn)?;
                    }
                }

                // 4. Insert progress
                for prg in progress {
                    let classroom_id = if let Some(&id) = classroom_name_to_id.get(&prg.classroom_name) {
                        id
                    } else {
                        let inserted_cls: ClassroomDb = diesel::insert_into(classrooms::table)
                            .values((
                                classrooms::name.eq(&prg.classroom_name),
                                classrooms::grade_level.eq(10),
                            ))
                            .on_conflict(classrooms::name)
                            .do_update()
                            .set(classrooms::name.eq(&prg.classroom_name))
                            .get_result(conn)?;
                        classroom_name_to_id.insert(prg.classroom_name.clone(), inserted_cls.id);
                        inserted_cls.id
                    };

                    let subject_id = if let Some(&id) = subject_name_to_id.get(&prg.subject_name) {
                        id
                    } else {
                        let inserted_sub: SubjectDb = diesel::insert_into(subjects::table)
                            .values((
                                subjects::name.eq(&prg.subject_name),
                                subjects::base_periods_per_week.eq(5),
                            ))
                            .on_conflict(subjects::name)
                            .do_update()
                            .set(subjects::name.eq(&prg.subject_name))
                            .get_result(conn)?;
                        subject_name_to_id.insert(prg.subject_name.clone(), inserted_sub.id);
                        inserted_sub.id
                    };

                    diesel::insert_into(academic_progress::table)
                        .values((
                            academic_progress::classroom_id.eq(classroom_id),
                            academic_progress::subject_id.eq(subject_id),
                            academic_progress::current_progress_ratio.eq(prg.current_progress_ratio),
                            academic_progress::expected_progress_ratio.eq(prg.expected_progress_ratio),
                        ))
                        .on_conflict((academic_progress::classroom_id, academic_progress::subject_id))
                        .do_update()
                        .set((
                            academic_progress::current_progress_ratio.eq(prg.current_progress_ratio),
                            academic_progress::expected_progress_ratio.eq(prg.expected_progress_ratio),
                        ))
                        .execute(conn)?;
                }
            }
            ImportedData::Teachers(teachers) => {
                for t in teachers {
                    let existing_teacher: Option<TeacherDb> = teachers::table
                        .filter(teachers::name.eq(&t.name))
                        .first(conn)
                        .optional()?;

                    let teacher_id = match existing_teacher {
                        Some(ext) => {
                            diesel::update(teachers::table.filter(teachers::id.eq(ext.id)))
                                .set((
                                    teachers::max_daily_periods.eq(t.max_daily_periods),
                                    teachers::max_weekly_periods.eq(t.max_weekly_periods),
                                ))
                                .get_result::<TeacherDb>(conn)?
                                .id
                        }
                        None => {
                            diesel::insert_into(teachers::table)
                                .values((
                                    teachers::name.eq(&t.name),
                                    teachers::max_daily_periods.eq(t.max_daily_periods),
                                    teachers::max_weekly_periods.eq(t.max_weekly_periods),
                                ))
                                .get_result::<TeacherDb>(conn)?
                                .id
                        }
                    };

                    for sub_name in &t.qualified_subjects {
                        let inserted_sub: SubjectDb = diesel::insert_into(subjects::table)
                            .values((
                                subjects::name.eq(sub_name),
                                subjects::base_periods_per_week.eq(5),
                            ))
                            .on_conflict(subjects::name)
                            .do_update()
                            .set(subjects::name.eq(sub_name))
                            .get_result(conn)?;

                        diesel::insert_into(teacher_qualifications::table)
                            .values((
                                teacher_qualifications::teacher_id.eq(teacher_id),
                                teacher_qualifications::subject_id.eq(inserted_sub.id),
                            ))
                            .on_conflict((teacher_qualifications::teacher_id, teacher_qualifications::subject_id))
                            .do_nothing()
                            .execute(conn)?;
                    }
                }
            }
            ImportedData::Classrooms(classrooms) => {
                for cls in classrooms {
                    diesel::insert_into(classrooms::table)
                        .values((
                            classrooms::name.eq(&cls.name),
                            classrooms::grade_level.eq(cls.grade_level),
                        ))
                        .on_conflict(classrooms::name)
                        .do_update()
                        .set(classrooms::grade_level.eq(cls.grade_level))
                        .execute(conn)?;
                }
            }
            ImportedData::Subjects(subjects_list) => {
                for sub in subjects_list {
                    diesel::insert_into(subjects::table)
                        .values((
                            subjects::name.eq(&sub.name),
                            subjects::base_periods_per_week.eq(sub.base_periods_per_week),
                        ))
                        .on_conflict(subjects::name)
                        .do_update()
                        .set(subjects::base_periods_per_week.eq(sub.base_periods_per_week))
                        .execute(conn)?;
                }
            }
            ImportedData::Progress(progress_list) => {
                for prg in progress_list {
                    let inserted_cls: ClassroomDb = diesel::insert_into(classrooms::table)
                        .values((
                            classrooms::name.eq(&prg.classroom_name),
                            classrooms::grade_level.eq(10),
                        ))
                        .on_conflict(classrooms::name)
                        .do_update()
                        .set(classrooms::name.eq(&prg.classroom_name))
                        .get_result(conn)?;

                    let inserted_sub: SubjectDb = diesel::insert_into(subjects::table)
                        .values((
                            subjects::name.eq(&prg.subject_name),
                            subjects::base_periods_per_week.eq(5),
                        ))
                        .on_conflict(subjects::name)
                        .do_update()
                        .set(subjects::name.eq(&prg.subject_name))
                        .get_result(conn)?;

                    diesel::insert_into(academic_progress::table)
                        .values((
                            academic_progress::classroom_id.eq(inserted_cls.id),
                            academic_progress::subject_id.eq(inserted_sub.id),
                            academic_progress::current_progress_ratio.eq(prg.current_progress_ratio),
                            academic_progress::expected_progress_ratio.eq(prg.expected_progress_ratio),
                        ))
                        .on_conflict((academic_progress::classroom_id, academic_progress::subject_id))
                        .do_update()
                        .set((
                            academic_progress::current_progress_ratio.eq(prg.current_progress_ratio),
                            academic_progress::expected_progress_ratio.eq(prg.expected_progress_ratio),
                        ))
                        .execute(conn)?;
                }
            }
        }
        Ok(())
    })
}
