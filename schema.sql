\connect postgres
DROP DATABASE media;
DROP role media;
CREATE DATABASE media;
\connect media
COMMENT ON DATABASE media IS 'Data on downloaded media';

CREATE ROLE media WITH LOGIN PASSWORD 'media';
ALTER DATABASE media OWNER TO media;
GRANT ALL PRIVILEGES ON DATABASE media TO media;
SET ROLE media;

CREATE TABLE roots (
       root text PRIMARY KEY,
       active boolean NOT NULL
);

CREATE TABLE paths (
       id bigserial PRIMARY KEY,
       hash text NOT NULL,
       path text NOT NULL,
       codec text,
       height integer,
       width integer,
       kbps real,
       extension text,
       bytes bigint NOT NULL,
       last_modified timestamp with time zone NOT NULL
);
CREATE UNIQUE INDEX paths_path ON paths (path);

CREATE TABLE video_extensions (
       extension text PRIMARY KEY
);
INSERT INTO video_extensions (extension) VALUES
('avi'),('mp4'),('m4v'),('mkv'),('iso'),('m2ts');

CREATE TABLE config (
       service text PRIMARY KEY,
       config jsonb NOT NULL
);
INSERT INTO config (service, config) VALUES
('scan',
'{
  "interval": 60
}'::jsonb),
('clean',
'{
  "interval": 60
}'::jsonb);
('convert',
'{
  "interval": 60,
  "args": [
    "-y",
    "-c:v", "libx265",
    "-crf", "28",
    "-c:a", "aac",
    "-hide_banner",
    "-nostats",
    "-v", "warning"
  ]
}'::jsonb);