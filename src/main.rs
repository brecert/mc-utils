use chrono::{TimeZone, Utc};
use owo_colors::OwoColorize;
use player::UsernameHistory;
use server::ServerPing;
use structopt::StructOpt;

use std::borrow::Cow;

mod api;
mod util;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn username_or_id(idname: &str, at: Option<i64>) -> Cow<str> {
    let timestamp = at.unwrap_or_else(|| chrono::Utc::now().timestamp());
    api::get_id_at(idname, timestamp)
        .map(Cow::from)
        .unwrap_or(Cow::Borrowed(idname))
}

fn find_uuid(username: String, at: Option<i64>) -> Result<()> {
    let timestamp = at.unwrap_or_else(|| chrono::Utc::now().timestamp());
    match api::get_id_at(&username, timestamp) {
        Ok(uuid) => {
            println!("{}", uuid);
        }
        Err(_) => {
            println!(
                "{}{} invalid username.",
                "error".bold().bright_red(),
                ":".bold()
            );
            std::process::exit(1)
        }
    };
    Ok(())
}

fn username_history(user: String, disable_date: bool, at: Option<i64>) -> Result<()> {
    let id = username_or_id(&user, at);
    let history = match api::get_username_history(&id) {
        Ok(val) => val,
        Err(_) => {
            println!(
                "{}{} invalid username or uuid",
                "error".bold().bright_red(),
                ":".bold()
            );
            std::process::exit(1)
        }
    };

    let mut entries = history.iter();
    let mut previous_name = &entries.next().unwrap().name;

    println!("Original name: {}", previous_name.bright_green());
    match history.len() {
        0 => println!("No name changes."),
        _ => {
            println!("Name changed {} times.", history.len() - 1);

            for entry in entries {
                let date = if disable_date {
                    String::new()
                } else {
                    format!(
                        "[{}] ",
                        Utc.timestamp_millis(entry.changed_to_at.unwrap())
                            .format("%x")
                    )
                };

                println!(
                    "{}Name changed from {} to {}",
                    date.bright_cyan(),
                    previous_name.yellow(),
                    &entry.name.green()
                );
                previous_name = &entry.name;
            }
        }
    }
    Ok(())
}

mod skin {
    use super::*;

    #[derive(StructOpt, Debug)]
    pub enum Opt {
        #[structopt(name = "json")]
        /// fetches the skin json that's typically base64 encoded
        FetchSkinData {
            /// username or user id
            user: String,
        },
        #[structopt(name = "info")]
        /// displays information about the skin
        ShowSkinInfo {
            /// username or user id
            user: String,
        },
    }

    fn get_skin_data(user: &str) -> Result<api::Textures> {
        let id = username_or_id(user, None);
        let skin = api::get_skin(&id)?;
        let tex = skin.properties[0].textures()?.textures;
        Ok(tex)
    }

    pub fn display_json(user: &str) -> Result<()> {
        let id = username_or_id(user, None);
        println!("{:?}", get_skin_data(&id)?);
        Ok(())
    }

    // todo(bree): figure out java timestamp
    pub fn display_info(user: &str) -> Result<()> {
        let id = username_or_id(user, None);
        let skin = get_skin_data(&id)?;
        println!(
            "skin_type: {}",
            skin.skin
                .metadata
                .map(|data| std::borrow::Cow::Owned(data.model))
                .unwrap_or(std::borrow::Cow::Borrowed("classic"))
        );
        println!("skin_url: {}", skin.skin.url.bright_cyan());
        if let Some(cape) = skin.cape {
            println!("has_cape: {}", "true".bright_green());
            println!("cape_url: {}", cape.url.bright_cyan());
        } else {
            println!("has_cape: {}", "false".bright_red());
        }

        Ok(())
    }
}

mod server {
    use super::*;
    use util::tty_style_chat;

    #[derive(StructOpt, Debug)]
    /// get a minecraft server's response
    pub struct ServerPing {
        pub host: String,
        pub port: Option<u16>,
    }

    #[derive(StructOpt, Debug)]
    pub enum Opt {
        #[structopt(name = "ping")]
        Ping(ServerPing),

        #[structopt(name = "blocked")]
        /// checks if a server is blocked by mojang
        ServerBlocked { addr: String },
    }

    pub fn ping_server(host: &str, port: u16) -> Result<()> {
        match craftping::sync::ping(host, port) {
            Err(err) => println!("{}{} {}", "error".bold().red(), ":".bold(), err),
            Ok(res) => {
                println!(
                    "[{}/{}] {}:{}",
                    res.online_players, res.max_players, host, port
                );
                println!("{}", tty_style_chat(&res.description));
            }
        };

        Ok(())
    }

    pub fn is_server_blocked(addr: &str) -> Result<()> {
        if let Some(pat) = api::find_blocked_pattern(addr)? {
            println!(
                "{} is blocked because of the pattern `{}`",
                addr.red().bold(),
                pat.cyan().bold()
            );
        } else {
            println!("{} is not blocked", addr.bright_green())
        }

        Ok(())
    }
}

mod player {
    use super::*;

    #[derive(StructOpt, Debug)]
    /// displays the username history of a user
    pub struct UsernameHistory {
        /// username or user id
        pub user: String,

        #[structopt(long)]
        /// removes the change date from the formatting
        pub no_date: bool,
    }

    #[derive(StructOpt, Debug)]
    pub enum Opt {
        #[structopt(name = "usernames")]
        UsernameHistory(UsernameHistory),
        /// gets the UUID of a user
        UUID { user: String },
    }
}

#[derive(StructOpt, Debug)]
enum Opt {
    /// utilities for fetching and viewing skin information
    #[structopt(name = "skin")]
    Skin(skin::Opt),

    /// utilities for pinging and checking if a server is blocked by mojang
    #[structopt(name = "server")]
    Server(server::Opt),

    /// utilities for fetching and viewing player information
    #[structopt(name = "player")]
    Player(player::Opt),

    /// alias for `server ping`
    /// get a minecraft server's response
    #[structopt(name = "ping", display_order = 1000)]
    Ping(server::ServerPing),

    /// alias for `player uuid`
    /// gets the UUID of a user
    #[structopt(name = "uuid")]
    UUID { user: String },

    /// alias for `player usernames`
    /// gets the username history of a user
    #[structopt(name = "usernames")]
    Usernames(player::UsernameHistory),
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    match args {
        Opt::Player(opt) => match opt {
            player::Opt::UsernameHistory(UsernameHistory { user, no_date }) => {
                username_history(user, no_date, None)?
            }
            player::Opt::UUID { user } => find_uuid(user, None)?,
        },
        Opt::Skin(opt) => match opt {
            skin::Opt::FetchSkinData { user } => skin::display_json(&user)?,
            skin::Opt::ShowSkinInfo { user } => skin::display_info(&user)?,
        },
        Opt::Server(opt) => match opt {
            server::Opt::ServerBlocked { addr } => server::is_server_blocked(&addr)?,
            server::Opt::Ping(server::ServerPing { host, port }) => {
                server::ping_server(&host, port.unwrap_or(25565))?
            }
        },
        Opt::UUID { user } => find_uuid(user, None)?,
        Opt::Usernames(player::UsernameHistory { user, no_date }) => {
            username_history(user, no_date, None)?
        }
        Opt::Ping(ServerPing { host, port }) => server::ping_server(&host, port.unwrap_or(25565))?,
    };
    Ok(())
}
