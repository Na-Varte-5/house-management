use yew::prelude::*;

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    // Keep for compatibility; encourage navbar dropdown usage
    html! {
        <div class="container mt-4" style="max-width: 480px; padding-top: 56px;">
            <div class="alert alert-info">{"Please use the Login button in the top bar to sign in or register."}</div>
        </div>
    }
}
