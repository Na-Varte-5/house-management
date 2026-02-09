use gloo_timers::callback::Timeout;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ToastLevel {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastLevel {
    fn css_class(&self) -> &'static str {
        match self {
            ToastLevel::Success => "bg-success text-white",
            ToastLevel::Error => "bg-danger text-white",
            ToastLevel::Warning => "bg-warning text-dark",
            ToastLevel::Info => "bg-info text-dark",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ToastLevel::Success => "bi-check-circle-fill",
            ToastLevel::Error => "bi-exclamation-triangle-fill",
            ToastLevel::Warning => "bi-exclamation-circle-fill",
            ToastLevel::Info => "bi-info-circle-fill",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u32,
    pub message: String,
    pub level: ToastLevel,
}

#[derive(Clone, PartialEq)]
pub struct ToastContext {
    pub toasts: Rc<Vec<Toast>>,
    pub show: Callback<(String, ToastLevel)>,
    pub dismiss: Callback<u32>,
}

impl ToastContext {
    pub fn success(&self, msg: impl Into<String>) {
        self.show.emit((msg.into(), ToastLevel::Success));
    }

    pub fn error(&self, msg: impl Into<String>) {
        self.show.emit((msg.into(), ToastLevel::Error));
    }

    #[allow(dead_code)]
    pub fn warning(&self, msg: impl Into<String>) {
        self.show.emit((msg.into(), ToastLevel::Warning));
    }

    #[allow(dead_code)]
    pub fn info(&self, msg: impl Into<String>) {
        self.show.emit((msg.into(), ToastLevel::Info));
    }
}

#[derive(Properties, PartialEq)]
pub struct ToastProviderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component(ToastProvider)]
pub fn toast_provider(props: &ToastProviderProps) -> Html {
    let toasts = use_state(Vec::<Toast>::new);
    let counter = use_state(|| 0u32);
    let _timers = use_mut_ref(Vec::<Timeout>::new);

    let dismiss = {
        let toasts = toasts.clone();
        Callback::from(move |id: u32| {
            let current = (*toasts).clone();
            toasts.set(current.into_iter().filter(|t| t.id != id).collect());
        })
    };

    let show = {
        let toasts = toasts.clone();
        let counter = counter.clone();
        let dismiss = dismiss.clone();
        let timers = _timers.clone();

        Callback::from(move |(message, level): (String, ToastLevel)| {
            let id = *counter + 1;
            counter.set(id);

            let toast = Toast { id, message, level };

            let mut current = (*toasts).clone();
            current.push(toast);
            toasts.set(current);

            let dismiss = dismiss.clone();
            let timeout = Timeout::new(4000, move || {
                dismiss.emit(id);
            });
            timers.borrow_mut().push(timeout);
        })
    };

    let context = ToastContext {
        toasts: Rc::new((*toasts).clone()),
        show,
        dismiss: dismiss.clone(),
    };

    html! {
        <ContextProvider<ToastContext> context={context}>
            {props.children.clone()}
            <ToastContainer toasts={(*toasts).clone()} on_dismiss={dismiss} />
        </ContextProvider<ToastContext>>
    }
}

#[derive(Properties, PartialEq)]
struct ToastContainerProps {
    toasts: Vec<Toast>,
    on_dismiss: Callback<u32>,
}

#[function_component(ToastContainer)]
fn toast_container(props: &ToastContainerProps) -> Html {
    if props.toasts.is_empty() {
        return html! {};
    }

    html! {
        <div class="position-fixed top-0 end-0 p-3" style="z-index: 1090; margin-top: 56px;">
            { for props.toasts.iter().map(|toast| {
                let id = toast.id;
                let on_close = {
                    let on_dismiss = props.on_dismiss.clone();
                    Callback::from(move |_: MouseEvent| on_dismiss.emit(id))
                };

                html! {
                    <div class={classes!("toast", "show", "mb-2")} role="alert">
                        <div class={classes!("toast-header", toast.level.css_class())}>
                            <i class={classes!("bi", toast.level.icon(), "me-2")}></i>
                            <strong class="me-auto">
                                {match &toast.level {
                                    ToastLevel::Success => "Success",
                                    ToastLevel::Error => "Error",
                                    ToastLevel::Warning => "Warning",
                                    ToastLevel::Info => "Info",
                                }}
                            </strong>
                            <button type="button" class="btn-close btn-close-white" onclick={on_close}></button>
                        </div>
                        <div class="toast-body">
                            {&toast.message}
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
