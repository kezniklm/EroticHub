CREATE TABLE IF NOT EXISTS user_table (
  id SERIAL PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  email TEXT UNIQUE NOT NULL,
  profile_picture_path TEXT
  -- added later for database consistency:
  -- artist_id INTEGER REFERENCES artist(id) ON DELETE SET NULL,
  -- paying_member_id INTEGER REFERENCES paying_member(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS artist (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES user_table(id) ON DELETE CASCADE,
  description TEXT
);

CREATE TABLE IF NOT EXISTS paying_member (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES user_table(id) ON DELETE CASCADE,
  valid_until TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS payment_method (
  id SERIAL PRIMARY KEY,
  paying_member_id INTEGER NOT NULL REFERENCES paying_member(id) ON DELETE CASCADE
);

ALTER TABLE user_table
ADD COLUMN artist_id INTEGER REFERENCES artist(id) ON DELETE SET NULL,
ADD COLUMN paying_member_id INTEGER REFERENCES paying_member(id) ON DELETE SET NULL;

CREATE TYPE visibility_type AS ENUM ('ALL', 'REGISTERED', 'PAYING');
CREATE TABLE IF NOT EXISTS video (
  id SERIAL PRIMARY KEY,
  artist_id INTEGER NOT NULL REFERENCES artist(id) ON DELETE CASCADE,
  visibility visibility_type NOT NULL,
  name TEXT NOT NULL,
  file_path TEXT NOT NULL,
  thumbnail_path TEXT NOT NULL,
  description TEXT
);

CREATE TABLE IF NOT EXISTS live_stream (
  id SERIAL PRIMARY KEY,
  video_id INTEGER NOT NULL REFERENCES video(id) ON DELETE CASCADE,
  start_time TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS comment (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL REFERENCES user_table(id) ON DELETE CASCADE,
  video_id INTEGER NOT NULL REFERENCES video(id) ON DELETE CASCADE,
  content TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS favorite (
  user_id INTEGER NOT NULL REFERENCES user_table(id) ON DELETE CASCADE,
  video_id INTEGER NOT NULL REFERENCES video(id) ON DELETE CASCADE,
  PRIMARY KEY (user_id, video_id)
);

CREATE TABLE IF NOT EXISTS video_category (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS video_category_video (
  video_id INTEGER NOT NULL REFERENCES video(id) ON DELETE CASCADE,
  category_id INTEGER NOT NULL REFERENCES video_category(id) ON DELETE CASCADE,
  PRIMARY KEY (video_id, category_id)
);
