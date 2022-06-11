use std::str::FromStr;

use chrono::NaiveDate;
use route_bucket_domain::model::{
    route::RouteInfo,
    types::{Email, Url},
    user::{Gender, UserId},
};
use route_bucket_usecase::user::UserCreateRequest;
use stylist::style;
use yew::prelude::*;

struct MyStyle;
impl MyStyle {
    fn h1() -> String {
        style!("color: navy;").unwrap().get_class_name().into()
    }

    fn div() -> String {
        style!("white-space: pre-wrap;")
            .unwrap()
            .get_class_name()
            .into()
    }
}

#[function_component(App)]
fn app() -> Html {
    let req = UserCreateRequest::from((
        UserId::from("hoge".to_string()),
        "hoge".to_string(),
        Email::try_from("sample@email.com".to_string()).unwrap(),
        Gender::Male,
        NaiveDate::from_str("2000-01-01").ok(),
        Url::try_from("https://google.com".to_string()).ok(),
        "password".to_string(),
    ));
    let (user, email, _) = req.into();
    html! {
        <>
            <h1 class={MyStyle::h1()}>
                { format!("Hello {} ({:?})!", user.name(), email) }
            </h1>
            <div class={MyStyle::div()}>
                { format!("User: {:#?}!", user) }
            </div>
            <br/>
            <div style={MyStyle::div()}>
                { format!("Route: {:#?}", RouteInfo::default()) }
            </div>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
