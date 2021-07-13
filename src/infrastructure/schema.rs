table! {
    operations (route_id, index) {
        route_id -> Varchar,
        index -> Unsigned<Integer>,
        code -> Char,
        pos -> Unsigned<Integer>,
        polyline -> Varchar,
    }
}

table! {
    routes (id) {
        id -> Varchar,
        name -> Varchar,
        operation_pos -> Unsigned<Integer>,
    }
}

table! {
    segments (route_id, index) {
        route_id -> Varchar,
        index -> Integer,
        polyline -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(operations, routes, segments,);
