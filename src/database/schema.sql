CREATE TABLE collection (
    id                  integer UNIQUE PRIMARY KEY,
    parent_collection   integer NULL,
    name                nvarchar(255),

    FOREIGN KEY(parent_collection) REFERENCES collection(id)
);

CREATE TABLE audio_files (
    id                  integer UNIQUE PRIMARY KEY,
    collection          integer,
    duration            integer,
    sample_rate         integer,
    bit_depth           integer,
    bpm                 integer NULL,
    key                 integer NULL,
    size                integer,

    FOREIGN KEY(collection) REFERENCES collection(name)
);

CREATE TABLE tags (
    id                  nvarchar(255) PRIMARY KEY,
    color               integer
);

CREATE TABLE audio_files_tags (
    audio_file          integer,
    tag                 varchar,

    PRIMARY KEY (audio_file, tag)

    FOREIGN KEY(audio_file) REFERENCES audio_files(id)
    FOREIGN KEY(tag) REFERENCES tags(id)
);