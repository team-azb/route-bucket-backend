use route_bucket_domain::model::route::RouteInfo;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <h1>{ format!("hello {:?}!", RouteInfo::default()) }</h1>
    }
}

fn main() {
    yew::start_app::<App>();
}
