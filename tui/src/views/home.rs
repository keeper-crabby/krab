use cli_clipboard::{ClipboardContext, ClipboardProvider};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::Text,
    widgets::Widget,
    Frame,
};

use crate::{
    components::scrollable_view::ScrollView,
    from,
    popups::{
        insert_domain_password::{InsertDomainPassword, InsertDomainPasswordExitState},
        insert_master::{InsertMaster, InsertMasterExitState},
        message::MessagePopup,
        Popup,
    },
    views::{login::Login, View},
    Application, ViewState, COLOR_BLACK, COLOR_ORANGE, COLOR_WHITE,
};
use krab_backend::user::{ReadOnlyRecords, RecordOperationConfig, User};

const DOMAIN_PASSWORD_LIST_ITEM_HEIGHT: u16 = 4;
const RIGHT_MARGIN: u16 = 6;
const LEFT_PADDING: u16 = 2;
const MAX_ENTRY_LENGTH: u16 = 256;
const DOMAIN_PASSWORD_MIDDLE_WIDTH: u16 = 3;
const MIN_WIDTH: u16 = 128;

/// Represents the operation over a secret
///
/// # Variants
/// * `Add` - The add operation
/// * `Remove` - The remove operation
/// * `Modify` - The modify operation
#[derive(Debug, Clone, PartialEq)]
enum Operation {
    Add,
    Remove,
    Modify,
}

/// Represents the position of the inner buffer
///
/// # Fields
/// * `offset_x` - The x offset
/// * `offset_y` - The y offset
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub offset_x: u16,
    pub offset_y: u16,
}

/// Represents the home view
///
/// # Fields
/// * `user` - The user
/// * `secrets` - Secrets
/// * `position` - The position of the inner buffer
/// * `area` - The area of the view
/// * `new_secret` - The new secret to add if any
/// * `operation` - The operation to perform if any
///
/// # Methods
///
/// * `new` - Creates a new `Home`
/// * `up` - Moves the cursor up
/// * `down` - Moves the cursor down
/// * `scroll_to_top` - Scrolls to the top
/// * `scroll_to_bottom` - Scrolls to the bottom
/// * `set_selected_secret` - Sets the selected secret
/// * `toggle_shown_secret` - Toggles the shown secret
/// * `separator` - Returns a separator
/// * `current_secret_cursor` - Returns the current secret cursor
/// * `width` - Returns the width
/// * `render_secrets` - Renders the secrets
/// * `render_legend` - Renders the legend
/// * `buffer_to_render` - Returns the buffer to render
///
/// # Implements
/// * `View` - The view trait
#[derive(Debug, Clone, PartialEq)]
pub struct Home {
    user: User,
    secrets: Secrets,
    position: Position,
    area: Rect,
    new_secret: Option<NewSecret>,
    operation: Option<Operation>,
}

/// Represents a new secret
///
/// # Fields
/// * `domain` - The domain
/// * `password` - The password
#[derive(Debug, Clone, PartialEq)]
struct NewSecret {
    domain: String,
    password: String,
}

/// Represents the secrets
///
/// # Fields
/// * `secrets` - The secrets
/// * `selected_secret` - The selected secret
/// * `shown_secrets` - The shown secrets
///
/// # Methods
/// * `max_length` - Returns the max length of the secrets
#[derive(Debug, Clone, PartialEq)]
struct Secrets {
    secrets: Vec<(String, String)>,
    selected_secret: usize,
    shown_secrets: Vec<usize>,
}

impl Home {
    /// Creates a new `Home`
    ///
    /// # Arguments
    /// * `user` - The user
    /// * `records` - The read only records
    /// * `position` - The position
    /// * `area` - The area
    ///
    /// # Returns
    /// A new `Home` view
    pub fn new(user: User, records: ReadOnlyRecords, position: Position, area: Rect) -> Self {
        let secrets = Secrets {
            secrets: records
                .records()
                .iter()
                .map(|x| (x.0.clone(), x.1.clone()))
                .collect(),
            selected_secret: 0,
            shown_secrets: vec![],
        };
        Self {
            user,
            secrets,
            position: Position {
                offset_x: position.offset_x,
                offset_y: position.offset_y,
            },
            area,
            new_secret: None,
            operation: None,
        }
    }

    /// Moves the cursor up
    ///
    /// # Arguments
    /// * `area` - The area
    fn up(&mut self, area: Rect) {
        if self.secrets.selected_secret <= 1 {
            return self.scroll_to_top();
        }
        self.set_selected_secret(
            self.secrets.selected_secret - 1,
            self.secrets.selected_secret,
            area,
        )
    }

    /// Scrolls to the top
    ///
    /// # Arguments
    /// * `area` - The area
    fn scroll_to_top(&mut self) {
        self.secrets.selected_secret = 0;
        self.position.offset_y = 0;
    }

    /// Moves the cursor down
    ///
    /// # Arguments
    /// * `area` - The area
    fn down(&mut self, area: Rect) {
        if self.secrets.selected_secret == self.secrets.secrets.len() - 1 {
            self.scroll_to_bottom(area);
            return;
        }
        self.set_selected_secret(
            self.secrets.selected_secret + 1,
            self.secrets.selected_secret,
            area,
        )
    }

    /// Scrolls to the bottom
    ///
    /// # Arguments
    /// * `area` - The area
    fn scroll_to_bottom(&mut self, area: Rect) {
        let (_, inner_buffer_height) = ScrollView::inner_buffer_bounding_box(area);
        let max_offset_y =
            self.buffer_to_render().area().height as i32 - inner_buffer_height as i32 + 1;
        let max_offset_y = if max_offset_y < 0 { 0 } else { max_offset_y };
        let max_offset_y = max_offset_y as u16;
        self.secrets.selected_secret = self.secrets.secrets.len() - 1;
        self.position.offset_y = max_offset_y;
    }

    /// Sets the selected secret
    ///
    /// # Arguments
    /// * `selected_secret` - The selected secret
    /// * `previous_selected_secret` - The previous selected secret
    /// * `area` - The area
    ///
    /// # Panics
    /// If the selected secret is out of bounds
    fn set_selected_secret(
        &mut self,
        selected_secret: usize,
        previous_selected_secret: usize,
        area: Rect,
    ) {
        assert!(selected_secret < self.secrets.secrets.len());
        let (_, inner_buffer_height) = ScrollView::inner_buffer_bounding_box(area);
        let mut position = self.position.clone();
        if selected_secret > previous_selected_secret {
            if selected_secret as u16 * DOMAIN_PASSWORD_LIST_ITEM_HEIGHT + 1
                >= inner_buffer_height + position.offset_y
            {
                position.offset_y += DOMAIN_PASSWORD_LIST_ITEM_HEIGHT;
            }
        } else {
            if selected_secret as u16 * DOMAIN_PASSWORD_LIST_ITEM_HEIGHT + 1 <= position.offset_y {
                position.offset_y -= DOMAIN_PASSWORD_LIST_ITEM_HEIGHT;
            }
        }
        self.secrets.selected_secret = selected_secret;
        self.position = position;
    }

    /// Toggles the shown secret
    ///
    /// # Panics
    /// If the selected secret is out of bounds
    fn toggle_shown_secret(&mut self) {
        assert!(self.secrets.selected_secret < self.secrets.secrets.len());

        let selected_secret = self.secrets.selected_secret;
        let mut shown_secrets = self.secrets.shown_secrets.clone();
        if shown_secrets.contains(&selected_secret) {
            shown_secrets.retain(|&x| x != selected_secret);
        } else {
            shown_secrets.push(selected_secret);
        }

        self.secrets.shown_secrets = shown_secrets;
    }

    /// Returns a separator
    ///
    /// # Arguments
    /// * `width` - The width of the separator
    ///
    /// # Returns
    /// A ascii separator
    fn separator(&self, width: u16) -> Text {
        let mut separator = String::new();
        for _ in 0..width {
            separator.push_str("╍");
        }
        Text::styled(
            separator,
            Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::Yellow)),
        )
    }

    /// Returns the current secret cursor
    ///
    /// # Arguments
    /// * `height` - The height of the cursor
    /// * `width` - The width of the cursor
    /// * `index` - The index of the secret where the cursor is
    /// * `style` - The style of the cursor
    ///
    /// # Returns
    /// The current ascii secret with the cursor
    fn current_secret_cursor(&self, height: u16, width: u16, index: u16, style: Style) -> Text {
        let mut cursor = String::new();
        for _ in 0..height {
            if self.secrets.selected_secret == index as usize {
                for _ in 0..width - 1 {
                    cursor.push_str(">");
                }
                cursor.push_str("\n");
            } else {
                for _ in 0..width - 1 {
                    cursor.push_str(" ");
                }
                cursor.push_str("\n");
            }
        }

        Text::styled(cursor, style)
    }

    /// Returns the width
    ///
    /// # Returns
    /// The width
    fn width(&self) -> u16 {
        let width = self.secrets.max_length() as u16 + RIGHT_MARGIN + LEFT_PADDING;
        if width > MIN_WIDTH {
            width
        } else {
            MIN_WIDTH
        }
    }

    /// Renders the secrets
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `cursor_offset` - The cursor offset
    /// * `y_offset` - The y offset
    fn render_secrets(&self, buffer: &mut Buffer, cursor_offset: u16, y_offset: u16) {
        let mut y = y_offset;
        let mut index = 0;
        let width = self.width();
        for (key, value) in self.secrets.secrets.iter() {
            let style = if self.secrets.selected_secret == index {
                Style::default()
                    .bg(from(COLOR_WHITE).unwrap_or(Color::White))
                    .fg(from(COLOR_BLACK).unwrap_or(Color::Black))
            } else {
                Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White))
            };
            let cursor = self.current_secret_cursor(3, cursor_offset, index as u16, style);
            if y == 0 {
                cursor.render(Rect::new(0, y + 1, cursor_offset, 3), buffer);
                let separator = self.separator(buffer.area().width);
                separator.render(Rect::new(cursor_offset, y, width, 1), buffer);
                y += 1;
            } else {
                cursor.render(Rect::new(0, y, cursor_offset, 3), buffer);
            }
            let text = if self.secrets.shown_secrets.contains(&index) {
                format!("\n  {} : {}", key, value)
            } else {
                "\n".to_string() + &hidden_value(key.to_string(), value.len())
            };
            let text = Text::styled(text, style);
            text.render(Rect::new(cursor_offset, y, width, 3), buffer);
            y += 3;
            let separator = self.separator(buffer.area().width);
            separator.render(Rect::new(cursor_offset, y, width, 1), buffer);
            y += 1;
            index += 1;
        }
    }

    /// Renders the legend
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `area` - The area
    /// * `cursor_offset` - The cursor offset
    ///
    /// # Returns
    /// The y offset
    fn render_legend(&self, buffer: &mut Buffer, area: Rect, cursor_offset: u16) -> u16 {
        let text = " ".repeat(cursor_offset as usize) + 
            "j - down | k - up | h - left | l - right | q - quit | a - add | d - delete selected | e - edit selected | c - copy selected";
        let legend = Text::styled(
            text,
            Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White)),
        );
        legend.render(Rect::new(0, 0, area.width, 1), buffer);

        let separator = self.separator(buffer.area().width);
        separator.render(Rect::new(cursor_offset, 1, self.width(), 1), buffer);

        2
    }

    /// Returns the buffer to render
    ///
    /// # Returns
    /// The buffer to render
    fn buffer_to_render(&self) -> Buffer {
        let cursor_offset = 4;
        let secrets_count = self.secrets.secrets.len();
        let rect = Rect::new(
            0,
            0,
            self.width() + cursor_offset,
            (secrets_count as u16 * DOMAIN_PASSWORD_LIST_ITEM_HEIGHT) + 3,
        );
        let mut buffer = Buffer::empty(rect);
        let y_offset = self.render_legend(&mut buffer, rect, cursor_offset);
        self.render_secrets(&mut buffer, cursor_offset, y_offset);

        buffer
    }
}

impl View for Home {
    fn render(&self, f: &mut Frame, app: &Application, area: Rect) {
        match app.immutable_app_state.rect {
            Some(_) => {
                let mut buffer = f.buffer_mut();
                let buffer_to_render = self.buffer_to_render();
                ScrollView::render(&mut buffer, &self.position, area, &buffer_to_render);
            }
            None => {}
        }
    }

    fn handle_key(&mut self, key: &KeyEvent, app: &Application) -> Application {
        let mut app = app.clone();
        let mut change_state = false;

        // TODO: rework this
        if key.code == KeyCode::Char('q') {
            app.state = ViewState::Login(Login::new(&app.immutable_app_state.db_path));
            change_state = true;
        }
        if key.code == KeyCode::Char('j') {
            self.down(app.immutable_app_state.rect.unwrap());
        }
        if key.code == KeyCode::Char('k') {
            self.up(app.immutable_app_state.rect.unwrap());
        }
        if key.code == KeyCode::Char('h') {
            if self.position.offset_x != 0 {
                self.position.offset_x -= 1;
            }
        }
        if key.code == KeyCode::Char('l') {
            if !ScrollView::check_if_width_out_of_bounds(
                &self.position,
                &self.buffer_to_render(),
                app.immutable_app_state.rect.unwrap_or(self.area),
            ) {
                self.position.offset_x += 1;
            }
        }
        if key.code == KeyCode::Enter {
            self.toggle_shown_secret();
        }
        if key.code == KeyCode::Char('a') {
            app.mutable_app_state
                .popups
                .push(Box::new(InsertDomainPassword::new()));
            self.operation = Some(Operation::Add);
        }
        if key.code == KeyCode::Char('d') {
            app.mutable_app_state
                .popups
                .push(Box::new(InsertMaster::new()));
            self.operation = Some(Operation::Remove);
        }
        if key.code == KeyCode::Char('c') {
            let current_secret = self
                .secrets
                .secrets
                .get(self.secrets.selected_secret)
                .unwrap();
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents(current_secret.1.clone()).unwrap();
        }

        if !change_state {
            app.state = ViewState::Home(self.clone());
        }

        app
    }

    fn needs_header(&self) -> bool {
        false
    }

    fn min_area(&self) -> (u16, u16) {
        (40, 40)
    }

    fn handle_insert_record_popup(
        &mut self,
        app: Application,
        popup: Box<dyn Popup>,
    ) -> Application {
        let domain: String;
        let password: String;
        let insert_password = popup.downcast::<InsertDomainPassword>();

        match insert_password {
            Ok(insert_password) => {
                if insert_password.exit_state() == Some(InsertDomainPasswordExitState::Quit) {
                    return app;
                }
                domain = insert_password.domain();
                password = insert_password.password();
            }
            Err(_) => {
                unreachable!();
            }
        }

        if domain.is_empty() || password.is_empty() {
            let mut app = app.clone();
            app.mutable_app_state
                .popups
                .push(Box::new(MessagePopup::new(
                    "Cannot create record".to_string(),
                )));
            return app;
        }

        self.new_secret = Some(NewSecret {
            domain: domain.clone(),
            password: password.clone(),
        });

        let mut app = app.clone();

        app.state = ViewState::Home(self.clone());

        app.mutable_app_state
            .popups
            .push(Box::new(InsertMaster::new()));

        app
    }

    fn handle_insert_master_popup(
        &mut self,
        app: Application,
        popup: Box<dyn Popup>,
    ) -> Application {
        let master_password: String;
        let insert_master = popup.downcast::<InsertMaster>();

        match insert_master {
            Ok(insert_master) => {
                if insert_master.exit_state() == Some(InsertMasterExitState::Quit) {
                    return app;
                }
                master_password = insert_master.master();
            }
            Err(_) => {
                unreachable!();
            }
        }

        if master_password.is_empty() {
            let mut app = app.clone();
            app.mutable_app_state
                .popups
                .push(Box::new(MessagePopup::new(
                    "Cannot create record".to_string(),
                )));
            return app;
        }

        match self.operation {
            None => {
                unreachable!();
            }
            Some(Operation::Add) => {
                let config = RecordOperationConfig::new(
                    &self.user.username(),
                    &master_password,
                    &self.new_secret.clone().unwrap().domain,
                    &self.new_secret.clone().unwrap().password,
                    &app.immutable_app_state.db_path,
                );

                let res = self.user.add_record(config);

                if res.is_err() {
                    let mut app = app.clone();
                    app.mutable_app_state
                        .popups
                        .push(Box::new(MessagePopup::new(
                            "Cannot create record".to_string(),
                        )));
                    return app;
                }

                self.secrets = Secrets {
                    secrets: res
                        .unwrap()
                        .records()
                        .iter()
                        .map(|x| (x.0.clone(), x.1.clone()))
                        .collect(),
                    selected_secret: self.secrets.selected_secret,
                    shown_secrets: self.secrets.shown_secrets.clone(),
                };

                let mut app = app.clone();
                app.state = ViewState::Home(self.clone());
                app
            }
            Some(Operation::Remove) => {
                let current_secret = self
                    .secrets
                    .secrets
                    .get(self.secrets.selected_secret)
                    .unwrap();

                let config = RecordOperationConfig::new(
                    &self.user.username(),
                    &master_password,
                    &current_secret.0,
                    "",
                    &app.immutable_app_state.db_path,
                );

                let res = self.user.remove_record(config);

                if res.is_err() {
                    let mut app = app.clone();
                    app.mutable_app_state
                        .popups
                        .push(Box::new(MessagePopup::new(
                            "Cannot remove record".to_string(),
                        )));
                    return app;
                }

                self.secrets = Secrets {
                    secrets: res
                        .unwrap()
                        .records()
                        .iter()
                        .map(|x| (x.0.clone(), x.1.clone()))
                        .collect(),
                    selected_secret: self.secrets.selected_secret,
                    shown_secrets: self.secrets.shown_secrets.clone(),
                };

                let mut app = app.clone();
                app.state = ViewState::Home(self.clone());
                app
            }
            Some(Operation::Modify) => app,
        }
    }
}

impl Secrets {
    /// Return the max length of the secrets
    ///
    /// # Returns
    /// The max length of the secrets
    fn max_length(&self) -> usize {
        self.secrets
            .iter()
            .map(|x| x.0.len() + x.1.len() + DOMAIN_PASSWORD_MIDDLE_WIDTH as usize)
            .max()
            .unwrap_or(0)
    }
}

/// Returns a hidden value
///
/// # Arguments
/// * `domain` - The domain
///
/// # Returns
/// A hidden value
fn hidden_value(domain: String, length: usize) -> String {
    assert!(domain.len() <= MAX_ENTRY_LENGTH as usize);

    let mut hidden_value = "  ".to_string() + &domain.clone();
    hidden_value.push_str(" : ");
    for _ in 0..length {
        hidden_value.push_str("•");
    }

    hidden_value
}
