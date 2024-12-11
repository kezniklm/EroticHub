# Erotic Hub

## Setup

### Prerequisites

Ensure you have `docker` and `docker-compose` installed.

### How to run

Start Postgres and Adminer:

```bash
docker-compose up -d
```

Set up environmental variables. You can use the example file:

```bash
cp .env-example .env
```

Create the database:

```bash
sqlx database create
```

Run migrations:

```bash
sqlx migrate run
```

Run with hot reload:

```bash
cargo watch -x run
```

The app is available at http://localhost:8000.

### Adminer

You can access the database through the Adminer running at http://localhost:8080.

### Edit migrations

Add a migration:

```bash
sqlx migrate add <migration_name>
```

### Install pre-commit for git hooks
```bash
pip install pre-commit
```
