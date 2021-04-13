-- Your SQL goes here
CREATE TABLE operations (
    PRIMARY KEY (`route_id`, `index`),
    `route_id` VARCHAR(11) NOT NULL,
    `index` INTEGER UNSIGNED NOT NULL,
    `code` CHAR(5) NOT NULL, -- add, rm, clear, init
    `pos` INTEGER UNSIGNED,
    `polyline` MEDIUMTEXT CHAR SET latin1 NOT NULL
);

ALTER TABLE routes ADD (
    `polyline` MEDIUMTEXT CHAR SET latin1 NOT NULL,
    `operation_pos` INTEGER UNSIGNED NOT NULL
);
DROP TABLE coordinates;