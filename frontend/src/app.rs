use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::navbar::Navbar;
use crate::pages::building_apartments::BuildingApartmentsPage;
use crate::pages::buildings::BuildingsPage;
use crate::pages::home::Home;
use crate::pages::login::LoginPage;
use crate::routes::Route;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Navbar />
            <Switch<Route> render={move |route| match route {
                Route::Home => html!{<Home />},
                Route::Buildings => html!{<BuildingsPage />},
                Route::BuildingApartments { .. } => html!{<BuildingApartmentsPage />},
                Route::Login => html!{<LoginPage />},
            }} />
            <footer class="app-footer py-4 mt-5 border-top">
                <div class="container d-flex justify-content-between small">
                    <span>{"© House Management"}</span>
                    <span class="text-muted">{"v0.1.0"}</span>
                </div>
            </footer>
        </BrowserRouter>
    }
}
