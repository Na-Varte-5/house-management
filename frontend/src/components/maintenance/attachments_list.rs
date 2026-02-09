use crate::i18n::t;
use crate::utils::datetime::format_dt_option;
use serde::Deserialize;
use web_sys::{FormData, HtmlInputElement};
use yew::prelude::*;

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub struct Attachment {
    pub id: u64,
    pub request_id: u64,
    pub original_filename: String,
    pub stored_filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub is_deleted: bool,
    pub created_at: Option<String>,
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[derive(Properties, PartialEq)]
pub struct AttachmentsListProps {
    pub attachments: Vec<Attachment>,
    pub loading: bool,
    pub request_id: u64,
    #[prop_or(false)]
    pub can_upload: bool,
    #[prop_or_default]
    pub on_upload: Callback<FormData>,
    #[prop_or(false)]
    pub uploading: bool,
}

#[function_component(AttachmentsList)]
pub fn attachments_list(props: &AttachmentsListProps) -> Html {
    let file_input_ref = use_node_ref();

    let on_file_change = {
        let on_upload = props.on_upload.clone();
        let file_input_ref = file_input_ref.clone();

        Callback::from(move |_: Event| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        let form_data = FormData::new().unwrap();
                        form_data.append_with_blob("file", &file).unwrap();
                        on_upload.emit(form_data);
                        input.set_value("");
                    }
                }
            }
        })
    };

    let on_upload_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    html! {
        <div class="card mt-3">
            <div class="card-header d-flex justify-content-between align-items-center">
                <h5 class="mb-0">{t("maintenance-attachments")}</h5>
                if props.can_upload {
                    <div>
                        <input
                            type="file"
                            ref={file_input_ref}
                            style="display: none;"
                            onchange={on_file_change}
                            accept="image/*,application/pdf"
                        />
                        <button
                            class="btn btn-sm btn-outline-primary"
                            onclick={on_upload_click}
                            disabled={props.uploading}
                        >
                            if props.uploading {
                                <span class="spinner-border spinner-border-sm me-1" role="status"></span>
                                {t("maintenance-uploading")}
                            } else {
                                <i class="bi bi-upload me-1"></i>
                                {t("maintenance-upload-file")}
                            }
                        </button>
                    </div>
                }
            </div>
            <div class="card-body">
                if props.loading {
                    <div class="text-center">
                        <div class="spinner-border spinner-border-sm" role="status"></div>
                    </div>
                } else if props.attachments.is_empty() {
                    <p class="text-muted small mb-0">{t("maintenance-no-attachments")}</p>
                } else {
                    <div class="list-group list-group-flush">
                        {
                            for props.attachments.iter().map(|att| {
                                let download_url = format!("/api/v1/requests/{}/attachments/{}/download", props.request_id, att.id);
                                let icon_class = if att.mime_type.starts_with("image/") {
                                    "bi bi-file-image text-success"
                                } else if att.mime_type == "application/pdf" {
                                    "bi bi-file-pdf text-danger"
                                } else {
                                    "bi bi-file-earmark text-secondary"
                                };

                                html! {
                                    <a
                                        href={download_url}
                                        target="_blank"
                                        class="list-group-item list-group-item-action d-flex justify-content-between align-items-center"
                                    >
                                        <div class="d-flex align-items-center">
                                            <i class={classes!(icon_class, "me-2")} style="font-size: 1.25rem;"></i>
                                            <div>
                                                <div class="fw-medium">{&att.original_filename}</div>
                                                <small class="text-muted">
                                                    {format_file_size(att.size_bytes)}
                                                    {" • "}
                                                    {format_dt_option(att.created_at.as_ref())}
                                                </small>
                                            </div>
                                        </div>
                                        <i class="bi bi-download text-muted"></i>
                                    </a>
                                }
                            })
                        }
                    </div>
                }
            </div>
        </div>
    }
}
