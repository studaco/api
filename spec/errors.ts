import { EntityType, IDOf } from "./common"

export type InternaError = {
    type: "internal_error"
    message?: string
}
export type UserDoesNotExist = { type: "user_does_not_exist" }
export type InvalidCredentials = { type: "invalid_credentials" }
export type LoginAlreadyPresent = { type: "login_already_present" }
export type LessonDoesNotExist = { type: "lesson_does_not_exist" }
type NoAccess<Entity extends EntityType> = {
    entity_type: Entity
    /**
     * @format uuid
     */
    enriry_id: IDOf<Entity>
}
export type NoReadAccess<Entity extends EntityType = EntityType> = NoAccess<Entity> & { type: "no_read_access" }
export type NoWriteAccess<Entity extends EntityType = EntityType> = NoAccess<Entity> & { type: "no_write_access" }
// export type NoAdminPermissions<Entity extends EntityType = EntityType> = NoAccess<Entity> & {
//     type: "no_admin_permissions"
// }
export type NoTokenPresent = { type: "no_token_present" }
export type InvalidToken = { type: "invalid_token" }
export type TokenExpired = { type: "token_expired" }
export type TokenRevoked = { type: "token_revoked" }

export type Unauthorized = NoTokenPresent | InvalidToken | TokenExpired | TokenRevoked

export type APIError =
    | InternaError
    | UserDoesNotExist
    | InvalidCredentials
    | InvalidToken
    | TokenExpired
    | NoTokenPresent
    | TokenRevoked
    | LoginAlreadyPresent
    | LessonDoesNotExist
    | NoReadAccess
    | NoWriteAccess
// | NoAdminPermissions

export type Error<E extends APIError> = { error: E }

export type SingleInternalError = Error<InternaError>
export type SingleUserDoesNotExist = Error<InternaError>
export type SingleInvalidCredentials = Error<InternaError>
export type SingleInvalidToken = Error<InternaError>
export type SingleLoginAlreadyPresent = Error<InternaError>
export type SingleLessonDoesNotExist = Error<LessonDoesNotExist>
export type SingleNoReadAccess<Entity extends EntityType = EntityType> = Error<NoReadAccess<Entity>>
export type SingleNoWriteAccess<Entity extends EntityType = EntityType> = Error<NoWriteAccess<Entity>>
// export type SingleNoAdminPermissions<Entity extends EntityType = EntityType> = Error<NoAdminPermissions>
export type SingleNoTokenPresent = Error<NoTokenPresent>
export type SingleTokenRevoked = Error<TokenRevoked>

export type SingleLessonNoReadAccess = SingleNoReadAccess<"lesson">
export type SingleLessonNoWriteAccess = SingleNoWriteAccess<"lesson">
// export type SingleLessonNoAdminPermissions = SingleNoAdminPermissions<"lesson">
