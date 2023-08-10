CREATE TABLE collections (
    id                  integer UNIQUE PRIMARY KEY,
    parent_collection   integer NULL,
    path                nvarchar(255),
    name                nvarchar(255),

    FOREIGN KEY(parent_collection) REFERENCES collections(id)
);

CREATE TABLE audio_files (
    id                  integer UNIQUE PRIMARY KEY,
    name                nvarchar(255),
    collection          integer,
    duration            integer,
    sample_rate         integer,
    bit_depth           integer,
    bpm                 integer NULL,
    key                 integer NULL,
    size                integer,

    FOREIGN KEY(collection) REFERENCES collections(id)
);

CREATE TABLE tags (
    id                  integer PRIMARY KEY,
    name                nvarchar(255),
    color               nvarchar(8)
);

CREATE TABLE audio_files_tags (
    audio_file          integer,
    tag                 integer,

    PRIMARY KEY (audio_file, tag)

    FOREIGN KEY(audio_file) REFERENCES audio_files(id)
    FOREIGN KEY(tag) REFERENCES tags(id)
);