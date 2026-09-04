package com.develata.ustccampusagent;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.Locale;
import java.util.Objects;

/** Validated, non-secret server origin used by the thin Android presentation shell. */
final class ServerEndpoint {
    static final String DEFAULT_URL = "http://127.0.0.1:8787/";

    private final URI origin;

    private ServerEndpoint(URI origin) {
        this.origin = origin;
    }

    static ServerEndpoint parse(String raw) {
        if (raw == null || raw.trim().isEmpty()) {
            throw new IllegalArgumentException("服务器地址不能为空");
        }

        final URI parsed;
        try {
            parsed = new URI(raw.trim());
        } catch (URISyntaxException error) {
            throw new IllegalArgumentException("服务器地址不是有效 URL", error);
        }

        String scheme = lower(parsed.getScheme());
        String host = lower(parsed.getHost());
        if (parsed.isOpaque()
                || host == null
                || parsed.getUserInfo() != null
                || parsed.getQuery() != null
                || parsed.getFragment() != null) {
            throw new IllegalArgumentException("只接受不含账号、查询参数或片段的服务器 origin");
        }
        String path = parsed.getPath();
        if (path != null && !path.isEmpty() && !"/".equals(path)) {
            throw new IllegalArgumentException("服务器地址不能包含路径");
        }
        int port = parsed.getPort();
        if (port == 0 || port < -1 || port > 65535) {
            throw new IllegalArgumentException("服务器端口无效");
        }
        boolean loopbackHttp = "http".equals(scheme) && isLoopbackHost(host);
        if (!"https".equals(scheme) && !loopbackHttp) {
            throw new IllegalArgumentException("远程服务器必须使用 HTTPS；HTTP 仅允许 127.0.0.1 或 localhost");
        }

        try {
            return new ServerEndpoint(new URI(scheme, null, host, port, "/", null, null));
        } catch (URISyntaxException impossible) {
            throw new IllegalArgumentException("服务器 origin 无法规范化", impossible);
        }
    }

    String url() {
        return origin.toASCIIString();
    }

    boolean contains(URI candidate) {
        return candidate != null
                && Objects.equals(lower(candidate.getScheme()), lower(origin.getScheme()))
                && Objects.equals(lower(candidate.getHost()), lower(origin.getHost()))
                && effectivePort(candidate) == effectivePort(origin);
    }

    static boolean isWebUri(URI candidate) {
        String scheme = candidate == null ? null : lower(candidate.getScheme());
        return "http".equals(scheme) || "https".equals(scheme);
    }

    private static int effectivePort(URI uri) {
        if (uri.getPort() >= 0) {
            return uri.getPort();
        }
        return "https".equals(lower(uri.getScheme())) ? 443 : 80;
    }

    private static boolean isLoopbackHost(String host) {
        return "127.0.0.1".equals(host) || "localhost".equals(host);
    }

    private static String lower(String value) {
        return value == null ? null : value.toLowerCase(Locale.ROOT);
    }
}
