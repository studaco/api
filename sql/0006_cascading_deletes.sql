ALTER TABLE repeats
    DROP CONSTRAINT repeats_lesson_id_fkey,
    ADD  CONSTRAINT lesson_id_fkey 
        FOREIGN KEY (lesson_id) REFERENCES Lesson(id) ON DELETE CASCADE;

ALTER TABLE teacher
    DROP CONSTRAINT teacher_account_id_fkey,
    ADD  CONSTRAINT account_id_fkey
        FOREIGN KEY (account_id) REFERENCES Account(id) ON DELETE SET NULL;

ALTER TABLE task
    DROP CONSTRAINT task_lesson_id_fkey,
    ADD  CONSTRAINT lesson_id_fkey
        -- Actually cascade even tho it can be null
        FOREIGN KEY (lesson_id) REFERENCES Lesson(id) ON DELETE CASCADE;

ALTER TABLE teacherlesson
    DROP CONSTRAINT teacherlesson_teacher_id_fkey,
    ADD  CONSTRAINT teacher_id_fkey
        FOREIGN KEY (teacher_id) REFERENCES Teacher(id) ON DELETE CASCADE,
    DROP CONSTRAINT teacherlesson_lesson_id_fkey,
    ADD  CONSTRAINT lesson_id_fkey
        FOREIGN KEY (lesson_id) REFERENCES Lesson(id) ON DELETE CASCADE;

ALTER TABLE lessonpermission 
    DROP CONSTRAINT lessonpermission_lesson_id_fkey,
    ADD  CONSTRAINT lesson_id_fkey 
        FOREIGN KEY (lesson_id) REFERENCES Lesson(id) ON DELETE CASCADE,
    DROP CONSTRAINT lessonpermission_account_id_fkey,
    ADD  CONSTRAINT account_id_fkey 
        FOREIGN KEY (account_id) REFERENCES Account(id) ON DELETE CASCADE;

ALTER TABLE teacherpermission 
    DROP CONSTRAINT teacherpermission_teacher_id_fkey,
    ADD  CONSTRAINT teacher_id_fkey 
        FOREIGN KEY (teacher_id) REFERENCES Teacher(id) ON DELETE CASCADE,
    DROP CONSTRAINT teacherpermission_account_id_fkey,
    ADD  CONSTRAINT account_id_fkey 
        FOREIGN KEY (account_id) REFERENCES Account(id) ON DELETE CASCADE;

ALTER TABLE taskpermission 
    DROP CONSTRAINT taskpermission_task_id_fkey,
    ADD  CONSTRAINT task_id_fkey 
        FOREIGN KEY (task_id) REFERENCES Task(id) ON DELETE CASCADE,
    DROP CONSTRAINT taskpermission_account_id_fkey,
    ADD  CONSTRAINT account_id_fkey 
        FOREIGN KEY (account_id) REFERENCES Account(id) ON DELETE CASCADE;