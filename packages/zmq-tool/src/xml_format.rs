use ratatui::text::{Line, Span};
use std::fmt::{Display, Write};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use syntect_tui::into_span;
use xml::{EmitterConfig, ParserConfig};

pub fn format_xml(msg: String) -> anyhow::Result<String> {
    let mut dest = Vec::new();

    let reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(false)
        .create_reader(msg.as_bytes());

    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .normalize_empty_elements(false)
        .autopad_comments(false)
        .create_writer(&mut dest);

    for event in reader {
        if let Some(event) = event?.as_writer_event() {
            writer.write(event).unwrap();
        }
    }

    Ok(String::from_utf8(dest)?)
}

pub enum Theme {
    Light,
    Dark,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Theme::Light => "InspiredGitHub",
                Theme::Dark => "base16-ocean.dark",
            }
        )
    }
}

pub fn highlight_xml(input: String, theme: Theme) -> anyhow::Result<String> {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set.find_syntax_by_extension("xml").unwrap();

    let theme = ThemeSet::load_defaults().themes[&theme.to_string()].clone();

    let mut highlight = HighlightLines::new(syntax, &theme);
    let mut output = String::new();

    for line in LinesWithEndings::from(input.as_str()) {
        let ranges = highlight.highlight_line(line, &syntax_set)?;
        output.write_str(as_24_bit_terminal_escaped(&ranges[..], true).as_str())?;
    }

    Ok(output)
}

pub fn highlight_xml_for_ratatui(input: &str, theme: Theme) -> anyhow::Result<Vec<Line<'static>>> {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set.find_syntax_by_extension("xml").unwrap();

    let theme = ThemeSet::load_defaults().themes[&theme.to_string()].clone();

    let mut highlight = HighlightLines::new(syntax, &theme);

    let output = LinesWithEndings::from(input)
        .filter_map(|line| {
            let spans = highlight
                .highlight_line(line, &syntax_set)
                .ok()?
                .into_iter()
                .filter_map(|segment| into_span(segment).ok())
                .map(|span| Span::styled(span.content.to_string(), span.style))
                .collect::<Vec<_>>();

            Some(Line::from(spans))
        })
        .collect::<Vec<_>>();

    Ok(output)
}
