CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS Lesson (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v1(),
    title TEXT NOT NULL,
    description TEXT
);

CREATE TYPE WeekDay AS ENUM ('MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT', 'SUN');

CREATE TABLE IF NOT EXISTS Repeats (
    every INTEGER NOT NULL DEFAULT 1,
    week_day WeekDay NOT NULL,
    scheduled_time time with time zone NOT NULL,
    lesson_id UUID NOT NULL REFERENCES Lesson(id),
    CONSTRAINT every_is_a_natural_number CHECK (every > 0)
);

CREATE INDEX repeats_idx_lesson_id ON Repeats(lesson_id);
CREATE INDEX repeats_idx_week_day ON Repeats(week_day);

CREATE TABLE IF NOT EXISTS Account (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v1(),
    first_name TEXT NOT NULL,
    last_name TEXT,
    login TEXT NOT NULL,
    pssword_hash TEXT NOT NULL
);

CREATE INDEX user_idx_login ON Account(login);

CREATE TABLE IF NOT EXISTS Teacher (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v1(),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    account_id UUID REFERENCES Account(id)
);

CREATE INDEX teacher_idx_account_id ON Teacher(account_id);

CREATE TABLE IF NOT EXISTS Task (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v1(),
    name TEXT NOT NULL,
    description TEXT,
    lesson_id UUID REFERENCES Lesson(id),
    lesson_date date NOT NULL
);

CREATE INDEX task_idx_lesson_id ON Task(lesson_id);
CREATE INDEX task_idx_lesson_date ON Task(lesson_date);

CREATE TABLE IF NOT EXISTS TeacherLesson (
    teacher_id UUID NOT NULL REFERENCES Teacher(id),
    lesson_id UUID NOT NULL REFERENCES Lesson(id)
);

CREATE INDEX teacherlesson_idx_teacher_id ON TeacherLesson(teacher_id);
CREATE INDEX teacherlesson_idx_lesson_id ON TeacherLesson(lesson_id);

CREATE TYPE PermissionType AS ENUM ('r', 'rw');
CREATE TYPE EntityType AS ENUM ('teacher', 'task', 'lesson');

CREATE TABLE IF NOT EXISTS TeacherPermission (
    type PermissionType NOT NULL,
    teacher_id UUID NOT NULL REFERENCES Teacher(id),
    account_id UUID NOT NULL REFERENCES Account(id)
);

CREATE INDEX teacherpermission_idx_type ON TeacherPermission(type);
CREATE INDEX teacherpermission_idx_teacher_id ON TeacherPermission(teacher_id);
CREATE INDEX teacherpermission_idx_account_id ON TeacherPermission(account_id);