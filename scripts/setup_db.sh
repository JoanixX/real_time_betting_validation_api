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

# Verifica si se ha establecido un usuario personalizado, de lo contrario, por defecto es 'postgres'
DB_USER="${POSTGRES_USER:=postgres}"
# Verifica si se ha establecido una contraseña personalizada, de lo contrario, por defecto es 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# Verifica si se ha establecido un nombre de base de datos personalizado, de lo contrario, por defecto es 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# Verifica si se ha establecido un puerto personalizado, de lo contrario, por defecto es '5432'
DB_PORT="${POSTGRES_PORT:=5432}"
# Verifica si se ha establecido un host personalizado, de lo contrario, por defecto es 'localhost'
DB_HOST="${POSTGRES_HOST:=localhost}"

# Permite saltar Docker si ya hay una base de datos Postgres funcionando
if [[ -z "${SKIP_DOCKER}" ]]
then
  # Lanza postgres usando Docker
  docker run \
      -e POSTGRES_USER=${DB_USER} \
      -e POSTGRES_PASSWORD=${DB_PASSWORD} \
      -e POSTGRES_DB=${DB_NAME} \
      -p "${DB_PORT}":5432 \
      -d postgres \
      postgres -N 1000
      # ^ Incrementado el número máximo de conexiones para propósitos de prueba
fi

# Sigue haciendo ping a Postgres hasta que esté listo para aceptar comandos
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres aún no está disponible - esperando"
  sleep 1
done

>&2 echo "¡Postgres está funcionando en el puerto ${DB_PORT}!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

>&2 echo "¡Postgres ha sido migrado y está listo!"
