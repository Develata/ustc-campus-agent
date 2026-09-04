package com.develata.ustccampusagent;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import java.net.URI;
import org.junit.Test;

public final class ServerEndpointTest {
    @Test
    public void defaultLoopbackOriginIsAcceptedAndNormalized() {
        ServerEndpoint endpoint = ServerEndpoint.parse(" http://127.0.0.1:8787 ");
        assertEquals("http://127.0.0.1:8787/", endpoint.url());
        assertTrue(endpoint.contains(URI.create("http://127.0.0.1:8787/api/v1/agent/chat")));
        assertFalse(endpoint.contains(URI.create("http://127.0.0.1:8788/")));
    }

    @Test
    public void remoteOriginRequiresHttps() {
        assertThrows(
                IllegalArgumentException.class,
                () -> ServerEndpoint.parse("http://campus.example/"));
        assertEquals(
                "https://campus.example/",
                ServerEndpoint.parse("https://CAMPUS.example").url());
    }

    @Test
    public void authorityConfusionAndNonOriginPartsAreRejected() {
        for (String value : new String[] {
            "https://user@campus.example/",
            "https://campus.example/path",
            "https://campus.example/?token=value",
            "https://campus.example/#fragment",
            "javascript:alert(1)",
            "file:///tmp/demo"
        }) {
            assertThrows(value, IllegalArgumentException.class, () -> ServerEndpoint.parse(value));
        }
    }

    @Test
    public void originComparisonUsesEffectivePorts() {
        ServerEndpoint endpoint = ServerEndpoint.parse("https://campus.example/");
        assertTrue(endpoint.contains(URI.create("https://campus.example:443/chat")));
        assertFalse(endpoint.contains(URI.create("http://campus.example/")));
        assertTrue(ServerEndpoint.isWebUri(URI.create("https://teach.ustc.edu.cn/")));
        assertFalse(ServerEndpoint.isWebUri(URI.create("intent://example")));
    }
}
