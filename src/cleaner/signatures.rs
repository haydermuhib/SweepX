/// Built-in signature definitions for common Linux desktop & developer tools.
pub struct AppSignature {
    pub id: &'static str,
    pub aliases: &'static [&'static str],
    pub config_dirs: &'static [&'static str],
    pub cache_dirs: &'static [&'static str],
    pub data_dirs: &'static [&'static str],
    pub state_dirs: &'static [&'static str],
    pub custom_user_paths: &'static [&'static str],
    pub system_paths: &'static [&'static str],
}

pub const KNOWN_SIGNATURES: &[AppSignature] = &[
    AppSignature {
        id: "vscode",
        aliases: &["code", "visual-studio-code", "vscodium", "code-oss"],
        config_dirs: &["Code", "Code - OSS", "VSCodium"],
        cache_dirs: &["Code", "Code - OSS", "VSCodium"],
        data_dirs: &["Code", "VSCodium"],
        state_dirs: &[],
        custom_user_paths: &[".vscode", ".vscode-oss"],
        system_paths: &[],
    },
    AppSignature {
        id: "chrome",
        aliases: &["google-chrome", "google-chrome-stable", "google-chrome-unstable", "chromium", "chromium-browser"],
        config_dirs: &["google-chrome", "google-chrome-beta", "google-chrome-unstable", "chromium"],
        cache_dirs: &["google-chrome", "google-chrome-beta", "google-chrome-unstable", "chromium"],
        data_dirs: &[],
        state_dirs: &[],
        custom_user_paths: &[],
        system_paths: &["/etc/opt/chrome"],
    },
    AppSignature {
        id: "firefox",
        aliases: &["firefox", "firefox-esr", "org.mozilla.firefox"],
        config_dirs: &["firefox"],
        cache_dirs: &["mozilla/firefox"],
        data_dirs: &[],
        state_dirs: &[],
        custom_user_paths: &[".mozilla/firefox", ".mozilla"],
        system_paths: &[],
    },
    AppSignature {
        id: "spotify",
        aliases: &["spotify", "spotify-client", "com.spotify.Client"],
        config_dirs: &["spotify"],
        cache_dirs: &["spotify"],
        data_dirs: &["spotify"],
        state_dirs: &[],
        custom_user_paths: &[],
        system_paths: &[],
    },
    AppSignature {
        id: "discord",
        aliases: &["discord", "discord-canary", "discord-ptb", "com.discordapp.Discord"],
        config_dirs: &["discord", "discordcanary", "discordptb"],
        cache_dirs: &["discord", "discordcanary", "discordptb"],
        data_dirs: &[],
        state_dirs: &[],
        custom_user_paths: &[],
        system_paths: &[],
    },
    AppSignature {
        id: "slack",
        aliases: &["slack", "slack-desktop", "com.slack.Slack"],
        config_dirs: &["Slack", "slack"],
        cache_dirs: &["Slack", "slack"],
        data_dirs: &["Slack"],
        state_dirs: &[],
        custom_user_paths: &[],
        system_paths: &[],
    },
    AppSignature {
        id: "steam",
        aliases: &["steam", "steam-launcher", "com.valvesoftware.Steam"],
        config_dirs: &["steam"],
        cache_dirs: &["steam"],
        data_dirs: &["Steam"],
        state_dirs: &[],
        custom_user_paths: &[".steam"],
        system_paths: &[],
    },
    AppSignature {
        id: "jetbrains",
        aliases: &[
            "idea", "intellij", "intellij-idea-community", "intellij-idea-ultimate",
            "pycharm", "pycharm-community", "pycharm-professional",
            "clion", "webstorm", "rider", "goland", "datagrip", "rustrover"
        ],
        config_dirs: &["JetBrains"],
        cache_dirs: &["JetBrains"],
        data_dirs: &["JetBrains"],
        state_dirs: &[],
        custom_user_paths: &[".local/share/JetBrains"],
        system_paths: &[],
    },
    AppSignature {
        id: "telegram",
        aliases: &["telegram", "telegram-desktop", "org.telegram.desktop"],
        config_dirs: &[],
        cache_dirs: &[],
        data_dirs: &["TelegramDesktop"],
        state_dirs: &[],
        custom_user_paths: &[],
        system_paths: &[],
    },
    AppSignature {
        id: "obsidian",
        aliases: &["obsidian", "md.obsidian.Obsidian"],
        config_dirs: &["obsidian"],
        cache_dirs: &["obsidian"],
        data_dirs: &["obsidian"],
        state_dirs: &[],
        custom_user_paths: &[],
        system_paths: &[],
    },
    AppSignature {
        id: "docker",
        aliases: &["docker", "docker-ce", "docker-desktop"],
        config_dirs: &["docker"],
        cache_dirs: &[],
        data_dirs: &["docker"],
        state_dirs: &[],
        custom_user_paths: &[".docker"],
        system_paths: &["/etc/docker", "/var/lib/docker"],
    },
];

/// Finds a known signature matching an app ID, executable name, or display name.
pub fn find_signature(query: &str) -> Option<&'static AppSignature> {
    let q = query.to_lowercase();
    for sig in KNOWN_SIGNATURES {
        if sig.id == q {
            return Some(sig);
        }
        for &alias in sig.aliases {
            if alias == q || q.contains(alias) || alias.contains(&q) {
                return Some(sig);
            }
        }
    }
    None
}
