#!/bin/sh

sqlx database create
sqlx migrate run

DATA_LOADED_FILE="/usr/data-loaded"
INIT_DATA_FOLDER="/usr/init-data"

# We want to load initial data only after the container is created, not after each start
if [[ ! -f "$DATA_LOADED_FILE" ]]; then
    echo "Initial data not loaded yet. Loading initial data from ${INIT_DATA_FOLDER}!"
    cp -r ${INIT_DATA_FOLDER} /usr/erotic-hub/resources
    touch ${DATA_LOADED_FILE}
else
    echo "Initial data already loaded. You must recreate the container to load them again"
fi

./erotic-hub