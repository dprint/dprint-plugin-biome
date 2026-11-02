use biome_css_formatter::context::CssFormatOptions;
use biome_css_parser::CssModulesKind;
use biome_css_parser::CssParserOptions;
use biome_formatter::IndentStyle;
use biome_formatter::LineEnding;
use biome_formatter::LineWidth;
use biome_formatter::QuoteStyle;
use biome_formatter::rowan::Printed;
use biome_graphql_formatter::context::GraphqlFormatOptions;
use biome_js_formatter::context::ArrowParentheses;
use biome_js_formatter::context::JsFormatOptions;
use biome_js_formatter::context::QuoteProperties;
use biome_js_formatter::context::Semicolons;
use biome_js_formatter::context::TrailingCommas;
use biome_js_parser::JsParserOptions;
use biome_json_formatter::context::JsonFormatOptions;
use biome_json_parser::JsonParserOptions;
use biome_json_parser::ParseDiagnostic;
use biome_json_parser::parse_json;
use biome_languages::CssFileSource;
use biome_languages::JsFileSource;
use biome_text_size::TextRange;
use biome_text_size::TextSize;
use camino::Utf8Path;
use std::ops::Range;
use std::path::Path;
use std::str::FromStr;

use crate::configuration::Configuration;
use crate::configuration::Language;

/// Error that occurs while formatting.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
  /// The file could not be parsed.
  #[error("{0}")]
  Parse(String),
  /// The provided configuration was not valid.
  #[error("{0}")]
  Config(String),
  /// Biome failed to format the file.
  #[error(transparent)]
  Format(#[from] biome_formatter::FormatError),
  /// Biome failed to print the formatted file.
  #[error(transparent)]
  Print(#[from] biome_formatter::PrintError),
}

pub fn format_text(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>, FormatError> {
  format(file_path, input_text, None, config)
}

/// Formats only the syntax nodes that the provided byte range overlaps, leaving the rest of the text as it was.
///
/// Biome widens the range to the closest nodes it can format on their own (JavaScript and TypeScript statements and
/// declarations, JSON values, or CSS rules, declarations and values) and keeps the indentation those nodes already
/// had. GraphQL isn't supported and returns `None`.
pub fn format_text_range(
  file_path: &Path,
  input_text: &str,
  range: Range<usize>,
  config: &Configuration,
) -> Result<Option<String>, FormatError> {
  let range = clamp_to_char_boundaries(input_text, range);
  let (Ok(start), Ok(end)) = (TextSize::try_from(range.start), TextSize::try_from(range.end)) else {
    return Ok(None);
  };
  format(file_path, input_text, Some(TextRange::new(start, end)), config)
}

fn format(
  file_path: &Path,
  input_text: &str,
  range: Option<TextRange>,
  config: &Configuration,
) -> Result<Option<String>, FormatError> {
  let lower_ext = file_path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|s| s.to_lowercase());
  let Some(language) = lower_ext.as_deref().and_then(Language::from_extension) else {
    return Ok(None);
  };
  if !config.is_language_enabled(language) {
    return Ok(None);
  }
  let output = match language {
    Language::Json | Language::Jsonc => {
      let tree = parse_json(
        input_text,
        JsonParserOptions {
          allow_comments: true,
          allow_trailing_commas: true,
        },
      );
      if tree.has_errors() {
        let trimmed_text = input_text.trim();
        if trimmed_text.is_empty() {
          // only formatting the whole file trims the whitespace
          return Ok(if range.is_some() || trimmed_text == input_text {
            None
          } else {
            Some(trimmed_text.to_string())
          });
        }
        return Err(FormatError::Parse(get_diagnostics_message(tree.into_diagnostics())));
      }

      let options = build_json_options(config)?;
      let syntax = tree.syntax();
      match range {
        Some(range) => {
          let printed = biome_json_formatter::format_range(options, &syntax, range)?;
          return Ok(splice_printed_range(input_text, printed));
        }
        None => biome_json_formatter::format_node(options, &syntax)?
          .print()?
          .into_code(),
      }
    }
    Language::JavaScript | Language::TypeScript => {
      let file_path = file_path.to_string_lossy();
      let file_path = Utf8Path::new(&file_path);
      let Ok(syntax) = JsFileSource::try_from(file_path) else {
        return Ok(None);
      };

      let options = build_js_options(config, syntax)?;
      let tree = biome_js_parser::parse(
        input_text,
        syntax,
        JsParserOptions {
          parse_class_parameter_decorators: true,
        },
      );
      if tree.has_errors() {
        return Err(FormatError::Parse(get_diagnostics_message(tree.into_diagnostics())));
      }
      let syntax = tree.syntax();
      match range {
        Some(range) => {
          let printed = biome_js_formatter::format_range(options, &syntax, range)?;
          return Ok(splice_printed_range(input_text, printed));
        }
        None => biome_js_formatter::format_node(options, &syntax, Vec::new())?
          .print()?
          .into_code(),
      }
    }
    Language::Css => {
      let file_path = file_path.to_string_lossy();
      let file_path = Utf8Path::new(&file_path);
      let Ok(syntax) = CssFileSource::try_from(file_path) else {
        return Ok(None);
      };

      let options = build_css_options(config, syntax)?;
      let tree = biome_css_parser::parse_css(
        input_text,
        syntax,
        CssParserOptions {
          allow_wrong_line_comments: true,
          css_modules: if config.css_css_modules.unwrap_or(false) {
            CssModulesKind::Classic
          } else {
            CssModulesKind::None
          },
          tailwind_directives: config.css_tailwind_directives.unwrap_or(false),
        },
      );
      if tree.has_errors() {
        return Err(FormatError::Parse(get_diagnostics_message(tree.into_diagnostics())));
      }
      let syntax = tree.syntax();
      match range {
        Some(range) => {
          let printed = biome_css_formatter::format_range(options, &syntax, range)?;
          return Ok(splice_printed_range(input_text, printed));
        }
        None => biome_css_formatter::format_node(options, &syntax)?.print()?.into_code(),
      }
    }
    Language::Graphql => {
      // Biome hasn't implemented range formatting for GraphQL yet. It formats the innermost node
      // the range touches and guesses its indentation from the whitespace before it on the line,
      // which mangles nodes that don't start a line.
      if range.is_some() {
        return Ok(None);
      }
      let options = build_graphql_options(config)?;
      let tree = biome_graphql_parser::parse_graphql(input_text);
      if tree.has_errors() {
        return Err(FormatError::Parse(get_diagnostics_message(tree.into_diagnostics())));
      }
      let formatted = biome_graphql_formatter::format_node(options, &tree.syntax())?;
      formatted.print()?.into_code()
    }
  };
  if output == input_text {
    Ok(None)
  } else {
    Ok(Some(output))
  }
}

/// Replaces the part of the input that Biome formatted with the formatted text.
///
/// Returns `None` when nothing changed.
fn splice_printed_range(input_text: &str, printed: Printed) -> Option<String> {
  // no range means there was nothing to format
  let source_range = printed.range()?;
  let source_range = usize::from(source_range.start())..usize::from(source_range.end());
  let old_text = input_text.get(source_range.clone())?;
  let new_text = printed.as_code();
  if old_text == new_text {
    return None;
  }
  let mut formatted = String::with_capacity(input_text.len() - old_text.len() + new_text.len());
  formatted.push_str(&input_text[..source_range.start]);
  formatted.push_str(new_text);
  formatted.push_str(&input_text[source_range.end..]);
  Some(formatted)
}

fn get_diagnostics_message(diagnostics: Vec<ParseDiagnostic>) -> String {
  let mut text = String::new();
  for (i, diagnostic) in diagnostics.into_iter().enumerate() {
    if i > 0 {
      text.push('\n');
    }
    let diagnostic: biome_diagnostics::Error = diagnostic.into();
    let error_text = biome_diagnostics::print_diagnostic_to_string(&diagnostic);
    text.push_str(&error_text);
  }
  text
}

fn build_json_options(config: &Configuration) -> Result<JsonFormatOptions, FormatError> {
  let mut options = JsonFormatOptions::default();
  if let Some(indent_style) = config.json_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.json_indent_width
    && let Ok(value) = value.try_into()
  {
    options = options.with_indent_width(value);
  }
  if let Some(line_ending) = config.line_ending {
    options = options.with_line_ending(match line_ending {
      crate::configuration::LineEnding::Lf => LineEnding::Lf,
      crate::configuration::LineEnding::Cr => LineEnding::Cr,
      crate::configuration::LineEnding::Crlf => LineEnding::Crlf,
    });
  }
  if let Some(line_width) = config.json_line_width {
    options = options.with_line_width(
      LineWidth::from_str(&line_width.to_string())
        .map_err(|err| FormatError::Config(format!("{} (Value: {})", err, line_width)))?,
    );
  }
  if let Some(value) = config.json_delimiter_spacing {
    options = options.with_delimiter_spacing(value.into());
  }
  if let Some(value) = config.json_trailing_newline {
    options = options.with_trailing_newline(value.into());
  }
  if let Some(value) = config.json_expand {
    options = options.with_expand(match value {
      crate::configuration::Expand::Auto => biome_formatter::Expand::Auto,
      crate::configuration::Expand::Always => biome_formatter::Expand::Always,
      crate::configuration::Expand::Never => biome_formatter::Expand::Never,
    });
  }
  if let Some(value) = config.json_trailing_commas {
    options = options.with_trailing_commas(match value {
      crate::configuration::JsonTrailingCommas::None => biome_json_formatter::context::TrailingCommas::None,
      crate::configuration::JsonTrailingCommas::All => biome_json_formatter::context::TrailingCommas::All,
    });
  }
  if let Some(value) = config.json_bracket_spacing {
    options = options.with_bracket_spacing(value.into());
  }

  Ok(options)
}

fn build_css_options(config: &Configuration, syntax: CssFileSource) -> Result<CssFormatOptions, FormatError> {
  let mut options = CssFormatOptions::new(syntax);
  if let Some(indent_style) = config.css_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.css_indent_width
    && let Ok(value) = value.try_into()
  {
    options = options.with_indent_width(value);
  }
  if let Some(line_width) = config.css_line_width {
    options = options.with_line_width(
      LineWidth::from_str(&line_width.to_string())
        .map_err(|err| FormatError::Config(format!("{} (Value: {})", err, line_width)))?,
    );
  }
  if let Some(quote_style) = &config.css_quote_style {
    options = options.with_quote_style(match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    })
  }
  if let Some(value) = config.css_delimiter_spacing {
    options = options.with_delimiter_spacing(value.into());
  }
  if let Some(value) = config.css_trailing_newline {
    options = options.with_trailing_newline(value.into());
  }
  if let Some(value) = config.line_ending {
    options = options.with_line_ending(match value {
      crate::configuration::LineEnding::Lf => LineEnding::Lf,
      crate::configuration::LineEnding::Cr => LineEnding::Cr,
      crate::configuration::LineEnding::Crlf => LineEnding::Crlf,
    });
  }

  Ok(options)
}

fn build_graphql_options(config: &Configuration) -> Result<GraphqlFormatOptions, FormatError> {
  let mut options = GraphqlFormatOptions::new();
  if let Some(indent_style) = config.graphql_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.graphql_indent_width
    && let Ok(value) = value.try_into()
  {
    options = options.with_indent_width(value);
  }
  if let Some(line_width) = config.graphql_line_width {
    options = options.with_line_width(
      LineWidth::from_str(&line_width.to_string())
        .map_err(|err| FormatError::Config(format!("{} (Value: {})", err, line_width)))?,
    );
  }
  if let Some(quote_style) = &config.graphql_quote_style {
    options = options.with_quote_style(match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    })
  }
  if let Some(bracket_spacing) = config.graphql_bracket_spacing {
    options = options.with_bracket_spacing(bracket_spacing.into());
  }
  if let Some(value) = config.graphql_trailing_newline {
    options = options.with_trailing_newline(value.into());
  }
  if let Some(value) = config.line_ending {
    options = options.with_line_ending(match value {
      crate::configuration::LineEnding::Lf => LineEnding::Lf,
      crate::configuration::LineEnding::Cr => LineEnding::Cr,
      crate::configuration::LineEnding::Crlf => LineEnding::Crlf,
    });
  }

  Ok(options)
}

fn build_js_options(config: &Configuration, syntax: JsFileSource) -> Result<JsFormatOptions, FormatError> {
  let mut options = JsFormatOptions::new(syntax);
  if let Some(line_ending) = config.line_ending {
    options = options.with_line_ending(match line_ending {
      crate::configuration::LineEnding::Crlf => LineEnding::Crlf,
      crate::configuration::LineEnding::Lf => LineEnding::Lf,
      crate::configuration::LineEnding::Cr => LineEnding::Cr,
    });
  }
  if let Some(indent_style) = config.javascript_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.javascript_indent_width
    && let Ok(value) = value.try_into()
  {
    options = options.with_indent_width(value);
  }
  if let Some(line_width) = config.javascript_line_width {
    options = options.with_line_width(
      LineWidth::from_str(&line_width.to_string())
        .map_err(|err| FormatError::Config(format!("{} (Value: {})", err, line_width)))?,
    );
  }

  if let Some(semi_colons) = config.semicolons {
    options = options.with_semicolons(match semi_colons {
      crate::configuration::Semicolons::AsNeeded => Semicolons::AsNeeded,
      crate::configuration::Semicolons::Always => Semicolons::Always,
    })
  }

  if let Some(quote_style) = &config.javascript_quote_style {
    options = options.with_quote_style(match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    })
  }

  if let Some(quote_style) = &config.jsx_quote_style {
    options = options.with_jsx_quote_style(match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    })
  }

  if let Some(quote_properties) = &config.quote_properties {
    options = options.with_quote_properties(match quote_properties {
      crate::configuration::QuoteProperties::AsNeeded => QuoteProperties::AsNeeded,
      crate::configuration::QuoteProperties::Preserve => QuoteProperties::Preserve,
    })
  }

  if let Some(arrow_parens) = &config.arrow_parentheses {
    options = options.with_arrow_parentheses(match arrow_parens {
      crate::configuration::ArrowParentheses::Always => ArrowParentheses::Always,
      crate::configuration::ArrowParentheses::AsNeeded => ArrowParentheses::AsNeeded,
    })
  }

  if let Some(trailing_commas) = &config.trailing_commas {
    options = options.with_trailing_commas(match trailing_commas {
      crate::configuration::TrailingComma::All => TrailingCommas::All,
      crate::configuration::TrailingComma::Es5 => TrailingCommas::Es5,
      crate::configuration::TrailingComma::None => TrailingCommas::None,
    })
  }

  if let Some(bracket_spacing) = &config.javascript_bracket_spacing {
    options = options.with_bracket_spacing((*bracket_spacing).into());
  }

  if let Some(bracket_same_line) = &config.bracket_same_line {
    options = options.with_bracket_same_line((*bracket_same_line).into());
  }

  if let Some(value) = config.javascript_delimiter_spacing {
    options = options.with_delimiter_spacing(value.into());
  }
  if let Some(value) = config.javascript_trailing_newline {
    options = options.with_trailing_newline(value.into());
  }
  if let Some(value) = config.javascript_attribute_position {
    options = options.with_attribute_position(match value {
      crate::configuration::AttributePosition::Auto => biome_formatter::AttributePosition::Auto,
      crate::configuration::AttributePosition::Multiline => biome_formatter::AttributePosition::Multiline,
    });
  }
  if let Some(value) = config.javascript_expand {
    options = options.with_expand(match value {
      crate::configuration::Expand::Auto => biome_formatter::Expand::Auto,
      crate::configuration::Expand::Always => biome_formatter::Expand::Always,
      crate::configuration::Expand::Never => biome_formatter::Expand::Never,
    });
  }
  if let Some(value) = config.javascript_operator_linebreak {
    options = options.with_operator_linebreak(match value {
      crate::configuration::OperatorLinebreak::After => biome_js_formatter::context::OperatorLinebreak::After,
      crate::configuration::OperatorLinebreak::Before => biome_js_formatter::context::OperatorLinebreak::Before,
    });
  }

  Ok(options)
}

/// Clamps the range to the text and widens it to the nearest char boundaries.
fn clamp_to_char_boundaries(text: &str, range: Range<usize>) -> Range<usize> {
  let mut end = range.end.min(text.len());
  let mut start = range.start.min(end);
  while !text.is_char_boundary(start) {
    start -= 1;
  }
  while !text.is_char_boundary(end) {
    end += 1;
  }
  start..end
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn handles_bom() {
    let input = "\u{FEFF}{}";
    let config = crate::configuration::Configuration::default();
    let result = format_text(std::path::Path::new("test.json"), input, &config)
      .unwrap()
      .unwrap();
    // biome chooses to keep the bom so respect that
    assert_eq!(result, "\u{FEFF}{}\n");
  }
}
