import { Response } from "./common"
import { InternaError, InvalidCredentials } from "./errors"

export type Login = Response<{ access_token: string }, InternaError | InvalidCredentials>