use dprint_core::configuration::ParseConfigurationError;
use dprint_core::generate_str_to_from;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
  Lf,
  Cr,
  Crlf,
}

generate_str_to_from![LineEnding, [Lf, "lf"], [Cr, "cr"], [Crlf, "crlf"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndentStyle {
  Tab,
  Space,
}

generate_str_to_from![IndentStyle, [Tab, "tab"], [Space, "space"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Semicolons {
  Always,
  AsNeeded,
}

generate_str_to_from![Semicolons, [Always, "always"], [AsNeeded, "asNeeded"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteStyle {
  Single,
  Double,
}

generate_str_to_from![QuoteStyle, [Single, "single"], [Double, "double"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteProperties {
  AsNeeded,
  Preserve,
}

generate_str_to_from![QuoteProperties, [AsNeeded, "asNeeded"], [Preserve, "preserve"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArrowParentheses {
  Always,
  AsNeeded,
}

generate_str_to_from![ArrowParentheses, [Always, "always"], [AsNeeded, "asNeeded"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrailingComma {
  All,
  Es5,
  None,
}

generate_str_to_from![TrailingComma, [All, "all"], [Es5, "es5"], [None, "none"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Expand {
  Auto,
  Always,
  Never,
}

generate_str_to_from![Expand, [Auto, "auto"], [Always, "always"], [Never, "never"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttributePosition {
  Auto,
  Multiline,
}

generate_str_to_from![AttributePosition, [Auto, "auto"], [Multiline, "multiline"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperatorLinebreak {
  After,
  Before,
}

generate_str_to_from![OperatorLinebreak, [After, "after"], [Before, "before"]];

#[derive(Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum JsonTrailingCommas {
  None,
  All,
}

generate_str_to_from![JsonTrailingCommas, [None, "none"], [All, "all"]];

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
  pub line_ending: Option<LineEnding>,
  pub css_delimiter_spacing: Option<bool>,
  pub css_trailing_newline: Option<bool>,
  pub css_enabled: Option<bool>,
  pub css_indent_style: Option<IndentStyle>,
  pub css_indent_width: Option<u8>,
  pub css_line_width: Option<u16>,
  pub css_quote_style: Option<QuoteStyle>,
  pub graphql_trailing_newline: Option<bool>,
  pub graphql_enabled: Option<bool>,
  pub graphql_indent_style: Option<IndentStyle>,
  pub graphql_indent_width: Option<u8>,
  pub graphql_line_width: Option<u16>,
  pub graphql_quote_style: Option<QuoteStyle>,
  pub graphql_bracket_spacing: Option<bool>,
  pub javascript_delimiter_spacing: Option<bool>,
  pub javascript_trailing_newline: Option<bool>,
  pub javascript_attribute_position: Option<AttributePosition>,
  pub javascript_expand: Option<Expand>,
  pub javascript_operator_linebreak: Option<OperatorLinebreak>,
  pub javascript_enabled: Option<bool>,
  pub javascript_indent_style: Option<IndentStyle>,
  pub javascript_indent_width: Option<u8>,
  pub javascript_line_width: Option<u16>,
  pub javascript_quote_style: Option<QuoteStyle>,
  pub json_delimiter_spacing: Option<bool>,
  pub json_trailing_newline: Option<bool>,
  pub json_expand: Option<Expand>,
  pub json_trailing_commas: Option<JsonTrailingCommas>,
  pub json_bracket_spacing: Option<bool>,
  pub json_enabled: Option<bool>,
  pub json_indent_style: Option<IndentStyle>,
  pub json_indent_width: Option<u8>,
  pub json_line_width: Option<u16>,
  pub semicolons: Option<Semicolons>,
  pub jsx_quote_style: Option<QuoteStyle>,
  pub quote_properties: Option<QuoteProperties>,
  pub arrow_parentheses: Option<ArrowParentheses>,
  pub trailing_commas: Option<TrailingComma>,
  pub bracket_same_line: Option<bool>,
  pub javascript_bracket_spacing: Option<bool>,
  pub css_css_modules: Option<bool>,
  pub css_tailwind_directives: Option<bool>,
  pub jsonc_enabled: Option<bool>,
  pub typescript_enabled: Option<bool>,
}

impl Configuration {
  /// Gets if the files of the language are formatted, which is decided
  /// by the language's `<language>.enabled` option.
  pub fn is_language_enabled(&self, language: Language) -> bool {
    match language {
      Language::TypeScript => self.typescript_enabled != Some(false),
      Language::JavaScript => self.javascript_enabled != Some(false),
      Language::Json => self.json_enabled != Some(false),
      Language::Jsonc => self.jsonc_enabled != Some(false),
      // these are opt-in
      Language::Css => self.css_enabled == Some(true),
      Language::Graphql => self.graphql_enabled == Some(true),
    }
  }
}

/// A language that can be turned on or off with its `<language>.enabled` option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
  TypeScript,
  JavaScript,
  Json,
  Jsonc,
  Css,
  Graphql,
}

impl Language {
  pub const ALL: [Language; 6] = [
    Language::TypeScript,
    Language::JavaScript,
    Language::Json,
    Language::Jsonc,
    Language::Css,
    Language::Graphql,
  ];

  /// Gets the language of a file with the provided lowercase extension.
  pub fn from_extension(lower_ext: &str) -> Option<Language> {
    Language::ALL
      .into_iter()
      .find(|language| language.extensions().contains(&lower_ext))
  }

  /// The file extensions of the language's files.
  pub fn extensions(&self) -> &'static [&'static str] {
    match self {
      Language::TypeScript => &["ts", "tsx", "cts", "mts"],
      Language::JavaScript => &["js", "jsx", "cjs", "mjs"],
      Language::Json => &["json"],
      Language::Jsonc => &["jsonc"],
      Language::Css => &["css"],
      Language::Graphql => &["graphql"],
    }
  }
}
