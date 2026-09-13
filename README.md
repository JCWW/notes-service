# notes-service
 
A REST backend for team note-taking.


## Prerequisites

```bash
sudo apt update && sudo apt install postgresql-client
cargo install sqlx-cli --no-default-features --features rustls,postgres

```

## Start the Application
```bash
cp .env.example .env
docker compose up -d
cargo sqlx prepare -- --all-targets
git add .sqlx
docker compose ps
cargo run
```

## Test 
``` bash
curl -s localhost:8080/health     # {"status":"ok"}
curl -s localhost:8080/ready      # {"status":"ok"}

```


## Confirm PSQL Commands

```bash
psql postgres://notes:notes@localhost:5432/notes -c 'select 1'
psql postgres://notes:notes@localhost:5432/notes -c '\dt'
psql postgres://notes:notes@localhost:5432/notes -c '\d notes' #test notes

#             List of relations
# Schema |       Name       | Type  | Owner 
#--------+------------------+-------+-------
# public | _sqlx_migrations | table | notes
# public | notes            | table | notes
# public | team_members     | table | notes
# public | teams            | table | notes
# public | users            | table | notes
#(5 rows)

```


## Overide Logging
```bash
RUST_LOG=warn cargo run    # request logs go quiet
```