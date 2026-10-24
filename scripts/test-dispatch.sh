#!/usr/bin/env bash
# Only creates new dedicated containers; never consumes DATABASE_URL/existing DB.
set -euo pipefail
pg_image=postgres@sha256:16bc17c64a573ef34162af9298258d1aec548232985b33ed7b1eac33ba35c229
my_image=mysql@sha256:2952e3be7807f06fc18de50b3ea1a632d5c70d63482ff7d7376fe3aa8999babf
run_id=$(cat /proc/sys/kernel/random/uuid)
pg_container="quent-dispatch-pg-$run_id"
my_container="quent-dispatch-my-$run_id"
created=()
cleanup() {
  result=$?
  trap - EXIT
  for container in "${created[@]}"; do
    docker rm --force --volumes "$container" >/dev/null || result=1
  done
  exit "$result"
}
trap cleanup EXIT
# Anonymous image-declared volumes are removed by --volumes, including on failure.
created+=("$pg_container")
docker run -d --name "$pg_container" -p 127.0.0.1::5432 -e POSTGRES_DB=quent_dispatch_test -e POSTGRES_HOST_AUTH_METHOD=trust "$pg_image" >/dev/null
created+=("$my_container")
docker run -d --name "$my_container" -p 127.0.0.1::3306 -e MYSQL_DATABASE=quent_dispatch_test -e MYSQL_ALLOW_EMPTY_PASSWORD=yes "$my_image" --character-set-server=utf8mb4 --collation-server=utf8mb4_bin --default-time-zone=+00:00 >/dev/null
ready=false
for attempt in {1..60}; do
  if docker exec "$pg_container" pg_isready -U postgres -d quent_dispatch_test >/dev/null 2>&1 && docker exec "$my_container" mysqladmin --host=127.0.0.1 ping --silent >/dev/null 2>&1; then ready=true; break; fi
  sleep 1
done
if ! "$ready"; then echo 'Dedicated DB readiness failed (no skip)' >&2; exit 1; fi
# Only these disposable DBs; the application/test role has no global privileges.
docker exec "$pg_container" psql -v ON_ERROR_STOP=1 -U postgres -d quent_dispatch_test -c 'CREATE ROLE quent_test LOGIN; ALTER DATABASE quent_dispatch_test OWNER TO quent_test; CREATE SCHEMA quent_dispatch_test AUTHORIZATION quent_test;' >/dev/null
docker exec "$my_container" mysql -e "CREATE USER 'quent_test'@'%' IDENTIFIED BY ''; GRANT ALL ON quent_dispatch_test.* TO 'quent_test'@'%';" >/dev/null
docker exec "$pg_container" psql -U postgres -d quent_dispatch_test -Atc "SELECT version(), current_setting('TimeZone'), current_setting('transaction_isolation'), current_setting('server_encoding'), datcollate FROM pg_database WHERE datname=current_database();"
docker exec "$my_container" mysql -N -e 'SELECT VERSION(), @@sql_mode, @@time_zone, @@transaction_isolation, @@character_set_server, @@collation_server;'
pg_port=$(docker port "$pg_container" 5432/tcp)
my_port=$(docker port "$my_container" 3306/tcp)
export QUENT_TEST_DISPOSABLE=quent_dispatch_test
export QUENT_TEST_POSTGRES_DSN="postgres://quent_test@${pg_port}/quent_dispatch_test"
export QUENT_TEST_MYSQL_DSN="mysql://quent_test@${my_port}/quent_dispatch_test"
if (( $# )); then "$@"; else cargo test -p quent --all-features --lib live_ --locked -- --test-threads=1; fi
