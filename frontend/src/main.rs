mod app;

use app::App;
use frontend::i18n::init_translations;

fn main() {
    init_translations();
    yew::Renderer::<App>::new().render();
}
