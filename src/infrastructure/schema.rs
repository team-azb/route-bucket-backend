table! {
    coordinates (route_id, index) {
        route_id -> Varchar,
        index -> Integer,
        latitude -> Decimal,
        longitude -> Decimal,
    }
}

table! {
    routes (id) {
        id -> Varchar,
        name -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    coordinates,
    routes,
);
