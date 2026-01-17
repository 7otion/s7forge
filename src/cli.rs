use lexopt::prelude::*;

use super::help;

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
    Combined {
        commands: Vec<Command>,
    },
}

pub fn parse_args() -> Result<Command, lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut app_id: Option<u32> = None;

    loop {
        match parser.next()? {
            Some(Long("help") | Short('h')) => {
                help::print_general_help();
                std::process::exit(0);
            }
            Some(Long("version") | Short('v')) => {
                help::print_version();
                std::process::exit(0);
            }
            Some(Long("app-id")) => {
                app_id = Some(parser.value()?.parse()?);
            }
            Some(Value(cmd)) => {
                let cmd_str = cmd.to_string_lossy().to_string();
                return parse_command(&cmd_str, app_id, &mut parser);
            }
            None => {
                help::print_general_help();
                return Err("Missing command".into());
            }
            _ => return Err("Unexpected argument".into()),
        }
    }
}

struct CommandBuilder {
    app_id: Option<u32>,
    item_id: Option<u64>,
    item_ids: Vec<u64>,
    query: String,
    sort_by: String,
    period: Option<String>,
    page: u32,
    tags: Option<String>,
}

impl CommandBuilder {
    fn new(global_app_id: Option<u32>) -> Self {
        Self {
            app_id: global_app_id,
            item_id: None,
            item_ids: Vec::new(),
            query: String::new(),
            sort_by: "relevance".to_string(),
            period: None,
            page: 1,
            tags: None,
        }
    }

    fn parse_item_ids(s: &str) -> Result<Vec<u64>, String> {
        s.split(',')
            .map(|s| {
                s.trim()
                    .parse()
                    .map_err(|_| format!("Invalid item ID: {}", s))
            })
            .collect()
    }
}

fn parse_command(
    command: &str,
    global_app_id: Option<u32>,
    parser: &mut lexopt::Parser,
) -> Result<Command, lexopt::Error> {
    match command {
        "combined" => parse_combined_command(global_app_id, parser),
        "check-item-download" => parse_simple_command(
            parser,
            global_app_id,
            help::print_check_item_help,
            |b, flag, p| {
                match flag {
                    "app-id" => b.app_id = Some(p.value()?.parse()?),
                    "item-id" => b.item_id = Some(p.value()?.parse()?),
                    _ => return Ok(false),
                }
                Ok(true)
            },
            |b| {
                Ok(Command::CheckItemDownload {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                    item_id: b.item_id.ok_or("Missing --item-id")?,
                })
            },
        ),
        "collection-items" => parse_simple_command(
            parser,
            global_app_id,
            help::print_collection_items_help,
            |b, flag, p| {
                match flag {
                    "app-id" => b.app_id = Some(p.value()?.parse()?),
                    "item-id" => b.item_id = Some(p.value()?.parse()?),
                    _ => return Ok(false),
                }
                Ok(true)
            },
            |b| {
                Ok(Command::CollectionItems {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                    item_id: b.item_id.ok_or("Missing --item-id")?,
                })
            },
        ),
        "search-workshop" => parse_simple_command(
            parser,
            global_app_id,
            help::print_search_workshop_help,
            |b, flag, p| {
                match flag {
                    "app-id" => b.app_id = Some(p.value()?.parse()?),
                    "query" => b.query = p.value()?.to_string_lossy().to_string(),
                    "sort-by" => b.sort_by = p.value()?.to_string_lossy().to_string(),
                    "period" => b.period = Some(p.value()?.to_string_lossy().to_string()),
                    "page" => b.page = p.value()?.parse()?,
                    "tags" => b.tags = Some(p.value()?.to_string_lossy().to_string()),
                    _ => return Ok(false),
                }
                Ok(true)
            },
            |b| {
                Ok(Command::SearchWorkshop {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                    query: b.query,
                    sort_by: b.sort_by,
                    period: b.period,
                    page: b.page,
                    tags: b.tags,
                })
            },
        ),
        "workshop-items" => parse_simple_command(
            parser,
            global_app_id,
            help::print_workshop_items_help,
            |b, flag, p| {
                match flag {
                    "app-id" => b.app_id = Some(p.value()?.parse()?),
                    "item-ids" => {
                        let ids_str = p.value()?.to_string_lossy().to_string();
                        b.item_ids = CommandBuilder::parse_item_ids(&ids_str)?;
                    }
                    _ => return Ok(false),
                }
                Ok(true)
            },
            |b| {
                Ok(Command::WorkshopItems {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                    item_ids: b.item_ids,
                })
            },
        ),
        "subscribe" => parse_simple_command(
            parser,
            global_app_id,
            help::print_subscribe_help,
            |b, flag, p| {
                match flag {
                    "app-id" => b.app_id = Some(p.value()?.parse()?),
                    "item-ids" => {
                        let ids_str = p.value()?.to_string_lossy().to_string();
                        b.item_ids = CommandBuilder::parse_item_ids(&ids_str)?;
                    }
                    _ => return Ok(false),
                }
                Ok(true)
            },
            |b| {
                Ok(Command::Subscribe {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                    item_ids: b.item_ids,
                })
            },
        ),
        "unsubscribe" => parse_simple_command(
            parser,
            global_app_id,
            help::print_unsubscribe_help,
            |b, flag, p| {
                match flag {
                    "app-id" => b.app_id = Some(p.value()?.parse()?),
                    "item-ids" => {
                        let ids_str = p.value()?.to_string_lossy().to_string();
                        b.item_ids = CommandBuilder::parse_item_ids(&ids_str)?;
                    }
                    _ => return Ok(false),
                }
                Ok(true)
            },
            |b| {
                Ok(Command::Unsubscribe {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                    item_ids: b.item_ids,
                })
            },
        ),
        "download-workshop-item" => parse_simple_command(
            parser,
            global_app_id,
            help::print_download_workshop_item_help,
            |b, flag, p| {
                match flag {
                    "app-id" => b.app_id = Some(p.value()?.parse()?),
                    "item-id" => b.item_id = Some(p.value()?.parse()?),
                    _ => return Ok(false),
                }
                Ok(true)
            },
            |b| {
                Ok(Command::DownloadWorkshopItem {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                    item_id: b.item_id.ok_or("Missing --item-id")?,
                })
            },
        ),
        "subscribed-items" => parse_no_arg_command(
            parser,
            global_app_id,
            help::print_subscribed_items_help,
            |b| {
                Ok(Command::SubscribedItems {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                })
            },
        ),
        "workshop-path" => {
            parse_no_arg_command(parser, global_app_id, help::print_workshop_path_help, |b| {
                Ok(Command::WorkshopPath {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                })
            })
        }
        "discover-tags" => {
            parse_no_arg_command(parser, global_app_id, help::print_discover_tags_help, |b| {
                Ok(Command::DiscoverTags {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                })
            })
        }
        "app-installation-path" => parse_no_arg_command(
            parser,
            global_app_id,
            help::print_app_installation_path_help,
            |b| {
                Ok(Command::AppInstallationPath {
                    app_id: b.app_id.ok_or("Missing --app-id")?,
                })
            },
        ),
        "clear-cache" => {
            if let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        help::print_clear_cache_help();
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
                        help::print_steam_library_paths_help();
                        std::process::exit(0);
                    }
                    _ => return Err(arg.unexpected()),
                }
            }
            Ok(Command::SteamLibraryPaths)
        }
        "help" | "--help" | "-h" => {
            help::print_main_help();
            std::process::exit(0);
        }
        _ => Err(format!("Unknown command: {}", command).into()),
    }
}

// Helper for commands with only --app-id
fn parse_no_arg_command<F>(
    parser: &mut lexopt::Parser,
    global_app_id: Option<u32>,
    help_fn: fn(),
    build_fn: F,
) -> Result<Command, lexopt::Error>
where
    F: FnOnce(CommandBuilder) -> Result<Command, lexopt::Error>,
{
    let mut builder = CommandBuilder::new(global_app_id);

    while let Some(arg) = parser.next()? {
        match arg {
            Long("app-id") => {
                let val = parser.value()?;
                builder.app_id = Some(val.parse()?);
            }
            Long("help") | Short('h') => {
                help_fn();
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    build_fn(builder)
}

fn parse_simple_command<F, G>(
    parser: &mut lexopt::Parser,
    global_app_id: Option<u32>,
    help_fn: fn(),
    mut parse_arg: F,
    build_fn: G,
) -> Result<Command, lexopt::Error>
where
    F: FnMut(&mut CommandBuilder, &str, &mut lexopt::Parser) -> Result<bool, lexopt::Error>,
    G: FnOnce(CommandBuilder) -> Result<Command, lexopt::Error>,
{
    let mut builder = CommandBuilder::new(global_app_id);

    while let Some(arg) = parser.next()? {
        match arg {
            Long("help") | Short('h') => {
                help_fn();
                std::process::exit(0);
            }
            Long(flag) => {
                let flag = flag.to_string();
                if !parse_arg(&mut builder, &flag, parser)? {
                    return Err(format!("Unknown option: --{}", flag).into());
                }
            }
            Short(flag) => {
                return Err(format!("Unknown option: -{}", flag).into());
            }
            Value(val) => {
                return Err(format!("Unexpected value: {}", val.to_string_lossy()).into());
            }
        }
    }

    build_fn(builder)
}

fn parse_combined_command(
    global_app_id: Option<u32>,
    parser: &mut lexopt::Parser,
) -> Result<Command, lexopt::Error> {
    let app_id = global_app_id.ok_or("--app-id required for combined command")?;

    const KNOWN_COMMANDS: &[&str] = &[
        "subscribed-items",
        "workshop-path",
        "search-workshop",
        "workshop-items",
        "check-item-download",
        "collection-items",
        "discover-tags",
    ];

    let mut command_blocks: Vec<(String, Vec<std::ffi::OsString>)> = Vec::new();
    let mut current_command: Option<String> = None;
    let mut current_args: Vec<std::ffi::OsString> = Vec::new();

    loop {
        match parser.next()? {
            Some(Long("help") | Short('h')) => {
                help::print_combined_help();
                std::process::exit(0);
            }
            Some(Long(flag)) => {
                if KNOWN_COMMANDS.contains(&flag) {
                    if let Some(cmd) = current_command.take() {
                        command_blocks.push((cmd, std::mem::take(&mut current_args)));
                    }
                    current_command = Some(flag.to_string());
                } else {
                    current_args.push(format!("--{}", flag).into());
                }
            }
            Some(Short(flag)) => current_args.push(format!("-{}", flag).into()),
            Some(Value(v)) => current_args.push(v),
            None => break,
        }
    }

    if let Some(cmd) = current_command {
        command_blocks.push((cmd, current_args));
    }

    if command_blocks.is_empty() {
        return Err("No subcommands specified for combined".into());
    }

    let commands = command_blocks
        .into_iter()
        .map(|(cmd_name, args)| parse_combined_subcommand(&cmd_name, app_id, args))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Command::Combined { commands })
}

fn parse_combined_subcommand(
    command: &str,
    app_id: u32,
    args: Vec<std::ffi::OsString>,
) -> Result<Command, lexopt::Error> {
    let mut iter = args.into_iter();
    let mut builder = CommandBuilder::new(Some(app_id));

    match command {
        "subscribed-items" => Ok(Command::SubscribedItems { app_id }),
        "workshop-path" => Ok(Command::WorkshopPath { app_id }),
        "discover-tags" => Ok(Command::DiscoverTags { app_id }),
        "search-workshop" => {
            while let Some(arg) = iter.next() {
                parse_arg_from_os(
                    &mut builder,
                    &arg,
                    &mut iter,
                    &[
                        ("--query", |b, v| b.query = v),
                        ("--sort-by", |b, v| b.sort_by = v),
                        ("--period", |b, v| b.period = Some(v)),
                        ("--tags", |b, v| b.tags = Some(v)),
                    ],
                    &[("--page", |b, v| {
                        b.page = v.parse().map_err(|_| "Invalid page")?;
                        Ok(())
                    })],
                )?;
            }
            Ok(Command::SearchWorkshop {
                app_id,
                query: builder.query,
                sort_by: builder.sort_by,
                period: builder.period,
                page: builder.page,
                tags: builder.tags,
            })
        }
        "workshop-items" => {
            while let Some(arg) = iter.next() {
                if arg.to_string_lossy() == "--item-ids" {
                    if let Some(val) = iter.next() {
                        builder.item_ids = CommandBuilder::parse_item_ids(&val.to_string_lossy())?;
                    }
                } else {
                    return Err(format!("Unexpected argument: {}", arg.to_string_lossy()).into());
                }
            }
            Ok(Command::WorkshopItems {
                app_id,
                item_ids: builder.item_ids,
            })
        }
        "check-item-download" | "collection-items" => {
            while let Some(arg) = iter.next() {
                if arg.to_string_lossy() == "--item-id" {
                    if let Some(val) = iter.next() {
                        builder.item_id = Some(
                            val.to_string_lossy()
                                .parse()
                                .map_err(|_| "Invalid item-id")?,
                        );
                    }
                } else {
                    return Err(format!("Unexpected argument: {}", arg.to_string_lossy()).into());
                }
            }
            let item_id = builder.item_id.ok_or("Missing --item-id")?;
            if command == "check-item-download" {
                Ok(Command::CheckItemDownload { app_id, item_id })
            } else {
                Ok(Command::CollectionItems { app_id, item_id })
            }
        }
        _ => Err(format!("Unknown subcommand: {}", command).into()),
    }
}

fn parse_arg_from_os<I>(
    builder: &mut CommandBuilder,
    arg: &std::ffi::OsString,
    iter: &mut I,
    string_args: &[(&str, fn(&mut CommandBuilder, String))],
    parse_args: &[(
        &str,
        fn(&mut CommandBuilder, String) -> Result<(), &'static str>,
    )],
) -> Result<(), lexopt::Error>
where
    I: Iterator<Item = std::ffi::OsString>,
{
    let arg_str = arg.to_string_lossy();

    for (flag, handler) in string_args {
        if arg_str == *flag {
            let val = iter
                .next()
                .ok_or_else(|| format!("Missing value for {}", flag))?;
            handler(builder, val.to_string_lossy().to_string());
            return Ok(());
        }
    }

    for (flag, handler) in parse_args {
        if arg_str == *flag {
            let val = iter
                .next()
                .ok_or_else(|| format!("Missing value for {}", flag))?;
            handler(builder, val.to_string_lossy().to_string())?;
            return Ok(());
        }
    }

    Err(format!("Unexpected argument: {}", arg_str).into())
}
