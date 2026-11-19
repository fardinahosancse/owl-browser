//! OwlKick's built-in base rules (Owl.'s tracker host list), the same as the WebView2 build's.

pub const BLOCKED_HOSTS: &[&str] = &[
    "doubleclick.net", "googlesyndication.com", "googleadservices.com", "google-analytics.com",
    "googletagmanager.com", "googletagservices.com", "adservice.google.com", "app-measurement.com",
    "adnxs.com", "adsrvr.org", "advertising.com", "amazon-adsystem.com", "taboola.com", "outbrain.com",
    "criteo.com", "criteo.net", "scorecardresearch.com", "quantserve.com", "hotjar.com", "mixpanel.com",
    "segment.io", "segment.com", "fullstory.com", "mouseflow.com", "crazyegg.com", "optimizely.com",
    "bluekai.com", "demdex.net", "krxd.net", "rubiconproject.com", "pubmatic.com", "openx.net",
    "casalemedia.com", "moatads.com", "adform.net", "smartadserver.com", "2mdn.net", "serving-sys.com",
    "yieldmo.com", "chartbeat.com", "ads-twitter.com", "ads.twitter.com", "analytics.twitter.com",
    "analytics.tiktok.com", "ads.linkedin.com", "bat.bing.com", "clarity.ms", "ct.pinterest.com",
    "tr.snapchat.com", "adsymptotic.com", "tapad.com", "bidswitch.net", "everesttech.net", "mathtag.com",
    "turn.com", "yadro.ru", "mc.yandex.ru", "adroll.com", "zedo.com", "sharethrough.com", "inmobi.com",
    "pagead2.googlesyndication.com", "stats.g.doubleclick.net", "hs-analytics.net",
    "ads.yahoo.com", "analytics.yahoo.com", "log.pinterest.com", "px.ads.linkedin.com", "snap.licdn.com",
    "ads.tiktok.com", "business-api.tiktok.com", "bttrack.com", "id5-sync.com", "liveramp.com", "rlcdn.com",
    "lijit.com", "33across.com", "indexww.com", "contextweb.com", "sitescout.com", "adcolony.com",
    "applovin.com", "unity3d.com/ads", "ads.mopub.com", "branch.io", "adjust.com", "appsflyer.com",
    "newrelic.com", "nr-data.net", "heap.io", "heapanalytics.com", "amplitude.com", "luckyorange.com",
    "inspectlet.com", "smartlook.com", "statcounter.com", "histats.com", "addthis.com", "sharethis.com",
];
pub const BLOCKED_PATHS: &[&str] = &[
    "facebook.com/tr", "youtube.com/api/stats/ads", "youtube.com/pagead/", "youtube.com/ptracking",
    "/pagead/", "/adsbygoogle", "/ads/ga-audiences",
];
