use crate::{AuxiliaryWorkspaceWindowId, Event as WorkspaceEvent, Pane, Workspace};
use gpui::{App, Context, Entity, Render, Subscription, WeakEntity, Window};
use project::Project;
use ui::prelude::*;

pub struct AuxiliaryWorkspaceWindow {
    workspace: WeakEntity<Workspace>,
    pane: Entity<Pane>,
    id: AuxiliaryWorkspaceWindowId,
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
            move |_, _, cx| {
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
}

impl Render for AuxiliaryWorkspaceWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id(("auxiliary-workspace-window", self.id.number()))
            .size_full()
            .when(self.workspace.upgrade().is_some(), |this| {
                this.child(self.pane.clone())
            })
    }
}
