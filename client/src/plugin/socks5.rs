use super::{ConnectionInfo, Plugin, PluginContext};
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use orbien_core::config::PluginConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Socks5Plugin {
    proxy_addr: String,
    username: String,
    password: String,
}

impl Socks5Plugin {
    pub fn new(_ctx: PluginContext, cfg: &PluginConfig) -> Result<Self> {
        let proxy_addr = cfg
            .plugin_addr
            .trim()
            .to_string();
        let proxy_addr = if proxy_addr.is_empty() {
            cfg.service.trim().to_string()
        } else {
            proxy_addr
        };
        if proxy_addr.is_empty() {
            bail!("socks5 requires plugin.service or pluginAddr (e.g. \"127.0.0.1:1080\")");
        }

        let username = cfg.plugin_user.trim().to_string();
        let password = cfg.plugin_passwd.trim().to_string();

        tracing::info!(
            tunnel = %_ctx.name,
            proxy = %proxy_addr,
            auth = %(!username.is_empty()),
            "plugin socks5 ready (forward through upstream SOCKS5 proxy)"
        );

        Ok(Self {
            proxy_addr,
            username,
            password,
        })
    }
}

#[async_trait]
impl Plugin for Socks5Plugin {
    fn name(&self) -> &str {
        "socks5"
    }

    async fn handle(&self, conn: ConnectionInfo) -> Result<()> {
        let mut upstream = TcpStream::connect(&self.proxy_addr)
            .await
            .map_err(|e| anyhow!("socks5 dial {}: {e}", self.proxy_addr))?;
        orbien_core::net::enable_nodelay(&upstream);

        self.do_handshake(&mut upstream).await?;
        let target = if conn.dst_addr.trim().is_empty() {
            "127.0.0.1".to_string()
        } else {
            conn.dst_addr.clone()
        };

        let req = socks5_connect_request(&target, conn.dst_port)?;
        upstream.write_all(&req).await?;

        let mut reply = [0u8; 4];
        let mut reader = &upstream;
        reader.read_exact(&mut reply).await?;
        if reply[0] != 0x05 {
            bail!("invalid SOCKS5 response version: {}", reply[0]);
        }
        if reply[1] != 0x00 {
            bail!("SOCKS5 connect rejected with code {}", reply[1]);
        }

        match reply[3] {
            0x01 => {
                let mut addr = [0u8; 4];
                reader.read_exact(&mut addr).await?;
            }
            0x03 => {
                let mut len = [0u8; 1];
                reader.read_exact(&mut len).await?;
                let len = len[0] as usize;
                let mut addr = vec![0u8; len];
                reader.read_exact(&mut addr).await?;
            }
            0x04 => {
                let mut addr = [0u8; 16];
                reader.read_exact(&mut addr).await?;
            }
            atyp => bail!("unsupported SOCKS5 destination type: {atyp}"),
        }
        let mut port = [0u8; 2];
        reader.read_exact(&mut port).await?;

        let (mut conn_r, mut conn_w) = tokio::io::split(conn.stream);
        let (mut upstream_r, mut upstream_w) = tokio::io::split(upstream);

        let _ = tokio::try_join!(
            async move { tokio::io::copy(&mut conn_r, &mut upstream_w).await },
            async move { tokio::io::copy(&mut upstream_r, &mut conn_w).await },
        );
        Ok(())
    }
}

impl Socks5Plugin {
    async fn do_handshake(&self, stream: &mut TcpStream) -> Result<()> {
        let mut methods = vec![0x00];
        if !self.username.is_empty() {
            methods.push(0x02);
        }

        let mut hello = vec![0x05, methods.len() as u8];
        hello.extend_from_slice(&methods);
        stream.write_all(&hello).await?;

        let mut resp = [0u8; 2];
        stream.read_exact(&mut resp).await?;
        if resp[0] != 0x05 {
            bail!("invalid SOCKS5 auth version: {}", resp[0]);
        }

        let method = resp[1];
        match method {
            0x00 => {}
            0x02 => {
                if self.username.is_empty() {
                    bail!("SOCKS5 upstream requires username/password authentication");
                }
                let mut auth = vec![0x01];
                let user = self.username.as_bytes();
                auth.push(user.len() as u8);
                auth.extend_from_slice(user);
                let pass = self.password.as_bytes();
                auth.push(pass.len() as u8);
                auth.extend_from_slice(pass);
                stream.write_all(&auth).await?;

                let mut auth_resp = [0u8; 2];
                stream.read_exact(&mut auth_resp).await?;
                if auth_resp[0] != 0x01 || auth_resp[1] != 0x00 {
                    bail!(
                        "SOCKS5 upstream authentication failed with code {}",
                        auth_resp[1]
                    );
                }
            }
            other => bail!("unsupported SOCKS5 auth method: {other}"),
        }
        Ok(())
    }
}

fn socks5_connect_request(target: &str, port: u16) -> Result<Vec<u8>> {
    let mut out = vec![0x05, 0x01, 0x00];
    if let Ok(ip) = target.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                out.push(0x01);
                out.extend_from_slice(&ipv4.octets());
            }
            std::net::IpAddr::V6(ipv6) => {
                out.push(0x04);
                out.extend_from_slice(&ipv6.octets());
            }
        }
    } else {
        let target = target.trim();
        let bytes = target.as_bytes();
        if bytes.is_empty() {
            bail!("empty SOCKS5 target host");
        }
        out.push(0x03);
        out.push(bytes.len() as u8);
        out.extend_from_slice(bytes);
    }
    out.extend_from_slice(&port.to_be_bytes());
    Ok(out)
}
