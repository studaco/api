type NamedEntity = {
    firstName: string
    lastName?: string
}

interface LoginRequest {
    login: string
    password: string
}

interface RegisterRequest extends NamedEntity {
    login: string
    password: string
}

type UUID = string

type TeacherID = UUID
type LessonID = UUID
type TaskID = UUID
type UserID = UUID

type Monday = 1
type Tuesday = 2
type Wednesday = 3
type Thursday = 4
type Friday = 5
type Saturday = 6
type Sunday = 7
type WeekDay = Monday | Tuesday | Wednesday | Thursday | Friday | Saturday | Sunday

type Time = {
    hour: number
    minute: number
}

type Weekly = 1
type BiWeekly = 2
type RepetitionFrequency = Weekly | BiWeekly

type Repeat = {
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

interface AddTask {
    name: string
    description?: string
}

interface AddLesson {
    title: string
    repeats: Repeat[]
    description?: string
}

interface AddTeacher extends NamedEntity {}

interface AddPermission {
    type: "r" | "rw"
    entityType: "task" | "lesson" | "teacher"
    entityID: TaskID | LessonID | TeacherID
    userID: UserID
}

type UpdateTask = AddTask
type UpdateLesson = AddLesson
type UpdateTeacher = AddTeacher

interface UpdateMe extends NamedEntity {}

interface AssignTaskLesson {
    taskID: TaskID
    lessonID: LessonID
}

interface AssignTeacherLesson {
    teacherID: TeacherID
    lessonID: LessonID
}

type DeassignTaskLesson = AssignTaskLesson
type DeassignTeacherLesson = AssignTeacherLesson