#!/usr/bin/env bash
set -euxo pipefail

# Database Configuration
DB_PORT="${DB_PORT:=5432}"
SUPERUSER="${SUPERUSER:=postgres}"
SUPERUSER_PWD="${SUPERUSER_PWD:=password}"
APP_USER="${APP_USER:=app}"
APP_USER_PWD="${APP_USER_PWD:=secret}"
APP_DB_NAME="${APP_DB_NAME:=newsletter}"
SKIP_DOCKER="${SKIP_DOCKER:-}"  # ✅ Ensure SKIP_DOCKER is always defined

# Check if Docker-based setup is required
if [[ -z "${SKIP_DOCKER}" ]]; then
  CONTAINER_NAME="postgres_$(date '+%s')"

  echo "Starting a new PostgreSQL container: ${CONTAINER_NAME}..."
  docker run --name "${CONTAINER_NAME}" -d \
    -e POSTGRES_USER=${SUPERUSER} \
    -e POSTGRES_PASSWORD=${SUPERUSER_PWD} \
    -p "${DB_PORT}":5432 \
    --health-cmd="pg_isready -U ${SUPERUSER}" \
    --health-interval=1s \
    --health-timeout=5s \
    --health-retries=5 \
    postgres:17-alpine

  # Wait for Postgres to be ready
  until docker exec "${CONTAINER_NAME}" pg_isready -U "${SUPERUSER}" >/dev/null 2>&1; do
    echo "Waiting for Postgres to become ready..."
    sleep 1
  done

  echo "Postgres is ready!"
fi

# Function to run commands inside the Postgres container or locally
exec_psql() {
  if [[ -z "${SKIP_DOCKER}" ]]; then
    docker exec -i "${CONTAINER_NAME}" psql -U "${SUPERUSER}" -d postgres -c "$1"
  else
    PGPASSWORD="${SUPERUSER_PWD}" psql -U "${SUPERUSER}" -h "localhost" -d postgres -c "$1"
  fi
}

# Create app user (only if it doesn't already exist)
exec_psql "DO \$\$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '${APP_USER}') THEN CREATE USER ${APP_USER} WITH PASSWORD '${APP_USER_PWD}'; END IF; END \$\$;"

# Grant necessary privileges
exec_psql "ALTER USER ${APP_USER} CREATEDB;"

# Set up application database
DATABASE_URL="postgres://${APP_USER}:${APP_USER_PWD}@localhost:${DB_PORT}/${APP_DB_NAME}"
export DATABASE_URL

sqlx database create
sqlx migrate run

echo "✅ Database migrations completed successfully!"
