# notes-service
 
A REST backend for team note-taking.

## Start the Application
```bash
cp .env.example .env
docker compose up -d
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
```


## Overide Logging
```bash
RUST_LOG=warn cargo run    # request logs go quiet
```