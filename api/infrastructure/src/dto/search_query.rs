use itertools::Itertools;
use route_bucket_domain::model::route::RouteSearchQuery;

#[derive(Clone, Debug)]
enum WhereCondition {
    Eq(&'static str, String),
    In(&'static str, Vec<String>),
    And(Vec<WhereCondition>),
    Or(Vec<WhereCondition>),
}

impl WhereCondition {
    fn to_query(&self) -> String {
        match self {
            Self::Eq(field_name, value) => {
                format!("{} = {}", field_name, value)
            }
            Self::In(field_name, values) => {
                if values.is_empty() {
                    format!("false")
                } else {
                    format!("{} IN ({})", field_name, values.join(","))
                }
            }
            Self::And(values) | Self::Or(values) => match values.len() {
                0 => String::new(),
                1 => values[0].to_query(),
                _ => {
                    let sep = if matches!(self, Self::And(_)) {
                        " AND "
                    } else {
                        " OR "
                    };
                    format!("({})", values.into_iter().map(Self::to_query).join(sep))
                }
            },
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
    where_condition: Option<WhereCondition>,
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

        if let Some(cond) = self.where_condition.clone() {
            query += &format!("WHERE {} ", cond.to_query());
        }

        if !is_for_counting {
            if let Some(order_by) = &self.order_by {
                query += &format!("ORDER BY {} ", order_by.to_query());
            }

            if let Some(limit) = self.limit {
                query += &format!("LIMIT {} ", limit);
            }

            if let Some(offset) = self.offset {
                query += &format!("OFFSET {} ", offset);
            }
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

        let mut visibility_conditions = vec![WhereCondition::Eq("is_public", "true".into())];

        if let Some(ids) = route_search_query.ids {
            visibility_conditions.push(WhereCondition::In(
                "id",
                ids.into_iter().map(|id| format!("'{}'", id)).collect(),
            ));
        }

        if let Some(caller_id) = route_search_query.caller_id {
            visibility_conditions.push(WhereCondition::Eq("owner_id", format!("'{}'", caller_id)));
        }

        let mut conditions = vec![WhereCondition::Or(visibility_conditions)];

        if let Some(owner_id) = route_search_query.owner_id {
            conditions.push(WhereCondition::Eq("owner_id", format!("'{}'", owner_id)));
        }

        search_query.where_condition = Some(WhereCondition::And(conditions));

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
