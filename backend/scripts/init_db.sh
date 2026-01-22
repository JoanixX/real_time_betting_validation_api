#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql no está instalado."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx no está instalado."
  echo >&2 "Usa:"
  echo >&2 "    cargo install --version='~0.7' sqlx-cli --no-default-features --features rustls,postgres"
  echo >&2 "para instalarlo."
  exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

# Permitimos saltar Docker si ya tenemos una base de datos corriendo
if [[ -z "${SKIP_DOCKER}" ]]
then
  docker-compose up -d db
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  echo >&2 "Postgres todavía no está listo - esperando un segundo..."
  sleep 1
done

echo >&2 "Postgres está corriendo en el puerto ${DB_PORT} - ¡ejecutando migraciones!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

echo >&2 "¡Postgres ha sido migrado y está listo para usarse!"
