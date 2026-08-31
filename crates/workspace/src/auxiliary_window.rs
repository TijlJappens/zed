use crate::{AuxiliaryWorkspaceWindowId, Event as WorkspaceEvent, Pane, Workspace};
use gpui::{App, Context, Entity, MouseButton, Render, Subscription, WeakEntity, Window};
use project::Project;
use ui::prelude::*;

pub struct AuxiliaryWorkspaceWindow {
    workspace: WeakEntity<Workspace>,
    pane: Entity<Pane>,
    id: AuxiliaryWorkspaceWindowId,
    workspace_integration_ready: bool,
    _subscriptions: Vec<Subscription>,
}

impl AuxiliaryWorkspaceWindow {
    pub(crate) fn new(
        workspace: WeakEntity<Workspace>,
        pane: Entity<Pane>,
        id: AuxiliaryWorkspaceWindowId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let pane_event_subscription = cx.subscribe_in(&pane, window, {
            let workspace = workspace.clone();
            move |_, pane, event, window, cx| {
                if let Some(workspace) = workspace.upgrade() {
                    workspace.update(cx, |workspace, cx| {
                        workspace.handle_pane_event(pane, event, window, cx)
                    });
                }
            }
        });

        cx.defer_in(window, {
            let workspace = workspace.clone();
            let pane = pane.clone();
            move |this, _, cx| {
                this.workspace_integration_ready = true;
                cx.notify();
                if let Some(workspace) = workspace.upgrade() {
                    workspace.update(cx, |_, cx| {
                        cx.emit(WorkspaceEvent::PaneAdded(pane));
                    });
                }
            }
        });

        Self {
            workspace,
            pane,
            id,
            workspace_integration_ready: false,
            _subscriptions: vec![pane_event_subscription],
        }
    }

    pub fn workspace(&self) -> Option<Entity<Workspace>> {
        self.workspace.upgrade()
    }

    pub fn pane(&self) -> &Entity<Pane> {
        &self.pane
    }

    pub fn project(&self, cx: &App) -> Option<Entity<Project>> {
        self.pane.read(cx).project()
    }

    pub fn id(&self) -> AuxiliaryWorkspaceWindowId {
        self.id
    }

    fn drag_handle(&self, cx: &App) -> impl IntoElement {
        div()
            .id(("auxiliary-window-drag-handle", self.id.number()))
            .debug_selector(|| "AUXILIARY_WINDOW_DRAG_HANDLE".to_owned())
            .h(px(30.))
            .w_full()
            .flex_none()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().title_bar_background)
            .on_mouse_down(MouseButton::Left, |_, window, _| {
                window.start_window_move();
            })
    }

    fn pane_container(&self) -> Div {
        div().flex_1().min_h_0().w_full().child(self.pane.clone())
    }
}

impl Render for AuxiliaryWorkspaceWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.workspace_integration_ready {
            return v_flex()
                .id(("auxiliary-workspace-window", self.id.number()))
                .size_full()
                .child(self.drag_handle(cx))
                .child(self.pane_container())
                .into_any_element();
        }
        let Some(workspace) = self.workspace.upgrade() else {
            return div().size_full().into_any_element();
        };
        let workspace_key_context =
            workspace.read_with(cx, |workspace, cx| workspace.key_context(cx));
        let root = workspace.update(cx, |workspace, cx| workspace.actions(v_flex(), window, cx));

        root.id(("auxiliary-workspace-window", self.id.number()))
            .key_context(workspace_key_context)
            .relative()
            .size_full()
            .font(theme_settings::setup_ui_font(window, cx))
            .text_color(cx.theme().colors().text)
            .child(self.drag_handle(cx))
            .child(self.pane_container())
            .into_any_element()
    }
}
