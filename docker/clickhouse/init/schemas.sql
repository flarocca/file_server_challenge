CREATE DATABASE IF NOT EXISTS file_server;

CREATE TABLE file_server.files
(
  id           UUID,
  files_order  Array(String),
  files        Map(String, String),
  leaf_hashes  Array(String),
  root         Nullable(String),
  updated_at   DateTime64(3) DEFAULT now()
)
ENGINE = ReplacingMergeTree
PRIMARY KEY id
ORDER BY id;
