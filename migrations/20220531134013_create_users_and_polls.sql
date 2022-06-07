-- Separate table users to allow for the addition 
-- of global users, not just scoped to the poll
CREATE TABLE users (
    user_id      UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (user_id)
);

CREATE TABLE polls (
    poll_id    UUID NOT NULL,
    creator_id UUID NOT NULL REFERENCES users(user_id),
    prompt     VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (poll_id)
);

CREATE TABLE poll_users (
    poll_id      UUID NOT NULL REFERENCES polls,
    user_id      UUID NOT NULL REFERENCES users,
    username VARCHAR(32) NOT NULL, 
    UNIQUE (poll_id, username)
);