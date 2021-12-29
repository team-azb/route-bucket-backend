use std::collections::HashMap;

use itertools::Itertools;
use route_bucket_domain::model::route::RouteSearchQuery;

#[derive(Clone, Debug)]
enum WhereCondition {
    Eq(String),
}

impl WhereCondition {
    fn to_query(&self, field_name: &'static str) -> String {
        match self {
            Self::Eq(value) => {
                format!("{} = \"{}\"", field_name, value)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct OrderBy {
    field_name: &'static str,
    descending: bool,
}

impl OrderBy {
    fn to_query(&self) -> String {
        format!(
            "`{}` {}",
            self.field_name,
            if self.descending { "DESC" } else { "" }
        )
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SearchQuery {
    table_name: &'static str,
    where_conditions: HashMap<&'static str, WhereCondition>,
    order_by: Option<OrderBy>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl SearchQuery {
    pub fn to_sql(&self, is_for_counting: bool) -> String {
        let mut query = format!(
            "SELECT {} FROM {} ",
            if is_for_counting { "COUNT(*)" } else { "*" },
            self.table_name
        );

        if !self.where_conditions.is_empty() {
            query += &format!(
                "WHERE {} ",
                self.where_conditions
                    .iter()
                    .map(|(field, cond)| cond.to_query(field))
                    .join(", "),
            );
        }

        if let Some(order_by) = &self.order_by {
            query += &format!("ORDER BY {} ", order_by.to_query());
        }

        if let Some(limit) = self.limit {
            query += &format!("LIMIT {} ", limit);
        }

        if let Some(offset) = self.offset {
            query += &format!("OFFSET {} ", offset);
        }

        query
    }
}

impl From<RouteSearchQuery> for SearchQuery {
    fn from(route_search_query: RouteSearchQuery) -> Self {
        let mut search_query = SearchQuery {
            table_name: "routes",
            ..Default::default()
        };

        if let Some(owner_id) = route_search_query.owner_id {
            search_query
                .where_conditions
                .insert("owner_id", WhereCondition::Eq(owner_id.to_string()));
        }

        // TODO: ここを指定できるようにする
        search_query.order_by = Some(OrderBy {
            field_name: "updated_at",
            descending: true,
        });

        if let Some(page_size) = route_search_query.page_size {
            search_query.limit = Some(page_size);
            search_query.offset = Some(page_size * route_search_query.page_offset);
        }

        search_query
    }
}
