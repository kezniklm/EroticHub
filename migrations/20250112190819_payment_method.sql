ALTER TABLE payment_method ADD COLUMN card_number TEXT NOT NULL;
ALTER TABLE payment_method ADD COLUMN card_expiration_date DATE NOT NULL;
ALTER TABLE payment_method ADD COLUMN card_cvc TEXT NOT NULL;
