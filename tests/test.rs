extern crate dprint_development;
extern crate dprint_plugin_biome;

use std::path::PathBuf;
use std::sync::Arc;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_biome::configuration::Configuration;
use dprint_plugin_biome::configuration::resolve_config;
use dprint_plugin_biome::*;
use pretty_assertions::assert_eq;

#[test]
fn test_specs() {
  let global_config = GlobalConfiguration::default();

  run_specs(
    &PathBuf::from("./tests/specs"),
    &ParseSpecOptions {
      default_file_name: "file.ts",
    },
    &RunSpecsOptions {
      fix_failures: false,
      format_twice: true,
    },
    {
      let global_config = global_config.clone();
      Arc::new(move |file_path, file_text, spec_config| {
        let spec_config: ConfigKeyMap = serde_json::from_value(spec_config.clone().into()).unwrap();
        let config_result = resolve_config(spec_config, &global_config);
        ensure_no_diagnostics(&config_result.diagnostics);

        format_text(file_path, file_text, &config_result.config).map_err(anyhow::Error::from)
      })
    },
    Arc::new(move |_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing.")),
  )
}

#[test]
fn should_fail_on_parse_error_js() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.ts"), "const t string = 5;", &config).unwrap_err();
  assert_eq!(
    err.to_string(),
    r#"parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Const declarations must have an initialized value.
  
  i This variable needs to be initialized.
  


parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Expected a semicolon or an implicit semicolon after a statement, but found none
  
  i An explicit or implicit semicolon is expected here...
  
  i ...Which is required to end this statement
  

"#
  );
}

#[test]
fn should_fail_on_parse_error_json() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.json"), "{", &config).unwrap_err();
  let err_str = err.to_string();
  assert!(
    err_str.contains("expected `}` but instead the file ends"),
    "Unexpected error: {}",
    err_str
  );
}

#[test]
fn should_not_format_disabled_languages() {
  let mut config_map = ConfigKeyMap::new();
  for key in ["typescript.enabled", "jsonc.enabled"] {
    config_map.insert(key.to_string(), ConfigKeyValue::from_bool(false));
  }
  let result = resolve_config(config_map, &GlobalConfiguration::default());
  assert!(result.diagnostics.is_empty());
  for (file_name, text, expected) in [
    ("file.ts", "a", None),
    ("file.tsx", "a", None),
    ("file.mts", "a", None),
    ("file.jsonc", "[1,2]", None),
    ("file.js", "a", Some("a;\n")),
    ("file.jsx", "a", Some("a;\n")),
    ("file.json", "[1,2]", Some("[1, 2]\n")),
  ] {
    let output = format_text(&PathBuf::from(file_name), text, &result.config).unwrap();
    assert_eq!(output.as_deref(), expected, "{file_name}");
  }

  let mut config_map = ConfigKeyMap::new();
  for key in ["javascript.enabled", "json.enabled"] {
    config_map.insert(key.to_string(), ConfigKeyValue::from_bool(false));
  }
  let result = resolve_config(config_map, &GlobalConfiguration::default());
  assert!(result.diagnostics.is_empty());
  for (file_name, text, expected) in [
    ("file.js", "a", None),
    ("file.cjs", "a", None),
    ("file.json", "[1,2]", None),
    ("file.ts", "a", Some("a;\n")),
    ("file.jsonc", "[1,2]", Some("[1, 2]\n")),
  ] {
    let output = format_text(&PathBuf::from(file_name), text, &result.config).unwrap();
    assert_eq!(output.as_deref(), expected, "{file_name}");
  }
}

#[test]
fn language_enabled_defaults() {
  use dprint_plugin_biome::configuration::Language;

  let config = Configuration::default();
  let enabled = Language::ALL
    .into_iter()
    .filter(|language| config.is_language_enabled(*language))
    .collect::<Vec<_>>();
  assert_eq!(
    enabled,
    vec![
      Language::TypeScript,
      Language::JavaScript,
      Language::Json,
      Language::Jsonc
    ]
  );
  let config = Configuration {
    css_enabled: Some(true),
    graphql_enabled: Some(true),
    ..Default::default()
  };
  assert!(
    Language::ALL
      .into_iter()
      .all(|language| config.is_language_enabled(language))
  );
  for language in Language::ALL {
    for ext in language.extensions() {
      assert_eq!(Language::from_extension(ext), Some(language), "{ext}");
    }
  }
}

#[test]
fn css_modules_config_resolves() {
  let mut config_map = ConfigKeyMap::new();
  config_map.insert("css.cssModules".to_string(), ConfigKeyValue::from_bool(true));
  let global_config = GlobalConfiguration::default();
  let result = resolve_config(config_map, &global_config);
  assert!(result.diagnostics.is_empty());
  assert_eq!(result.config.css_css_modules, Some(true));
}

#[test]
fn range_inside_statement_only_formats_that_statement() {
  let text = "const a  =  1;\nconst b  =  2;\nconst c  =  3;\n";
  assert_eq!(
    format_range("file.ts", text, range_of(text, "b  =", "2;")),
    Some("const a  =  1;\nconst b = 2;\nconst c  =  3;\n".to_string())
  );
}

#[test]
fn range_spanning_statements_formats_each_statement() {
  let text = "function f() {\n\ta  ;\n\tb  ;\n\tc  ;\n\td  ;\n}\n";
  assert_eq!(
    format_range("file.ts", text, range_of(text, "b", "c")),
    Some("function f() {\n\ta  ;\n\tb;\n\tc;\n\td  ;\n}\n".to_string())
  );
}

#[test]
fn range_already_formatted_returns_no_change() {
  let text = "const a  =  1;\nconst b = 2;\nconst c  =  3;\n";
  assert_eq!(format_range("file.ts", text, range_of(text, "const b", "2;")), None);
  // an empty range has nothing to format
  let index = text.find('b').unwrap();
  assert_eq!(format_range("file.ts", text, index..index), None);
}

#[test]
fn range_after_multi_byte_chars() {
  let text = "const s  =  \"héllo 日本\";\nconst b  =  2;\nconst c  =  3;\n";
  assert_eq!(
    format_range("file.ts", text, range_of(text, "b  =", "2;")),
    Some("const s  =  \"héllo 日本\";\nconst b = 2;\nconst c  =  3;\n".to_string())
  );
}

#[test]
fn range_inside_multi_byte_char_is_widened() {
  let text = "const a  =  1;\nconst s  =  \"日本\";\n";
  let index = text.find('日').unwrap();
  assert_eq!(
    format_range("file.ts", text, index + 1..index + 2),
    Some("const a  =  1;\nconst s = \"日本\";\n".to_string())
  );
}

#[test]
fn range_past_end_of_text_is_clamped() {
  let text = "const a  =  1;\n";
  assert_eq!(
    format_range("file.ts", text, 0..1000),
    Some("const a = 1;\n".to_string())
  );
}

#[test]
fn range_json() {
  let text = "{\n\t\"a\":  [1,2],\n\t\"b\":  [3,4],\n\t\"c\":  [5,6]\n}\n";
  assert_eq!(
    format_range("file.json", text, range_of(text, "[3", "4]")),
    Some("{\n\t\"a\":  [1,2],\n\t\"b\":  [3, 4],\n\t\"c\":  [5,6]\n}\n".to_string())
  );
}

#[test]
fn range_css() {
  let text = "a {\n\tcolor:red;\n\tbackground:blue;\n}\nb { color:green; }\n";
  assert_eq!(
    format_range("file.css", text, range_of(text, "red", "blue")),
    Some("a {\n\tcolor: red;\n\tbackground: blue;\n}\nb { color:green; }\n".to_string())
  );
}

#[test]
fn range_graphql_is_not_supported() {
  let text = "query A { a }\nquery   B {   b   }\n";
  assert_eq!(format_range("file.graphql", text, 0..text.len()), None);
}

fn format_range(file_name: &str, text: &str, range: std::ops::Range<usize>) -> Option<String> {
  let config = Configuration {
    css_enabled: Some(true),
    graphql_enabled: Some(true),
    ..Default::default()
  };
  format_text_range(&PathBuf::from(file_name), text, range, &config).unwrap()
}

/// Gets the range from the start of `start` to the end of the first `end` after it.
fn range_of(text: &str, start: &str, end: &str) -> std::ops::Range<usize> {
  let start = text.find(start).unwrap();
  let end = start + text[start..].find(end).unwrap() + end.len();
  start..end
}
