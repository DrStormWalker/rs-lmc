use core::fmt;
use std::iter;

use colored::Color;

pub struct SourceBuffer<'a> {
    source: &'a str,
    lines: Vec<usize>,
}
impl<'a> SourceBuffer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut almost_lines = source
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if c == '\n' { Some(i + 1) } else { None })
            .collect();

        let mut lines = vec![0];

        lines.append(&mut almost_lines);

        Self { source, lines }
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn get_line(&self, line: usize) -> Option<&str> {
        let line_pos = *self.lines.get(line)?;
        let line_end = self
            .lines
            .get(line + 1)
            .map(|l| *l - 1)
            .unwrap_or(self.source.len());

        Some(&self.source[line_pos..line_end])
    }

    pub fn get_lines(&self, span: Span) -> Option<&str> {
        let line_pos = *self.lines.get(span.line)?;
        let line_end = *self.lines.get(span.line + 1).unwrap_or(&self.source.len());

        Some(&self.source[line_pos + 1..line_end])
    }
}

#[derive(Copy, Clone, Debug, Hash)]
pub struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) line: usize,
}
impl Span {
    pub fn new(start: usize, end: usize, line: usize) -> Self {
        Self { start, end, line }
    }

    pub fn union(&self, other: Self) -> Self {
        if self.line != other.line {
            panic!("Two spans that are unionized must be from the same line");
        }

        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
        }
    }

    pub fn render<'a, 'b>(
        &self,
        buffer: &'b SourceBuffer<'a>,
        label: Option<RenderLabel<'b>>,
        padding: &'b str,
    ) -> SpanRenderer<'a, 'b> {
        SpanRenderer {
            source: buffer,
            span: *self,
            label,
            padding,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RenderLabel<'a> {
    Error(&'a str),
    Info(&'a str),
}
impl<'a> RenderLabel<'a> {
    pub fn label(&self) -> &'a str {
        match self {
            Self::Error(s) => s,
            Self::Info(s) => s,
        }
    }

    pub fn underline_char(&self) -> char {
        match self {
            Self::Error(_) => '^',
            Self::Info(_) => '-',
        }
    }

    pub fn get_colour(&self) -> Color {
        match self {
            Self::Error(_) => Color::BrightRed,
            Self::Info(_) => Color::BrightBlue,
        }
    }
}
impl<'a> fmt::Display for RenderLabel<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::Colorize;

        match self {
            Self::Error(err) => write!(
                f,
                "{}{} {}",
                "error".bright_red().bold(),
                ":".bold(),
                err.bold()
            ),
            Self::Info(info) => write!(
                f,
                "{}{} {}",
                "info".bright_blue().bold(),
                ":".bold(),
                info.bold(),
            ),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SpanRenderer<'a, 'b> {
    source: &'b SourceBuffer<'a>,
    pub(crate) span: Span,
    label: Option<RenderLabel<'b>>,
    padding: &'b str,
}
impl<'a, 'b> fmt::Display for SpanRenderer<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::Colorize;

        let line = self.source.get_line(self.span.line).unwrap();

        let line_number = format!("{}", self.span.line + 1);

        write!(
            f,
            "{: <padding_width$} {} {}",
            line_number.bright_blue().bold(),
            "|".bright_blue().bold(),
            line,
            padding_width = self.padding.len(),
        )?;

        if let Some(label) = self.label {
            let underline = iter::repeat(' ').take(self.span.start).collect::<String>()
                + &iter::repeat(label.underline_char())
                    .take(self.span.len())
                    .collect::<String>();

            write!(
                f,
                "\n{} {} {}",
                self.padding,
                "|".bright_blue().bold(),
                underline.color(label.get_colour()).bold(),
            )?;

            if label.label().len() > 0 {
                write!(f, " {}", label.label().color(label.get_colour()).bold(),)?;
            }
        }

        Ok(())
    }
}
