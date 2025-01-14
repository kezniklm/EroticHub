CREATE TABLE IF NOT EXISTS deal (
  id SERIAL PRIMARY KEY,
  label TEXT UNIQUE NOT NULL,
  price_per_month DECIMAL(10, 2) NOT NULL,
  number_of_months INTEGER NOT NULL
);

INSERT INTO deal (label, price_per_month, number_of_months) VALUES
('One Month', 10.00, 1),
('Three Months', 9.00, 3),
('One Year', 8.00, 12);
