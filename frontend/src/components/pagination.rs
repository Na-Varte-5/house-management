use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PaginationProps {
    pub current_page: i64,
    pub total_pages: i64,
    pub on_page_change: Callback<i64>,
    #[prop_or(0)]
    pub total_items: i64,
}

#[function_component(Pagination)]
pub fn pagination(props: &PaginationProps) -> Html {
    if props.total_pages <= 1 {
        return html! {};
    }

    let current = props.current_page;
    let total = props.total_pages;

    let pages = build_page_numbers(current, total);

    let on_prev = {
        let cb = props.on_page_change.clone();
        let page = current;
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if page > 1 {
                cb.emit(page - 1);
            }
        })
    };

    let on_next = {
        let cb = props.on_page_change.clone();
        let page = current;
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            if page < total {
                cb.emit(page + 1);
            }
        })
    };

    html! {
        <div class="d-flex justify-content-between align-items-center mt-3">
            if props.total_items > 0 {
                <small class="text-muted">{format!("{} items total", props.total_items)}</small>
            } else {
                <div></div>
            }
            <nav>
                <ul class="pagination pagination-sm mb-0">
                    <li class={classes!("page-item", (current <= 1).then_some("disabled"))}>
                        <a class="page-link" href="#" onclick={on_prev}>{"‹"}</a>
                    </li>
                    { for pages.iter().map(|p| {
                        match p {
                            PageNumber::Page(n) => {
                                let n = *n;
                                let is_active = n == current;
                                let cb = props.on_page_change.clone();
                                let on_click = Callback::from(move |e: MouseEvent| {
                                    e.prevent_default();
                                    cb.emit(n);
                                });
                                html! {
                                    <li class={classes!("page-item", is_active.then_some("active"))}>
                                        <a class="page-link" href="#" onclick={on_click}>{n}</a>
                                    </li>
                                }
                            }
                            PageNumber::Ellipsis => {
                                html! {
                                    <li class="page-item disabled">
                                        <span class="page-link">{"…"}</span>
                                    </li>
                                }
                            }
                        }
                    })}
                    <li class={classes!("page-item", (current >= total).then_some("disabled"))}>
                        <a class="page-link" href="#" onclick={on_next}>{"›"}</a>
                    </li>
                </ul>
            </nav>
        </div>
    }
}

enum PageNumber {
    Page(i64),
    Ellipsis,
}

fn build_page_numbers(current: i64, total: i64) -> Vec<PageNumber> {
    if total <= 7 {
        return (1..=total).map(PageNumber::Page).collect();
    }

    let mut pages = Vec::new();
    pages.push(PageNumber::Page(1));

    if current > 3 {
        pages.push(PageNumber::Ellipsis);
    }

    let start = (current - 1).max(2);
    let end = (current + 1).min(total - 1);

    for i in start..=end {
        pages.push(PageNumber::Page(i));
    }

    if current < total - 2 {
        pages.push(PageNumber::Ellipsis);
    }

    pages.push(PageNumber::Page(total));
    pages
}
