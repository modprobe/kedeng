use crate::receiver::ZmqMessage;
use crate::tui::list::MessageListWidget;
use crate::tui::view::MessageViewWidget;
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use futures_util::StreamExt;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::{Frame, Terminal};
use std::time::Duration;
use tokio::sync::broadcast::Receiver;

mod list;
mod view;

#[derive(Debug)]
pub struct TuiMode {
    should_quit: bool,
    receiver: Receiver<ZmqMessage<String>>,
    messages: Vec<ZmqMessage<String>>,

    current_screen: Screen,

    message_list_widget: MessageListWidget,
    message_view_widget: Option<MessageViewWidget>,
}

impl TuiMode {
    const FPS: f32 = 60.0;

    pub fn new(receiver: Receiver<ZmqMessage<String>>) -> Self {
        Self {
            receiver,
            should_quit: false,
            messages: Vec::new(),
            current_screen: Screen::default(),

            message_list_widget: MessageListWidget::default(),
            message_view_widget: None,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let mut terminal = ratatui::init();
        let period = Duration::from_secs_f32(1.0 / Self::FPS);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                biased;
                msg = self.receiver.recv() => {
                    match msg {
                        Ok(message) => {
                            self.messages.push(message);
                            while let Ok(msg) = self.receiver.try_recv() {
                                self.messages.push(msg);
                            }

                            self.message_list_widget.update_messages(self.messages.clone());
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                Some(Ok(event)) = events.next() => { self.handle_event(&event); }
                _ = interval.tick() => {
                    self.draw(&mut terminal)?;
                },
            }
        }

        Ok(())
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        terminal.draw(|frame| self.render(frame))?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);

        let [title_area, body_area] = layout.areas(frame.area());
        let title = Line::from("zMQ Messages");

        frame.render_widget(title, title_area);
        match &self.current_screen {
            Screen::List => {
                frame.render_widget(&mut self.message_list_widget, body_area);
            }
            Screen::View(_) => {
                frame.render_widget(self.message_view_widget.as_mut().unwrap(), body_area);
            }
        }
    }

    fn handle_event(&mut self, event: &Event) {
        if let Some(key) = event.as_key_event() {
            match self.current_screen {
                Screen::List => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Up => self.message_list_widget.scroll_up(),
                    KeyCode::Down => self.message_list_widget.scroll_down(),
                    KeyCode::Enter => self.view_selected_message(),
                    _ => {}
                },
                Screen::View(_) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.current_screen = Screen::List,
                    KeyCode::Up => self.message_view_widget.as_mut().unwrap().scroll_up(),
                    KeyCode::Down => self.message_view_widget.as_mut().unwrap().scroll_down(),
                    _ => {}
                },
            }
        }
    }

    fn view_selected_message(&mut self) {
        let selected = self.message_list_widget.state.table_state.selected();
        if let Some(selected) = selected {
            self.message_view_widget =
                Some(MessageViewWidget::new(self.messages[selected].clone()));
            self.current_screen = Screen::View(selected);
        }
    }
}

#[derive(Debug, Default)]
enum Screen {
    #[default]
    List,
    View(usize),
}
