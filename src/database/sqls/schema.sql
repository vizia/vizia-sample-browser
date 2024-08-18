CREATE TABLE collections (
    id                  integer UNIQUE PRIMARY KEY,
    parent_collection   integer NULL,
    name                nvarchar(255),
    path                nvarchar(255),

    CONSTRAINT fk_coll_coll 
        FOREIGN KEY(parent_collection) 
        REFERENCES collections(id)
        ON DELETE CASCADE
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

    CONSTRAINT fk_af_coll 
        FOREIGN KEY(collection) 
        REFERENCES collections(id)
        ON DELETE CASCADE
);

CREATE TABLE tags (
    id                  integer PRIMARY KEY,
    name                nvarchar(255),
    color               nvarchar(8),
    number              integer
);

CREATE TABLE audio_files_tags (
    audio_file          integer,
    tag                 integer,

    PRIMARY KEY (audio_file, tag)

    CONSTRAINT fk_aft_af 
        FOREIGN KEY(audio_file) 
        REFERENCES audio_files(id) 
        ON DELETE CASCADE
    CONSTRAINT fk_aft_tag 
        FOREIGN KEY(tag) 
        REFERENCES tags(id) 
        ON DELETE CASCADE
);