CREATE TABLE routes
(
    `id`            VARCHAR(11)      NOT NULL,
    `name`          VARCHAR(50)      NOT NULL,
    `operation_pos` INTEGER UNSIGNED NOT NULL,
    PRIMARY KEY (`id`)
);

CREATE TABLE segments
(
    `id`       VARCHAR(21)                        NOT NULL,
    `route_id` VARCHAR(11)                        NOT NULL,
    `index`    INTEGER UNSIGNED                   NOT NULL,
    `mode`     VARCHAR(15)    CHARACTER SET ascii NOT NULL,
    `polyline` VARCHAR(65000) CHARACTER SET ascii NOT NULL,
    INDEX segment_idx (`route_id`, `index`),
    PRIMARY KEY (`id`)
);

CREATE TABLE operations
(
    `id`       VARCHAR(21)                        NOT NULL,
    `route_id` VARCHAR(11)                        NOT NULL,
    `index`    INTEGER UNSIGNED                   NOT NULL,
    `code`     CHAR(2) CHARACTER SET ascii        NOT NULL,
    `pos`      INTEGER UNSIGNED                   NOT NULL,
    `org_seg_templates` JSON                      NOT NULL,
    `new_seg_templates` JSON                      NOT NULL,
    INDEX segment_idx (`route_id`, `index`),
    PRIMARY KEY (`id`)
);