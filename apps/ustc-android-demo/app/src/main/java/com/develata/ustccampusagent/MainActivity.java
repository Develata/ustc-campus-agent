package com.develata.ustccampusagent;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.ActivityNotFoundException;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.graphics.Color;
import android.net.Uri;
import android.net.http.SslError;
import android.os.Bundle;
import android.text.InputType;
import android.view.Gravity;
import android.view.KeyEvent;
import android.view.View;
import android.view.ViewGroup;
import android.webkit.CookieManager;
import android.webkit.SslErrorHandler;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Button;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.ProgressBar;
import android.widget.TextView;
import java.net.URI;

/**
 * Thin Android demonstration shell for the existing server-owned Web MVP.
 *
 * <p>The shell validates the server origin, handles platform navigation and lifecycle, and never
 * implements campus/Agent authority locally. It intentionally exposes no JavaScript bridge.
 */
public final class MainActivity extends Activity {
    private static final String PREFS = "uca_android_demo";
    private static final String SERVER_URL_KEY = "server_url";

    private WebView webView;
    private ProgressBar progressBar;
    private View errorPanel;
    private TextView errorMessage;
    private TextView connectionState;
    private ServerEndpoint endpoint;
    private boolean mainFrameFailed;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        endpoint = loadEndpoint();
        setContentView(buildContent());
        configureWebView();

        if (savedInstanceState == null || webView.restoreState(savedInstanceState) == null) {
            loadHome();
        }
    }

    private View buildContent() {
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setBackgroundColor(getColor(R.color.uca_background));

        LinearLayout toolbar = new LinearLayout(this);
        toolbar.setGravity(Gravity.CENTER_VERTICAL);
        toolbar.setPadding(dp(16), dp(8), dp(8), dp(8));
        toolbar.setBackgroundColor(getColor(R.color.uca_surface));

        TextView title = new TextView(this);
        title.setText(R.string.app_name);
        title.setTextColor(getColor(R.color.uca_text));
        title.setTextSize(18);
        title.setTypeface(title.getTypeface(), android.graphics.Typeface.BOLD);
        toolbar.addView(
                title,
                new LinearLayout.LayoutParams(0, dp(48), 1));

        Button serverButton = toolbarButton(getString(R.string.server_settings));
        serverButton.setContentDescription("设置校园 Agent 服务器地址");
        serverButton.setOnClickListener(view -> showEndpointDialog());
        toolbar.addView(serverButton, new LinearLayout.LayoutParams(dp(88), dp(48)));

        Button reloadButton = toolbarButton(getString(R.string.reload));
        reloadButton.setContentDescription("重新载入校园 Agent");
        reloadButton.setOnClickListener(view -> loadHome());
        toolbar.addView(reloadButton, new LinearLayout.LayoutParams(dp(72), dp(48)));
        root.addView(
                toolbar,
                new LinearLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.WRAP_CONTENT));

        TextView prototypeNotice = new TextView(this);
        prototypeNotice.setId(R.id.prototype_disclaimer);
        prototypeNotice.setText(R.string.prototype_disclaimer);
        prototypeNotice.setContentDescription(getString(R.string.prototype_disclaimer));
        prototypeNotice.setGravity(Gravity.CENTER);
        prototypeNotice.setPadding(dp(16), dp(6), dp(16), dp(6));
        prototypeNotice.setTextColor(getColor(R.color.uca_accent));
        prototypeNotice.setTextSize(12);
        prototypeNotice.setTypeface(
                prototypeNotice.getTypeface(), android.graphics.Typeface.BOLD);
        prototypeNotice.setBackgroundColor(getColor(R.color.uca_surface));
        root.addView(
                prototypeNotice,
                new LinearLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.WRAP_CONTENT));

        connectionState = new TextView(this);
        connectionState.setPadding(dp(16), dp(4), dp(16), dp(6));
        connectionState.setTextColor(getColor(R.color.uca_muted));
        connectionState.setTextSize(12);
        connectionState.setSingleLine(true);
        connectionState.setText(connectionLabel("准备连接"));
        connectionState.setBackgroundColor(getColor(R.color.uca_surface));
        root.addView(
                connectionState,
                new LinearLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.WRAP_CONTENT));

        FrameLayout content = new FrameLayout(this);
        webView = new WebView(this);
        content.addView(
                webView,
                new FrameLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT));

        progressBar = new ProgressBar(this);
        FrameLayout.LayoutParams progressParams =
                new FrameLayout.LayoutParams(dp(44), dp(44), Gravity.CENTER);
        content.addView(progressBar, progressParams);

        errorPanel = buildErrorPanel();
        errorPanel.setVisibility(View.GONE);
        content.addView(
                errorPanel,
                new FrameLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT));

        root.addView(
                content,
                new LinearLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        0,
                        1));
        return root;
    }

    private View buildErrorPanel() {
        LinearLayout panel = new LinearLayout(this);
        panel.setOrientation(LinearLayout.VERTICAL);
        panel.setGravity(Gravity.CENTER);
        panel.setPadding(dp(28), dp(28), dp(28), dp(28));
        panel.setBackgroundColor(getColor(R.color.uca_background));

        TextView heading = new TextView(this);
        heading.setText(R.string.offline_heading);
        heading.setTextColor(getColor(R.color.uca_text));
        heading.setTextSize(21);
        heading.setGravity(Gravity.CENTER);
        heading.setTypeface(heading.getTypeface(), android.graphics.Typeface.BOLD);
        panel.addView(heading);

        errorMessage = new TextView(this);
        errorMessage.setTextColor(getColor(R.color.uca_muted));
        errorMessage.setTextSize(15);
        errorMessage.setGravity(Gravity.CENTER);
        errorMessage.setPadding(0, dp(12), 0, dp(20));
        panel.addView(
                errorMessage,
                new LinearLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.WRAP_CONTENT));

        Button retry = toolbarButton(getString(R.string.retry));
        retry.setContentDescription("重试连接校园 Agent");
        retry.setOnClickListener(view -> loadHome());
        panel.addView(retry, new LinearLayout.LayoutParams(dp(160), dp(48)));
        return panel;
    }

    private Button toolbarButton(String text) {
        Button button = new Button(this);
        button.setText(text);
        button.setTextSize(14);
        button.setTextColor(getColor(R.color.uca_accent));
        button.setAllCaps(false);
        button.setGravity(Gravity.CENTER);
        button.setMinHeight(dp(48));
        button.setMinWidth(dp(48));
        button.setPadding(dp(8), 0, dp(8), 0);
        button.setBackgroundColor(Color.TRANSPARENT);
        return button;
    }

    @SuppressLint("SetJavaScriptEnabled") // Required by the same-origin Web MVP; no JS bridge exists.
    private void configureWebView() {
        WebSettings settings = webView.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setAllowFileAccess(false);
        settings.setAllowContentAccess(false);
        settings.setJavaScriptCanOpenWindowsAutomatically(false);
        settings.setSupportMultipleWindows(false);
        settings.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);
        settings.setSafeBrowsingEnabled(true);
        settings.setMediaPlaybackRequiresUserGesture(true);
        settings.setUserAgentString(settings.getUserAgentString() + " USTCCampusAgentAndroid/0.1");

        CookieManager.getInstance().setAcceptCookie(true);
        CookieManager.getInstance().setAcceptThirdPartyCookies(webView, false);
        WebView.setWebContentsDebuggingEnabled(BuildConfig.DEBUG);

        webView.setWebChromeClient(
                new WebChromeClient() {
                    @Override
                    public void onProgressChanged(WebView view, int newProgress) {
                        progressBar.setProgress(newProgress);
                        progressBar.setVisibility(newProgress < 100 ? View.VISIBLE : View.GONE);
                    }
                });
        webView.setWebViewClient(
                new WebViewClient() {
                    @Override
                    public void onPageStarted(WebView view, String url, android.graphics.Bitmap favicon) {
                        mainFrameFailed = false;
                        errorPanel.setVisibility(View.GONE);
                        progressBar.setVisibility(View.VISIBLE);
                        connectionState.setText(connectionLabel("正在连接"));
                    }

                    @Override
                    public void onPageFinished(WebView view, String url) {
                        progressBar.setVisibility(View.GONE);
                        if (!mainFrameFailed) {
                            errorPanel.setVisibility(View.GONE);
                            connectionState.setText(connectionLabel("已连接"));
                        }
                    }

                    @Override
                    public boolean shouldOverrideUrlLoading(
                            WebView view, WebResourceRequest request) {
                        if (!request.isForMainFrame()) {
                            return false;
                        }
                        URI target;
                        try {
                            target = URI.create(request.getUrl().toString());
                        } catch (IllegalArgumentException error) {
                            connectionState.setText(connectionLabel("已阻止无效链接"));
                            return true;
                        }
                        if (endpoint.contains(target)) {
                            return false;
                        }
                        if (ServerEndpoint.isWebUri(target)) {
                            openExternal(request.getUrl());
                        } else {
                            connectionState.setText(connectionLabel("已阻止非 Web 链接"));
                        }
                        return true;
                    }

                    @Override
                    public void onReceivedError(
                            WebView view,
                            WebResourceRequest request,
                            WebResourceError error) {
                        if (request.isForMainFrame()) {
                            showConnectionError("无法访问 " + endpoint.url() + "\n\n开发演示：先启动 Web MVP，再执行 adb reverse tcp:8787 tcp:8787。");
                        }
                    }

                    @Override
                    public void onReceivedHttpError(
                            WebView view,
                            WebResourceRequest request,
                            WebResourceResponse errorResponse) {
                        if (request.isForMainFrame() && errorResponse.getStatusCode() >= 400) {
                            showConnectionError(
                                    "服务器返回 HTTP " + errorResponse.getStatusCode() + "。请核对服务状态和地址。");
                        }
                    }

                    @Override
                    public void onReceivedSslError(
                            WebView view, SslErrorHandler handler, SslError error) {
                        handler.cancel();
                        showConnectionError("HTTPS 证书校验失败；应用不会绕过证书错误。");
                    }
                });
    }

    private void loadHome() {
        mainFrameFailed = false;
        errorPanel.setVisibility(View.GONE);
        connectionState.setText(connectionLabel("正在连接"));
        webView.loadUrl(endpoint.url());
    }

    private void showConnectionError(String message) {
        mainFrameFailed = true;
        progressBar.setVisibility(View.GONE);
        errorMessage.setText(message);
        errorPanel.setVisibility(View.VISIBLE);
        connectionState.setText(connectionLabel("连接失败"));
    }

    private void openExternal(Uri target) {
        try {
            startActivity(new Intent(Intent.ACTION_VIEW, target));
        } catch (ActivityNotFoundException error) {
            connectionState.setText(connectionLabel("没有可打开链接的浏览器"));
        }
    }

    private ServerEndpoint loadEndpoint() {
        SharedPreferences preferences = getSharedPreferences(PREFS, Context.MODE_PRIVATE);
        String stored = preferences.getString(SERVER_URL_KEY, ServerEndpoint.DEFAULT_URL);
        try {
            return ServerEndpoint.parse(stored);
        } catch (IllegalArgumentException error) {
            preferences.edit().remove(SERVER_URL_KEY).apply();
            return ServerEndpoint.parse(ServerEndpoint.DEFAULT_URL);
        }
    }

    private void showEndpointDialog() {
        EditText input = new EditText(this);
        input.setSingleLine(true);
        input.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_URI);
        input.setText(endpoint.url());
        input.setSelectAllOnFocus(true);
        int inset = dp(20);

        FrameLayout holder = new FrameLayout(this);
        holder.setPadding(inset, 0, inset, 0);
        holder.addView(
                input,
                new FrameLayout.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.WRAP_CONTENT));

        AlertDialog dialog =
                new AlertDialog.Builder(this)
                        .setTitle("服务器 origin")
                        .setMessage("远程地址必须使用 HTTPS；本机演示可使用 127.0.0.1。地址不是凭据。")
                        .setView(holder)
                        .setNegativeButton("取消", null)
                        .setPositiveButton("保存并连接", null)
                        .create();
        dialog.setOnShowListener(
                ignored ->
                        dialog.getButton(AlertDialog.BUTTON_POSITIVE)
                                .setOnClickListener(
                                        view -> {
                                            final ServerEndpoint candidate;
                                            try {
                                                candidate = ServerEndpoint.parse(input.getText().toString());
                                            } catch (IllegalArgumentException error) {
                                                input.setError(error.getMessage());
                                                return;
                                            }
                                            endpoint = candidate;
                                            getSharedPreferences(PREFS, Context.MODE_PRIVATE)
                                                    .edit()
                                                    .putString(SERVER_URL_KEY, endpoint.url())
                                                    .apply();
                                            dialog.dismiss();
                                            loadHome();
                                        }));
        dialog.show();
    }

    private String connectionLabel(String state) {
        String source = BuildConfig.SOURCE_COMMIT;
        String shortSource = source.length() >= 8 ? source.substring(0, 8) : source;
        return state + " · " + endpoint.url() + " · " + shortSource;
    }

    private int dp(int value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }

    @Override
    protected void onSaveInstanceState(Bundle outState) {
        webView.saveState(outState);
        super.onSaveInstanceState(outState);
    }

    @Override
    protected void onPause() {
        webView.onPause();
        super.onPause();
    }

    @Override
    protected void onResume() {
        super.onResume();
        webView.onResume();
    }

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK && webView.canGoBack()) {
            webView.goBack();
            return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    @Override
    protected void onDestroy() {
        webView.stopLoading();
        webView.setWebChromeClient(null);
        webView.setWebViewClient(null);
        webView.destroy();
        super.onDestroy();
    }
}
