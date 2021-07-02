-- This file should undo anything in `up.sql`
ALTER TABLE routes
    ADD `waypoint_polyline` VARCHAR(65000) CHAR SET ascii NOT NULL;
ALTER TABLE segments
    ADD `distance` DOUBLE NOT NULL;
ALTER TABLE operations
    CHANGE `pos` `pos`   INTEGER UNSIGNED,
    CHANGE `code` `code` CHAR(5) NOT NULL;