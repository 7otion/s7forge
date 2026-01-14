#[derive(Debug)]
pub enum Command {
    CheckItemDownload {
        app_id: u32,
        item_id: u64,
    },
    CollectionItems {
        app_id: u32,
        item_id: u64,
    },
    WorkshopItems {
        app_id: u32,
        item_ids: Vec<u64>,
    },
    Subscribe {
        app_id: u32,
        item_ids: Vec<u64>,
    },
    Unsubscribe {
        app_id: u32,
        item_ids: Vec<u64>,
    },
    DownloadWorkshopItem {
        app_id: u32,
        item_id: u64,
    },
    SubscribedItems {
        app_id: u32,
    },
    SearchWorkshop {
        app_id: u32,
        query: String,
        sort_by: String,
        period: Option<String>,
        page: u32,
        tags: Option<String>,
    },
    WorkshopPath {
        app_id: u32,
    },
    AppInstallationPath {
        app_id: u32,
    },
    SteamLibraryPaths,
    ClearCache,
    DiscoverTags {
        app_id: u32,
    },
}

pub fn parse_args() -> Result<Command, lexopt::Error> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();

    // Handle global --help and --version before command parsing
    let command = match parser.next()? {
        Some(Long("help") | Short('h')) => {
            print_general_help();
            std::process::exit(0);
        }
        Some(Long("version") | Short('v')) => {
            print_version();
            std::process::exit(0);
        }
        Some(Value(cmd)) => cmd.to_string_lossy().to_string(),
        _ => {
            print_general_help();
            return Err("Missing command".into());
        }
    };

    match command.as_str() {
        "check-item-download" => {
            let mut app_id = None;
            let mut item_id = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("item-id") => item_id = Some(parser.value()?.parse()?),
                    Long("help") | Short('h') => {
                        print_check_item_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::CheckItemDownload {
                app_id: app_id.ok_or("Missing --app-id")?,
                item_id: item_id.ok_or("Missing --item-id")?,
            })
        }

        "collection-items" => {
            let mut app_id = None;
            let mut item_id = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("item-id") => item_id = Some(parser.value()?.parse()?),
                    Long("help") | Short('h') => {
                        print_collection_items_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::CollectionItems {
                app_id: app_id.ok_or("Missing --app-id")?,
                item_id: item_id.ok_or("Missing --item-id")?,
            })
        }

        "search-workshop" => {
            let mut app_id = None;
            let mut query = String::new();
            let mut sort_by = "relevance".to_string();
            let mut period = None;
            let mut page = 1;
            let mut tags = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("query") => query = parser.value()?.to_string_lossy().to_string(),
                    Long("sort-by") => sort_by = parser.value()?.to_string_lossy().to_string(),
                    Long("period") => period = Some(parser.value()?.to_string_lossy().to_string()),
                    Long("page") => page = parser.value()?.parse()?,
                    Long("tags") => tags = Some(parser.value()?.to_string_lossy().to_string()),
                    Long("help") | Short('h') => {
                        print_search_workshop_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::SearchWorkshop {
                app_id: app_id.ok_or("Missing --app-id")?,
                query,
                sort_by,
                period,
                page,
                tags,
            })
        }

        "clear-cache" => {
            if let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        print_clear_cache_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }
            Ok(Command::ClearCache)
        }

        "steam-library-paths" => {
            if let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        print_steam_library_paths_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }
            Ok(Command::SteamLibraryPaths)
        }

        "workshop-items" => {
            let mut app_id = None;
            let mut item_ids = Vec::new();

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("item-ids") => {
                        let ids_value = parser.value()?;
                        let ids_str = ids_value.to_string_lossy();
                        item_ids = ids_str
                            .split(',')
                            .map(|s| {
                                s.trim()
                                    .parse()
                                    .map_err(|_| format!("Invalid item ID: {}", s))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                    }
                    Long("help") | Short('h') => {
                        print_workshop_items_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::WorkshopItems {
                app_id: app_id.ok_or("Missing --app-id")?,
                item_ids,
            })
        }

        "subscribe" => {
            let mut app_id = None;
            let mut item_ids = Vec::new();

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("item-ids") => {
                        let ids_value = parser.value()?;
                        let ids_str = ids_value.to_string_lossy();
                        item_ids = ids_str
                            .split(',')
                            .map(|s| {
                                s.trim()
                                    .parse()
                                    .map_err(|_| format!("Invalid item ID: {}", s))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                    }
                    Long("help") | Short('h') => {
                        print_subscribe_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::Subscribe {
                app_id: app_id.ok_or("Missing --app-id")?,
                item_ids,
            })
        }

        "unsubscribe" => {
            let mut app_id = None;
            let mut item_ids = Vec::new();

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("item-ids") => {
                        let ids_value = parser.value()?;
                        let ids_str = ids_value.to_string_lossy();
                        item_ids = ids_str
                            .split(',')
                            .map(|s| {
                                s.trim()
                                    .parse()
                                    .map_err(|_| format!("Invalid item ID: {}", s))
                            })
                            .collect::<Result<Vec<u64>, String>>()?;
                    }
                    Long("help") | Short('h') => {
                        print_unsubscribe_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::Unsubscribe {
                app_id: app_id.ok_or("Missing --app-id")?,
                item_ids,
            })
        }

        "download-workshop-item" => {
            let mut app_id = None;
            let mut item_id = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("item-id") => item_id = Some(parser.value()?.parse()?),
                    Long("help") | Short('h') => {
                        print_download_workshop_item_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::DownloadWorkshopItem {
                app_id: app_id.ok_or("Missing --app-id")?,
                item_id: item_id.ok_or("Missing --item-id")?,
            })
        }

        "subscribed-items" => {
            let mut app_id = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("help") | Short('h') => {
                        print_subscribed_items_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::SubscribedItems {
                app_id: app_id.ok_or("Missing --app-id")?,
            })
        }

        "workshop-path" => {
            let mut app_id = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("help") | Short('h') => {
                        print_workshop_path_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::WorkshopPath {
                app_id: app_id.ok_or("Missing --app-id")?,
            })
        }

        "discover-tags" => {
            let mut app_id = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("help") | Short('h') => {
                        print_discover_tags_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::DiscoverTags {
                app_id: app_id.ok_or("Missing --app-id")?,
            })
        }

        "app-installation-path" => {
            let mut app_id = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("app-id") => app_id = Some(parser.value()?.parse()?),
                    Long("help") | Short('h') => {
                        print_app_installation_path_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }

            Ok(Command::AppInstallationPath {
                app_id: app_id.ok_or("Missing --app-id")?,
            })
        }

        "help" | "--help" | "-h" => {
            print_main_help();
            std::process::exit(0);
        }

        _ => Err(format!("Unknown command: {}", command).into()),
    }
}

fn print_main_help() {
    println!("s7forge - Steam utility\n");
    println!("USAGE:");
    println!("    s7forge <COMMAND>\n");
    println!("COMMANDS:");
    println!("    check-item-download     Check download status of a workshop item");
    println!("    collection-items        Get items from a workshop collection");
    println!("    workshop-items          Get detailed information about workshop items");
    println!("    subscribe               Subscribe to workshop items");
    println!("    unsubscribe             Unsubscribe from workshop items");
    println!("    download-workshop-item  Download a workshop item you own");
    println!("    subscribed-items        List all items you're subscribed to for a game");
    println!("    search-workshop         Search workshop content by text query");
    println!("    workshop-path           Get the local workshop path for a game");
    println!("    app-installation-path   Get the installation path for a Steam app");
    println!("    steam-library-paths     List all Steam library folder paths");
    println!("    clear-cache             Clear all cached data");
    println!("    discover-tags           Discover all available workshop tags for a game");
    println!("    help                    Print this message\n");
    println!("For more information on a specific command, use: s7forge <COMMAND> --help");
}

fn print_check_item_help() {
    println!("Check download status of a workshop item\n");
    println!("USAGE:");
    println!("    s7forge check-item-download --app-id <APP_ID> --item-id <ITEM_ID>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>      Steam App ID of the game");
    println!("    --item-id <ITEM_ID>    Workshop item ID to check download status for");
    println!("    -h, --help             Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge check-item-download --app-id 548430 --item-id 123456789");
}

fn print_collection_items_help() {
    println!("Get items from a workshop collection\n");
    println!("USAGE:");
    println!("    s7forge collection-items --app-id <APP_ID> --item-id <ITEM_ID>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>      Steam App ID of the game");
    println!("    --item-id <ITEM_ID>    Collection ID to get items from");
    println!("    -h, --help             Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge collection-items --app-id 548430 --item-id 987654321");
}

fn print_search_workshop_help() {
    println!("Search workshop content by text query with flexible sorting options\n");
    println!("USAGE:");
    println!("    s7forge search-workshop --app-id <APP_ID> [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>        Steam App ID of the game");
    println!("    --query <QUERY>          Text to search for (optional for most sort methods)");
    println!(
        "    --sort-by <SORT>         Sort by: relevance, recent, popular, most-subscribed, recently-updated [default: relevance]"
    );
    println!(
        "    --period <PERIOD>        Time period filter: today, one-week, three-months, six-months, one-year (only for 'popular' sort)"
    );
    println!("    --page <PAGE>            Page number for pagination [default: 1]");
    println!("    --tags <TAGS>            Filter by tags, comma-separated (e.g., 'mod,weapon')");
    println!("    -h, --help               Print help\n");
    println!("EXAMPLES:");
    println!("    s7forge search-workshop --app-id 548430 --query \"tank\" --sort-by relevance");
    println!("    s7forge search-workshop --app-id 548430 --sort-by recent --tags \"mod,weapon\"");
    println!("    s7forge search-workshop --app-id 548430 --sort-by popular --period one-week");
}

fn print_clear_cache_help() {
    println!("Clear all cached data (creator names, workshop items)\n");
    println!("USAGE:");
    println!("    s7forge clear-cache\n");
    println!("OPTIONS:");
    println!("    -h, --help    Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge clear-cache");
}

fn print_steam_library_paths_help() {
    println!("List all Steam library folder paths\n");
    println!("USAGE:");
    println!("    s7forge steam-library-paths\n");
    println!("OPTIONS:");
    println!("    -h, --help    Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge steam-library-paths");
}

fn print_workshop_items_help() {
    println!("Get detailed information about workshop items\n");
    println!("USAGE:");
    println!("    s7forge workshop-items --app-id <APP_ID> --item-ids <ITEM_IDS>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>          Steam App ID of the game");
    println!("    --item-ids <ITEM_IDS>      Workshop item IDs (comma-separated)");
    println!("    -h, --help                 Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge workshop-items --app-id 548430 --item-ids 123,456,789");
}

fn print_subscribe_help() {
    println!("Subscribe to workshop items\n");
    println!("USAGE:");
    println!("    s7forge subscribe --app-id <APP_ID> --item-ids <ITEM_IDS>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>          Steam App ID of the game");
    println!("    --item-ids <ITEM_IDS>      Workshop item IDs to subscribe to (comma-separated)");
    println!("    -h, --help                 Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge subscribe --app-id 548430 --item-ids 123,456,789");
}

fn print_unsubscribe_help() {
    println!("Unsubscribe from workshop items\n");
    println!("USAGE:");
    println!("    s7forge unsubscribe --app-id <APP_ID> --item-ids <ITEM_IDS>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>          Steam App ID of the game");
    println!(
        "    --item-ids <ITEM_IDS>      Workshop item IDs to unsubscribe from (comma-separated)"
    );
    println!("    -h, --help                 Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge unsubscribe --app-id 548430 --item-ids 123,456,789");
}

fn print_download_workshop_item_help() {
    println!("Download a workshop item you own\n");
    println!("USAGE:");
    println!("    s7forge download-workshop-item --app-id <APP_ID> --item-id <ITEM_ID>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>      Steam App ID of the game");
    println!("    --item-id <ITEM_ID>    Workshop item ID to download");
    println!("    -h, --help             Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge download-workshop-item --app-id 548430 --item-id 123456789");
}

fn print_subscribed_items_help() {
    println!("List all items you're subscribed to for a game\n");
    println!("USAGE:");
    println!("    s7forge subscribed-items --app-id <APP_ID>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>      Steam App ID of the game");
    println!("    -h, --help             Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge subscribed-items --app-id 548430");
}

fn print_workshop_path_help() {
    println!("Get the local workshop path for a game\n");
    println!("USAGE:");
    println!("    s7forge workshop-path --app-id <APP_ID>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>      Steam App ID of the game");
    println!("    -h, --help             Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge workshop-path --app-id 548430");
}

fn print_discover_tags_help() {
    println!("Discover all available workshop tags for a game\n");
    println!("USAGE:");
    println!("    s7forge discover-tags --app-id <APP_ID>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>      Steam App ID of the game");
    println!("    -h, --help             Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge discover-tags --app-id 548430");
}

fn print_app_installation_path_help() {
    println!("Get the installation path for a Steam app\n");
    println!("USAGE:");
    println!("    s7forge app-installation-path --app-id <APP_ID>\n");
    println!("OPTIONS:");
    println!("    --app-id <APP_ID>      Steam App ID of the game");
    println!("    -h, --help             Print help\n");
    println!("EXAMPLE:");
    println!("    s7forge app-installation-path --app-id 548430");
}

fn print_general_help() {
    println!("s7forge - Steam utility for managing workshop content and Steam app data\n");
    println!("USAGE:");
    println!("    s7forge <COMMAND> [OPTIONS]\n");
    println!("COMMANDS:");
    println!("    search-workshop          Search for workshop items");
    println!("    discover-tags            Discover available workshop tags for a game");
    println!("    workshop-items           Get details about workshop items");
    println!("    collection-items         Get items from a workshop collection");
    println!("    subscribed-items         List all items you're subscribed to");
    println!("    check-item-download      Check if a workshop item is downloaded");
    println!("    subscribe                Subscribe to workshop items");
    println!("    unsubscribe              Unsubscribe from workshop items");
    println!("    download-workshop-item   Download a workshop item you own");
    println!("    clear-cache              Clear the Steam workshop cache");
    println!("    workshop-path            Get the local workshop path for a game");
    println!("    steam-library-paths      List all Steam library paths");
    println!("    app-installation-path    Get the installation path for a Steam app\n");
    println!("OPTIONS:");
    println!("    -h, --help               Print help");
    println!("    -v, --version            Print version\n");
    println!("Use 's7forge <COMMAND> --help' for more information on a specific command.");
}

fn print_version() {
    println!("s7forge {}", env!("CARGO_PKG_VERSION"));
}
