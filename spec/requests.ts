import { NamedEntity, Repeat, TaskID, LessonID, TeacherID, UserID } from "./common"

export interface Login {
    login: string
    password: string
}

export interface Register extends NamedEntity {
    login: string
    password: string
}

export interface AddTask {
    name: string
    description?: string
}

export interface AddLesson {
    title: string
    repeats: Repeat[]
    description?: string
}

export interface AddTeacher extends NamedEntity {}

export interface AddPermission {
    type: "r" | "rw"
    entity_type: "task" | "lesson" | "teacher"
    entity_id: TaskID | LessonID | TeacherID
    user_id: UserID
}

export type UpdateTask = AddTask
export type UpdateLesson = AddLesson
export type UpdateTeacher = AddTeacher

export interface UpdateMe extends NamedEntity {}

export interface AssignTaskLesson {
    task_id: TaskID
    lesson_id: LessonID
}

export interface AssignTeacherLesson {
    teacher_id: TeacherID
    lesson_id: LessonID
}

export type DeassignTaskLesson = AssignTaskLesson
export type DeassignTeacherLesson = AssignTeacherLesson