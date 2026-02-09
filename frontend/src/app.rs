use yew::prelude::*;
use yew_router::prelude::*;

use frontend::components::app_layout::AppLayout;
use frontend::components::navbar::Navbar;
use frontend::components::toast::ToastProvider;
use frontend::contexts::{AuthContext, AuthProvider, LanguageContext, LanguageProvider};
use frontend::i18n::t;
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
use frontend::pages::meters::{
    MeterCalibrationPage, MeterDetailPage, MeterListPage, MeterManagementPage, MeterNewPage,
};
use frontend::pages::my_properties::MyProperties;
use frontend::pages::my_property_detail::MyPropertyDetailPage;
use frontend::pages::voting::{VotingDetailPage, VotingListPage, VotingNewPage};
use frontend::routes::Route;

#[function_component(AppContent)]
fn app_content() -> Html {
    let auth = use_context::<AuthContext>().expect("AuthContext not found");
    let lang_ctx = use_context::<LanguageContext>().expect("LanguageContext not found");
    let is_authenticated = auth.is_authenticated();
    let current_route = use_route::<Route>();

    html! {
        <div key={lang_ctx.language.clone()}>
            if !matches!(current_route, Some(Route::Login)) {
                <Navbar />
            }
            <Switch<Route> render={move |route| {
                if matches!(route, Route::Login) {
                    return match route {
                        Route::Login => html!{<LoginPage />},
                        _ => html!{<div>{t("page-not-found")}</div>},
                    };
                }

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

                html!{
                    <AppLayout active_route={route.clone()}>
                        {match route {
                            Route::Buildings => html!{<BuildingsPage />},
                            Route::BuildingApartments { .. } => html!{<BuildingApartmentsPage />},
                            Route::Admin => html!{<AdminPage />},
                            Route::AdminAnnouncements => html!{<AdminAnnouncementsPage />},
                            Route::AdminProperties => html!{<AdminPropertiesPage />},
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
                            Route::MyProperties => html!{<MyProperties />},
                            Route::MyPropertyDetail { apartment_id } => html!{<MyPropertyDetailPage apartment_id={apartment_id} />},
                            _ => html!{<div>{t("page-not-found")}</div>},
                        }}
                    </AppLayout>
                }
            }} />
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <AuthProvider>
            <LanguageProvider>
                <ToastProvider>
                    <BrowserRouter>
                        <AppContent />
                    </BrowserRouter>
                </ToastProvider>
            </LanguageProvider>
        </AuthProvider>
    }
}
