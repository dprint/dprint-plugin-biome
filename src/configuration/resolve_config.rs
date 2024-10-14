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
  let indent_width = get_nullable_value(&mut config, "indentWidth", &mut diagnostics)
    .or_else(|| get_nullable_value(&mut config, "indentSize", &mut diagnostics))
    .or(global_config.indent_width);
  let line_width = get_nullable_value(&mut config, "lineWidth", &mut diagnostics).or(
    global_config
      .line_width
      .map(|l| std::cmp::min(u16::MAX as u32, l) as u16),
  );
  let quote_style = get_nullable_value(&mut config, "quoteStyle", &mut diagnostics);
  let jsx_quote_style = get_nullable_value(&mut config, "jsxQuoteStyle", &mut diagnostics);
  let bracket_spacing = get_nullable_value(&mut config, "bracketSpacing", &mut diagnostics);

  let resolved_config = Configuration {
    line_ending: get_nullable_value(&mut config, "lineEnding", &mut diagnostics).or(
      match global_config.new_line_kind {
        Some(NewLineKind::CarriageReturnLineFeed) => Some(LineEnding::Crlf),
        Some(NewLineKind::LineFeed) => Some(LineEnding::Lf),
        _ => None,
      },
    ),
    css_enabled: get_nullable_value(&mut config, "css.enabled", &mut diagnostics),
    css_indent_width: get_nullable_value(&mut config, "css.indentWidth", &mut diagnostics).or(indent_width),
    css_line_width: get_nullable_value(&mut config, "css.lineWidth", &mut diagnostics).or(line_width),
    css_quote_style: get_nullable_value(&mut config, "css.quoteStyle", &mut diagnostics).or(quote_style),
    css_indent_style: get_nullable_value(&mut config, "css.indentStyle", &mut diagnostics).or(indent_style),
    graphql_enabled: get_nullable_value(&mut config, "graphql.enabled", &mut diagnostics),
    graphql_indent_width: get_nullable_value(&mut config, "graphql.indentWidth", &mut diagnostics).or(indent_width),
    graphql_line_width: get_nullable_value(&mut config, "graphql.lineWidth", &mut diagnostics).or(line_width),
    graphql_quote_style: get_nullable_value(&mut config, "graphql.quoteStyle", &mut diagnostics).or(quote_style),
    graphql_indent_style: get_nullable_value(&mut config, "graphql.indentStyle", &mut diagnostics).or(indent_style),
    graphql_bracket_spacing: get_nullable_value(&mut config, "graphql.bracketSpacing", &mut diagnostics)
      .or(bracket_spacing),
    javascript_indent_style: get_nullable_value(&mut config, "javascript.indentStyle", &mut diagnostics)
      .or(indent_style),
    javascript_indent_width: get_nullable_value(&mut config, "javascript.indentWidth", &mut diagnostics)
      .or_else(|| get_nullable_value(&mut config, "javascript.indentSize", &mut diagnostics))
      .or(indent_width),
    javascript_line_width: get_nullable_value(&mut config, "javascript.lineWidth", &mut diagnostics).or(line_width),
    javascript_quote_style: get_nullable_value(&mut config, "javascript.quoteStyle", &mut diagnostics).or(quote_style),
    json_indent_style: get_nullable_value(&mut config, "json.indentStyle", &mut diagnostics).or(indent_style),
    json_indent_width: get_nullable_value(&mut config, "json.indentWidth", &mut diagnostics)
      .or_else(|| get_nullable_value(&mut config, "json.indentSize", &mut diagnostics))
      .or(indent_width),
    json_line_width: get_nullable_value(&mut config, "json.lineWidth", &mut diagnostics).or(line_width),
    quote_properties: get_nullable_value(&mut config, "quoteProperties", &mut diagnostics),
    semicolons: get_nullable_value(&mut config, "semicolons", &mut diagnostics),
    arrow_parentheses: get_nullable_value(&mut config, "arrowParentheses", &mut diagnostics),
    jsx_quote_style,
    trailing_commas: get_nullable_value(&mut config, "trailingCommas", &mut diagnostics)
      .or_else(|| get_nullable_value(&mut config, "trailingComma", &mut diagnostics)),
    bracket_same_line: get_nullable_value(&mut config, "bracketSameLine", &mut diagnostics),
    javascript_bracket_spacing: get_nullable_value(&mut config, "javascript.bracketSpacing", &mut diagnostics)
      .or(bracket_spacing),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}
