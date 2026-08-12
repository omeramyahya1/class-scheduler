CREATE TABLE teachers (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    max_daily_periods INT NOT NULL,
    max_weekly_periods INT NOT NULL
);

CREATE TABLE classrooms (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    grade_level INT NOT NULL
);

CREATE TABLE subjects (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    base_periods_per_week INT NOT NULL
);

CREATE TABLE teacher_qualifications (
    teacher_id INT NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    subject_id INT NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    PRIMARY KEY (teacher_id, subject_id)
);

CREATE TABLE academic_progress (
    classroom_id INT NOT NULL REFERENCES classrooms(id) ON DELETE CASCADE,
    subject_id INT NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    current_progress_ratio FLOAT NOT NULL,
    expected_progress_ratio FLOAT NOT NULL,
    PRIMARY KEY (classroom_id, subject_id)
);
