CREATE TABLE routes
(
    `id`                     VARCHAR(11)      NOT NULL,
    `name`                   VARCHAR(50)      NOT NULL,
    `owner_id`               VARCHAR(40)      NOT NULL,
    `operation_pos`          INTEGER UNSIGNED NOT NULL,
    `ascent_elevation_gain`  INTEGER UNSIGNED NOT NULL,
    `descent_elevation_gain` INTEGER UNSIGNED NOT NULL,
    `total_distance`         DOUBLE           NOT NULL,
    `is_public`              TINYINT(1)       NOT NULL,
    `created_at`    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `updated_at`    TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX updated_idx (`updated_at`),
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

CREATE TABLE users
(
    `id`        VARCHAR(40)      NOT NULL,
    `name`      VARCHAR(50)      NOT NULL,
    `gender`    VARCHAR(6)       NOT NULL,
    `birthdate` DATE                     ,
    `icon_url`  TEXT                     ,
    PRIMARY KEY (`id`)
);

CREATE TABLE permissions
(
    `route_id`        VARCHAR(11) NOT NULL,
    `user_id`         VARCHAR(40) NOT NULL,
    `permission_type` VARCHAR(6)  NOT NULL,
    PRIMARY KEY (`user_id`, `route_id`)
);
