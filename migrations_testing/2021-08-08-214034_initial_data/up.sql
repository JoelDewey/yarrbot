INSERT INTO users (id, service_username, user_role)
VALUES ('33a370ef-e309-4b8f-ab72-0e75632282af',
        'testuser@localhost',
        'system_administrator');

INSERT INTO webhooks (id, username, password, user_id)
VALUES ('0891fdfa-3612-41ae-adf1-c2277db90ea2',
        'testuser',
        -- Null terminator-padded bytea representation of the password hash for: myP@ssw0rd123
        -- $argon2id$v=19$m=65536,t=2,p=1$y0EyhsaVL2Pq8V/lblKO+w$rymdaAlnBambjBXQuu+n+x0y9Xle1Bt6dF2WrNeJgiU
        E'\\x246172676F6E32696424763D3139246D3D36353533362C743D322C703D312479304579687361564C32507138562F6C626C4B4F2B772472796D6461416C6E42616D626A42585175752B6E2B78307939586C653142743664463257724E654A67695500000000000000000000000000000000000000000000000000000000000000',
        '33a370ef-e309-4b8f-ab72-0e75632282af');
INSERT INTO webhooks (id, username, password, user_id)
VALUES ('464afbf9-3ef3-451a-8992-a47cb95e72a3',
        'testuser',
           -- Null terminator-padded bytea representation of the password hash for: myP@ssw0rd123
           -- $argon2id$v=19$m=65536,t=2,p=1$y0EyhsaVL2Pq8V/lblKO+w$rymdaAlnBambjBXQuu+n+x0y9Xle1Bt6dF2WrNeJgiU
        E'\\x246172676F6E32696424763D3139246D3D36353533362C743D322C703D312479304579687361564C32507138562F6C626C4B4F2B772472796D6461416C6E42616D626A42585175752B6E2B78307939586C653142743664463257724E654A67695500000000000000000000000000000000000000000000000000000000000000',
        '33a370ef-e309-4b8f-ab72-0e75632282af');

INSERT INTO matrix_rooms (id, room_id, webhook_id)
VALUES ('a3f5b09b-7891-4212-8c50-72f4eb05c80b', '!tFdiCkSzfpqSyeABCk:localhost',
        '0891fdfa-3612-41ae-adf1-c2277db90ea2');
INSERT INTO matrix_rooms (id, room_id, webhook_id)
VALUES ('f8c19435-ace4-4f74-967e-e26a05c63186', '!tFdiCkSzfpqSyeABCk:localhost',
        '464afbf9-3ef3-451a-8992-a47cb95e72a3');