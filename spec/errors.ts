export type InternaError = {
    type: "internal_error"
    message?: string
}
export type UserDoesNotExist = {
    type: "user_does_not_exist"
}
export type InvalidCredentials = {
    type: "invalid_credentials"
}
export type InvalidToken = {
    type: "invalid_token"
}
export type TokenExpired = {
    type: "token_expired"
}

export type APIError = InternaError | UserDoesNotExist | InvalidCredentials | InvalidToken | TokenExpired
