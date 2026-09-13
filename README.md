# notes-service
 
A REST backend for team note-taking.

## Start the Application
```bash
docker compose up -d
docker compose ps   
```

## PSWL Commands

```bash
psql postgres://notes:notes@localhost:5432/notes -c 'select 1'
```


## Overide Logging
```bash
RUST_LOG=warn cargo run    # request logs go quiet
```