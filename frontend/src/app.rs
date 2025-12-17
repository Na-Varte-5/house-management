use yew::prelude::*;
use yew_router::prelude::*;

use frontend::components::navbar::Navbar;
use frontend::contexts::AuthProvider;
use frontend::pages::admin::AdminPage;
use frontend::pages::admin::AdminAnnouncementsPage;
use frontend::pages::admin::AdminPropertiesPage;
use frontend::pages::building_apartments::BuildingApartmentsPage;
use frontend::pages::buildings::BuildingsPage;
use frontend::pages::home::Home;
use frontend::pages::login::LoginPage;
use frontend::pages::manage::ManagePage;
use frontend::pages::health::HealthPage;
use frontend::pages::maintenance::{MaintenanceListPage, MaintenanceNewPage, MaintenanceDetailPage};
use frontend::pages::voting::{VotingListPage, VotingNewPage, VotingDetailPage};
use frontend::routes::Route;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <AuthProvider>
            <BrowserRouter>
                <Navbar />
                <Switch<Route> render={move |route| match route {
                    Route::Home => html!{<Home />},
                    Route::Buildings => html!{<BuildingsPage />},
                    Route::BuildingApartments { .. } => html!{<BuildingApartmentsPage />},
                    Route::Login => html!{<LoginPage />},
                    Route::Admin => html!{<AdminPage />},
                    Route::AdminAnnouncements => html!{<AdminAnnouncementsPage />},
                    Route::AdminProperties => html!{<AdminPropertiesPage />},
                    Route::Manage => html!{<ManagePage />},
                    Route::Health => html!{<HealthPage />},
                    Route::Maintenance => html!{<MaintenanceListPage />},
                    Route::MaintenanceNew => html!{<MaintenanceNewPage />},
                    Route::MaintenanceDetail { id } => html!{<MaintenanceDetailPage id={id} />},
                    Route::Voting => html!{<VotingListPage />},
                    Route::VotingNew => html!{<VotingNewPage />},
                    Route::VotingDetail { id } => html!{<VotingDetailPage id={id} />},
                }} />
                <footer class="app-footer py-4 mt-5 border-top">
                    <div class="container d-flex justify-content-between small">
                        <span>{"© House Management"}</span>
                        <span class="text-muted">{"v0.1.0"}</span>
                    </div>
                </footer>
            </BrowserRouter>
        </AuthProvider>
    }
}
