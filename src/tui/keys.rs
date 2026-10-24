//! Keys: one handler for every tab and the popup.

use super::*;

impl App {
    /// One key. Returns false to quit.
    pub(super) fn key(&mut self, code: KeyCode, mods: KeyModifiers) -> bool {
        if self.typing {
            match code {
                KeyCode::Esc => {
                    self.typing = false;
                    self.filter.clear();
                }
                KeyCode::Enter => self.typing = false,
                KeyCode::Backspace => {
                    self.filter.pop();
                }
                KeyCode::Char(c) => self.filter.push(c),
                _ => {}
            }
            self.state.select(Some(0));
            return true;
        }
        if self.open.is_some() {
            match code {
                KeyCode::Char('c') => {
                    let s = &self.sess[self.open.unwrap()];
                    let t = if s.tier == 0 { format!("claude --resume {}", s.sid) } else { s.sid.clone() };
                    self.copy(&t);
                }
                KeyCode::Char('q') if mods.is_empty() => return false,
                _ => self.open = None,
            }
            return true;
        }
        let n = self.visible().len();
        let at = self.state.selected().unwrap_or(0);
        let go = |s: &mut TableState, x: usize| s.select(Some(x.min(n.saturating_sub(1))));
        match code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => return false,
            KeyCode::Char('1') => self.switch(Tab::Overview),
            KeyCode::Char('2') => self.switch(Tab::Repos),
            KeyCode::Char('3') => {
                self.repo_pick = None;
                self.switch(Tab::Sessions)
            }
            KeyCode::Char('4') => {
                self.dir_pick = None;
                self.switch(Tab::Paths)
            }
            KeyCode::Char('5') => self.switch(Tab::S3),
            KeyCode::Char('c') if self.tab == Tab::S3 => self.s3_start(true),
            KeyCode::Char('s') if self.tab == Tab::S3 => self.s3_start(false),
            KeyCode::Tab | KeyCode::Right => self.switch(match self.tab {
                Tab::Overview => Tab::Repos,
                Tab::Repos => Tab::Sessions,
                Tab::Sessions => Tab::Paths,
                Tab::Paths => Tab::S3,
                Tab::S3 => Tab::Overview,
            }),
            KeyCode::BackTab | KeyCode::Left => self.switch(match self.tab {
                Tab::Overview => Tab::S3,
                Tab::S3 => Tab::Paths,
                Tab::Repos => Tab::Overview,
                Tab::Sessions => Tab::Repos,
                Tab::Paths => Tab::Sessions,
            }),
            KeyCode::Char('/') => self.typing = true,
            KeyCode::Esc => {
                self.filter.clear();
                self.state.select(Some(0));
            }
            KeyCode::Down | KeyCode::Char('j') => go(&mut self.state, at + 1),
            KeyCode::Up | KeyCode::Char('k') => go(&mut self.state, at.saturating_sub(1)),
            KeyCode::PageDown | KeyCode::Char(' ') => go(&mut self.state, at + 20),
            KeyCode::PageUp => go(&mut self.state, at.saturating_sub(20)),
            KeyCode::Home | KeyCode::Char('g') => go(&mut self.state, 0),
            KeyCode::End | KeyCode::Char('G') => go(&mut self.state, usize::MAX),
            KeyCode::Enter => {
                let v = self.visible();
                let Some(&k) = v.get(at) else { return true };
                match self.tab {
                    Tab::Repos => {
                        self.repo_pick = Some(self.repos[k].name.clone());
                        self.switch(Tab::Sessions);
                    }
                    Tab::Sessions => self.open = Some(k),
                    Tab::Paths if self.dir_pick.is_none() => {
                        self.dir_pick = Some(k);
                        self.filter.clear();
                        self.state.select(Some(0));
                    }
                    Tab::Paths => {
                        let f = &self.files[k];
                        let p = format!("{}/{}", self.dirs[f.dir].path, f.name);
                        self.copy(&p);
                    }
                    Tab::Overview | Tab::S3 => {}
                }
            }
            KeyCode::Backspace => {
                if self.tab == Tab::Paths && self.dir_pick.is_some() {
                    self.dir_pick = None;
                    self.state.select(Some(0));
                } else if self.tab == Tab::Sessions && self.repo_pick.is_some() {
                    self.repo_pick = None;
                    self.switch(Tab::Repos);
                }
            }
            _ => {}
        }
        true
    }

}
