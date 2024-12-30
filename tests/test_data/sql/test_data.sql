BEGIN;
INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id) VALUES (1, 'John', 'hash', 'email@email.cz', 'path/pic.png', null, null);

INSERT INTO artist(id, user_id, description) VALUES (1, 1, 'Description');

COMMIT;