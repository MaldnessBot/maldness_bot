use std::env;
use std::thread::sleep;
use std::time::Duration;

use frankenstein::{Api, GetUpdatesParams, TelegramApi, Update};

use commands::{up, CommandExecutor};
use errors::HandleUpdateError;

mod commands;
mod errors;

const BOT_COMMAND: &str = "bot_command";

fn handle_updates(
    updates: Vec<Update>,
    executor: &CommandExecutor,
) -> Result<Option<u32>, HandleUpdateError> {
    let mut last_update_id: Option<u32> = None;

    for update in updates {
        let message = update.message.as_ref().ok_or(HandleUpdateError::Skip)?;
        let text = message.text.as_ref().ok_or(HandleUpdateError::Skip)?;

        for entity in message.entities.as_ref().ok_or(HandleUpdateError::Skip)? {
            if entity.type_field.as_str() != BOT_COMMAND {
                continue;
            }

            let text_str = text.as_str();
            let offset = entity.offset as usize;
            let length = entity.length as usize;
            let command = &text_str[offset..length];
            executor.execute_command(&update, command, &text_str[length..]);
        }

        last_update_id = Some(update.update_id + 1);
    }

    Ok(last_update_id)
}

fn main() {
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN is not defined");
    let api = Api::new(token.as_str());

    let mut executor = CommandExecutor::new(&api);
    executor.register_command("up".to_string(), up::up);

    let mut update_params = GetUpdatesParams::new();
    update_params.set_allowed_updates(Some(vec!["message".to_string()]));

    loop {
        let result = api.get_updates(&update_params);
        match result {
            Ok(response) => match handle_updates(response.result, &executor) {
                Ok(last_update_id) => update_params.set_offset(last_update_id),
                Err(error) => println!("Error: {:?}", error),
            },
            Err(error) => println!("Error: {:?}", error),
        };
        sleep(Duration::new(3, 0));
    }
}
