BEGIN;
INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id)
VALUES (1, 'JohnArtist', '$2b$12$jc1StD0ppsiVerHTwQ2x/.VA7jG9v7zSfLzF8Y61n.xdhz0Mi72UK', 'john1@email.cz', 'path/pic.png', null, null);

INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id)
VALUES (2, 'JohnNotArtist', '$2b$12$jc1StD0ppsiVerHTwQ2x/.VA7jG9v7zSfLzF8Y61n.xdhz0Mi72UK', 'john2@email.cz', 'path/pic.png', null, null);

INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id)
VALUES (3, 'CharlesArtist', '$2b$12$jc1StD0ppsiVerHTwQ2x/.VA7jG9v7zSfLzF8Y61n.xdhz0Mi72UK', 'charles@email.cz', 'path/pic.png', null, null);

INSERT INTO user_table (id, username, password_hash, email, profile_picture_path, artist_id, paying_member_id)
VALUES (4, 'JohnPaying', '$2b$12$jc1StD0ppsiVerHTwQ2x/.VA7jG9v7zSfLzF8Y61n.xdhz0Mi72UK', 'johnPaying@email.cz', 'path/pic.png', null, null);

-- Artists
INSERT INTO artist(id, user_id, description)
VALUES (1, 1, 'Description');

INSERT INTO artist(id, user_id, description)
VALUES (2, 3, 'Description');

-- Paying members

INSERT INTO paying_member(id, user_id, valid_until, payment_method_id)
VALUES (1, 4, '2211020212', null);

INSERT INTO payment_method(id, paying_member_id, card_number, card_expiration_date, card_cvc) 
VALUES (1, 1, '1111222233334444', '2099-12-31', '123');

-- SET ARTIST REFERENCE FOR USERS
UPDATE user_table SET artist_id = 1 WHERE id = 1;
UPDATE user_table SET artist_id = 2 WHERE id = 3;

-- SET PAYING MEMBER REFERENCE FOR USERS
UPDATE user_table SET paying_member_id = 1 WHERE id = 4; 
COMMIT;