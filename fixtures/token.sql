INSERT INTO tokens (token, created_at, expires_at) VALUES ('foo', NOW(), NOW() + interval '1 year') ON CONFLICT DO NOTHING;

