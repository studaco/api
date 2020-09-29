ALTER TABLE Repeats RENAME COLUMN end_day TO end_date;
ALTER TABLE Repeats RENAME COLUMN start_day TO start_date;
ALTER TABLE Repeats RENAME TO LessonWeeklyRepeat;
ALTER INDEX repeats_idx_start_day RENAME TO lessonweeklyrepeat_idx_start_date;
ALTER INDEX repeats_idx_end_day RENAME TO lessonweeklyrepeat_idx_end_date;
ALTER INDEX repeats_idx_lesson_id RENAME TO lessonweeklyrepeat_idx_lesson_id;
ALTER INDEX repeats_idx_week_day RENAME TO lessonweeklyrepeat_idx_week_day;

CREATE TABLE IF NOT EXISTS LessonDailyRepeat (
    scheduled_time TIME NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    lesson_id UUID NOT NULL REFERENCES Lesson(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS LessonMonthlyRepeat (
    scheduled_time TIMESTAMP NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    every INT NOT NULL,
    lesson_id UUID NOT NULL REFERENCES Lesson(id) ON DELETE CASCADE,

    CONSTRAINT every_is_a_natural_number CHECK (every > 0)
);