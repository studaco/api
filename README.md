# Ne Student API

Attempts to learn Rust on a real-world example. Plus making a somewhat decent product.

### How would one go about building this one?
`cargo build`. duh...

### How to launch dev environment?
1. Setup PosgreSQL database.
2. Set following environment variables. You can simply put 'em into `.env` file:
- `HOST` - hostname to bind application to. Use `0.0.0.0` to make it accessible from everywhere.
- `PORT` - network port which application will listen to.
- `DATABASE_URL` - URL to a postgres database in fomat `postgresql://<username>:<password>@<host>[:<port>]/<database name>[?schema=<schema name>]`
- `TOKEN_SECRET` - secret which will be used to sign access tokens
3. make sure you have `cargo make` installed
4. `cargo make dev`

### How to deploy this bad boi?
`docker-compose up`. You don't even need to clone repo. Just yoink the `docker-compose.yml`.  
Make sure you have docker, and docker-compose installed. duh.
You would need to populate following env variables. _Trick with `.env` file still works_
- `DB_USER` - postgres admin username
- `DB_PASSWORD` - postgres admin password
- `TOKEN_SECRET` - secret which will be used to sign access tokens
- `PORT` - network port at which application will be running

Happy sailing 
