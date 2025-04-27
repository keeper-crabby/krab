use ratatui::{
    buffer::Cell,
    prelude::{Buffer, Position as RatatuiPosition, Rect},
    style::{Color, Style},
    widgets::Block,
    widgets::{Borders, Widget},
};

use crate::{centered_absolute_rect, from, views::home::Position, COLOR_ORANGE, COLOR_WHITE};

/// Represents a scrollable view
///
/// # Methods
/// * `check_if_width_out_of_bounds` - Checks if the width is out of bounds
/// * `inner_buffer_bounding_box` - Returns the inner buffer bounding box
/// * `render` - Renders the scrollable view
/// * `render_borders` - Renders the borders
/// * `render_scrollbars` - Renders the scrollbars
/// * `render_view` - Renders the view
pub struct ScrollView {}

impl ScrollView {
    /// Checks if the width is out of bounds
    ///
    /// # Arguments
    /// * `position` - The position
    /// * `buffer_to_render` - The buffer to render
    /// * `area` - The area
    ///
    /// # Returns
    /// `true` if the width is out of bounds, otherwise `false`
    pub fn check_if_width_out_of_bounds(
        position: &Position,
        buffer_to_render: &Buffer,
        area: Rect,
    ) -> bool {
        // - 2 for the borders, not the best approach but works for now
        let area = centered_absolute_rect(area, area.width - 2, area.height - 2);
        if position.offset_x() + area.width - 4 > buffer_to_render.area().width {
            return true;
        }
        false
    }

    /// Returns the inner buffer bounding box
    ///
    /// # Arguments
    /// * `area` - The area
    ///
    /// # Returns
    /// The inner buffer bounding box
    pub fn inner_buffer_bounding_box(area: Rect) -> (u16, u16) {
        (area.width - 4, area.height - 3)
    }

    // TODO: area is maybe not needed

    /// Renders the scrollable view
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `position` - The position
    /// * `area` - The area
    /// * `buffer_to_render` - The buffer to render
    pub fn render(buffer: &mut Buffer, position: &Position, area: Rect, buffer_to_render: &Buffer) {
        let area = ScrollView::render_borders(buffer, area);
        let area = ScrollView::render_scrollbars(buffer, position, area, buffer_to_render);
        ScrollView::render_view(buffer, position, area, buffer_to_render);
    }

    /// Renders the borders
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `area` - The area
    ///
    /// # Returns
    /// The inner area
    fn render_borders(buffer: &mut Buffer, area: Rect) -> Rect {
        let b = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::White)));

        b.render(area, buffer);

        Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2)
    }

    /// Renders the scrollbars
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `position` - The position
    /// * `area` - The area
    /// * `buffer_to_render` - The buffer to render
    ///
    /// # Returns
    /// The inner area
    fn render_scrollbars(
        buffer: &mut Buffer,
        position: &Position,
        area: Rect,
        buffer_to_render: &Buffer,
    ) -> Rect {
        let scrollbar_x_start = area.x;
        let scrollbar_x_end = area.x + area.width;
        let scrollbar_y_start = area.y;
        let scrollbar_y_end = area.y + area.height;
        for i in scrollbar_x_start..scrollbar_x_end - 2 {
            if i == scrollbar_x_start
                || i == scrollbar_x_end - 3
                || i == scrollbar_x_start + 1
                || i == scrollbar_x_end - 4
            {
                buffer[(i, scrollbar_y_end - 1)] = Cell::new("█")
                    .set_style(Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::Yellow)))
                    .clone();
            } else {
                buffer[(i, scrollbar_y_end - 1)] = Cell::new("━")
                    .set_style(Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White)))
                    .clone();
            }
        }
        for i in scrollbar_y_start..scrollbar_y_end - 1 {
            if i == scrollbar_y_start || i == scrollbar_y_end - 2 {
                buffer[(scrollbar_x_end - 2, i)] = Cell::new("██")
                    .set_style(Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::Yellow)))
                    .clone();
            } else {
                buffer[(scrollbar_x_end - 2, i)] = Cell::new("▕▏")
                    .set_style(Style::default().fg(from(COLOR_WHITE).unwrap_or(Color::White)))
                    .clone();
            }
        }

        let buffer_to_render_width = buffer_to_render.area().width;
        let buffer_to_render_height = buffer_to_render.area().height;

        let mut scrollbar_x_size = (area.width as f32 - 2.0) / buffer_to_render_width as f32;
        if scrollbar_x_size > 1.0 {
            scrollbar_x_size = 1.0;
        }
        let mut scrollbar_y_size = (area.height as f32 - 1.0) / buffer_to_render_height as f32;
        if scrollbar_y_size > 1.0 {
            scrollbar_y_size = 1.0;
        }

        if scrollbar_x_size < 1.0 {
            let scrollbar_x_position_start = (position.offset_x() as f32
                / buffer_to_render_width as f32)
                * (area.width as f32 - 2.0) as f32
                + area.x as f32;
            let scrollbar_x_position_end =
                scrollbar_x_position_start + scrollbar_x_size * (area.width as f32 - 2.0) as f32;

            for i in scrollbar_x_position_start as u16..scrollbar_x_position_end as u16 {
                buffer[(i, scrollbar_y_end - 1)] = Cell::new("▒")
                    .set_style(Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::Yellow)))
                    .clone();
            }
        }

        if scrollbar_y_size < 1.0 {
            let scrollbar_y_position_start = (position.offset_y() as f32
                / buffer_to_render_height as f32)
                * (area.height as f32 - 1.0) as f32
                + area.y as f32;
            let scrollbar_y_position_end =
                scrollbar_y_position_start + scrollbar_y_size * (area.height as f32 - 1.0) as f32;

            for i in scrollbar_y_position_start as u16..scrollbar_y_position_end as u16 {
                buffer[(scrollbar_x_end - 2, i)] = Cell::new("▒▒")
                    .set_style(Style::default().fg(from(COLOR_ORANGE).unwrap_or(Color::Yellow)))
                    .clone();
            }
        }

        let bottom_right_corner = "  ";
        buffer[(scrollbar_x_end - 2, scrollbar_y_end - 1)] = Cell::new(bottom_right_corner)
            .set_style(Style::default().fg(Color::Reset))
            .clone();

        Rect::new(
            scrollbar_x_start,
            scrollbar_y_start,
            scrollbar_x_end - scrollbar_x_start - 2,
            scrollbar_y_end - scrollbar_y_start - 1,
        )
    }

    /// Renders the buffer_to_render to the inner buffer
    ///
    /// # Arguments
    /// * `buffer` - The mutable buffer to render to
    /// * `position` - The position
    /// * `area` - The area
    /// * `buffer_to_render` - The buffer to render
    fn render_view(
        buffer: &mut Buffer,
        position: &Position,
        area: Rect,
        buffer_to_render: &Buffer,
    ) {
        for i in 0 + area.x..area.width + area.x {
            for j in 0 + area.y..area.height + area.y {
                let cell = buffer_to_render.cell(RatatuiPosition {
                    x: i - area.x + position.offset_x(),
                    y: j - area.y + position.offset_y(),
                });
                if cell.is_none() || i >= area.width + area.x || j >= area.height + area.y {
                    continue;
                }
                buffer[(i, j)] = buffer_to_render
                    .cell(RatatuiPosition {
                        x: i - area.x + position.offset_x(),
                        y: j - area.y + position.offset_y(),
                    })
                    .unwrap()
                    .clone();
            }
        }
    }
}
