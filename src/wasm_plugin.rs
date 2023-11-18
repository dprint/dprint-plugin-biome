use super::configuration::{resolve_config, Configuration};

use dprint_core::configuration::{ConfigKeyMap, GlobalConfiguration, ResolveConfigurationResult};
use dprint_core::generate_plugin_code;
use dprint_core::plugins::FileMatchingInfo;
use dprint_core::plugins::FormatResult;
use dprint_core::plugins::PluginInfo;
use dprint_core::plugins::SyncPluginHandler;
use dprint_core::plugins::SyncPluginInfo;
use std::path::Path;

struct BiomePluginHandler;

impl SyncPluginHandler<Configuration> for BiomePluginHandler {
  fn resolve_config(
    &mut self,
    config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
  ) -> ResolveConfigurationResult<Configuration> {
    resolve_config(config, global_config)
  }

  fn plugin_info(&mut self) -> SyncPluginInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    SyncPluginInfo {
      info: PluginInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: version.clone(),
        config_key: "biome".to_string(),
        help_url: "https://dprint.dev/plugins/biome".to_string(),
        config_schema_url: format!(
          "https://plugins.dprint.dev/dprint/dprint-plugin-biome/{}/schema.json",
          version
        ),
        update_url: Some("https://plugins.dprint.dev/dprint/dprint-plugin-biome/latest.json".to_string()),
      },
      file_matching: FileMatchingInfo {
        file_extensions: vec![
          "ts".to_string(),
          "tsx".to_string(),
          "cts".to_string(),
          "mts".to_string(),
          "js".to_string(),
          "jsx".to_string(),
          "cjs".to_string(),
          "mjs".to_string(),
          "json".to_string(),
          "jsonc".to_string(),
        ],
        file_names: vec![],
      },
    }
  }

  fn license_text(&mut self) -> String {
    std::str::from_utf8(include_bytes!("../LICENSE")).unwrap().into()
  }

  fn format(
    &mut self,
    file_path: &Path,
    file_text: &str,
    config: &Configuration,
    _format_with_host: impl FnMut(&Path, String, &ConfigKeyMap) -> FormatResult,
  ) -> FormatResult {
    super::format_text(file_path, file_text, config)
  }
}

generate_plugin_code!(BiomePluginHandler, BiomePluginHandler);
