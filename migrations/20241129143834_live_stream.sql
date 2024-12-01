ALTER TABLE live_stream ALTER COLUMN start_time SET DEFAULT now();

CREATE TYPE live_stream_status AS ENUM ('PENDING', 'RUNNING', 'ENDED');
ALTER TABLE live_stream ADD COLUMN status live_stream_status NOT NULL DEFAULT 'PENDING';