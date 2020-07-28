import { APIError } from "./errors"

export type NamedEntity = {
    firstName: string
    lastName?: string
}

export type Payload<T> = { payload: T }
export type Error<E extends APIError> = { error: E }
export type Response<T, E extends APIError> = Payload<T> | Error<E>

export type UUID = string

export type TeacherID = UUID
export type LessonID = UUID
export type TaskID = UUID
export type UserID = UUID

export type Monday = 1
export type Tuesday = 2
export type Wednesday = 3
export type Thursday = 4
export type Friday = 5
export type Saturday = 6
export type Sunday = 7
export type WeekDay = Monday | Tuesday | Wednesday | Thursday | Friday | Saturday | Sunday

export type Time = {
    hour: number
    minute: number
}

export type Weekly = 1
export type BiWeekly = 2
export type RepetitionFrequency = Weekly | BiWeekly

export type Repeat = {
    every: RepetitionFrequency
    day: WeekDay
    time: Time
}

interface Lesson {
    id: LessonID
    title: string
    repeats: Repeat[]
    // if you have read access to the lesson, 
    // you get read access to the teachers assigned to it
    teachers: TeacherID[]
    description?: string
}

interface Teacher extends NamedEntity {
    id: TeacherID
    userID?: UserID
    // lessons: LessonID[] // not MVP
}

interface Task {
    id: TaskID
    name: string
    description?: string
    lesson?: LessonID
    // teacher?: TeacherID // not MVP
}

interface User extends NamedEntity {
    id: UserID
}