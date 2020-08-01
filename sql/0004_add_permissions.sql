DROP TYPE EntityType;

CREATE TABLE IF NOT EXISTS LessonPermission (
    type PermissionType NOT NULL,
    lesson_id UUID NOT NULL REFERENCES Lesson(id),
    account_id UUID NOT NULL REFERENCES Account(id)
);
CREATE INDEX lessonpermission_idx_type ON LessonPermission(type);
CREATE INDEX lessonpermission_idx_lesson_id ON LessonPermission(lesson_id);
CREATE INDEX lessonpermission_idx_account_id ON LessonPermission(account_id);

CREATE TABLE IF NOT EXISTS TaskPermission (
    type PermissionType NOT NULL,
    task_id UUID NOT NULL REFERENCES Task(id),
    account_id UUID NOT NULL REFERENCES Account(id)
);
CREATE INDEX taskpermission_idx_type ON TaskPermission(type);
CREATE INDEX taskpermission_idx_teacher_id ON TaskPermission(task_id);
CREATE INDEX taskpermission_idx_account_id ON TaskPermission(account_id);