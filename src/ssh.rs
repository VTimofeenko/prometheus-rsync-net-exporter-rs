use anyhow::{Context, Result};
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

pub trait RsyncFetcher: Send + Sync {
    fn fetch_quota(&self) -> Result<String>;
}

pub struct SshFetcher {
    username: String,
    host: String,
    ssh_key_path: String,
}

impl SshFetcher {
    pub fn new(username: String, host: String, ssh_key_path: String) -> Self {
        Self {
            username,
            host,
            ssh_key_path,
        }
    }
}

impl RsyncFetcher for SshFetcher {
    fn fetch_quota(&self) -> Result<String> {
        let tcp = TcpStream::connect(format!("{}:22", self.host))
            .with_context(|| format!("Failed to connect to {}:22", self.host))?;

        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("SSH handshake failed")?;

        // Authenticate with private key

        sess.userauth_pubkey_file(
            &self.username,
            None,
            Path::new(&self.ssh_key_path),
            None, // Passphrase (optional, currently not supported by config)
        )
        .with_context(|| format!("SSH authentication failed using key: {}", self.ssh_key_path))?;

        if !sess.authenticated() {
            anyhow::bail!("SSH authentication failed (generic error)");
        }

        let mut channel = sess
            .channel_session()
            .context("Failed to open SSH channel")?;
        channel
            .exec("quota")
            .context("Failed to execute 'quota' command")?;

        let mut s = String::new();
        channel
            .read_to_string(&mut s)
            .context("Failed to read output from SSH channel")?;

        channel.wait_close()?;
        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            anyhow::bail!("Command 'quota' exited with status {}", exit_status);
        }

        Ok(s)
    }
}
