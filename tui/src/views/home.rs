use cli_clipboard::{ClipboardContext, ClipboardProvider};
use directories::UserDirs;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::{Buffer, Rect},
    style::{Color, Style},
    text::Text,
    widgets::Widget,
    Frame,
};

use crate::{
    components::{
        input::{Input, InputConfig},
        scrollable_view::ScrollView,
    },
    from,
    popups::{
        insert_domain_password::{InsertDomainPassword, InsertDomainPasswordExitState},
        insert_master::{InsertMaster, InsertMasterExitState},
        insert_password::{InsertPassword, InsertPasswordExitState},
        message::MessagePopup,
        Popup,
    },
    views::{login::Login, View},
    Application, ViewState, COLOR_BLACK, COLOR_ORANGE, COLOR_WHITE,
};
use chrono;
use krab_backend::user::{ReadOnlyRecords, RecordOperationConfig, User};

const DOMAIN_PASSWORD_LIST_ITEM_HEIGHT: u16 = 4;
const RIGHT_MARGIN: u16 = 6;
const LEFT_PADDING: u16 = 2;
const MAX_ENTRY_LENGTH: u16 = 256;
const DOMAIN_PASSWORD_MIDDLE_WIDTH: u16 = 3;
const MIN_WIDTH: u16 = 128;
const FILTER_INPUT_WIDTH: u16 = 64;
const LEGEND_TEXT: &str = "Press ? for help";

/// Represents the home view state
///
/// # Variants
/// * `Normal` - The normal state
/// * `Filter` - The filter state
#[derive(Debug, Clone, PartialEq)]
enum HomeViewState {
    Normal,
    Filter,
}

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
///
/// # Methods
/// * `offset_x` - Returns the x offset
/// * `offset_y` - Returns the y offset
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    offset_x: u16,
    offset_y: u16,
}

/// Represents the home view
///
/// # Fields
/// * `user` - The user
/// * `secrets` - Secrets
/// * `position` - The position of the inner buffer
/// * `area` - The area of the view
/// * `state` - The state of the view
/// * `cursor` - The cursor position
/// * `input_offset` - The input offset
/// * `filter_value` - The filter value
/// * `new_secret` - The new secret to add if any
/// * `operation` - The operation to perform if any
///
/// # Methods
///
/// * `new` - Creates a new `Home`
/// * `generate_input_config` - Returns the input config for the popup
/// * `filter_value` - Returns the input value
/// * `set_shown_upwards` - Sets the shown upwards
/// * `largest_prefix` - Returns the largest prefix
/// * `fuzzy_filter` - Filters the secrets
/// * `up` - Moves the cursor up
/// * `scroll_to_top` - Scrolls to the top
/// * `down` - Moves the cursor down
/// * `scroll_to_bottom` - Scrolls to the bottom
/// * `set_selected_secret` - Sets the selected secret
/// * `toggle_shown_secret` - Toggles the shown secret
/// * `separator` - Returns a separator
/// * `current_secret_cursor` - Returns the current secret cursor
/// * `width` - Returns the width
/// * `render_secrets` - Renders the secrets
/// * `render_header` - Renders the header
/// * `header_height` - Returns the header height
/// * `render_legend` - Renders the legend
/// * `legend_height` - Returns the legend height
/// * `buffer_to_render` - Returns the buffer to render
/// * `index_offset` - Returns the index offset
///
/// # Implements
/// * `View` - The view trait
#[derive(Debug, Clone, PartialEq)]
pub struct Home {
    user: User,
    secrets: Vec<Secrets>,
    position: Position,
    area: Rect,
    state: HomeViewState,
    cursor: u16,
    input_offset: u16,
    filter_value: String,
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

/// Represents a secret
///
/// # Fields
/// * `key` - The key
/// * `value` - The value
/// * `last_suffix` - The last suffix
#[derive(Debug, Clone, PartialEq)]
struct Secret {
    key: String,
    value: String,
    last_suffix: String,
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
    secrets: Vec<Secret>,
    selected_secret: usize,
    shown_secrets: Vec<usize>,
}

impl Position {
    /// Returns the x offset
    ///
    /// # Returns
    /// The x offset
    pub fn offset_x(&self) -> u16 {
        self.offset_x
    }

    /// Returns the y offset
    ///
    /// # Returns
    /// The y offset
    pub fn offset_y(&self) -> u16 {
        self.offset_y
    }
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
                .map(|x| Secret {
                    key: x.0.clone(),
                    value: x.1.clone(),
                    last_suffix: x.0.clone(),
                })
                .collect(),
            selected_secret: 0,
            shown_secrets: vec![],
        };
        let secrets = vec![secrets];
        Self {
            user,
            secrets,
            position: Position {
                offset_x: position.offset_x,
                offset_y: position.offset_y,
            },
            area,
            state: HomeViewState::Normal,
            cursor: 0,
            input_offset: 0,
            filter_value: "".to_string(),
            new_secret: None,
            operation: None,
        }
    }

    /// Returns the input config for the popup
    ///
    /// # Returns
    /// The input config for the popup
    fn generate_input_config(&self) -> InputConfig {
        InputConfig::new(
            self.state == HomeViewState::Filter,
            self.filter_value(),
            false,
            "Filter".to_string(),
            if self.state == HomeViewState::Filter {
                Some(self.cursor)
            } else {
                None
            },
            self.input_offset,
            Some(FILTER_INPUT_WIDTH),
        )
    }

    /// Returns the input value
    ///
    /// # Returns
    /// The input value
    fn filter_value(&self) -> String {
        self.filter_value.clone()
    }

    /// Generates the help text with all keybindings grouped by category
    ///
    /// # Returns
    /// A formatted string with categorized keybindings
    fn generate_help_text() -> String {
        let help = vec![
            "NAVIGATION:",
            "  j / Down     Move down",
            "  k / Up       Move up",
            "  h / Left     Scroll left",
            "  l / Right    Scroll right",
            "",
            "ACTIONS:",
            "  a            Add new secret",
            "  d            Delete selected secret",
            "  e            Edit selected secret",
            "  c            Copy password to clipboard",
            "  Enter        Toggle password visibility",
            "",
            "OTHER:",
            "  f            Enter filter/search mode",
            "  x            Export secrets to CSV",
            "  q            Quit application",
            "  ?            Show this help",
        ];
        help.join("\n")
    }

    /// Toggles the shown secret upwards to the root
    ///
    /// # Arguments
    /// * `secret` - The secret
    /// * `is_shown` - The shown state
    ///
    /// # Panics
    /// If the secret is out of bounds
    fn set_shown_upwards(&mut self, secret: String, is_shown: bool) {
        for i in 0..self.secrets.len() - 1 {
            let index = self.secrets[i]
                .secrets
                .iter()
                .position(|x| x.key == secret)
                .unwrap_or(0);
            if !is_shown {
                self.secrets[i].shown_secrets.retain(|&x| x != index);
            } else if !self.secrets[i].shown_secrets.contains(&index) {
                self.secrets[i].shown_secrets.push(index);
            }
        }
    }

    /// Returns the largest prefix of the new value and the previous value
    ///
    /// # Arguments
    /// * `previous_value` - The previous value
    /// * `new_value` - The new value
    ///
    /// # Returns
    /// The largest prefix
    fn largest_prefix(&self, previous_value: &str, new_value: &str) -> String {
        let mut prefix = String::new();
        for (i, c) in new_value.chars().enumerate() {
            if i < previous_value.len() && c == previous_value.chars().nth(i).unwrap() {
                prefix.push(c);
            } else {
                break;
            }
        }
        prefix
    }

    /// Filters the secrets
    ///
    /// # Arguments
    /// * `previous_value` - The previous value
    /// * `new_value` - The new value
    ///
    /// # Panics
    /// If the secrets are out of bounds
    fn fuzzy_filter(&mut self, previous_value: String, new_value: String) -> () {
        let largest_prefix = self.largest_prefix(&previous_value, &new_value);
        let rest = new_value
            .chars()
            .skip(largest_prefix.len())
            .collect::<String>();

        assert!(
            self.secrets.len() >= largest_prefix.len()
                || self.secrets.last().unwrap().secrets.len() == 0,
        );
        for i in largest_prefix.len()..self.secrets.len() - 1 {
            self.secrets.remove(i + 1);
        }

        let mut current_secrets = self
            .secrets
            .get(largest_prefix.len())
            .unwrap()
            .secrets
            .clone();
        let mut current_shown_secrets = self
            .secrets
            .get(largest_prefix.len())
            .unwrap()
            .shown_secrets
            .clone();
        for i in 0..rest.len() {
            let mut new_secrets = vec![];
            let mut shown_secrets = vec![];
            for (index, secret) in current_secrets.iter().enumerate() {
                if secret.last_suffix.contains(rest.chars().nth(i).unwrap()) {
                    let new_suffix = secret
                        .last_suffix
                        .chars()
                        .skip_while(|&c| c != rest.chars().nth(i).unwrap())
                        .skip(1)
                        .collect::<String>();

                    assert!(new_suffix.len() != secret.last_suffix.len() || new_suffix.len() == 0,);
                    let new_secret = Secret {
                        key: secret.key.clone(),
                        value: secret.value.clone(),
                        last_suffix: new_suffix,
                    };
                    new_secrets.push(new_secret);
                    if current_shown_secrets.contains(&index) {
                        shown_secrets.push(new_secrets.len() - 1);
                    }
                }
            }
            self.secrets.push(Secrets {
                secrets: new_secrets.clone(),
                selected_secret: 0,
                shown_secrets: shown_secrets.clone(),
            });
            if new_secrets.is_empty() {
                break;
            }
            current_secrets = new_secrets;
            current_shown_secrets = shown_secrets;
        }
    }

    /// Moves the cursor up
    ///
    /// # Arguments
    /// * `area` - The area
    fn up(&mut self, area: Rect) {
        if self.secrets.last().unwrap().selected_secret <= 1 {
            return self.scroll_to_top();
        }
        self.set_selected_secret(
            self.secrets.last().unwrap().selected_secret - 1,
            self.secrets.last().unwrap().selected_secret,
            area,
        )
    }

    /// Scrolls to the top
    ///
    /// # Arguments
    /// * `area` - The area
    fn scroll_to_top(&mut self) {
        self.secrets.last_mut().unwrap().selected_secret = 0;
        self.position.offset_y = 0;
    }

    /// Moves the cursor down
    ///
    /// # Arguments
    /// * `area` - The area
    fn down(&mut self, area: Rect) {
        let current_selected_secret = self.secrets.last().unwrap().selected_secret;
        if current_selected_secret == self.secrets.last().unwrap().secrets.len() - 1
            || current_selected_secret == self.secrets.last().unwrap().secrets.len() - 2
        {
            self.scroll_to_bottom(area);
            return;
        }
        self.set_selected_secret(
            self.secrets.last().unwrap().selected_secret + 1,
            self.secrets.last().unwrap().selected_secret,
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
        self.secrets.last_mut().unwrap().selected_secret =
            self.secrets.last().unwrap().secrets.len() - 1;
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
        assert!(selected_secret < self.secrets.last().unwrap().secrets.len());
        let (_, inner_buffer_height) = ScrollView::inner_buffer_bounding_box(area);
        let mut position = self.position.clone();
        if selected_secret > previous_selected_secret {
            if self.index_offset(selected_secret as u16) + 2 * DOMAIN_PASSWORD_LIST_ITEM_HEIGHT
                >= inner_buffer_height + position.offset_y
                && selected_secret != self.secrets.last().unwrap().secrets.len() - 1
            {
                position.offset_y += DOMAIN_PASSWORD_LIST_ITEM_HEIGHT;
            }
        } else {
            if self.index_offset(selected_secret as u16) <= position.offset_y {
                position.offset_y -= DOMAIN_PASSWORD_LIST_ITEM_HEIGHT;
            }
        }
        self.secrets.last_mut().unwrap().selected_secret = selected_secret;
        self.position = position;
    }

    /// Toggles the shown secret
    ///
    /// # Panics
    /// If the selected secret is out of bounds
    fn toggle_shown_secret(&mut self) {
        assert!(
            self.secrets.last().unwrap().selected_secret
                < self.secrets.last().unwrap().secrets.len()
        );

        let selected_secret = self.secrets.last().unwrap().selected_secret;
        let selecred_secret_domain = self.secrets.last().unwrap().secrets[selected_secret]
            .key
            .clone();
        let mut shown_secrets = self.secrets.last().unwrap().shown_secrets.clone();
        if shown_secrets.contains(&selected_secret) {
            shown_secrets.retain(|&x| x != selected_secret);
            self.set_shown_upwards(selecred_secret_domain, false);
        } else {
            shown_secrets.push(selected_secret);
            self.set_shown_upwards(selecred_secret_domain, true);
        }

        self.secrets.last_mut().unwrap().shown_secrets = shown_secrets;
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
            if self.secrets.last().unwrap().selected_secret == index as usize {
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
        let width =
            (self.secrets.last().unwrap().max_length() as u16 + RIGHT_MARGIN + LEFT_PADDING)
                .max(LEGEND_TEXT.len() as u16 + 4);
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
        for secret in self.secrets.last().unwrap().secrets.iter() {
            let style = if self.secrets.last().unwrap().selected_secret == index {
                Style::default()
                    .bg(from(COLOR_WHITE).unwrap_or(Color::White))
                    .fg(from(COLOR_BLACK).unwrap_or(Color::Black))
            } else {
                Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White))
            };
            let cursor = self.current_secret_cursor(3, cursor_offset, index as u16, style);
            if index == 0 {
                let separator = self.separator(buffer.area().width);
                separator.render(Rect::new(cursor_offset, y, width, 1), buffer);
                cursor.render(Rect::new(0, y + 1, cursor_offset, 3), buffer);
                y += 1;
            } else {
                cursor.render(Rect::new(0, y, cursor_offset, 3), buffer);
            }
            let text = if self.secrets.last().unwrap().shown_secrets.contains(&index) {
                format!("\n  {} : {}", secret.key, secret.value)
            } else {
                "\n".to_string() + &hidden_value(secret.key.to_string(), secret.value.len())
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

    /// Renders the header
    ///
    /// # Arguments
    ///
    /// * `buffer` - The mutable buffer to render to
    /// * `area` - The area
    /// * `cursor_offset` - The cursor offset
    ///
    /// # Returns
    /// The cursor offset
    fn render_header(&self, buffer: &mut Buffer, area: Rect, cursor_offset: u16) -> u16 {
        let mut username = self.user.username().clone();
        username.truncate(MAX_ENTRY_LENGTH as usize - cursor_offset as usize - "Welcome ".len());
        let text = " ".repeat(cursor_offset as usize) + "Welcome " + username.as_str();
        let header = Text::styled(
            text,
            Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White)),
        );
        header.render(Rect::new(0, 0, area.width, 1), buffer);

        let separator = self.separator(buffer.area().width);
        separator.render(Rect::new(cursor_offset, 1, self.width(), 1), buffer);

        self.header_height()
    }

    /// Returns the header height
    ///
    /// # Returns
    /// The header height
    fn header_height(&self) -> u16 {
        3
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
    fn render_legend(
        &self,
        buffer: &mut Buffer,
        area: Rect,
        cursor_offset: u16,
        y_offset: u16,
    ) -> u16 {
        let text = " ".repeat(cursor_offset as usize) + LEGEND_TEXT;
        let legend = Text::styled(
            text,
            Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White)),
        );
        legend.render(Rect::new(0, y_offset, area.width, 1), buffer);

        let separator = self.separator(buffer.area().width);
        separator.render(
            Rect::new(cursor_offset, y_offset + 1, self.width(), 1),
            buffer,
        );

        self.legend_height()
    }

    /// Returns the legend height
    ///
    /// # Returns
    /// The legend height
    fn legend_height(&self) -> u16 {
        3
    }

    /// Returns the buffer to render
    ///
    /// # Returns
    /// The buffer to render
    fn buffer_to_render(&self) -> Buffer {
        let cursor_offset = 4;
        let secrets_count = self.secrets.last().unwrap().secrets.len();
        let rect = Rect::new(
            0,
            0,
            self.width() + cursor_offset,
            (secrets_count as u16 * DOMAIN_PASSWORD_LIST_ITEM_HEIGHT + 1)
                + self.header_height()
                + self.legend_height()
                + InputConfig::height(),
        );
        let mut buffer = Buffer::empty(rect);
        let y_offset_header = self.render_header(&mut buffer, rect, cursor_offset);
        let y_offset_legend = self.render_legend(&mut buffer, rect, cursor_offset, y_offset_header);
        let input_config = self.generate_input_config();
        let input_rect = Rect::new(
            cursor_offset,
            y_offset_header + y_offset_legend,
            input_config.width(),
            InputConfig::height(),
        );
        Input::render(&mut buffer, input_rect, &input_config);
        self.render_secrets(
            &mut buffer,
            cursor_offset,
            y_offset_header + y_offset_legend + InputConfig::height(),
        );

        buffer
    }

    /// Returns the index offset
    ///
    /// # Arguments
    /// * `index` - The index
    ///
    /// # Returns
    /// The index offset
    fn index_offset(&self, index: u16) -> u16 {
        index * DOMAIN_PASSWORD_LIST_ITEM_HEIGHT + 1 + self.header_height() + self.legend_height()
    }

    /// Exports secrets to csv file
    ///
    /// # Returns
    /// Error if something went wrong
    fn export_csv(&self) -> Result<(), String> {
        let user_dirs = UserDirs::new().ok_or_else(|| "Could not create user dirs".to_string())?;

        let download_dir = user_dirs
            .download_dir()
            .ok_or_else(|| "Could not find download dir".to_string())?;

        let now = chrono::Local::now();
        let formatted_date = now.format("%Y-%m-%d-%H-%M-%S").to_string();
        let filename = format!("krab-secrets-{}.csv", formatted_date);
        let file_path = download_dir.join(filename);

        let mut result = "domain,password\n".to_string();
        if let Some(root_secrets) = self.secrets.first() {
            for secret in &root_secrets.secrets {
                result.push_str(&format!("{},{}\n", secret.key, secret.value));
            }
        }

        std::fs::write(file_path, result).map_err(|e| e.to_string())?;

        Ok(())
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

        match self.state {
            HomeViewState::Normal => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.state = ViewState::Login(Login::new(&app.immutable_app_state.db_path));
                    change_state = true;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.down(app.immutable_app_state.rect.unwrap());
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.up(app.immutable_app_state.rect.unwrap());
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if self.position.offset_x != 0 {
                        self.position.offset_x -= 1;
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if !ScrollView::check_if_width_out_of_bounds(
                        &self.position,
                        &self.buffer_to_render(),
                        app.immutable_app_state.rect.unwrap_or(self.area),
                    ) {
                        self.position.offset_x += 1;
                    }
                }
                KeyCode::Enter => {
                    self.toggle_shown_secret();
                }
                KeyCode::Char('a') => {
                    app.mutable_app_state
                        .popups
                        .push(Box::new(InsertDomainPassword::new()));
                    self.operation = Some(Operation::Add);
                }
                KeyCode::Char('d') => {
                    app.mutable_app_state
                        .popups
                        .push(Box::new(InsertMaster::new()));
                    self.operation = Some(Operation::Remove);
                }
                KeyCode::Char('e') => {
                    app.mutable_app_state
                        .popups
                        .push(Box::new(InsertPassword::new(
                            self.secrets.last().unwrap().secrets
                                [self.secrets.last().unwrap().selected_secret]
                                .key
                                .clone(),
                        )));
                    self.operation = Some(Operation::Modify);
                }
                KeyCode::Char('c') => {
                    let current_secret = self
                        .secrets
                        .last()
                        .unwrap()
                        .secrets
                        .get(self.secrets.last().unwrap().selected_secret)
                        .unwrap();
                    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                    ctx.set_contents(current_secret.value.clone()).unwrap();
                }
                KeyCode::Char('f') => {
                    self.state = HomeViewState::Filter;
                }
                KeyCode::Char('x') => match self.export_csv() {
                    Ok(_) => {
                        app.mutable_app_state
                            .popups
                            .push(Box::new(MessagePopup::new(
                                "Secrets exported\nsuccessfully to the\ndownloads folder"
                                    .to_string(),
                            )));
                    }
                    Err(e) => {
                        app.mutable_app_state
                            .popups
                            .push(Box::new(MessagePopup::new(format!(
                                "Failed to export secrets: {}",
                                e
                            ))));
                    }
                },
                KeyCode::Char('?') => {
                    let rect = app.immutable_app_state.rect.unwrap_or(self.area);
                    // Calculate popup size: use 80% of screen width and height, but cap at reasonable maximums
                    let help_width = std::cmp::min((rect.width * 4) / 5, 60);
                    let help_height = std::cmp::min((rect.height * 4) / 5, 24);
                    app.mutable_app_state
                        .popups
                        .push(Box::new(MessagePopup::new_with_size(
                            Self::generate_help_text(),
                            help_width,
                            help_height,
                        )));
                }
                _ => {}
            },
            HomeViewState::Filter => match key.code {
                KeyCode::Esc => {
                    self.state = HomeViewState::Normal;
                }
                _ => {
                    let previous_value = self.filter_value.clone();
                    let config = self.generate_input_config();
                    let (value, cursor_position, input_offset) =
                        Input::handle_key(key, &config, previous_value.as_str());
                    self.filter_value = value;
                    self.cursor = cursor_position;
                    self.input_offset = input_offset;

                    self.fuzzy_filter(previous_value, self.filter_value.clone());
                }
            },
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

                let secrets = self.secrets.last_mut().unwrap();
                secrets.secrets = res
                    .unwrap()
                    .records()
                    .iter()
                    .map(|x| Secret {
                        key: x.0.clone(),
                        value: x.1.clone(),
                        last_suffix: x.0.clone(),
                    })
                    .collect();
                secrets.selected_secret = secrets.selected_secret;
                secrets.shown_secrets = secrets.shown_secrets.clone();

                let mut app = app.clone();
                app.state = ViewState::Home(self.clone());
                app
            }
            Some(Operation::Remove) => {
                let current_secret = self
                    .secrets
                    .last()
                    .unwrap()
                    .secrets
                    .get(self.secrets.last().unwrap().selected_secret)
                    .unwrap();

                let config = RecordOperationConfig::new(
                    &self.user.username(),
                    &master_password,
                    &current_secret.key,
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

                let secrets = self.secrets.last_mut().unwrap();
                secrets.secrets = res
                    .unwrap()
                    .records()
                    .iter()
                    .map(|x| Secret {
                        key: x.0.clone(),
                        value: x.1.clone(),
                        last_suffix: x.0.clone(),
                    })
                    .collect();
                secrets.selected_secret = secrets.selected_secret;
                secrets.shown_secrets = secrets.shown_secrets.clone();

                let mut app = app.clone();
                app.state = ViewState::Home(self.clone());
                app
            }
            Some(Operation::Modify) => {
                let current_secret = self
                    .secrets
                    .last()
                    .unwrap()
                    .secrets
                    .get(self.secrets.last().unwrap().selected_secret)
                    .unwrap();

                let config = RecordOperationConfig::new(
                    &self.user.username(),
                    &master_password,
                    &current_secret.key,
                    &self.new_secret.clone().unwrap().password,
                    &app.immutable_app_state.db_path,
                );

                let res = self.user.modify_record(config);

                if res.is_err() {
                    let mut app = app.clone();
                    app.mutable_app_state
                        .popups
                        .push(Box::new(MessagePopup::new(
                            "Cannot modify record".to_string(),
                        )));
                    return app;
                }

                let secrets = self.secrets.last_mut().unwrap();
                secrets.secrets = res
                    .unwrap()
                    .records()
                    .iter()
                    .map(|x| Secret {
                        key: x.0.clone(),
                        value: x.1.clone(),
                        last_suffix: x.0.clone(),
                    })
                    .collect();
                secrets.selected_secret = secrets.selected_secret;
                secrets.shown_secrets = secrets.shown_secrets.clone();

                let mut app = app.clone();
                app.state = ViewState::Home(self.clone());
                app
            }
        }
    }

    fn handle_insert_password_popup(
        &mut self,
        app: Application,
        popup: Box<dyn Popup>,
    ) -> Application {
        let password: String;
        let insert_password = popup.downcast::<InsertPassword>();

        match insert_password {
            Ok(insert_password) => {
                if insert_password.exit_state() == Some(InsertPasswordExitState::Quit) {
                    return app;
                }
                password = insert_password.password();
            }
            Err(_) => {
                unreachable!();
            }
        }

        if password.is_empty() {
            let mut app = app.clone();
            app.mutable_app_state
                .popups
                .push(Box::new(MessagePopup::new(
                    "Password cannot be empty".to_string(),
                )));
            return app;
        }

        self.new_secret = Some(NewSecret {
            domain: "".to_string(),
            password: password.clone(),
        });

        let mut app = app.clone();

        app.state = ViewState::Home(self.clone());

        app.mutable_app_state
            .popups
            .push(Box::new(InsertMaster::new()));

        app
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
            .map(|x| x.key.len() + x.value.len() + DOMAIN_PASSWORD_MIDDLE_WIDTH as usize)
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

// WARNING: Make sure to remove files created by the tests
// so that you don't clutter your filesystem with test files
#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;
    use std::{env, path::PathBuf};

    fn random_number() -> u32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(10000000..99999999)
    }

    fn generate_random_username() -> String {
        format!("krab-{}", random_number())
    }

    fn setup_user_data(domain: &str) -> Result<RecordOperationConfig, String> {
        let username = generate_random_username();
        let username = username.as_str().to_owned();
        let master_password = "password";
        let password = "password";
        let path = PathBuf::from(env::var("KRAB_TEMP_DIR").unwrap());
        let user =
            RecordOperationConfig::new(username.as_str(), master_password, domain, password, &path);
        match User::new(&user) {
            Ok(_) => Ok(user.clone()),
            Err(e) => Err(e),
        }
    }

    fn create_user(config: &RecordOperationConfig) -> Result<(User, ReadOnlyRecords), String> {
        User::from(&config.path, &config.username, &config.master_password)
    }

    #[test]
    fn test_home_largest_prefix() {
        let user_data = setup_user_data("example.com").unwrap();
        let (user, ror) = create_user(&user_data).unwrap();

        let home = Home::new(user, ror, Position::default(), Rect::default());

        let previous_value = "0123".to_string();
        let new_value = "01234".to_string();
        let largest_prefix = home.largest_prefix(&previous_value, &new_value);
        assert_eq!(largest_prefix, "0123".to_string());

        let previous_value = "01234".to_string();
        let new_value = "0123".to_string();
        let largest_prefix = home.largest_prefix(&previous_value, &new_value);
        assert_eq!(largest_prefix, "0123".to_string());

        let previous_value = "01234".to_string();
        let new_value = "0129345".to_string();
        let largest_prefix = home.largest_prefix(&previous_value, &new_value);
        assert_eq!(largest_prefix, "012".to_string());
    }

    #[test]
    fn test_home_fuzzy_filter() {
        let user_data = setup_user_data("example.com").unwrap();
        let (user, ror) = create_user(&user_data).unwrap();

        let mut home = Home::new(user, ror, Position::default(), Rect::default());

        assert!(home.secrets.len() == 1);

        let previous_value = "".to_string();
        let new_value = "e".to_string();
        home.fuzzy_filter(previous_value, new_value);
        assert_eq!(home.secrets.len(), 2);

        let previous_value = "e".to_string();
        let new_value = "".to_string();
        home.fuzzy_filter(previous_value, new_value);
        assert_eq!(home.secrets.len(), 1);
    }
}
