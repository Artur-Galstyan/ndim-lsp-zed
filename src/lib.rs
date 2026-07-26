use std::fs;
use zed_extension_api::{
    self as zed, settings::LspSettings, Command, LanguageServerId, Result, Worktree,
};

const BINARY_NAME: &str = "ndim-lsp";

struct NdimLspExtension;

impl NdimLspExtension {
    fn resolve_binary(&self, worktree: &Worktree) -> Result<String> {
        if let Ok(settings) = LspSettings::for_worktree(BINARY_NAME, worktree) {
            if let Some(binary) = settings.binary {
                if let Some(path) = binary.path {
                    if fs::metadata(&path).is_ok_and(|m| m.is_file()) {
                        return Ok(path);
                    }
                    return Err(format!(
                        "configured ndim-lsp binary at `{path}` does not exist"
                    ));
                }
            }
        }

        worktree.which(BINARY_NAME).ok_or_else(|| {
            format!(
                "`{BINARY_NAME}` not found in $PATH. Build it from \
                     https://github.com/Artur-Galstyan/ndim-lsp and either install it on PATH \
                     or set `lsp.{BINARY_NAME}.binary.path` in your Zed settings."
            )
        })
    }
}

impl zed::Extension for NdimLspExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let command = self.resolve_binary(worktree)?;

        let (args, env) = LspSettings::for_worktree(BINARY_NAME, worktree)
            .ok()
            .and_then(|s| s.binary)
            .map(|b| {
                (
                    b.arguments.unwrap_or_default(),
                    b.env
                        .map(|m| m.into_iter().collect::<Vec<(String, String)>>())
                        .unwrap_or_default(),
                )
            })
            .unwrap_or_default();

        Ok(Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(LspSettings::for_worktree(BINARY_NAME, worktree)
            .ok()
            .and_then(|s| s.initialization_options))
    }
}

zed::register_extension!(NdimLspExtension);
