//! CLI subcommand definitions.

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show status of all harnesses.
    Status,

    /// Initialize bridle configuration.
    Init,

    /// Manage profiles.
    #[command(subcommand)]
    Profile(ProfileCommands),

    /// Launch terminal UI.
    Tui,

    /// Manage bridle settings.
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Configure API providers for harnesses (e.g., Z.AI, OpenRouter).
    #[command(subcommand)]
    Provider(ProviderCommands),

    /// Install skills from a GitHub repository.
    Install {
        /// GitHub repository URL or owner/repo shorthand.
        source: String,
        /// Force overwrite existing skills.
        #[arg(long, short)]
        force: bool,
    },

    /// Uninstall components from a profile.
    Uninstall {
        /// Harness name (claude-code, opencode, goose).
        harness: String,
        /// Profile name.
        profile: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Set a configuration value.
    Set {
        /// Setting name (e.g., profile_marker).
        key: String,
        /// Value to set (true/false for booleans).
        value: String,
    },

    /// Get a configuration value.
    Get {
        /// Setting name.
        key: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List profiles for a harness.
    List {
        /// Harness name (claude-code, opencode, goose).
        harness: String,
    },

    /// Show details of a specific profile.
    Show {
        /// Harness name.
        harness: String,
        /// Profile name.
        name: String,
    },

    /// Create a new profile.
    Create {
        /// Harness name.
        harness: String,
        /// Profile name.
        name: String,
        /// Copy current harness config to the new profile.
        #[arg(long)]
        from_current: bool,
    },

    /// Delete a profile.
    Delete {
        /// Harness name.
        harness: String,
        /// Profile name.
        name: String,
    },

    /// Switch to a profile (set as active).
    Switch {
        /// Harness name.
        harness: String,
        /// Profile name.
        name: String,
    },

    /// Edit a profile with $EDITOR.
    Edit {
        /// Harness name.
        harness: String,
        /// Profile name.
        name: String,
    },

    /// Compare two profiles or profile vs current config.
    Diff {
        /// Harness name.
        harness: String,
        /// First profile name.
        name: String,
        /// Second profile name (optional, defaults to current config).
        other: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProviderCommands {
    /// Set an API provider for a harness (configures base URL and auth).
    Set {
        /// Harness name (claude-code, opencode, goose).
        harness: String,
        /// Provider name (z.ai, openrouter, custom).
        provider: String,
        /// API key for the provider.
        #[arg(long, short = 'k')]
        api_key: Option<String>,
        /// Custom base URL (required for 'custom' provider).
        #[arg(long)]
        base_url: Option<String>,
    },

    /// Remove provider configuration (restore default).
    Remove {
        /// Harness name (claude-code, opencode, goose).
        harness: String,
    },

    /// Show current provider configuration.
    Show {
        /// Harness name (claude-code, opencode, goose).
        harness: String,
    },

    /// List available provider presets.
    List,
}
