//! `o` and the mouse: open the page (`relic-scan up`) on what the cursor is on, in the browser.

use super::*;
use crate::links;

impl App {
    /// The page address for what is selected: a session in the tree, a repo in the tree, a folder in paths, or the view of this tab.
    pub(super) fn page_link(&self) -> String {
        let base = links::base();
        let at = self.state.selected().unwrap_or(0);
        let k = self.visible().get(at).copied();
        match (self.tab, self.open.or(k)) {
            (Tab::Sessions, Some(k)) => links::session_url(&base, &self.sess[k].sid),
            (Tab::Repos, Some(k)) => links::query_url(&base, "tree", &format!("repo:{}", self.repos[k].name)),
            (Tab::Paths, _) => match self.dir_pick.or(k.filter(|_| self.dir_pick.is_none())) {
                Some(d) => links::query_url(&base, "paths", &self.dirs[d].path),
                None => links::view_url(&base, "paths"),
            },
            (Tab::S3, _) => links::view_url(&base, "export"),
            (Tab::Ops, _) => links::view_url(&base, "ops"),
            _ => links::view_url(&base, "scan"),
        }
    }

    pub(super) fn open_page(&mut self) {
        let url = self.page_link();
        let t = if !links::up(&links::base()) {
            format!("the page is not running: relic-scan up   ({url})")
        } else if links::open(&url).is_ok() {
            format!("opened {url}")
        } else {
            format!("could not start a browser; {url}")
        };
        self.toast = Some((t, Instant::now()));
    }

    /// A left click on a list row selects it; a click on the selected row opens the page. The wheel moves the selection.
    pub(super) fn mouse(&mut self, m: ratatui::crossterm::event::MouseEvent) {
        use ratatui::crossterm::event::{MouseButton, MouseEventKind};
        let n = self.visible().len();
        let at = self.state.selected().unwrap_or(0);
        match m.kind {
            MouseEventKind::ScrollDown => self.state.select(Some((at + 3).min(n.saturating_sub(1)))),
            MouseEventKind::ScrollUp => self.state.select(Some(at.saturating_sub(3))),
            MouseEventKind::Down(MouseButton::Left) if self.open.is_none() => {
                let a = self.list_area;
                // border and header take the first two lines of the table
                if n == 0 || m.column < a.x || m.column >= a.x + a.width || m.row < a.y + 2 || m.row >= a.y + a.height.saturating_sub(1) {
                    return;
                }
                let row = (m.row - a.y - 2) as usize + self.state.offset();
                if row >= n {
                    return;
                }
                if row == at { self.open_page() } else { self.state.select(Some(row)) }
            }
            _ => {}
        }
    }
}
