#![allow(unused)]
#![allow(dead_code)]

use std::collections::HashMap;

enum School {
    Primary,
    Middle,
    Secondary,
}

enum Schedule {
    Teacher,
    ClassRoom,
    Teachers,
}

enum Day {
    Sunaday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

struct Teacher {
    name: String,
    subjects: Vec<Subject>,
}

struct Class(Subject, Teacher, ClassRoom);

struct Subject {
    name: String,
}

struct ClassRoom(char);

struct Grade {
    school: School,
    class_rooms: Vec<ClassRoom>,
}

struct TimeTable(HashMap<Day, Vec<Class>>);

fn main() {
    println!("Hello, world!");
}
