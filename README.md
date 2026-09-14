# notes-service

A REST backend for note-taking shared across small teams. Rust, axum,
Postgres.

I chose this stack partly because I have been putting off getting familiar
with Rust for over 3 years. As a language I find it strangely familiar in the
features it borrows — generics, trait-based polymorphism, pattern matching —
and also quite foreign because of the borrow checker and the ownership model.
Working through a small but complete service seemed like a better way to
learn that than reading about it.

The structure reflects how I normally build a web service: organized by
domain, with each domain owning its own routes, handlers, model, and data
access.

---

## Running it

Prerequisites: Rust stable, Docker, and `sqlx-cli`:

```
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

Then:

```
docker compose up -d
cp .env.example .env
cargo run
```

The service migrates the database at boot and listens on `127.0.0.1:8080`.

```
curl -s localhost:8080/health     # {"status":"ok"}
curl -s localhost:8080/ready      # {"status":"ok"}
```

`/health` is liveness and never touches the database. `/ready` pings the pool,
so it returns 503 when Postgres is down. Keeping those separate means a
database outage does not get a healthy process killed and restarted.

Logging is controlled by `RUST_LOG`:

```
RUST_LOG=warn cargo run    # request logs go quiet
```

### Confirming the database came up

```bash
docker compose ps
psql postgres://notes:notes@localhost:5432/notes -c 'select 1'
psql postgres://notes:notes@localhost:5432/notes -c '\dt'
```

```
             List of relations
 Schema |       Name       | Type  | Owner
--------+------------------+-------+-------
 public | _sqlx_migrations | table | notes
 public | notes            | table | notes
 public | team_members     | table | notes
 public | teams            | table | notes
 public | users            | table | notes
(5 rows)
```

## Testing

```
docker compose up -d
cargo test
```

Each test creates its own database and spawns the server on an ephemeral
port, so the suite runs in parallel with nothing shared between tests. Set
`TEST_LOG=1` to see server logs during a run.

## Authentication is stubbed

**The bearer token is the caller's user id.** There is no password, no JWT,
no session:

```
curl -H "Authorization: Bearer $USER_ID" localhost:8080/notes
```

Real authentication is a solved problem, and building it would have eaten the
time budget without demonstrating anything about the design. Identity enters
through a single extractor in `src/auth.rs`, so replacing the mechanism
touches one file and no handler.

Create a user to get started:

```
psql postgres://notes:notes@localhost:5432/notes -c \
  "INSERT INTO users (email, display_name) VALUES ('you@example.com', 'You') RETURNING id"
```

## Endpoints

| Method | Path | Notes |
|---|---|---|
| GET | `/health` | Liveness. Never touches the database. |
| GET | `/ready` | Readiness. Pings the pool. |
| POST | `/notes` | `{ title, body?, team_id? }` |
| GET | `/notes` | `?team_id=&q=&cursor=&limit=` |
| GET | `/notes/{id}` | Returns an `ETag` |
| PATCH | `/notes/{id}` | Honors `If-Match` |
| DELETE | `/notes/{id}` | Soft delete. Author or team admin. |
| POST | `/teams` | `{ name, slug? }`. Creator becomes admin. |
| GET | `/teams` | Teams you belong to. |
| GET | `/teams/{id}/members` | Members only. |
| POST | `/teams/{id}/members` | Admins only. |
| DELETE | `/teams/{id}/members/{user_id}` | Admins only. |

Errors share one envelope:

```json
{ "error": { "code": "conflict", "message": "note is now at version 3, re-read before writing" } }
```

---

## Design choices I spent the most time on

### 1. Rust, and organizing it by domain

Ultimately I chose Rust because I wanted to learn it, and because I think it
belongs in scenarios that need highly performant, predictable services. Under
normal circumstances I would have utilized C# or Node, where I am faster.
For a service this size the runtime performance argument is not the real one —
what I got out of Rust here was that the type system made and my unfamiliarity with Rust libraries 
forced me to spend more time exercising my critical thinking cap.

Given the simplicity of the task, organizing the project into domains was
probably overkill. A single set of handlers and routes would have worked. I
did it anyway for two reasons. It is how I would structure a real service,
because separating concerns is what makes it maintainable a year later, and
it gave me a reason to work through module visibility and cross-module wiring
in Rust, which is one of the parts I understood least going in.


### 2.  **sqlx rather than an ORM.**

 Queries are SQL, verified against the live schema
at compile time. The visibility predicate and the index-aware ordering are the
interesting parts of this service, and an ORM's DSL would have hidden both.
`cargo sqlx prepare` caches that verification in `.sqlx/`, which is committed,
so `SQLX_OFFLINE=true cargo build` works with no database running.


---

## What I would change or add with more time

- **More unit tests.** I spent my testing time on integration tests, because
  getting a per-test database and a real HTTP client working in Rust was the
  part I most wanted to learn. That was the right call for confidence in the
  behavior, but it leaves the pure logic verification thinner than would be
  appropiate for a production system.

- **A CI pipeline** running `cargo clippy -- -D warnings`, `cargo test`, and
  `cargo sqlx prepare --check`. That last one matters most, since a stale
  `.sqlx/` builds fine on my machine and breaks for everyone else.
- **A Dockerfile for the service itself.** Compose only brings up Postgres,
  and the app runs from `cargo run`. A multi-stage Rust build is fiddly
  enough that I chose to spend the time on behavior instead.
- **Migrations as a separate step rather than at boot.** Migrating at startup
  is convenient for `cargo run` and for whoever is reviewing this. It is not
  what I would do with more than one replica.

---

## Notes

Any time a `sqlx::query!` macro is added or changed, the cached query data has
to be regenerated and committed:

```bash
docker compose up -d
cargo sqlx prepare -- --all-targets
git add .sqlx
```

The `-- --all-targets` matters. Without it, `prepare` skips the test code and
the test build quietly falls back to needing a live database.

---
