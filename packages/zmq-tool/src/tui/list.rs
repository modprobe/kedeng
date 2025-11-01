use crate::receiver::ZmqMessage;
use color_eyre::owo_colors::OwoColorize;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{
    Block, HighlightSpacing, Row, ScrollDirection, Scrollbar, ScrollbarOrientation, ScrollbarState,
    StatefulWidget, Table, TableState, Widget,
};

#[derive(Default, Debug)]
pub struct MessageListWidget {
    pub state: MessageListWidgetState,
}

#[derive(Default, Debug)]
pub struct MessageListWidgetState {
    messages: Vec<ZmqMessage<String>>,
    pub table_state: TableState,
    scroll_state: ScrollbarState,
}

impl MessageListWidget {
    pub(crate) fn new(messages: Vec<ZmqMessage<String>>) -> MessageListWidget {
        MessageListWidget {
            state: MessageListWidgetState {
                messages,
                ..Default::default()
            },
        }
    }

    pub fn update_messages(&mut self, messages: Vec<ZmqMessage<String>>) {
        self.state.messages = messages;
        self.state.scroll_state = self
            .state
            .scroll_state
            .content_length(self.state.messages.len());
    }

    pub fn scroll_down(&mut self) {
        if self.state.table_state.selected().is_none() {
            self.state.table_state.select_first();
            self.state.scroll_state.first();
        } else {
            self.state.table_state.scroll_down_by(1);
            self.state.scroll_state.scroll(ScrollDirection::Forward);
        }
    }

    pub fn scroll_up(&mut self) {
        if self.state.table_state.selected().is_none() {
            self.state.table_state.select_last();
            self.state.scroll_state.last();
        } else {
            self.state.table_state.scroll_up_by(1);
            self.state.scroll_state.scroll(ScrollDirection::Backward);
        }
    }
}

impl Widget for &mut MessageListWidget {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Messages")
            .title_bottom("q to quit");

        let rows = self.state.messages.iter();
        let widths = [
            Constraint::Length(10),
            Constraint::Percentage(15),
            Constraint::Fill(1),
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">> ")
            .row_highlight_style(Style::new().black().on_blue());

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);

        StatefulWidget::render(table, area, buf, &mut self.state.table_state);
        StatefulWidget::render(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            buf,
            &mut self.state.scroll_state,
        );
    }
}

impl From<&ZmqMessage<String>> for Row<'_> {
    fn from(message: &ZmqMessage<String>) -> Self {
        Row::new(vec![
            message.timestamp.clone().time().to_string(),
            message.topic.clone(),
            message.payload.clone().get(0..100).unwrap().to_string(),
        ])
    }
}
