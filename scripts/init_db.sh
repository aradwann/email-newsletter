#!/usr/bin/env bash
set -x 
set -eo pipefail

# check if psql is installed
if ! [ -x "$(command -v psql)" ]; then
  echo 'Error: psql is not installed.' >&2
  exit 1
fi

# Check if sqlx-cli is installed
if ! [ -x "$(command -v sqlx)" ]; then
  echo 'Error: sqlx is not installed.' >&2
  exit 1
fi

# Check if docker is installed
if ! [ -x "$(command -v docker)" ]; then
  echo 'Error: docker is not installed.' >&2
  exit 1
fi

# Check if a custom user has been set, otherwise default to 'postgres'
DB_USER="${POSTGRES_USER:=postgres}"

# Check if a custom password has been set, otherwise default to 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"

# Check if a custom database name has been set, otherwise default to 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"

# Check if a custom port has been set, otherwise default to '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

# Launch postgres using Docker

# Check if a postgres container is already running else start a new one
if docker ps | grep -q "postgres"; then
  echo "A postgres container is already running"
else
  echo "Starting a new postgres container"
  docker run \
  -e POSTGRES_USER=${DB_USER} \
  -e POSTGRES_PASSWORD=${DB_PASSWORD} \
  -e POSTGRES_DB=${DB_NAME} \
  -p "${DB_PORT}":5432 \
  -d postgres:17-alpine -N 1000 # Increase max number of connections for testing purposes
fi


# Keep pinging Postgres until it's ready to accept commands
until PGPASSWORD="${DB_PASSWORD}" psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

export DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run