package io.github.lxien.orbien.boot;

import io.github.lxien.orbien.client.OrbienClientConfig;

import java.util.ArrayList;
import java.util.List;

import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.boot.context.properties.NestedConfigurationProperty;
import org.springframework.util.StringUtils;

@ConfigurationProperties(prefix = "orbien")
public class OrbienProperties {
    private static final String DEFAULT_LOCAL_IP = "127.0.0.1";
    private static final String DEFAULT_SERVER = "127.0.0.1:9527";

    private boolean enabled = true;
    private String server = DEFAULT_SERVER;
    private String token = "";
    private boolean tcpMux = false;
    private int poolCount = 1;
    private String user = "";
    private int heartbeatIntervalSecs = 30;

    @NestedConfigurationProperty
    private final Tunnel tunnel = new Tunnel();

    public boolean isEnabled() {
        return enabled;
    }

    public void setEnabled(boolean enabled) {
        this.enabled = enabled;
    }

    public String getServer() {
        return server;
    }

    public void setServer(String server) {
        this.server = StringUtils.hasText(server) ? server.trim() : DEFAULT_SERVER;
    }

    public String getToken() {
        return token;
    }

    public void setToken(String token) {
        this.token = token;
    }

    public boolean isTcpMux() {
        return tcpMux;
    }

    public void setTcpMux(boolean tcpMux) {
        this.tcpMux = tcpMux;
    }

    public int getPoolCount() {
        return poolCount;
    }

    public void setPoolCount(int poolCount) {
        this.poolCount = poolCount;
    }

    public String getUser() {
        return user;
    }

    public void setUser(String user) {
        this.user = user;
    }

    public int getHeartbeatIntervalSecs() {
        return heartbeatIntervalSecs;
    }

    public void setHeartbeatIntervalSecs(int heartbeatIntervalSecs) {
        this.heartbeatIntervalSecs = heartbeatIntervalSecs;
    }

    public Tunnel getTunnel() {
        return tunnel;
    }

    public boolean hasTunnel() {
        return StringUtils.hasText(tunnel.getName())
                || tunnel.getLocalPort() > 0
                || tunnel.getRemotePort() > 0
                || (tunnel.getDomains() != null && !tunnel.getDomains().isEmpty());
    }

    public OrbienClientConfig toClientConfig() {
        OrbienClientConfig cfg = new OrbienClientConfig();
        cfg.setServer(server);
        cfg.setToken(token);
        cfg.setTcpMux(tcpMux);
        cfg.setPoolCount(poolCount);
        cfg.setUser(user);
        cfg.setHeartbeatIntervalSecs(heartbeatIntervalSecs);
        if (hasTunnel()) {
            OrbienClientConfig.TunnelConfig p = new OrbienClientConfig.TunnelConfig();
            String name = tunnel.getName();
            if (!StringUtils.hasText(name)) {
                String type = StringUtils.hasText(tunnel.getProtocol()) ? tunnel.getProtocol() : "tcp";
                name = "orbien-" + type.toLowerCase();
            }
            p.setName(name);
            p.setProtocol(tunnel.getProtocol());
            String localIp =
                    StringUtils.hasText(tunnel.getLocalIp()) ? tunnel.getLocalIp() : DEFAULT_LOCAL_IP;
            p.setLocalIp(localIp);
            p.setLocalPort(tunnel.getLocalPort());
            p.setRemotePort(tunnel.getRemotePort());
            p.setDomains(new ArrayList<>(tunnel.getDomains()));
            cfg.getTunnels().add(p);
        }
        return cfg;
    }

    public static class Tunnel {
        private String name;
        private String protocol = "tcp";
        private String localIp = DEFAULT_LOCAL_IP;
        private int localPort;
        private int remotePort;
        private List<String> domains = new ArrayList<>();

        public String getName() {
            return name;
        }

        public void setName(String name) {
            this.name = name;
        }

        public String getProtocol() {
            return protocol;
        }

        public void setProtocol(String protocol) {
            this.protocol = protocol;
        }

        public String getLocalIp() {
            return localIp;
        }

        public void setLocalIp(String localIp) {
            this.localIp = localIp;
        }

        public int getLocalPort() {
            return localPort;
        }

        public void setLocalPort(int localPort) {
            this.localPort = localPort;
        }

        public int getRemotePort() {
            return remotePort;
        }

        public void setRemotePort(int remotePort) {
            this.remotePort = remotePort;
        }

        public List<String> getDomains() {
            return domains;
        }

        public void setDomains(List<String> domains) {
            this.domains = domains == null ? new ArrayList<>() : domains;
        }
    }
}
