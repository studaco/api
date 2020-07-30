import { NamedEntity, Repeat, TaskID, LessonID, TeacherID, UserID, EntityID, EntityType } from "./common"

export interface Login {
    login: string
    /**
     * @format password
     */
    password: string
}

export interface Register extends NamedEntity {
    login: string
    /**
     * @format password
     */
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
    entity_type: EntityType
    /**
     * @format uuid
     */
    entity_id: EntityID
    /**
     * @format uuid
     */
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