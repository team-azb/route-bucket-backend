-- This file should undo anything in `up.sql`
DROP TABLE operations;

ALTER TABLE routes DROP COLUMN `polyline`,
                    DROP COLUMN `operation_pos`;
CREATE TABLE coordinates (
     PRIMARY KEY (`route_id`, `index`),
     `route_id` VARCHAR(11) NOT NULL,
     `index` INTEGER UNSIGNED NOT NULL,
     `latitude` DECIMAL NOT NULL,
     `longitude` DECIMAL NOT NULL
);
