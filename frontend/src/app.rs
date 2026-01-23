use yew::prelude::*;
use yew_router::prelude::*;

use frontend::components::app_layout::AppLayout;
use frontend::components::navbar::Navbar;
use frontend::contexts::{AuthContext, AuthProvider};
use frontend::pages::admin::AdminAnnouncementsPage;
use frontend::pages::admin::AdminPage;
use frontend::pages::admin::AdminPropertiesPage;
use frontend::pages::building_apartments::BuildingApartmentsPage;
use frontend::pages::buildings::BuildingsPage;
use frontend::pages::health::HealthPage;
use frontend::pages::home::Home;
use frontend::pages::login::LoginPage;
use frontend::pages::maintenance::{
    MaintenanceDetailPage, MaintenanceListPage, MaintenanceNewPage,
};
use frontend::pages::manage::ManagePage;
use frontend::pages::meters::{
    MeterCalibrationPage, MeterDetailPage, MeterListPage, MeterManagementPage, MeterNewPage,
};
use frontend::pages::voting::{VotingDetailPage, VotingListPage, VotingNewPage};
use frontend::routes::Route;

#[function_component(AppContent)]
fn app_content() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let is_authenticated = auth.is_authenticated();

    html! {
        <>
            <Navbar />
            <Switch<Route> render={move |route| {
                // Routes that don't need AppLayout (public/login)
                if matches!(route, Route::Login) {
                    return match route {
                        Route::Login => html!{<LoginPage />},
                        _ => html!{<div>{"Not found"}</div>},
                    };
                }

                // Home page: use layout only if authenticated
                if matches!(route, Route::Home) {
                    return if is_authenticated {
                        html!{
                            <AppLayout active_route={route.clone()}>
                                <Home />
                            </AppLayout>
                        }
                    } else {
                        html!{<Home />}
                    };
                }

                // All other routes: wrap with AppLayout
                html!{
                    <AppLayout active_route={route.clone()}>
                        {match route {
                            Route::Buildings => html!{<BuildingsPage />},
                            Route::BuildingApartments { .. } => html!{<BuildingApartmentsPage />},
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
                            Route::ApartmentMeters { apartment_id } => html!{<MeterListPage apartment_id={apartment_id} />},
                            Route::MeterDetail { id } => html!{<MeterDetailPage id={id} />},
                            Route::MeterManagement => html!{<MeterManagementPage />},
                            Route::MeterNew => html!{<MeterNewPage />},
                            Route::MeterCalibration => html!{<MeterCalibrationPage />},
                            _ => html!{<div>{"Not found"}</div>},
                        }}
                    </AppLayout>
                }
            }} />
        </>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <AuthProvider>
            <BrowserRouter>
                <AppContent />
            </BrowserRouter>
        </AuthProvider>
    }
}
