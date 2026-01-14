mod cli;
mod commands;
mod core;
mod utils;

use cli::{Command, parse_args};

#[tokio::main]
async fn main() {
    let command = match parse_args() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    let result = match command {
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
    };

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
