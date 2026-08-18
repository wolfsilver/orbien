use super::server::{parse_host_port, QuicOptions};
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(default = "default_server")]
    pub server: String,

    #[serde(default)]
    pub user: String,

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default)]
    pub transport: TransportConfig,

    #[serde(default)]
    pub tunnels: Vec<TunnelConfig>,

    #[serde(
        default = "default_udp_packet_size",
        rename = "udpPacketSize",
        alias = "udp_packet_size"
    )]
    pub udp_packet_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default = "default_auth_type", rename = "type", alias = "auth_type")]
    pub auth_type: String,
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(default = "default_protocol")]
    pub protocol: String,

    #[serde(
        default = "default_pool_count",
        rename = "poolCount",
        alias = "pool_count"
    )]
    pub pool_count: i32,

    #[serde(default = "default_tcp_mux", rename = "tcpMux", alias = "tcp_mux")]
    pub tcp_mux: bool,

    #[serde(
        default = "default_mux_keepalive_secs",
        rename = "muxKeepaliveSecs",
        alias = "mux_keepalive_secs"
    )]
    pub mux_keepalive_secs: i64,

    #[serde(
        default = "default_heartbeat_interval",
        rename = "heartbeatInterval",
        alias = "heartbeat_interval"
    )]
    pub heartbeat_interval: i64,
    #[serde(
        default = "default_heartbeat_timeout",
        rename = "heartbeatTimeout",
        alias = "heartbeat_timeout"
    )]
    pub heartbeat_timeout: i64,
    #[serde(default)]
    pub quic: QuicOptions,

    #[serde(default)]
    pub tls: ClientTlsConfig,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            protocol: default_protocol(),
            pool_count: default_pool_count(),
            tcp_mux: default_tcp_mux(),
            mux_keepalive_secs: default_mux_keepalive_secs(),
            heartbeat_interval: default_heartbeat_interval(),
            heartbeat_timeout: default_heartbeat_timeout(),
            quic: QuicOptions::default(),
            tls: ClientTlsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTlsConfig {
    #[serde(default = "default_tls_enable")]
    pub enable: bool,
    #[serde(default, rename = "certFile", alias = "cert_file")]
    pub cert_file: String,
    #[serde(default, rename = "keyFile", alias = "key_file")]
    pub key_file: String,
    #[serde(default, rename = "trustedCaFile", alias = "trusted_ca_file")]
    pub trusted_ca_file: String,
    #[serde(default, rename = "serverName", alias = "server_name")]
    pub server_name: String,
}

impl Default for ClientTlsConfig {
    fn default() -> Self {
        Self {
            enable: default_tls_enable(),
            cert_file: String::new(),
            key_file: String::new(),
            trusted_ca_file: String::new(),
            server_name: String::new(),
        }
    }
}

fn default_tls_enable() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub name: String,

    pub protocol: String,

    #[serde(default)]
    pub service: String,

    #[serde(default, rename = "remotePort", alias = "remote_port")]
    pub remote_port: u16,

    #[serde(default)]
    pub domains: Vec<String>,

    #[serde(default)]
    pub locations: Vec<String>,

    #[serde(default, rename = "basicAuthUser", alias = "basic_auth_user")]
    pub basic_auth_user: String,
    #[serde(default, rename = "basicAuthPassword", alias = "basic_auth_password")]
    pub basic_auth_password: String,

    #[serde(default, rename = "hostHeaderRewrite", alias = "host_header_rewrite")]
    pub host_header_rewrite: String,
    #[serde(default, rename = "routeByHTTPUser", alias = "route_by_http_user")]
    pub route_by_http_user: String,

    #[serde(default)]
    pub transport: TunnelTransportConfig,

    #[serde(default)]
    pub plugin: Option<PluginConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    #[serde(rename = "type", alias = "plugin_type")]
    pub plugin_type: String,

    #[serde(default)]
    pub service: String,

    #[serde(
        default,
        alias = "pluginAddr",
        alias = "pluginServerAddr",
        alias = "plugin_server_addr",
        alias = "server_addr"
    )]
    pub plugin_addr: String,

    #[serde(
        default,
        alias = "pluginUser",
        alias = "plugin_username",
        alias = "username"
    )]
    pub plugin_user: String,

    #[serde(
        default,
        alias = "pluginPasswd",
        alias = "plugin_password",
        alias = "password",
        alias = "passwd"
    )]
    pub plugin_passwd: String,

    #[serde(default, rename = "certFile", alias = "cert_file")]
    pub cert_file: String,
    #[serde(default, rename = "keyFile", alias = "key_file")]
    pub key_file: String,
    #[serde(default, rename = "hostHeaderRewrite", alias = "host_header_rewrite")]
    pub host_header_rewrite: String,

    #[serde(default, rename = "requestHeaders", alias = "request_headers")]
    pub request_headers: PluginRequestHeaders,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginRequestHeaders {
    #[serde(default)]
    pub set: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TunnelTransportConfig {
    #[serde(default)]
    pub bandwidth: f64,

    #[serde(
        default = "default_bandwidth_limit_side",
        rename = "bandwidthLimitSide",
        alias = "bandwidth_limit_side"
    )]
    pub bandwidth_limit_side: String,

    #[serde(
        default,
        rename = "proxyProtocolVersion",
        alias = "proxy_protocol_version"
    )]
    pub proxy_protocol_version: String,
}

fn default_bandwidth_limit_side() -> String {
    "client".into()
}

fn default_auth_type() -> String {
    "token".into()
}

fn default_protocol() -> String {
    "tcp".into()
}

fn default_pool_count() -> i32 {
    1
}

fn default_tcp_mux() -> bool {
    true
}

fn default_mux_keepalive_secs() -> i64 {
    30
}

fn default_heartbeat_interval() -> i64 {
    -1
}

fn default_heartbeat_timeout() -> i64 {
    -1
}

fn default_udp_packet_size() -> usize {
    1500
}

fn default_server() -> String {
    "127.0.0.1:9527".into()
}

impl TunnelConfig {
    pub fn service_host_port(&self) -> anyhow::Result<(String, u16)> {
        let raw = self.service.trim();
        if raw.is_empty() {
            return Ok(("127.0.0.1".into(), 0));
        }
        let (host, port) = parse_host_port(raw, 0)
            .map_err(|e| anyhow!("tunnel `{}` invalid service: {e}", self.name))?;
        if port == 0 {
            return Err(anyhow!(
                "tunnel `{}` service must include a port (got {raw:?})",
                self.name
            ));
        }
        if host.is_empty() {
            return Err(anyhow!("tunnel `{}` service has empty host", self.name));
        }
        Ok((host, port))
    }

    pub fn has_plugin(&self) -> bool {
        matches!(
            self.plugin
                .as_ref()
                .map(|p| p.plugin_type.trim().is_empty()),
            Some(false)
        )
    }

    pub fn requires_local_service(&self) -> bool {
        !self.has_plugin()
    }
}

impl ClientConfig {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::load_with(path, true)
    }

    pub fn load_for_edit(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::load_with(path, false)
    }

    fn load_with(path: impl AsRef<Path>, resolve: bool) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let raw = super::read_toml_file(path)?;
        let mut cfg: Self = toml::from_str(&raw)
            .with_context(|| format!("failed to parse config file '{}'", path.display()))?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        if resolve {
            cfg.resolve_paths(base);
        }
        cfg.complete();
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn prepare_runtime(&mut self, config_path: &Path) {
        let base = config_path.parent().unwrap_or_else(|| Path::new("."));
        self.resolve_paths(base);
        self.complete();
    }

    fn resolve_paths(&mut self, base: &Path) {
        let tls = &mut self.transport.tls;
        tls.cert_file = super::resolve_maybe_relative(base, &tls.cert_file);
        tls.key_file = super::resolve_maybe_relative(base, &tls.key_file);
        tls.trusted_ca_file = super::resolve_maybe_relative(base, &tls.trusted_ca_file);
        for t in &mut self.tunnels {
            if let Some(ref mut plugin) = t.plugin {
                plugin.cert_file = super::resolve_maybe_relative(base, &plugin.cert_file);
                plugin.key_file = super::resolve_maybe_relative(base, &plugin.key_file);
            }
        }
    }

    pub fn complete(&mut self) {
        if matches!(self.protocol().ok(), Some(crate::transport::Protocol::Quic)) {
            self.transport.tcp_mux = false;
            self.transport.tls.enable = true;
        }

        if !self.transport.tcp_mux {
            if self.transport.heartbeat_interval < 0 {
                self.transport.heartbeat_interval = 30;
            }
            if self.transport.heartbeat_timeout < 0 {
                self.transport.heartbeat_timeout = 90;
            }
        }

        if self.auth.auth_type.trim().is_empty() {
            self.auth.auth_type = default_auth_type();
        }

        if self.udp_packet_size == 0 {
            self.udp_packet_size = default_udp_packet_size();
        }

        if self.transport.tls.server_name.trim().is_empty() {
            if let Ok(host) = self.server_host() {
                self.transport.tls.server_name = host;
            }
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.server.trim().is_empty() {
            return Err(anyhow!("server is required (host:port)"));
        }
        let (host, port) = parse_host_port(&self.server, 9527)
            .map_err(|e| anyhow!("invalid server {:?}: {e}", self.server))?;
        if host.is_empty() {
            return Err(anyhow!("server host is empty"));
        }
        if port == 0 {
            return Err(anyhow!("server port must be > 0"));
        }
        let _ = self.protocol()?;
        if self.transport.pool_count < 1 {
            return Err(anyhow!(
                "transport.poolCount must be >= 1, got {}",
                self.transport.pool_count
            ));
        }
        if self.udp_packet_size == 0 || self.udp_packet_size > 65535 {
            return Err(anyhow!(
                "udpPacketSize out of range: {}",
                self.udp_packet_size
            ));
        }
        if self.transport.heartbeat_timeout > 0
            && self.transport.heartbeat_interval > 0
            && self.transport.heartbeat_timeout < self.transport.heartbeat_interval
        {
            return Err(anyhow!(
                "heartbeatTimeout ({}) must be >= heartbeatInterval ({})",
                self.transport.heartbeat_timeout,
                self.transport.heartbeat_interval
            ));
        }
        for t in &self.tunnels {
            if t.name.trim().is_empty() {
                return Err(anyhow!("tunnel name is required"));
            }
            let proto = t.protocol.trim().to_ascii_lowercase();
            if !matches!(proto.as_str(), "tcp" | "udp" | "http" | "https") {
                return Err(anyhow!(
                    "tunnel `{}` unsupported protocol {:?}",
                    t.name,
                    t.protocol
                ));
            }
            if !t.transport.bandwidth.is_finite() || t.transport.bandwidth < 0.0 {
                return Err(anyhow!(
                    "tunnel `{}` invalid bandwidth {}",
                    t.name,
                    t.transport.bandwidth
                ));
            }
            let side = t.transport.bandwidth_limit_side.trim().to_ascii_lowercase();
            if !side.is_empty() && side != "client" && side != "server" {
                return Err(anyhow!(
                    "tunnel `{}` invalid bandwidthLimitSide {:?}",
                    t.name,
                    t.transport.bandwidth_limit_side
                ));
            }
            match proto.as_str() {
                "tcp" | "udp" => {
                    if t.remote_port == 0 {
                        return Err(anyhow!(
                            "tunnel `{}` remotePort is required for {}",
                            t.name,
                            proto
                        ));
                    }
                    if t.requires_local_service() {
                        let _ = t.service_host_port()?;
                    }
                }
                "http" | "https" => {
                    if t.domains.is_empty() {
                        return Err(anyhow!(
                            "tunnel `{}` domains is required for {}",
                            t.name,
                            proto
                        ));
                    }
                    if t.requires_local_service() {
                        let _ = t.service_host_port()?;
                    }
                    if let Some(plugin) = &t.plugin {
                        let pt = plugin.plugin_type.trim().to_ascii_lowercase();
                        if !pt.is_empty() && pt != "tls-term" && pt != "socks5" && pt != "socks" {
                            return Err(anyhow!(
                                "tunnel `{}` unsupported plugin.type {:?}",
                                t.name,
                                plugin.plugin_type
                            ));
                        }
                        if pt == "tls-term" {
                            if plugin.service.trim().is_empty() {
                                return Err(anyhow!(
                                    "tunnel `{}` plugin.service is required for tls-term",
                                    t.name
                                ));
                            }
                            let (h, p) = parse_host_port(&plugin.service, 0).map_err(|e| {
                                anyhow!("tunnel `{}` invalid plugin.service: {e}", t.name)
                            })?;
                            if h.is_empty() || p == 0 {
                                return Err(anyhow!(
                                    "tunnel `{}` plugin.service must be host:port",
                                    t.name
                                ));
                            }
                        }
                        if pt == "socks5" || pt == "socks" {
                            let addr = plugin.plugin_addr.trim();
                            let fallback = plugin.service.trim();
                            let target = if !addr.is_empty() { addr } else { fallback };
                            if target.is_empty() {
                                return Err(anyhow!(
                                    "tunnel `{}` plugin.service or pluginServerAddr is required for socks5",
                                    t.name
                                ));
                            }
                            let (h, p) = parse_host_port(target, 0).map_err(|e| {
                                anyhow!("tunnel `{}` invalid socks5 plugin address: {e}", t.name)
                            })?;
                            if h.is_empty() || p == 0 {
                                return Err(anyhow!(
                                    "tunnel `{}` plugin address must be host:port",
                                    t.name
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn server_host(&self) -> anyhow::Result<String> {
        Ok(parse_host_port(&self.server, 9527)?.0)
    }

    pub fn server_port(&self) -> anyhow::Result<u16> {
        Ok(parse_host_port(&self.server, 9527)?.1)
    }

    pub fn tls_server_name(&self) -> String {
        let sn = self.transport.tls.server_name.trim();
        if sn.is_empty() {
            self.server_host().unwrap_or_else(|_| "localhost".into())
        } else {
            sn.to_string()
        }
    }

    pub fn server_endpoint(&self) -> String {
        self.server.trim().to_string()
    }

    pub fn protocol(&self) -> anyhow::Result<crate::transport::Protocol> {
        crate::transport::Protocol::parse(&self.transport.protocol).ok_or_else(|| {
            anyhow::anyhow!(
                "unsupported transport.protocol {:?}, use tcp|quic|websocket|kcp",
                self.transport.protocol
            )
        })
    }

    pub fn uses_yamux(&self) -> bool {
        self.transport.tcp_mux
            && matches!(
                self.protocol().ok(),
                Some(
                    crate::transport::Protocol::Tcp
                        | crate::transport::Protocol::Websocket
                        | crate::transport::Protocol::Kcp
                )
            )
    }
}
