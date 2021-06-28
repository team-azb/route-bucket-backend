-- This file should undo anything in `up.sql`
DROP TABLE segments;

ALTER TABLE routes CHANGE `waypoint_polyline` `polyline` MEDIUMTEXT CHAR SET latin1 NOT NULL;

ALTER TABLE operations CHANGE `polyline` `polyline` MEDIUMTEXT CHAR SET latin1 NOT NULL;