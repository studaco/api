import { Login as LoginRequest, Register as RegisterRequest, AddLesson } from "./requests"
import { Login as LoginReponse, Register as RegisterResponse, GetLesson as GetLessonResponse } from "./responses"
import {
    InternaError,
    Error,
    InvalidCredentials,
    LoginAlreadyPresent,
    LessonDoesNotExist,
    NoReadAccess,
    NoWriteAccess,
    NoTokenPresent,
    InvalidToken,
    TokenExpired,
    TokenRevoked,
} from "./errors"
import { Payload, Lesson } from "./common"

export const login: LoginRequest = {
    login: "yarik",
    password: "LegitPasword",
}

export const loginSuccess: LoginReponse = {
    payload: {
        access_token:
            "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6ImI2OWJkODdlLWQwNDQtMTFlYS05NWJlLTE0MTA5ZmQ2NzRkMSJ9.xYJaZTwhvncTGCLJFc2_xCnA9AYegn88VW3akEQnp18",
    },
}

export const loginFailure: Error<InvalidCredentials> = {
    error: {
        type: "invalid_credentials",
    },
}

export const register: RegisterRequest = {
    firstName: "John",
    lastName: "Appleseed",
    login: "johny",
    password: "LegitPassword",
}

export const registerWithoutName: RegisterRequest = {
    firstName: "John",
    login: "johny",
    password: "LegitPassword",
}

export const registerSuccess: RegisterResponse = {
    payload: {
        access_token:
            "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6ImI2OWJkODdlLWQwNDQtMTFlYS05NWJlLTE0MTA5ZmQ2NzRkMSJ9.xYJaZTwhvncTGCLJFc2_xCnA9AYegn88VW3akEQnp18",
    },
}

export const registerFailure: Error<LoginAlreadyPresent> = {
    error: {
        type: "login_already_present",
    },
}

export const internalError: Error<InternaError> = {
    error: {
        type: "internal_error",
    },
}

export const getLesson: GetLessonResponse = {
    payload: {
        id: "a7262da1-33ed-448c-8b7d-97263d0974f7",
        title: "Math",
        repeats: [
            {
                day: 1,
                every: 1,
                time: {
                    hour: 12,
                    minute: 0,
                },
            },
            {
                day: 3,
                every: 2,
                time: {
                    hour: 15,
                    minute: 10,
                },
            },
        ],
        teachers: ["cdf033af-e625-4fa4-b7e0-08ad096ba6dd", "291d3192-3cbf-4749-ae3f-f4834f220fda"],
    },
}

export const lessonNotFound: Error<LessonDoesNotExist> = {
    error: {
        type: "lesson_does_not_exist",
    },
}

export const noLessonReadAccess: Error<NoReadAccess<"lesson">> = {
    error: {
        type: "no_read_access",
        entity_type: "lesson",
        enriry_id: "a7262da1-33ed-448c-8b7d-97263d0974f7",
    },
}
export const noLessonWriteAccess: Error<NoWriteAccess<"lesson">> = {
    error: {
        type: "no_write_access",
        entity_type: "lesson",
        enriry_id: "a7262da1-33ed-448c-8b7d-97263d0974f7",
    },
}
// export const noLessonAdminPermission: Error<NoAdminPermissions<"lesson">> = {
//     error: {
//         type: "no_admin_permissions",
//         entity_type: "lesson",
//         enriry_id: "a7262da1-33ed-448c-8b7d-97263d0974f7",
//     },
// }

export const noTokenPresent: Error<NoTokenPresent> = {
    error: {
        type: "no_token_present",
    },
}
export const invalidToken: Error<InvalidToken> = {
    error: {
        type: "invalid_token",
    },
}
export const tokenExpired: Error<TokenExpired> = {
    error: {
        type: "token_expired",
    },
}
export const tokenRevoked: Error<TokenRevoked> = {
    error: {
        type: "token_revoked",
    },
}

export const addLesson: AddLesson = {
    title: "Math",
    repeats: [
        {
            day: 4,
            every: 2,
            time: {
                hour: 10,
                minute: 30,
            },
        },
        {
            day: 6,
            every: 1,
            time: {
                hour: 8,
                minute: 0,
            },
        },
    ],
}

export const lessonList: Payload<Lesson[]> = {
    payload: [
        getLesson.payload,
        {
            id: "cb571a0e-d057-4e4e-a592-7d6343875a7e",
            title: "English",
            repeats: [
                {
                    day: 2,
                    every: 1,
                    time: {
                        hour: 13,
                        minute: 10,
                    },
                },
            ],
            teachers: ["47477195-1de3-4a67-8e8d-1060a44593d5"],
        },
    ],
}

export const emptyList: Payload<[]> = {
    payload: []
}
