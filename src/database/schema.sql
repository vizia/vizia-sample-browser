CREATE TABLE collection (
    name nvarchar(255)  PRIMARY KEY,
    sub_collections     nvarchar(255) NULL,
    audio_files         nvarchar(255) NULL,
    created_at          timestamp,
    FOREIGN KEY(sub_collections) REFERENCES collection(name)
);

-- CREATE TABLE audio_files (
--   location nvarchar(255) UNIQUE PRIMARY KEY,
--   tags nvarchar(255),
--   duration integer,
--   sample_rate integer,
--   bit_depth integer,
--   bpm integer,
--   key integer,
--   size integer,
--   created_at timestamp
-- );

-- CREATE TABLE tags (
--   id nvarchar(255) PRIMARY KEY,
--   color integer
-- );

-- ALTER TABLE collection ADD FOREIGN KEY (audio_files) REFERENCES audio_files (location);

-- ALTER TABLE audio_files ADD FOREIGN KEY (tags) REFERENCES tags (id);

-- ALTER TABLE collection ADD FOREIGN KEY (sub_collections) REFERENCES collection (name);