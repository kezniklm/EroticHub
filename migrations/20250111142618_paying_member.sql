ALTER TABLE paying_member ADD COLUMN payment_method_id INTEGER REFERENCES payment_method(id) ON DELETE SET NULL;
ALTER TABLE paying_member ALTER COLUMN valid_until DROP NOT NULL;
ALTER TABLE paying_member ALTER COLUMN valid_until TYPE TIMESTAMP WITH TIME ZONE;
