use super::Configuration;
use super::IndentStyle;
use super::LineEnding;
use dprint_core::configuration::*;

/// Resolves configuration from a collection of key value strings.
///
/// # Example
///
/// ```
/// use dprint_core::configuration::ConfigKeyMap;
/// use dprint_core::configuration::resolve_global_config;
/// use dprint_plugin_biome::configuration::resolve_config;
///
/// let mut config_map = ConfigKeyMap::new(); // get a collection of key value pairs from somewhere
/// let global_config_result = resolve_global_config(&mut config_map);
///
/// // check global_config_result.diagnostics here...
///
/// let config_result = resolve_config(
///     config_map,
///     &global_config_result.config
/// );
///
/// // check config_result.diagnostics here and use config_result.config
/// ```
pub fn resolve_config(
  config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
  let mut diagnostics = Vec::new();
  let mut config = config;
  let indent_style = get_nullable_value(&mut config, "indentStyle", &mut diagnostics).or(global_config.use_tabs.map(
    |value| match value {
      true => IndentStyle::Tab,
      false => IndentStyle::Space,
    },
  ));
  let indent_size = get_nullable_value(&mut config, "indentSize", &mut diagnostics).or(global_config.indent_width);
  let line_width = get_nullable_value(&mut config, "lineWidth", &mut diagnostics).or(
    global_config
      .line_width
      .map(|l| std::cmp::max(u16::MAX as u32, l) as u16),
  );

  let resolved_config = Configuration {
    line_ending: get_nullable_value(&mut config, "lineEnding", &mut diagnostics).or_else(|| {
      match global_config.new_line_kind {
        Some(NewLineKind::CarriageReturnLineFeed) => Some(LineEnding::Crlf),
        Some(NewLineKind::LineFeed) => Some(LineEnding::Lf),
        _ => None,
      }
    }),
    javascript_indent_style: get_nullable_value(&mut config, "javascript.indentStyle", &mut diagnostics)
      .or(indent_style),
    javascript_indent_size: get_nullable_value(&mut config, "javascript.indentSize", &mut diagnostics).or(indent_size),
    javascript_line_width: get_nullable_value(&mut config, "javascript.lineWidth", &mut diagnostics).or(line_width),
    json_indent_style: get_nullable_value(&mut config, "json.indentStyle", &mut diagnostics).or(indent_style),
    json_indent_size: get_nullable_value(&mut config, "json.indentSize", &mut diagnostics).or(indent_size),
    json_line_width: get_nullable_value(&mut config, "json.lineWidth", &mut diagnostics).or(line_width),
    quote_style: get_nullable_value(&mut config, "quoteStyle", &mut diagnostics),
    jsx_quote_style: get_nullable_value(&mut config, "jsxQuoteStyle", &mut diagnostics),
    quote_properties: get_nullable_value(&mut config, "quoteProperties", &mut diagnostics),
    semicolons: get_nullable_value(&mut config, "semicolons", &mut diagnostics),
    arrow_parentheses: get_nullable_value(&mut config, "arrowParentheses", &mut diagnostics),
    trailing_comma: get_nullable_value(&mut config, "trailingComma", &mut diagnostics),
    bracket_same_line: get_nullable_value(&mut config, "bracketSameLine", &mut diagnostics),
    bracket_spacing: get_nullable_value(&mut config, "bracketSpacing", &mut diagnostics),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}
