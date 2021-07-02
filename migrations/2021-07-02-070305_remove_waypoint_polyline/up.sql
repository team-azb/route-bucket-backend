-- Your SQL goes here
ALTER TABLE routes
    DROP COLUMN `waypoint_polyline`;
ALTER TABLE segments
    DROP COLUMN `distance`;
ALTER TABLE operations
    CHANGE `pos` `pos`   INTEGER UNSIGNED       NOT NULL,
    CHANGE `code` `code` CHAR(2) CHAR SET ascii NOT NULL;