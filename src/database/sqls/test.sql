INSERT INTO collections(id, parent_collection, name, path) VALUES (0, NULL, "Sample Library", "");
INSERT INTO collections(id, parent_collection, name, path) VALUES (1, 0, "Library 1", "");
INSERT INTO collections(id, parent_collection, name, path) VALUES (2, 0, "Library 2", "");
INSERT INTO collections(id, parent_collection, name, path) VALUES (3, 1, "Sub Library 1.1", "");
INSERT INTO collections(id, parent_collection, name, path) VALUES (4, 1, "Sub Library 1.2", "");

INSERT INTO audio_files(id, name, collection, duration, sample_rate, bit_depth, bpm, key, size) VALUES (0, "Audio File 0", 0, 0, 0, 0, 0, 0, 0);
INSERT INTO audio_files(id, name, collection, duration, sample_rate, bit_depth, bpm, key, size) VALUES (1, "Audio File 1", 1, 0, 0, 0, 0, 0, 0);

INSERT INTO tags(id, name, color) VALUES (0, "Tag 0", "f00");
INSERT INTO tags(id, name, color) VALUES (1, "Tag 1", "0f0");
INSERT INTO tags(id, name, color) VALUES (2, "Tag 2", "00f");

INSERT INTO audio_files_tags(audio_file, tag) VALUES (0, 0);
INSERT INTO audio_files_tags(audio_file, tag) VALUES (1, 1);
INSERT INTO audio_files_tags(audio_file, tag) VALUES (1, 2);