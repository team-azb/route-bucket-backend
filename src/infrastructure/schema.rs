table! {
    operations (route_id, index) {
        route_id -> Varchar,
        index -> Unsigned<Integer>,
        code -> Char,
        pos -> Nullable<Unsigned<Integer>>,
        polyline -> Mediumtext,
    }
}

table! {
    routes (id) {
        id -> Varchar,
        name -> Varchar,
        polyline -> Mediumtext,
        operation_pos -> Unsigned<Integer>,
    }
}

allow_tables_to_appear_in_same_query!(
    operations,
    routes,
);
