use core::fmt;
use std::iter;

use thiserror::Error;

use crate::{
    instruction::OperandParseError,
    span::{RenderLabel, SourceBuffer, Span},
};

pub type InterpreterResult<'a, T> = Result<T, InterpreterError<'a>>;

#[derive(Clone, Debug, Error)]
pub enum InterpreterError<'a> {
    #[error("Unexpected tokens")]
    UnexpectedTokens(Span),

    #[error("Invalid label name")]
    InvalidLabel(&'a str, Span),

    #[error("Expected opcode")]
    ExpectedOpCode(Span),

    #[error("{1}")]
    OperandParseError(Span, OperandParseError),

    #[error("Duplicate label `{0}`")]
    DuplicateLabel(&'a str, Span, Span),

    #[error("Use of undefined label")]
    UndefinedLabel(&'a str, Span),
}
impl<'a> InterpreterError<'a> {
    pub fn render(
        self,
        source: &'a SourceBuffer,
        filepath: &'a str,
    ) -> InterpreterErrorRenderer<'a> {
        InterpreterErrorRenderer {
            error: self,
            source,
            filepath,
        }
    }
}

pub struct InterpreterErrorRenderer<'a> {
    error: InterpreterError<'a>,
    source: &'a SourceBuffer<'a>,
    filepath: &'a str,
}
impl<'a> fmt::Display for InterpreterErrorRenderer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error {
            InterpreterError::UnexpectedTokens(span) => write!(
                f,
                "{}",
                InterpreterErrorRenderHelper {
                    label: Some(RenderLabel::Error("unexpected tokens")),
                    filepath: self.filepath,
                    source: self.source,

                    main_span: span,
                    spans: &[(span, Some(RenderLabel::Error("unexpected tokens")))],

                    notes: &["each instruction can only contain at most a label, an opcode, and an operand"],
                }
            ),
            InterpreterError::ExpectedOpCode(span) => write!(
                f,
                "{}",
                InterpreterErrorRenderHelper {
                    label: Some(RenderLabel::Error("expected an opcode")),
                    filepath: self.filepath,
                    source: self.source,

                    main_span: span,
                    spans: &[(span, Some(RenderLabel::Error("expected an opcode here")))],

                    notes: &["each instruction must contain an opcode"],
                }
            ),
            InterpreterError::OperandParseError(span, ref e) => match e {
                OperandParseError::InvalidIntegerLiteral(e) => write!(
                    f,
                    "{}",
                    InterpreterErrorRenderHelper {
                        label: Some(RenderLabel::Error("invalid integer literal")),
                        filepath: self.filepath,
                        source: self.source,

                        main_span: span,
                        spans: &[(span, Some(RenderLabel::Error("")))],

                        notes: &[&e.to_string()],
                    },
                ),
                OperandParseError::InvalidLabel(label) => write!(
                    f,
                    "{}",
                    InterpreterErrorRenderHelper {
                        label: Some(RenderLabel::Error("invalid label")),
                        filepath: self.filepath,
                        source: self.source,

                        main_span: span,
                        spans: &[(span, Some(RenderLabel::Error("")))],

                        notes: &[&format!("the label `{}` contains invalid characters", label)],
                    },
                ),
            },
            InterpreterError::InvalidLabel(label, span) => write!(
                f,
                "{}",
                InterpreterErrorRenderHelper {
                    label: Some(RenderLabel::Error("invalid label")),
                    filepath: self.filepath,
                    source: self.source,

                    main_span: span,
                    spans: &[(span, Some(RenderLabel::Error("")))],

                    notes: &[&format!("opcodes, such as `{}`, cannot be used as labels", label)]
                },
            ),
            InterpreterError::DuplicateLabel(label, first, again) => write!(
                f,
                "{}",
                InterpreterErrorRenderHelper {
                    label: Some(RenderLabel::Error(&format!("the label `{}` has been defined multiple times", label))),
                    filepath: self.filepath,
                    source: self.source,

                    main_span: again,
                    spans: &[
                        (first, Some(RenderLabel::Info("label first defined here"))),
                        (again, Some(RenderLabel::Error("label redefined here")))
                    ],

                    notes: &[],
                }
            ),
            InterpreterError::UndefinedLabel(label, span) => write!(
                f,
                "{}",
                InterpreterErrorRenderHelper {
                    label: Some(RenderLabel::Error(&format!("use of undefined label `{}`", label))),
                    filepath: self.filepath,
                    source: self.source,

                    main_span: span,
                    spans: &[(span, Some(RenderLabel::Error("undefined label used here")))],

                    notes: &[],
                }
            )
        }
    }
}

struct InterpreterErrorRenderHelper<'a> {
    label: Option<RenderLabel<'a>>,
    filepath: &'a str,
    source: &'a SourceBuffer<'a>,

    main_span: Span,
    spans: &'a [(Span, Option<RenderLabel<'a>>)],

    notes: &'a [&'a str],
}
impl<'a> fmt::Display for InterpreterErrorRenderHelper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::Colorize;

        let padding_width = self
            .spans
            .iter()
            .map(|(s, _)| s.line.to_string().len())
            .max()
            .unwrap_or(0);

        let padding = iter::repeat(' ').take(padding_width).collect::<String>();

        if let Some(label) = self.label {
            writeln!(f, "{}", label)?;
        }
        writeln!(
            f,
            "{}{} {}:{}:{}",
            padding,
            "-->".bright_blue().bold(),
            self.filepath,
            self.main_span.line,
            self.main_span.start,
        )?;

        writeln!(f, "{} {}", padding, "|".bright_blue().bold())?;

        let mut spans = self.spans.to_vec();

        spans.sort_by_key(|(s, _)| s.line);

        for window in self.spans.windows(2) {
            let &[(span, label), (next, _)] = window else {
                unreachable!();
            };

            writeln!(f, "{}", span.render(self.source, label, &padding))?;

            match next.line - span.line {
                0 | 1 => {}
                2 => {
                    let mut next = span;
                    next.line += 1;

                    writeln!(f, "{}", next.render(self.source, None, &padding))?
                }
                _ => writeln!(f, " {}", "...".bright_blue().bold())?,
            }
        }

        if let Some((span, label)) = spans.last() {
            let render = span.render(self.source, *label, &padding);

            writeln!(f, "{}", render)?;
        }

        if self.notes.len() > 0 {
            writeln!(f, "{} {}", padding, "|".bright_blue().bold())?;
        }

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
