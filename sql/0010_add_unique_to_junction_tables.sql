ALTER TABLE teacherlesson ADD CONSTRAINT teacherlesson_unique_connection UNIQUE (teacher_id, lesson_id);
ALTER TABLE teacherpermission ADD CONSTRAINT teacherpermission_unique_connection UNIQUE (teacher_id, account_id);
ALTER TABLE lessonpermission ADD CONSTRAINT lessonpermission_unique_connection UNIQUE (lesson_id, account_id);
ALTER TABLE taskpermission ADD CONSTRAINT taskpermission_unique_connection UNIQUE (task_id, account_id);
