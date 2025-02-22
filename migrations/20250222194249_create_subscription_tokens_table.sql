CREATE TABLE subscription_tokens
(
    id uuid PRIMARY KEY,
    token VARCHAR(255) NOT NULL,
    subscription_id uuid NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions (id)
);