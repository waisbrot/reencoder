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
       last_modified timestamp with time zone NOT NULL,
       in_progress boolean NOT NULL DEFAULT false
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
  "interval": 3600
}'::jsonb),
('clean',
'{
  "interval": 3600
}'::jsonb),
('reencode',
'{
  "interval": 60,
  "target_extension": "mkv",
  "target_codec": "hevc"
}'::jsonb);
