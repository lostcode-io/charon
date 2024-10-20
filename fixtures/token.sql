INSERT INTO tokens (token, addr, created_at, expires_at) VALUES ('foo', 'some_kind_of_and_address', NOW(), NOW() + interval '1 year') ON CONFLICT DO NOTHING;

