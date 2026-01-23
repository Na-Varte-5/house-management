use yew::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct DeletedAnnouncement {
    pub id: u64,
    pub title: String,
}

#[derive(Properties, PartialEq)]
pub struct DeletedAnnouncementsListProps {
    pub announcements: Vec<DeletedAnnouncement>,
    pub on_restore: Callback<u64>,
    pub on_purge: Callback<u64>,
}

/// Component for displaying deleted announcements with restore/purge actions
#[function_component(DeletedAnnouncementsList)]
pub fn deleted_announcements_list(props: &DeletedAnnouncementsListProps) -> Html {
    html! {
        <div class="mt-3">
            <h6>{"Deleted"}</h6>
            { if props.announcements.is_empty() {
                html!{<div class="text-muted small">{"None"}</div>}
            } else {
                html!{
                    <>
                        { for props.announcements.iter().map(|d| {
                            let restore_cb = props.on_restore.clone();
                            let purge_cb = props.on_purge.clone();
                            let id = d.id;
                            html!{
                                <div class="border rounded p-2 mb-2 bg-light" key={id}>
                                    <div class="d-flex justify-content-between align-items-center">
                                        <span>{&d.title}</span>
                                        <div class="btn-group btn-group-sm">
                                            <button
                                                class="btn btn-outline-success"
                                                onclick={Callback::from(move |_| restore_cb.emit(id))}
                                            >
                                                {"Restore"}
                                            </button>
                                            <button
                                                class="btn btn-outline-danger"
                                                onclick={Callback::from(move |_| purge_cb.emit(id))}
                                            >
                                                {"Purge"}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }) }
                    </>
                }
            } }
        </div>
    }
}
