-- Create table for subscriptions
CREATE TABLE subscriptions (
    id uuid PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);