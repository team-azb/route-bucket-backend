use route_bucket_domain::model::user::Gender;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <h1>{ format!("{:#?}", Gender::default()) }</h1>
    }
}

fn main() {
    yew::start_app::<App>();
}
