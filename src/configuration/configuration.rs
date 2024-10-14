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

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
  pub line_ending: Option<LineEnding>,
  pub css_enabled: Option<bool>,
  pub css_indent_style: Option<IndentStyle>,
  pub css_indent_width: Option<u8>,
  pub css_line_width: Option<u16>,
  pub css_quote_style: Option<QuoteStyle>,
  pub graphql_enabled: Option<bool>,
  pub graphql_indent_style: Option<IndentStyle>,
  pub graphql_indent_width: Option<u8>,
  pub graphql_line_width: Option<u16>,
  pub graphql_quote_style: Option<QuoteStyle>,
  pub graphql_bracket_spacing: Option<bool>,
  pub javascript_indent_style: Option<IndentStyle>,
  pub javascript_indent_width: Option<u8>,
  pub javascript_line_width: Option<u16>,
  pub javascript_quote_style: Option<QuoteStyle>,
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
}
