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
            .filter_map(|(i, c)| if c == '\n' { Some(i) } else { None })
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
        let line_end = *self.lines.get(line + 1).unwrap_or(&self.source.len());

        Some(&self.source[line_pos + 1..line_end])
    }

    pub fn get_lines(&self, span: Span) -> Option<&str> {
        let line_pos = *self.lines.get(span.start.line)?;
        let line_end = *self
            .lines
            .get(span.end.line + 1)
            .unwrap_or(&self.source.len());

        Some(&self.source[line_pos + 1..line_end])
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Ord)]
pub struct Position {
    pub(crate) line: usize,
    pub(crate) char: usize,
}
impl Position {
    pub fn new(line: usize, char: usize) -> Self {
        Self { line, char }
    }
}
impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.char)
    }
}
impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.line.cmp(&other.line).then(self.char.cmp(&other.char)))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub(crate) start: Position,
    pub(crate) end: Position,
}
impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn union(&self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn render<'a, 'b>(
        &self,
        buffer: &'b SourceBuffer<'a>,
        label: Option<RenderLabel<'b>>,
        file: &'b str,
        notes: &'b [&'b str],
    ) -> SpanRenderer<'a, 'b> {
        SpanRenderer {
            source: buffer,
            span: *self,
            label,
            file,
            notes,
        }
    }

    pub fn len(&self) -> usize {
        self.end.char - self.start.char
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RenderLabel<'a> {
    Error(&'a str),
}
impl<'a> RenderLabel<'a> {
    pub fn get_colour(&self) -> Color {
        match self {
            Self::Error(_) => Color::BrightRed,
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
        }
    }
}

pub struct SpanRenderer<'a, 'b> {
    source: &'b SourceBuffer<'a>,
    span: Span,
    label: Option<RenderLabel<'b>>,
    file: &'b str,

    notes: &'b [&'b str],
}
impl<'a, 'b> fmt::Display for SpanRenderer<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::Colorize;

        let line = self.source.get_line(self.span.start.line).unwrap();

        let underline = iter::repeat(' ')
            .take(self.span.start.char)
            .collect::<String>()
            + &iter::repeat('^').take(self.span.len()).collect::<String>();

        let line_number = format!("{}", self.span.start.line);

        let padding = iter::repeat(' ')
            .take(line_number.len())
            .collect::<String>();

        if let Some(label) = self.label {
            writeln!(f, "{}", label)?;
        }
        writeln!(
            f,
            "{}{} {}:{}:{}",
            padding,
            "-->".bright_blue().bold(),
            self.file,
            self.span.start.line,
            self.span.start.char,
        )?;
        writeln!(f, "{} {}", padding, "|".bright_blue().bold())?;
        writeln!(
            f,
            "{} {} {}",
            line_number.bright_blue().bold(),
            "|".bright_blue().bold(),
            line,
        )?;
        writeln!(
            f,
            "{} {} {}",
            padding,
            "|".bright_blue().bold(),
            underline
                .color(
                    self.label
                        .map_or(Color::BrightBlue, |label| label.get_colour())
                )
                .bold()
        )?;

        for note in self.notes {
            writeln!(
                f,
                "{} {} {}: {}",
                padding,
                "=".bright_blue().bold(),
                "note".bold(),
                note
            )?;
        }

        Ok(())
    }
}
