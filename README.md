# Erotic Hub

## Prerequisites

Ensure you have `docker` and `docker-compose` installed.

## Run in release mode

If you want to add data from an existing database into the released application, add the resources by copying them into the [docker/erotic-hub/data/](docker/erotic-hub/data/) folder. Add the database dump into the [docker/postgres/db-data](docker/postgres/db-data) folder. Use this command to create such a dump:

```bash
docker exec -t erotic-hub-db-1 pg_dump -U postgres -d erotic-hub --column-inserts --create > docker/postgres/db-data/dump2.sql
```

Before running the application, set up the environmental variables accordingly. You can copy either of these files:

- [.env.release-without-seed](.env.release-without-seed), and
- [.env.release-with-seed](.env.release-with-seed).

If you set `SEED_DATA=true`, you have to copy the resources from [seed_resources](seed_resources) to [docker/erotic-hub/data/](docker/erotic-hub/data/).

Use the docker compose script to run the application:

### With seed data

```bash
cp .env.release-with-seed .env
cp -r seed_resources/ docker/erotic-hub/data/
docker compose --profile release up -d
```

The app should now run at [http://localhost:8000/](http://localhost:8000/).

### Without seed data

```bash
cp .env.release-without-seed .env
docker compose --profile release up -d
```

The app should now run at [http://localhost:8000/](http://localhost:8000/).

## Run in development mode

In development mode, the EroticHub container is not built and created.

1. Start all docker services

```bash
docker compose --profile dev up -d
```

2. Set up environmental variables. You can use the example file:

```bash
cp .env.dev .env
```

3. Create the database

```bash
sqlx database create
```

4. Run migrations

```bash
sqlx migrate run
```

5. Prepare seed data

```bash
cp -r seed_resources/ resources/
```

6. Run the application

```bash
cargo run
```

The app should now run at [http://localhost:8000/](http://localhost:8000/).

You can access the database through the Adminer running at [http://localhost:8080/](http://localhost:8080/).

**Important!!!** After you update repositories, add migration, or add other SQLx commands, you must run following command to generate offline SQLx files. Otherwise, it's not possible to release the EroticHub! Following command creates files in the [.sqlx](.sqlx) folder, don't forget to commit them!

```bash
cargo sqlx prepare
```
