CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE team_role AS ENUM ('member', 'admin');

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE teams (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE team_members (
    team_id    UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role       team_role NOT NULL DEFAULT 'member',
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (team_id, user_id)
);

CREATE INDEX team_members_user_idx ON team_members (user_id);

CREATE TABLE notes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_id   UUID NOT NULL REFERENCES users(id),
    team_id     UUID REFERENCES teams(id) ON DELETE RESTRICT,
    title       TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    body        TEXT NOT NULL DEFAULT '',
    version     BIGINT NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ
);

CREATE INDEX notes_author_idx
    ON notes (author_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX notes_team_idx
    ON notes (team_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL AND team_id IS NOT NULL;

CREATE INDEX notes_search_idx
    ON notes USING GIN (to_tsvector('english', title || ' ' || body))
    WHERE deleted_at IS NULL;