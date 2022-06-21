-- Add migration script here
CREATE TABLE suggestions (
    suggestion_id UUID NOT NULL,
    poll_id       UUID NOT NULL REFERENCES polls(poll_id),
    creator_id    UUID NOT NULL REFERENCES users(user_id),
    suggestion    TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (suggestion_id)
);

CREATE TABLE votes (
    user_id       UUID NOT NULL REFERENCES users(user_id),
    suggestion_id UUID NOT NULL REFERENCES suggestions(suggestion_id),
    PRIMARY KEY (user_id)
);