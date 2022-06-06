set -x
set -eo pipefail

# set default env variables if they don't exist
DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:=password}
DB_NAME=${POSTGRES_DB:=apoll}
DB_PORT=${POSTGRES_PORT:=5432}

# run docker server
docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -e POSTGRES_PORT=${DB_PORT} \
    -d postgres \
    # increase max number of connections for testing
    postgres -N 1000

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable. Retrying in 1 second."
    sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}."

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create