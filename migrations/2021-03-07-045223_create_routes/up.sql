-- Your SQL goes here

CREATE TABLE routes (
   `id` VARCHAR(11) PRIMARY KEY,
   `name` VARCHAR(50) NOT NULL
);

CREATE TABLE coordinates (
    PRIMARY KEY (`route_id`, `index`),
    `route_id` VARCHAR(11) NOT NULL,
    `index` INTEGER NOT NULL,
    `latitude` DECIMAL NOT NULL,
    `longitude` DECIMAL NOT NULL
);