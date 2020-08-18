ALTER TABLE Repeats 
    ALTER COLUMN every SET DEFAULT 7,
    ADD COLUMN start_day DATE NOT NULL DEFAULT CURRENT_DATE,
    ADD COLUMN end_day DATE;

ALTER TABLE Repeats
    ALTER COLUMN start_day DROP DEFAULT;
-- INDEXES
CREATE INDEX repeats_idx_start_day ON Repeats(start_day);
CREATE INDEX repeats_idx_end_day ON Repeats(end_day);

CREATE TABLE IF NOT EXISTS SingleOccurance (
    occures_at TIMESTAMP NOT NULL,
    lesson_id UUID NOT NULL,
    CONSTRAINT lesson_id_fkey FOREIGN KEY (lesson_id) REFERENCES Lesson(id) ON DELETE CASCADE
);

CREATE INDEX singleoccurance_idx_occurs_at ON SingleOccurance(occures_at);
CREATE INDEX singleoccurance_idx_lesson_id ON SingleOccurance(lesson_id);