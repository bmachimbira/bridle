use std::io::{self, Write};
use std::path::PathBuf;

use harness_locate::{Harness, HarnessKind};

use crate::error::{Error, Result};
use crate::harness::HarnessConfig;

#[derive(Debug, Clone)]
pub struct ProviderPreset {
    pub name: &'static str,
    pub display_name: &'static str,
    pub base_url: &'static str,
    pub auth_env_var: &'static str,
}

const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        name: "z.ai",
        display_name: "Z.AI (GLM Coding Plan)",
        base_url: "https://api.z.ai/api/anthropic",
        auth_env_var: "ANTHROPIC_AUTH_TOKEN",
    },
    ProviderPreset {
        name: "openrouter",
        display_name: "OpenRouter",
        base_url: "https://openrouter.ai/api/v1",
        auth_env_var: "OPENROUTER_API_KEY",
    },
    ProviderPreset {
        name: "kimi",
        display_name: "Kimi K2 (Moonshot)",
        base_url: "https://api.moonshot.ai/anthropic",
        auth_env_var: "ANTHROPIC_AUTH_TOKEN",
    },
    ProviderPreset {
        name: "minimax",
        display_name: "MiniMax M2.1",
        base_url: "https://api.minimax.io/anthropic",
        auth_env_var: "ANTHROPIC_AUTH_TOKEN",
    },
    ProviderPreset {
        name: "glm",
        display_name: "GLM (Zhipu AI)",
        base_url: "https://api.z.ai/api/anthropic",
        auth_env_var: "ANTHROPIC_AUTH_TOKEN",
    },
];

fn find_preset(name: &str) -> Option<&'static ProviderPreset> {
    PROVIDER_PRESETS
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(name))
}

fn resolve_harness(name: &str) -> Result<Harness> {
    let kind = match name.to_lowercase().as_str() {
        "claude-code" | "claude" => HarnessKind::ClaudeCode,
        "opencode" => HarnessKind::OpenCode,
        "goose" => HarnessKind::Goose,
        "amp-code" | "amp" => HarnessKind::AmpCode,
        _ => return Err(Error::UnknownHarness(name.to_string())),
    };
    Ok(Harness::new(kind))
}

fn get_settings_path(harness: &Harness) -> Result<PathBuf> {
    let config_dir = harness.config_dir()?;
    Ok(config_dir.join("settings.json"))
}

fn read_settings(path: &PathBuf) -> Result<serde_json::Value> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| Error::Config(e.to_string()))
    } else {
        Ok(serde_json::json!({}))
    }
}

fn write_settings(path: &PathBuf, settings: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| Error::Config(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

fn prompt_api_key() -> Result<String> {
    print!("Enter API key: ");
    io::stdout().flush()?;
    let mut key = String::new();
    io::stdin().read_line(&mut key)?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(Error::Config("API key cannot be empty".to_string()));
    }
    Ok(key)
}

pub fn set_provider(
    harness_name: &str,
    provider_name: &str,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Result<()> {
    let harness = resolve_harness(harness_name)?;

    if harness.id() != "claude-code" {
        return Err(Error::Config(format!(
            "Provider configuration is currently only supported for claude-code, not {}",
            harness.id()
        )));
    }

    let (resolved_base_url, auth_var, display_name) = if provider_name.eq_ignore_ascii_case("custom") {
        let url = base_url.ok_or_else(|| {
            Error::Config("--base-url is required for custom provider".to_string())
        })?;
        (url, "ANTHROPIC_AUTH_TOKEN".to_string(), "Custom".to_string())
    } else if let Some(preset) = find_preset(provider_name) {
        (
            preset.base_url.to_string(),
            preset.auth_env_var.to_string(),
            preset.display_name.to_string(),
        )
    } else {
        return Err(Error::Config(format!(
            "Unknown provider '{}'. Use 'bridle provider list' to see available presets, or use 'custom' with --base-url.",
            provider_name
        )));
    };

    let api_key = match api_key {
        Some(k) => k,
        None => prompt_api_key()?,
    };

    let settings_path = get_settings_path(&harness)?;
    let mut settings = read_settings(&settings_path)?;

    let env = settings
        .as_object_mut()
        .ok_or_else(|| Error::Config("Invalid settings.json format".to_string()))?
        .entry("env")
        .or_insert(serde_json::json!({}));

    let env_obj = env
        .as_object_mut()
        .ok_or_else(|| Error::Config("Invalid env format in settings.json".to_string()))?;

    let provider_env_vars = [
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_AUTH_TOKEN",
        "OPENROUTER_API_KEY",
        "API_TIMEOUT_MS",
        "ANTHROPIC_MODEL",
        "ANTHROPIC_SMALL_FAST_MODEL",
        "ANTHROPIC_DEFAULT_OPUS_MODEL",
        "ANTHROPIC_DEFAULT_SONNET_MODEL",
        "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        "CLAUDE_CODE_SUBAGENT_MODEL",
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    ];
    for var in &provider_env_vars {
        env_obj.remove(*var);
    }

    env_obj.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        serde_json::Value::String(resolved_base_url.clone()),
    );
    env_obj.insert(
        auth_var.clone(),
        serde_json::Value::String(api_key),
    );
    env_obj.insert(
        "API_TIMEOUT_MS".to_string(),
        serde_json::Value::String("3000000".to_string()),
    );

    if provider_name.eq_ignore_ascii_case("kimi") {
        let kimi_model = "kimi-k2-thinking-turbo";
        env_obj.insert(
            "ANTHROPIC_MODEL".to_string(),
            serde_json::Value::String(kimi_model.to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            serde_json::Value::String(kimi_model.to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            serde_json::Value::String(kimi_model.to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            serde_json::Value::String(kimi_model.to_string()),
        );
        env_obj.insert(
            "CLAUDE_CODE_SUBAGENT_MODEL".to_string(),
            serde_json::Value::String(kimi_model.to_string()),
        );
    }

    if provider_name.eq_ignore_ascii_case("minimax") {
        let minimax_model = "MiniMax-M2.1";
        env_obj.insert(
            "ANTHROPIC_MODEL".to_string(),
            serde_json::Value::String(minimax_model.to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
            serde_json::Value::String(minimax_model.to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            serde_json::Value::String(minimax_model.to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            serde_json::Value::String(minimax_model.to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            serde_json::Value::String(minimax_model.to_string()),
        );
        env_obj.insert(
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
            serde_json::Value::String("1".to_string()),
        );
    }

    if provider_name.eq_ignore_ascii_case("glm") {
        env_obj.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            serde_json::Value::String("glm-4.7".to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            serde_json::Value::String("glm-4.7".to_string()),
        );
        env_obj.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            serde_json::Value::String("glm-4.5-air".to_string()),
        );
    }

    write_settings(&settings_path, &settings)?;

    println!("✓ Configured {} as provider for {}", display_name, harness.id());
    println!("  Base URL: {}", resolved_base_url);
    println!("\nRestart Claude Code for changes to take effect.");

    Ok(())
}

pub fn remove_provider(harness_name: &str) -> Result<()> {
    let harness = resolve_harness(harness_name)?;

    if harness.id() != "claude-code" {
        return Err(Error::Config(format!(
            "Provider configuration is currently only supported for claude-code, not {}",
            harness.id()
        )));
    }

    let settings_path = get_settings_path(&harness)?;
    
    if !settings_path.exists() {
        println!("No provider configuration found for {}", harness.id());
        return Ok(());
    }

    let mut settings = read_settings(&settings_path)?;

    let env_vars_to_remove = [
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_AUTH_TOKEN",
        "OPENROUTER_API_KEY",
        "API_TIMEOUT_MS",
        "ANTHROPIC_MODEL",
        "ANTHROPIC_SMALL_FAST_MODEL",
        "ANTHROPIC_DEFAULT_OPUS_MODEL",
        "ANTHROPIC_DEFAULT_SONNET_MODEL",
        "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        "CLAUDE_CODE_SUBAGENT_MODEL",
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    ];

    if let Some(env) = settings.get_mut("env").and_then(|e| e.as_object_mut()) {
        for var in &env_vars_to_remove {
            env.remove(*var);
        }
        if env.is_empty() {
            settings.as_object_mut().unwrap().remove("env");
        }
    }

    write_settings(&settings_path, &settings)?;

    println!("✓ Removed provider configuration for {}", harness.id());
    println!("  Claude Code will use default Anthropic API.");
    println!("\nRestart Claude Code for changes to take effect.");

    Ok(())
}

pub fn show_provider(harness_name: &str) -> Result<()> {
    let harness = resolve_harness(harness_name)?;

    if harness.id() != "claude-code" {
        return Err(Error::Config(format!(
            "Provider configuration is currently only supported for claude-code, not {}",
            harness.id()
        )));
    }

    let settings_path = get_settings_path(&harness)?;
    
    if !settings_path.exists() {
        println!("Provider: Default (Anthropic)");
        println!("No custom provider configured for {}", harness.id());
        return Ok(());
    }

    let settings = read_settings(&settings_path)?;

    let env = settings.get("env").and_then(|e| e.as_object());

    let base_url = env
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .and_then(|v| v.as_str());

    let has_auth = env
        .map(|e| e.contains_key("ANTHROPIC_AUTH_TOKEN") || e.contains_key("OPENROUTER_API_KEY"))
        .unwrap_or(false);

    match base_url {
        Some(url) => {
            let provider_name = PROVIDER_PRESETS
                .iter()
                .find(|p| p.base_url == url)
                .map(|p| p.display_name)
                .unwrap_or("Custom");

            println!("Provider: {}", provider_name);
            println!("Base URL: {}", url);
            println!("API Key:  {}", if has_auth { "configured" } else { "not set" });

            if let Some(e) = env {
                let model_vars = [
                    ("ANTHROPIC_DEFAULT_OPUS_MODEL", "Opus"),
                    ("ANTHROPIC_DEFAULT_SONNET_MODEL", "Sonnet"),
                    ("ANTHROPIC_DEFAULT_HAIKU_MODEL", "Haiku"),
                ];
                let mut has_mappings = false;
                for (var, label) in &model_vars {
                    if let Some(model) = e.get(*var).and_then(|v| v.as_str()) {
                        if !has_mappings {
                            println!("\nModel Mappings:");
                            has_mappings = true;
                        }
                        println!("  {} → {}", label, model);
                    }
                }
            }
        }
        None => {
            println!("Provider: Default (Anthropic)");
            println!("No custom provider configured.");
        }
    }

    Ok(())
}

pub fn list_providers() {
    println!("Available provider presets:\n");
    for preset in PROVIDER_PRESETS {
        println!("  {} - {}", preset.name, preset.display_name);
        println!("    Base URL: {}", preset.base_url);
        println!();
    }
    println!("  custom - Use any OpenAI-compatible API");
    println!("    Requires: --base-url <url>");
    println!();
    println!("Usage:");
    println!("  bridle provider set claude-code z.ai --api-key <key>");
    println!("  bridle provider set claude-code custom --base-url https://... --api-key <key>");
}
