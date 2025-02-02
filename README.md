# Erotic Hub

## Setup

### Prerequisites

Ensure you have `docker` and `docker-compose` installed.

### How to run in release mode

Release mode automatically prepares build of EroticHub in the Docker container. Then inserts database dump into
`PostgreSQL` container and inserts all desired resources files
into EroticHub folders. This is done only when the container is created for the first time.

1. Copy (or check if done) your database dump into `docker/postgres/db-data/` folder.
2. Copy (or check if done) your resources (content of `resources` folder - images, thumbnails, videos) into
   `docker/erotic-hub/data/` folder.
3. In root directory of the project, run

```bash
docker compose --profile release up -d
```

4. Wait until the all services are built and start.

### How to run in development mode

In development mode, the EroticHub container is not built and created.

1. Start all docker services

```bash
docker compose --profile dev up -d
```

2. Set up environmental variables. You can use the example file:

```bash
cp .env-example .env
```

3. Create the database

```bash
sqlx database create
```

4. Run migrations

```bash
sqlx migrate run
```

5. Run the application

```bash
cargo run
```

The app is available at http://localhost:8000.

You can access the database through the Adminer running at http://localhost:8080.

**Important!!!** After you update repositories / add migration, you must run following command to generate offline SQLx
files. Otherwise, it's not possible to release the EroticHub! Following command creates files in `.sqlx` folder,
don't forget to commit them!

```bash
cargo sqlx prepare
```

### How to seed data

To seed the app with showcase data, set the value of the `SEED_DATA` variable in [.env](.env) to `true` _before running the application_:

```bash
SEED_DATA=true
```

It is recommended to only use this right after creating the database. Otherwise, it is possible to run into conflicting data.

### How to create an admin account

To create the initial admin account, set up the `ADMIN_USERNAME`, `ADMIN_PASSWORD`, and `ADMIN_EMAIL` variables in [.env](.env) accordingly _before running the application_:

```bash
ADMIN_USERNAME=admin
ADMIN_PASSWORD=admin
ADMIN_EMAIL=admin@test.com
```

It is recommended to only use this right after creating the database. Otherwise, it is possible to run into conflicting data.

It is recommended to only use this to create the first admin account. Other users can be granted admin permissions by using the admin section of the application.

### How to make database

Example

```bash
docker exec -t erotic-hub-db-1 pg_dump -U postgres -d erotic-hub --column-inserts --create > docker/postgres/db-data/dump2.sql
```

### Install pre-commit for git hooks

```bash
pip install pre-commit
```
