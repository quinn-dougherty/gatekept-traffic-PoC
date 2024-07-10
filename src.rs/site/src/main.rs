use yew::prelude::*;
mod components;
use components::pixels::PixelsComponent;

#[function_component]
fn App() -> Html {
    html! {
        <div>
            <h1>{"Pixels in Yew"}</h1>
            <PixelsComponent />
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
