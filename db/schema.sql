CREATE TABLE routes
(
    `id`            VARCHAR(11)      NOT NULL,
    `name`          VARCHAR(50)      NOT NULL,
    `operation_pos` INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (`id`)
);

CREATE TABLE segments
(
    `route_id` VARCHAR(11)                        NOT NULL,
    `index`    INTEGER UNSIGNED                   NOT NULL,
    `polyline` VARCHAR(65000) CHARACTER SET ascii NOT NULL,
    INDEX segment_idx (`route_id`, `index`)
);

CREATE TABLE operations
(
    `route_id` VARCHAR(11)                        NOT NULL,
    `index`    INTEGER UNSIGNED                   NOT NULL,
    `code`     CHAR(2) CHARACTER SET ascii        NOT NULL,
    `pos`      INTEGER UNSIGNED                   NOT NULL,
    `polyline` VARCHAR(65000) CHARACTER SET ascii NOT NULL,
    PRIMARY KEY (`route_id`, `index`)
);

CREATE TABLE users
(
    `id` VARCHAR(28) NOT NULL,
    PRIMARY KEY (`id`)
);