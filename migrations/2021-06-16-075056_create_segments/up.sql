-- Your SQL goes here

CREATE TABLE segments (
    PRIMARY KEY (`route_id`, `index`),
    `route_id` VARCHAR(11) NOT NULL,
    # UNSIGNEDだとなぜかdieselでインクリメントができないので
    `index` INTEGER NOT NULL,
    `polyline` VARCHAR(65000) CHAR SET ascii NOT NULL,
    `distance` DOUBLE NOT NULL
);

ALTER TABLE routes CHANGE `polyline` `waypoint_polyline` VARCHAR(65000) CHAR SET ascii NOT NULL;

ALTER TABLE operations CHANGE `polyline` `polyline` VARCHAR(100) CHAR SET ascii NOT NULL;