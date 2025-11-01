use crate::receiver::ZmqMessage;
use crate::xml_format::{format_xml, highlight_xml_for_ratatui, Theme};
use ratatui::buffer::Buffer;
use ratatui::layout::{Margin, Rect};
use ratatui::prelude::StatefulWidget;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget};

#[derive(Debug)]
pub struct MessageViewWidget {
    message: ZmqMessage<String>,

    formatted_xml: String,
    highlighted_content: Vec<Line<'static>>,

    scroll_position: usize,
    content_length: usize,
    scroll_state: ScrollbarState,
}

impl MessageViewWidget {
    pub fn new(message: ZmqMessage<String>) -> Self {
        let formatted_xml = format_xml(message.payload.clone()).unwrap();
        let highlighted_content =
            highlight_xml_for_ratatui(formatted_xml.as_str(), Theme::Light).unwrap();

        Self {
            message: message.clone(),

            formatted_xml,
            highlighted_content,

            scroll_position: 0,
            content_length: 0,
            scroll_state: ScrollbarState::default(),
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_state.next();
        self.scroll_position = self.scroll_position.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_state.prev();
        self.scroll_position = self.scroll_position.saturating_sub(1);
    }
}

impl Widget for &mut MessageViewWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(format!(
                "{} - {}",
                self.message.timestamp.time(),
                self.message.topic
            ))
            .title_bottom("Esc/q = return to list | s = Save | e = Open in $EDITOR");

        self.content_length = self.highlighted_content.len();

        self.scroll_state = self
            .scroll_state
            .content_length(self.content_length)
            .position(self.scroll_position);

        let paragraph = Paragraph::new(self.highlighted_content.clone())
            .block(block)
            .scroll((self.scroll_position as u16, 0));
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

        paragraph.render(area, buf);
        scrollbar.render(
            area.inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
            buf,
            &mut self.scroll_state,
        );
    }
}
