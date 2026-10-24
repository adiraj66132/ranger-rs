use mlua::{Function, Lua, RegistryKey, Table};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub enum Action {
    Cd(PathBuf),
    Notify(String),
    Quit,
    Reload,
    ToggleHidden,
    Mkdir(String),
    Touch(String),
}

pub struct PluginHost {
    lua: Lua,
    commands: HashMap<String, RegistryKey>,
    keys: HashMap<char, String>,
}

impl PluginHost {
    pub fn load(
        dir: &Path,
        actions: Arc<Mutex<Vec<Action>>>,
        cwd: Arc<Mutex<PathBuf>>,
    ) -> (Option<Self>, Vec<String>) {
        let mut files = Vec::new();
        if let Ok(read) = std::fs::read_dir(dir) {
            for item in read.flatten() {
                let path = item.path();
                if path.extension().is_some_and(|ext| ext == "lua") {
                    files.push(path);
                }
            }
        }
        if files.is_empty() {
            return (None, Vec::new());
        }
        files.sort();

        let lua = Lua::new();
        let mut warnings = Vec::new();
        let (command_table, key_table) = match setup(&lua, actions, cwd) {
            Ok(tables) => tables,
            Err(err) => return (None, vec![format!("plugin api: {err}")]),
        };

        for path in files {
            let name = path.to_string_lossy().into_owned();
            match std::fs::read_to_string(&path) {
                Ok(source) => {
                    if let Err(err) = lua.load(&source).set_name(&name).exec() {
                        warnings.push(format!("{name}: {err}"));
                    }
                }
                Err(err) => warnings.push(format!("{name}: {err}")),
            }
        }

        let mut commands = HashMap::new();
        for pair in command_table.pairs::<String, Function>() {
            match pair {
                Ok((name, function)) => match lua.create_registry_value(function) {
                    Ok(key) => {
                        commands.insert(name, key);
                    }
                    Err(err) => warnings.push(format!("plugin command {name}: {err}")),
                },
                Err(err) => warnings.push(format!("plugin commands: {err}")),
            }
        }

        let mut keys = HashMap::new();
        for pair in key_table.pairs::<String, String>() {
            match pair {
                Ok((key, command)) => {
                    if let Some(ch) = key.chars().next() {
                        keys.insert(ch, command);
                    } else {
                        warnings.push("plugin key: empty key".into());
                    }
                }
                Err(err) => warnings.push(format!("plugin keys: {err}")),
            }
        }

        (
            Some(Self {
                lua,
                commands,
                keys,
            }),
            warnings,
        )
    }

    pub fn key_command(&self, key: char) -> Option<&str> {
        self.keys.get(&key).map(String::as_str)
    }

    pub fn run_command(&self, name: &str, args: &str) -> Result<(), String> {
        let Some(key) = self.commands.get(name) else {
            return Err(format!("no such command: {name}"));
        };
        let function: Function = self
            .lua
            .registry_value(key)
            .map_err(|err| err.to_string())?;
        function.call::<_, ()>(args).map_err(|err| err.to_string())
    }
}

fn setup<'lua>(
    lua: &'lua Lua,
    actions: Arc<Mutex<Vec<Action>>>,
    cwd: Arc<Mutex<PathBuf>>,
) -> mlua::Result<(Table<'lua>, Table<'lua>)> {
    let commands = lua.create_table()?;
    let keys = lua.create_table()?;
    lua.globals()
        .set("__ranger_rs_commands", commands.clone())?;
    lua.globals().set("__ranger_rs_keys", keys.clone())?;

    let api = lua.create_table()?;

    let queue = actions.clone();
    api.set(
        "notify",
        lua.create_function(move |_, message: String| {
            queue.lock().unwrap().push(Action::Notify(message));
            Ok(())
        })?,
    )?;

    let queue = actions.clone();
    api.set(
        "cd",
        lua.create_function(move |_, path: String| {
            queue.lock().unwrap().push(Action::Cd(PathBuf::from(path)));
            Ok(())
        })?,
    )?;

    let queue = actions.clone();
    api.set(
        "quit",
        lua.create_function(move |_, ()| {
            queue.lock().unwrap().push(Action::Quit);
            Ok(())
        })?,
    )?;

    let queue = actions.clone();
    api.set(
        "reload",
        lua.create_function(move |_, ()| {
            queue.lock().unwrap().push(Action::Reload);
            Ok(())
        })?,
    )?;

    let queue = actions.clone();
    api.set(
        "toggle_hidden",
        lua.create_function(move |_, ()| {
            queue.lock().unwrap().push(Action::ToggleHidden);
            Ok(())
        })?,
    )?;

    let queue = actions.clone();
    api.set(
        "mkdir",
        lua.create_function(move |_, path: String| {
            queue.lock().unwrap().push(Action::Mkdir(path));
            Ok(())
        })?,
    )?;

    let queue = actions.clone();
    api.set(
        "touch",
        lua.create_function(move |_, path: String| {
            queue.lock().unwrap().push(Action::Touch(path));
            Ok(())
        })?,
    )?;

    let current = cwd.clone();
    api.set(
        "cwd",
        lua.create_function(move |_, ()| {
            Ok(current.lock().unwrap().to_string_lossy().into_owned())
        })?,
    )?;

    api.set(
        "register_command",
        lua.create_function(|lua, (name, function): (String, Function)| {
            let table: Table = lua.globals().get("__ranger_rs_commands")?;
            table.set(name, function)?;
            Ok(())
        })?,
    )?;

    api.set(
        "register_key",
        lua.create_function(|lua, (key, command): (String, String)| {
            let table: Table = lua.globals().get("__ranger_rs_keys")?;
            table.set(key, command)?;
            Ok(())
        })?,
    )?;

    lua.globals().set("ranger", api)?;
    Ok((commands, keys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_commands_keys_and_actions() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ranger-rs-plugin-{stamp}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("hello.lua"),
            r#"
                ranger.register_command("hello", function(args)
                    ranger.notify("hi " .. args)
                end)
                ranger.register_key("H", "hello")
            "#,
        )
        .unwrap();

        let actions = Arc::new(Mutex::new(Vec::new()));
        let cwd = Arc::new(Mutex::new(PathBuf::from("/tmp")));
        let (host, warnings) = PluginHost::load(&dir, actions.clone(), cwd);
        assert!(warnings.is_empty(), "{warnings:?}");
        let host = host.unwrap();
        assert_eq!(host.key_command('H'), Some("hello"));
        host.run_command("hello", "there").unwrap();
        assert!(matches!(
            &actions.lock().unwrap()[0],
            Action::Notify(message) if message == "hi there"
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
