use crate::player::detect::Player;
use std::process::{Command, Stdio};

pub fn spawn_player(player: Player, url: &str, title: &str) -> std::io::Result<()> {
    let mut cmd = match player {
        Player::Mpv => {
            let mut c = Command::new("mpv");
            c.arg("--force-media-title").arg(title).arg(url);
            c
        }
        Player::Iina => {
            let mut c = Command::new("iina");
            c.arg(url);
            c
        }
        Player::Vlc => {
            let mut c = Command::new("vlc");
            c.arg("--play-and-exit").arg(url);
            c
        }
    };

    cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
    Ok(())
}
