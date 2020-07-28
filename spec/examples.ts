import { Login as LoginRequest } from "./requests"
import { Login as LoginReponse } from "./responses"

export const login: LoginRequest = {
    login: "yarik",
    password: "LegitPasword"
}

export const loginSuccess: LoginReponse = {
    payload: {
        access_token:
            "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6ImI2OWJkODdlLWQwNDQtMTFlYS05NWJlLTE0MTA5ZmQ2NzRkMSJ9.xYJaZTwhvncTGCLJFc2_xCnA9AYegn88VW3akEQnp18"
    }
}

export const loginFailure: LoginReponse = {
    error: {
        type: "invalid_credentials"
    }
}