use anyhow::bail;
use anyhow::Result;
use biome_formatter::IndentStyle;
use biome_formatter::LineWidth;
use biome_js_formatter::context::trailing_comma::TrailingComma;
use biome_js_formatter::context::ArrowParentheses;
use biome_js_formatter::context::JsFormatOptions;
use biome_js_formatter::context::QuoteProperties;
use biome_js_formatter::context::QuoteStyle;
use biome_js_formatter::context::Semicolons;
use biome_js_parser::parse;
use biome_js_parser::JsParserOptions;
use biome_js_syntax::JsFileSource;
use biome_json_formatter::context::JsonFormatOptions;
use biome_json_parser::parse_json;
use biome_json_parser::JsonParserOptions;
use biome_json_parser::ParseDiagnostic;
use std::fmt::Write;
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
        bail!("{}", get_diagnostics_message(tree.into_diagnostics()));
      }

      let options = build_json_options(config);
      let formatted = biome_json_formatter::format_node(options, &tree.syntax())?;
      let printed = formatted.print()?;
      let output = printed.into_code();
      output
    }
    Some("js" | "jsx" | "ts" | "tsx" | "cjs" | "mjs" | "cts" | "mts") => {
      let Ok(syntax) = JsFileSource::try_from(file_path) else {
        return Ok(None);
      };

      let options = build_js_options(config, syntax);
      let tree = parse(
        &input_text,
        syntax,
        JsParserOptions {
          parse_class_parameter_decorators: true,
        },
      );
      if tree.has_errors() {
        bail!("{}", get_diagnostics_message(tree.into_diagnostics()));
      }
      let formatted = biome_js_formatter::format_node(options, &tree.syntax())?;
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
  for diagnostic in diagnostics.into_iter() {
    let diagnostic: biome_diagnostics::Error = diagnostic.into();
    let error_text = biome_diagnostics::print_diagnostic_to_string(&diagnostic);
    writeln!(&mut text, "{}", error_text).unwrap();
  }
  text
}

fn build_json_options(config: &Configuration) -> JsonFormatOptions {
  let mut options = JsonFormatOptions::default();
  if let Some(indent_style) = config.json_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(value) = config.json_indent_size {
    options = options.with_indent_width(value.into());
  }
  if let Some(line_width) = config.json_line_width {
    options = options.with_line_width(LineWidth::from_str(&line_width.to_string()).unwrap());
  }
  options
}

fn build_js_options(config: &Configuration, syntax: JsFileSource) -> JsFormatOptions {
  let mut options = JsFormatOptions::new(syntax);
  if let Some(indent_style) = config.javascript_indent_style {
    options = options.with_indent_style(match indent_style {
      crate::configuration::IndentStyle::Tab => IndentStyle::Tab,
      crate::configuration::IndentStyle::Space => IndentStyle::Space,
    });
  }
  if let Some(indent_width) = config.javascript_indent_size {
    options = options.with_indent_width(indent_width.into());
  }
  if let Some(line_width) = config.javascript_line_width {
    options = options.with_line_width(LineWidth::from_str(&line_width.to_string()).unwrap());
  }

  if let Some(semi_colons) = config.semicolons {
    options = options.with_semicolons(match semi_colons {
      crate::configuration::Semicolons::AsNeeded => Semicolons::AsNeeded,
      crate::configuration::Semicolons::Always => Semicolons::Always,
    })
  }

  if let Some(quote_style) = &config.quote_style {
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

  if let Some(trailing_comma) = &config.trailing_comma {
    options = options.with_trailing_comma(match trailing_comma {
      crate::configuration::TrailingComma::All => TrailingComma::All,
      crate::configuration::TrailingComma::Es5 => TrailingComma::Es5,
      crate::configuration::TrailingComma::None => TrailingComma::None,
    })
  }

  options
}
