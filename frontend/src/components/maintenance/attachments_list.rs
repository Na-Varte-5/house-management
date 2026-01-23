use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq)]
pub struct Attachment {
    pub id: u64,
    pub filename: String,
    pub uploaded_by: u64,
    pub uploaded_at: String,
    pub url: String,
}

#[derive(Properties, PartialEq)]
pub struct AttachmentsListProps {
    pub attachments: Vec<Attachment>,
    pub loading: bool,
}

/// Component for displaying maintenance request attachments
#[function_component(AttachmentsList)]
pub fn attachments_list(props: &AttachmentsListProps) -> Html {
    html! {
        <div class="card mt-3">
            <div class="card-header">
                <h5 class="mb-0">{"Attachments"}</h5>
            </div>
            <div class="card-body">
                if props.loading {
                    <div class="text-center">
                        <div class="spinner-border spinner-border-sm" role="status"></div>
                    </div>
                } else if props.attachments.is_empty() {
                    <p class="text-muted small mb-0">{"No attachments"}</p>
                } else {
                    <div class="row">
                        {
                            for props.attachments.iter().map(|att| {
                                html! {
                                    <div class="col-md-4 mb-2">
                                        <div class="card">
                                            <div class="card-body p-2">
                                                <p class="mb-1 small"><strong>{&att.filename}</strong></p>
                                                <p class="mb-0 text-muted" style="font-size: 0.75rem;">
                                                    {"Uploaded "}{&att.uploaded_at}
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })
                        }
                    </div>
                }
            </div>
        </div>
    }
}
