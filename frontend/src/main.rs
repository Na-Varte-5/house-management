mod app;
mod i18n;

use app::App;
use i18n::init_translations;

fn main() {
    // Initialize translations
    init_translations();

    // Render the app
    yew::Renderer::<App>::new().render();
}
