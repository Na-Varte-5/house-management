use yew::prelude::*;
use crate::components::announcement_list::AnnouncementList;

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <div class="container mt-4">
            <h1>{"Announcements"}</h1>
            <AnnouncementList />
        </div>
    }
}
