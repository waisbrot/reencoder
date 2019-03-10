CREATE DATABASE media;
\connect media
COMMENT ON DATABASE media IS 'Data on downloaded media';

CREATE ROLE media WITH LOGIN PASSWORD 'media';
GRANT ALL PRIVILEGES ON DATABASE media TO media;
SET ROLE media;

CREATE TABLE paths (hash text PRIMARY KEY, path text NOT NULL);
CREATE INDEX paths_path ON paths (path);
