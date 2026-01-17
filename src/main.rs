mod cli;
mod commands;
mod core;
mod help;
mod utils;

use cli::{Command, parse_args};
use serde_json::json;

#[tokio::main]
async fn main() {
    let command = match parse_args() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    let result = execute_command(command).await;

    match result {
        Ok(output) => {
            println!("{}", output);
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("Error: {:?}", error);
            std::process::exit(1);
        }
    }
}

async fn execute_command(command: Command) -> Result<String, String> {
    match command {
        Command::Combined { commands } => {
            let mut results = serde_json::Map::new();

            for (idx, cmd) in commands.into_iter().enumerate() {
                let key = match &cmd {
                    Command::SubscribedItems { .. } => "subscribed-items".to_string(),
                    Command::WorkshopPath { .. } => "workshop-path".to_string(),
                    Command::SearchWorkshop { .. } => format!("search-workshop-{}", idx),
                    Command::WorkshopItems { .. } => format!("workshop-items-{}", idx),
                    Command::CheckItemDownload { .. } => format!("check-item-download-{}", idx),
                    Command::CollectionItems { .. } => format!("collection-items-{}", idx),
                    Command::DiscoverTags { .. } => format!("discover-tags-{}", idx),
                    _ => format!("command-{}", idx),
                };

                match execute_single_command(cmd).await {
                    Ok(output) => {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&output) {
                            results.insert(key, value);
                        } else {
                            results.insert(key, json!(output));
                        }
                    }
                    Err(error) => {
                        results.insert(key, json!({ "error": error }));
                    }
                }
            }

            Ok(serde_json::to_string_pretty(&results).unwrap())
        }
        cmd => execute_single_command(cmd).await,
    }
}

async fn execute_single_command(command: Command) -> Result<String, String> {
    match command {
        Command::CheckItemDownload { app_id, item_id } => {
            commands::check_item_download::check_item_download(app_id, item_id)
                .await
                .map(|info| serde_json::to_string_pretty(&info).unwrap())
        }
        Command::CollectionItems { app_id, item_id } => {
            commands::collection_items::collection_items(app_id, item_id)
                .await
                .map(|items| serde_json::to_string_pretty(&items).unwrap())
        }
        Command::WorkshopItems { app_id, item_ids } => {
            commands::workshop_items::workshop_items(app_id, item_ids)
                .await
                .map(|items| serde_json::to_string_pretty(&items).unwrap())
        }
        Command::Subscribe { app_id, item_ids } => commands::subscribe::subscribe(app_id, item_ids)
            .await
            .map(|results| serde_json::to_string_pretty(&results).unwrap()),
        Command::Unsubscribe { app_id, item_ids } => {
            commands::unsubscribe::unsubscribe(app_id, item_ids)
                .await
                .map(|results| serde_json::to_string_pretty(&results).unwrap())
        }
        Command::DownloadWorkshopItem { app_id, item_id } => {
            commands::download_workshop_item::download_workshop_item(app_id, item_id)
                .await
                .map(|_| "\"Workshop item download completed successfully\"".to_string())
        }
        Command::SubscribedItems { app_id } => commands::subscribed_items::subscribed_items(app_id)
            .await
            .map(|items| serde_json::to_string_pretty(&items).unwrap()),
        Command::SearchWorkshop {
            app_id,
            query,
            sort_by,
            period,
            page,
            tags,
        } => commands::search_workshop::search_workshop(app_id, query, sort_by, period, page, tags)
            .await
            .map(|items| serde_json::to_string_pretty(&items).unwrap()),
        Command::WorkshopPath { app_id } => match commands::workshop_path::workshop_path(app_id) {
            Some(path) => Ok(serde_json::to_string_pretty(&path).unwrap()),
            None => Err(format!("Workshop path not found for app ID {}", app_id)),
        },
        Command::AppInstallationPath { app_id } => {
            commands::app_installation_path::app_installation_path(app_id)
                .map(|path| serde_json::to_string_pretty(&path).unwrap())
        }
        Command::SteamLibraryPaths => commands::steam_library_paths::steam_library_paths()
            .map(|paths| serde_json::to_string_pretty(&paths).unwrap()),
        Command::ClearCache => commands::clear_cache::clear_cache()
            .map(|message| serde_json::to_string_pretty(&message).unwrap()),
        Command::DiscoverTags { app_id } => commands::discover_tags::discover_tags(app_id)
            .await
            .map(|tags| serde_json::to_string_pretty(&tags).unwrap()),
        Command::Combined { .. } => unreachable!("Combined should be handled in execute_command"),
    }
}
