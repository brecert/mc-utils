use chrono::{TimeZone, Utc};
use craftping;
use owo_colors::OwoColorize;
use structopt::StructOpt;
mod api;
mod util;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn username_or_id(idname: &str, at: Option<i64>) -> String {
    let timestamp = at.unwrap_or(chrono::Utc::now().timestamp());
    api::get_id_at(idname, timestamp).unwrap_or(idname.to_string())
}

fn find_uuid(username: String, at: Option<i64>) -> Result<()> {
    let timestamp = at.unwrap_or(chrono::Utc::now().timestamp());
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
    let history = match api::get_username_history(id) {
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

    fn get_skin_data(user: String) -> Result<api::Textures> {
        let id = username_or_id(&user, None);
        let skin = api::get_skin(id)?;
        let tex = skin.properties[0].textures()?.textures;
        Ok(tex)
    }

    pub fn display_json(user: String) -> Result<()> {
        let id = username_or_id(&user, None);
        println!("{:?}", get_skin_data(id)?);
        Ok(())
    }

    // todo(bree): figure out java timestamp
    pub fn display_info(user: String) -> Result<()> {
        let id = username_or_id(&user, None);
        let skin = get_skin_data(id)?;
        println!(
            "skin_type: {}",
            skin.skin
                .metadata
                .map(|data| data.model)
                .unwrap_or(String::from("classic"))
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
    pub enum Opt {
        #[structopt(name = "ping")]
        /// get a minecraft server's response
        Ping { host: String, port: Option<u16> },
        #[structopt(name = "blocked")]
        /// checks if a server is blocked by mojang
        ServerBlocked { addr: String },
    }

    pub fn ping_server(host: &str, port: u16) -> Result<()> {
        match craftping::sync::ping(host, port) {
            Err(err) => println!("{}{} {}", "error".bold().red(), ":".bold(), err.to_string()),
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

    pub fn is_server_blocked(addr: String) -> Result<()> {
        if let Some(pat) = api::find_blocked_pattern(&addr)? {
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

#[derive(StructOpt, Debug)]
enum Opt {
    #[structopt(name = "uuid")]
    /// gets the uuid of a user
    FindUuid {
        /// username or user id
        user: String,
        #[structopt(long)]
        /// return the uuid as it was at the timestamp provided (currently doesn't seemt to do anything)
        at: Option<i64>,
    },
    #[structopt(name = "usernames")]
    /// displays the username history of a user
    UsernameHistory {
        /// username or user id
        user: String,
        #[structopt(long)]
        /// removes the change date from the formatting
        no_date: bool,
        #[structopt(long)]
        /// return the username_history of the user as it was at the timestamp provided (currently doesn't seemt to do anything)
        at: Option<i64>,
    },

    #[structopt(name = "skin")]
    /// utilities for fetching and looking at skin information
    Skin(skin::Opt),
    #[structopt(name = "server")]
    /// utilities for pinging and checking if a server is blocked by mojang
    Server(server::Opt),
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    match args {
        Opt::UsernameHistory { user, no_date, at } => username_history(user, no_date, at)?,
        Opt::FindUuid { user, at } => find_uuid(user, at)?,
        Opt::Skin(skin_opt) => match skin_opt {
            skin::Opt::FetchSkinData { user } => skin::display_json(user)?,
            skin::Opt::ShowSkinInfo { user } => skin::display_info(user)?,
        },
        Opt::Server(server_opt) => match server_opt {
            server::Opt::ServerBlocked { addr } => server::is_server_blocked(addr)?,
            server::Opt::Ping { host, port } => server::ping_server(&host, port.unwrap_or(25565))?,
        },
    };
    Ok(())
}
