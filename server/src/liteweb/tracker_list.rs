use once_cell::sync::Lazy;
use std::collections::HashSet;

pub static TRACKER_DOMAINS: Lazy<HashSet<String>> = Lazy::new(|| {
    let domains = [
        // Google
        "google-analytics.com", "googletagmanager.com", "googleadservices.com",
        "googlesyndication.com", "doubleclick.net", "googletagservices.com",
        "pagead2.googlesyndication.com", "adservice.google.com",
        "stats.g.doubleclick.net", "cm.g.doubleclick.net",
        // Facebook / Meta
        "facebook.net", "connect.facebook.net", "pixel.facebook.com",
        "graph.facebook.com", "an.facebook.com",
        // Twitter / X
        "analytics.twitter.com", "ads-twitter.com", "static.ads-twitter.com",
        "t.co",
        // Microsoft / LinkedIn
        "bat.bing.com", "ads.linkedin.com", "snap.licdn.com",
        "clarity.ms",
        // Yandex
        "mc.yandex.ru", "metrika.yandex.ru",
        // Session recording
        "hotjar.com", "static.hotjar.com", "fullstory.com",
        "mouseflow.com", "luckyorange.com",
        // APM / Error tracking
        "newrelic.com", "nr-data.net", "bam.nr-data.net",
        "sentry.io", "browser.sentry-cdn.com", "bugsnag.com",
        // Analytics platforms
        "segment.io", "segment.com", "cdn.segment.com",
        "mixpanel.com", "api.mixpanel.com", "cdn.mxpnl.com",
        "amplitude.com", "api.amplitude.com",
        "heapanalytics.com", "app.pendo.io",
        "plausible.io", "matomo.cloud",
        "chartbeat.com", "parsely.com",
        "scorecardresearch.com", "sb.scorecardresearch.com",
        "b.scorecardresearch.com",
        "quantserve.com", "secure.quantserve.com", "pixel.quantserve.com",
        // A/B testing
        "crazyegg.com", "optimizely.com", "cdn.optimizely.com",
        "cloudflareinsights.com",
        // Advertising networks
        "adsrvr.org", "adnxs.com", "openx.net", "pubmatic.com",
        "rubiconproject.com", "contextweb.com", "indexexchange.com",
        "casalemedia.com", "moatads.com", "doubleverify.com",
        "adsafeprotected.com", "serving-sys.com", "turn.com",
        "amazon-adsystem.com", "media.net",
        "criteo.com", "criteo.net", "adroll.com",
        "perfectaudience.com", "steelhousemedia.com",
        // Content recommendation / native ads
        "taboola.com", "outbrain.com", "mgid.com", "revcontent.com",
        // Social sharing / engagement
        "sharethis.com", "addthis.com", "shareaholic.com",
        "disqus.com", "disquscdn.com", "spot.im",
        // Chat / support widgets
        "intercom.io", "widget.intercom.io",
        "crisp.chat", "drift.com", "tawk.to",
        "zendesk.com",
        // Push notifications
        "onesignal.com", "pusher.com", "pushwoosh.com",
        // Cookie consent
        "cookiebot.com", "cookieinformation.com",
        "onetrust.com", "cookielaw.org",
        "trustarc.com", "evidon.com",
        // Mobile attribution
        "branch.io", "app.link", "adjust.com",
        "appsflyer.com", "kochava.com", "singular.net",
        // Adobe
        "demdex.net", "omtrdc.net", "2o7.net", "sc.omtrdc.net",
        // Misc
        "recaptcha.net",
        "cdn.embedly.com",
        "static.chartbeat.com",
        "widgets.outbrain.com",
        "cdn.taboola.com",
    ];
    domains.iter().map(|s| s.to_string()).collect()
});
