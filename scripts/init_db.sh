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
