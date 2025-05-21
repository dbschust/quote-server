-- Add migration script here

CREATE TABLE IF NOT EXISTS quotes (
  words VARCHAR(200) NOT NULL,
  author VARCHAR(200) NOT NULL
);