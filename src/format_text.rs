use anyhow::Result;
use anyhow::bail;
use biome_css_formatter::context::CssFormatOptions;
use biome_css_parser::CssParserOptions;
use biome_css_syntax::CssFileSource;
use biome_formatter::IndentStyle;
use biome_formatter::LineEnding;
use biome_formatter::LineWidth;
use biome_formatter::QuoteStyle;
use biome_graphql_formatter::context::GraphqlFormatOptions;
use biome_graphql_syntax::GraphqlFileSource;
use biome_js_formatter::context::ArrowParentheses;
use biome_js_formatter::context::JsFormatOptions;
use biome_js_formatter::context::QuoteProperties;
use biome_js_formatter::context::Semicolons;
use biome_js_formatter::context::TrailingCommas;
use biome_js_parser::JsParserOptions;
use biome_js_syntax::JsFileSource;
use biome_json_formatter::context::JsonFormatOptions;
use biome_json_parser::JsonParserOptions;
use biome_json_parser::ParseDiagnostic;
use biome_json_parser::parse_json;
use camino::Utf8Path;
use std::path::Path;
use std::str::FromStr;

use crate::configuration::Configuration;

pub fn format_text(file_path: &Path, input_text: &str, config: &Configuration) -> Result<Option<String>> {
  let lower_ext = file_path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|s| s.to_lowercase());
  let output = match lower_ext.as_deref() {
    Some("json" | "jsonc") => {
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
          return Ok(if trimmed_text == input_text {
            None
          } else {
            Some(trimmed_text.to_string())
          });
        }
        bail!("{}", get_diagnostics_message(tree.into_diagnostics()));
      }

      let options = build_json_options(config)?;
      let formatted = biome_json_formatter::format_node(options, &tree.syntax())?;
      let printed = formatted.print()?;
      printed.into_code()
    }
    Some("js" | "jsx" | "ts" | "tsx" | "cjs" | "mjs" | "cts" | "mts") => {
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
          grit_metavariables: false,
        },
      );
      if tree.has_errors() {
        bail!("{}", get_diagnostics_message(tree.into_diagnostics()));
      }
      let formatted = biome_js_formatter::format_node(options, &tree.syntax())?;
      formatted.print()?.into_code()
    }
    Some("css") => {
      if config.css_enabled != Some(true) {
        return Ok(None);
      }

      let file_path = file_path.to_string_lossy();
      let file_path = Utf8Path::new(&file_path);
      let Ok(syntax) = CssFileSource::try_from(file_path) else {
        return Ok(None);
      };

      let options = build_css_options(config, syntax)?;
      let tree = biome_css_parser::parse_css(
        input_text,
        CssParserOptions {
          allow_wrong_line_comments: true,
          css_modules: true,
          grit_metavariables: false,
          tailwind_directives: Default::default(),
        },
      );
      if tree.has_errors() {
        bail!("{}", get_diagnostics_message(tree.into_diagnostics()));
      }
      let formatted = biome_css_formatter::format_node(options, &tree.syntax())?;
      formatted.print()?.into_code()
    }
    Some("graphql") => {
      if config.graphql_enabled != Some(true) {
        return Ok(None);
      }

      let file_path = file_path.to_string_lossy();
      let file_path = Utf8Path::new(&file_path);
      let Ok(syntax) = GraphqlFileSource::try_from(file_path) else {
        return Ok(None);
      };

      let options = build_graphql_options(config, syntax)?;
      let tree = biome_graphql_parser::parse_graphql(input_text);
      if tree.has_errors() {
        bail!("{}", get_diagnostics_message(tree.into_diagnostics()));
      }
      let formatted = biome_graphql_formatter::format_node(options, &tree.syntax())?;
      formatted.print()?.into_code()
    }
    _ => return Ok(None),
  };
  if output == input_text {
    Ok(None)
  } else {
    Ok(Some(output))
  }
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

fn build_json_options(config: &Configuration) -> Result<JsonFormatOptions> {
  let mut options = JsonFormatOptions::default();
  if let Some(indent_style) = config.json_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.json_indent_width {
    if let Ok(value) = value.try_into() {
      options = options.with_indent_width(value);
    }
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
      LineWidth::from_str(&line_width.to_string()).map_err(|err| anyhow::anyhow!("{} (Value: {})", err, line_width))?,
    );
  }
  Ok(options)
}

fn build_css_options(config: &Configuration, syntax: CssFileSource) -> Result<CssFormatOptions> {
  let mut options = CssFormatOptions::new(syntax);
  if let Some(indent_style) = config.css_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.css_indent_width {
    if let Ok(value) = value.try_into() {
      options = options.with_indent_width(value);
    }
  }
  if let Some(line_width) = config.css_line_width {
    options = options.with_line_width(
      LineWidth::from_str(&line_width.to_string()).map_err(|err| anyhow::anyhow!("{} (Value: {})", err, line_width))?,
    );
  }
  if let Some(quote_style) = &config.css_quote_style {
    options = options.with_quote_style(match quote_style {
      crate::configuration::QuoteStyle::Single => QuoteStyle::Single,
      crate::configuration::QuoteStyle::Double => QuoteStyle::Double,
    })
  }
  Ok(options)
}

fn build_graphql_options(config: &Configuration, syntax: GraphqlFileSource) -> Result<GraphqlFormatOptions> {
  let mut options = GraphqlFormatOptions::new(syntax);
  if let Some(indent_style) = config.graphql_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.graphql_indent_width {
    if let Ok(value) = value.try_into() {
      options = options.with_indent_width(value);
    }
  }
  if let Some(line_width) = config.graphql_line_width {
    options = options.with_line_width(
      LineWidth::from_str(&line_width.to_string()).map_err(|err| anyhow::anyhow!("{} (Value: {})", err, line_width))?,
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
  Ok(options)
}

fn build_js_options(config: &Configuration, syntax: JsFileSource) -> Result<JsFormatOptions> {
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
  if let Some(value) = config.javascript_indent_width {
    if let Ok(value) = value.try_into() {
      options = options.with_indent_width(value);
    }
  }
  if let Some(line_width) = config.javascript_line_width {
    options = options.with_line_width(
      LineWidth::from_str(&line_width.to_string()).map_err(|err| anyhow::anyhow!("{} (Value: {})", err, line_width))?,
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

  Ok(options)
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
