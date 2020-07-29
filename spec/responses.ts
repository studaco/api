import { Payload, Lesson } from "./common"

export type Login = Payload<{ access_token: string }>

export type Register = Login

export type GetLesson = Payload<Lesson>

export type NewLesson = GetLesson

export type GetLessonList = Payload<Lesson[]>