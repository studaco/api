ALTER TABLE teacher ALTER COLUMN last_name DROP NOT NULL;

ALTER TABLE SingleOccurance RENAME TO SingleOccurrence;
ALTER TABLE SingleOccurrence RENAME occures_at TO occurs_at;

ALTER INDEX singleoccurance_idx_occurs_at RENAME TO singleoccurrence_idx_occurs_at;
ALTER INDEX singleoccurance_idx_lesson_id RENAME TO singleoccurrence_idx_lesson_id;