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

5. Important!!! After you update repositories / add migration, you must run following command to generate offline SQLx
   files. Otherwise, it's not possible to release the EroticHub! Following command creates files in `.sqlx` folder,
   don't forget to commit them!

```bash
cargo sqlx prepare
```

The app is available at http://localhost:8000.

You can access the database through the Adminer running at http://localhost:8080.

### How to make database

Example

```bash
docker exec -t erotic-hub-db-1 pg_dump -U postgres -d erotic-hub --column-inserts --create > docker/postgres/db-data/dump2.sql
```

### Install pre-commit for git hooks

```bash
pip install pre-commit
```
