use super::login_state::Login;
use crate::{ImutableAppState, MutableAppState};
use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

pub enum ScreenState {
    Login(Login),
}

pub trait State {
    fn render(
        &self,
        f: &mut Frame,
        immutable_state: &ImutableAppState,
        mutable_state: &MutableAppState,
        rect: Rect,
    );
    fn handle_key(
        &mut self,
        key: KeyEvent,
        immutable_state: &ImutableAppState,
        mutable_state: &MutableAppState,
    ) -> MutableAppState;
}
