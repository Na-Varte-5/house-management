use crate::components::spinner::Spinner;
use crate::i18n::t;
use crate::utils::api::api_url;
use crate::utils::datetime::format_dt_local;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct AnnouncementDto {
    pub id: u64,
    pub title: String,
    pub body_md: String,
    pub body_html: String,
    pub author_id: u64,
    pub author_name: String,
    pub pinned: bool,
    pub public: bool,
    pub publish_at: Option<String>,
    pub expire_at: Option<String>,
    pub roles_csv: Option<String>,
    pub building_id: Option<u64>,
    pub building_address: Option<String>,
    pub apartment_id: Option<u64>,
    pub apartment_number: Option<String>,
    pub comments_enabled: bool,
}

#[function_component(AnnouncementList)]
pub fn announcement_list() -> Html {
    let announcements = use_state(|| None::<Vec<AnnouncementDto>>);
    let loading = use_state(|| false);
    let expanded = use_state(|| Vec::<u64>::new());

    {
        let announcements = announcements.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match reqwasm::http::Request::get(&api_url("/api/v1/announcements/public"))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.ok() {
                            if let Ok(list) = resp.json::<Vec<AnnouncementDto>>().await {
                                let mapped: Vec<AnnouncementDto> = list
                                    .into_iter()
                                    .map(|mut a| {
                                        if a.body_html.trim().is_empty() {
                                            a.body_html = format!(
                                                "<pre>{}</pre>",
                                                html_escape::encode_text(&a.body_md)
                                            );
                                        }
                                        a
                                    })
                                    .collect();
                                announcements.set(Some(mapped));
                            }
                        }
                        loading.set(false);
                    }
                    Err(_) => {
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    if *loading {
        return html! {<Spinner center={true} />};
    }
    let anns_opt = (*announcements).clone();
    if anns_opt.as_ref().map(|v| v.is_empty()).unwrap_or(true) {
        return html! {<div class="alert alert-info">{ t("announcement-no-items") }</div>};
    }

    let now_str: String = js_sys::Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_default();

    let rendered_cards: Html = {
        let list_iter = anns_opt.unwrap();
        let mut v: Vec<Html> = Vec::new();
        for a in list_iter.iter() {
            let status_badges: Html = {
                let mut badges: Vec<Html> = Vec::new();
                if a.pinned {
                    badges.push(html!{<span class="badge bg-warning text-dark me-2">{ t("announcement-status-pinned") }</span>});
                }
                if let Some(p) = &a.publish_at {
                    if p > &now_str {
                        badges.push(html!{<span class="badge bg-info text-dark me-2">{ t("announcement-status-scheduled") }</span>});
                    }
                }
                if let Some(e) = &a.expire_at {
                    if e < &now_str {
                        badges.push(html!{<span class="badge bg-dark me-2">{ t("announcement-status-expired") }</span>});
                    }
                }
                html! {<>{ for badges }</>}
            };
            let audience_badges: Html = {
                let mut badges: Vec<Html> = Vec::new();
                if a.public {
                    badges.push(html!{<span class="badge bg-success me-1">{ t("announcement-public-label") }</span>});
                } else {
                    badges.push(html!{<span class="badge bg-secondary me-1">{ t("announcement-private-label") }</span>});
                }
                if let Some(csv) = &a.roles_csv {
                    for role in csv.split(',').map(|r| r.trim()).filter(|r| !r.is_empty()) {
                        badges.push(html! {
                            <span class="badge bg-primary me-1">{role}</span>
                        });
                    }
                }
                if let Some(addr) = &a.building_address {
                    badges.push(html!{<span class="badge bg-info text-dark me-1">{format!("{} {}", t("announcement-building-prefix"), addr)}</span>});
                }
                if let Some(num) = &a.apartment_number {
                    badges.push(html!{<span class="badge bg-warning text-dark me-1">{format!("{} {}", t("announcement-apartment-prefix"), num)}</span>});
                }
                html! {<div class="mt-1">{ for badges }</div>}
            };

            let is_expanded = expanded.contains(&a.id);
            let toggle_expanded = {
                let expanded = expanded.clone();
                let id = a.id;
                Callback::from(move |_| {
                    let mut vv = (*expanded).clone();
                    if let Some(pos) = vv.iter().position(|x| *x == id) {
                        vv.remove(pos);
                    } else {
                        vv.push(id);
                    }
                    expanded.set(vv);
                })
            };

            v.push(html! {
                <div class="card mb-3 border-secondary" key={a.id}>
                    <div class="card-header">
                        <div class="d-flex align-items-center gap-2 flex-wrap">
                            {status_badges}
                            <h5 class="mb-0 fw-bold">{ &a.title }</h5>
                        </div>
                        <div class="small text-muted">{format!("{} {}", t("announcement-by-prefix"), a.author_name)}</div>
                        { audience_badges }
                    </div>
                    <div class="card-body">
                        <div class="announcement-body">
                            { Html::from_html_unchecked(a.body_html.clone().into()) }
                        </div>
                        <div class="mt-2 small text-muted">
                            { if let Some(p) = a.publish_at.clone() { html!{<span class="me-2">{format!("{} {}", t("announcement-published-prefix"), format_dt_local(&p))}</span>} } else { html!{<span class="me-2">{ t("announcement-published-now") }</span>} } }
                            { if let Some(e) = a.expire_at.clone() { html!{<span>{format!("{} {}", t("announcement-expires-prefix"), format_dt_local(&e))}</span>} } else { html!{<span>{ t("announcement-no-expiry") }</span>} } }
                        </div>
                        { if a.comments_enabled { html!{<div class="mt-2"><button class="btn btn-sm btn-outline-primary" onclick={toggle_expanded.clone()}>{ if is_expanded { t("announcement-hide-comments") } else { t("announcement-show-comments") } }</button></div>} } else { html!{} } }
                        { if is_expanded && a.comments_enabled { html!{<div class="mt-3"><crate::components::comment_list::CommentList announcement_id={a.id} comments_enabled={true} /></div>} } else { html!{} } }
                    </div>
                </div>
            });
        }
        html! {<>{ for v }</>}
    };
    html! { <div class="announcement-list">{ rendered_cards }</div> }
}
