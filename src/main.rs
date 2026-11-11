#![windows_subsystem = "windows"]

mod crypt;
mod dl;
mod media;
mod native;
mod pages;
mod profile;
mod themes;
mod vault;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    platform::windows::IconExtWindows,
    window::{Fullscreen, Icon, ResizeDirection, Window, WindowBuilder},
};
use vault::Cred;
use wry::{
    http::Response, NewWindowResponse, Rect, WebContext, WebView, WebViewBuilder, WebViewBuilderExtWindows,
    WebViewExtWindows,
};

const UI_BASE: f64 = 72.0;
const UI_BAR: f64 = 26.0;
const UI_HTML: &str = include_str!("ui.html");
const SEARCH: &str = "https://duckduckgo.com/?q=";
const HOST: &str = "owl.localhost";
const DEFAULT_PIN: &str = "1966";

const KEYS_JS: &str = r#"
addEventListener('keydown',e=>{const c=e.ctrlKey||e.metaKey;let k=null;
 if(c&&e.key==='t')k='new';else if(c&&e.key==='w')k='close';else if(c&&e.key==='l'&&!e.shiftKey)k='addr';
 else if(c&&e.key==='r'||e.key==='F5')k='reload';else if(c&&e.key==='f')k='find';else if(c&&e.key==='d')k='bookmark';
 else if(c&&e.key==='h')k='history';else if(c&&e.key==='b')k='bar';else if(e.key==='F12')k='dev';else if(e.key==='F11')k='fs';
 else if(c&&e.shiftKey&&e.key.toLowerCase()==='l')k='lock';
 else if(e.altKey&&e.key==='ArrowLeft')k='back';else if(e.altKey&&e.key==='ArrowRight')k='forward';
 else if(c&&(e.key==='='||e.key==='+'))k='zin';else if(c&&e.key==='-')k='zout';else if(c&&e.key==='0')k='z0';
 if(k){e.preventDefault();window.ipc.postMessage(JSON.stringify({cmd:'key',k}))}},true);
document.addEventListener('fullscreenchange',()=>window.ipc.postMessage(JSON.stringify({cmd:'fs',on:!!document.fullscreenElement})));
addEventListener('load',()=>{if(!location.protocol.startsWith('http')||top!==window)return;
 const l=document.querySelector('link[rel~=icon]');
 window.ipc.postMessage(JSON.stringify({cmd:'icon',href:l?l.href:location.origin+'/favicon.ico'}))});
"#;

const PW_JS: &str = r#"
(function(){
 if(top!==window||!/^https?:$/.test(location.protocol)||location.hostname==='owl.localhost')return;
 const post=o=>window.ipc.postMessage(JSON.stringify(o));
 const pwf=()=>[...document.querySelectorAll('input[type=password]')].filter(i=>i.offsetParent!==null);
 const userOf=p=>{const f=p.form||document;const c=[...f.querySelectorAll('input:not([type=password]):not([type=hidden]):not([type=checkbox]):not([type=radio]):not([type=submit]):not([type=button])')].filter(i=>i.offsetParent!==null&&(p.compareDocumentPosition(i)&2));return c.length?c[c.length-1]:null};
 let asked=false,last='';
 const set=(el,v)=>{if(!el)return;const d=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value');d.set.call(el,v);el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}))};
 const hostNow=()=>location.hostname.replace(/^www\./,'').toLowerCase();
 window.__owlFill=(u,p,h)=>{if(h!==hostNow())return;const ps=pwf()[0];if(!ps)return;set(userOf(ps),u);set(ps,p)};
 // Autofill only after a real (trusted) click on, or Tab into, a visible sign-in field. A page that
 // calls focus() on a hidden field, or fakes events, cannot trigger it.
 let gesture={t:0,el:null};
 const real=el=>{if(!el||el.offsetParent===null)return false;const r=el.getBoundingClientRect(),s=getComputedStyle(el);
  return r.width>=40&&r.height>=14&&r.bottom>0&&r.right>0&&r.top<innerHeight&&r.left<innerWidth&&parseFloat(s.opacity)>0.3&&s.visibility==='visible'};
 addEventListener('pointerdown',e=>{if(e.isTrusted)gesture={t:Date.now(),el:e.target}},true);
 addEventListener('keydown',e=>{if(e.isTrusted&&e.key==='Tab')gesture={t:Date.now(),el:null}},true);
 addEventListener('focusin',e=>{if(asked||!e.isTrusted)return;const t=e.target;if(!(t instanceof HTMLInputElement)||!real(t))return;
  if(Date.now()-gesture.t>700||(gesture.el&&gesture.el!==t&&!t.contains(gesture.el)&&!gesture.el.contains(t)))return;
  const ps=pwf();if(!ps.length)return;
  if(t.type==='password'||ps.some(p=>userOf(p)===t)){asked=true;post({cmd:'pw_query'})}},true);
 const cap=()=>{const ps=pwf().find(p=>p.value);if(!ps)return;const u=userOf(ps);const k=(u?u.value:'')+'\n'+ps.value;if(k===last)return;last=k;post({cmd:'pw_capture',user:u?u.value:'',pass:ps.value})};
 addEventListener('submit',cap,true);
 addEventListener('click',e=>{const b=e.target.closest&&e.target.closest('button,input[type=submit],[role=button]');if(b&&pwf().some(p=>p.value))cap()},true);
 addEventListener('keydown',e=>{if(e.key==='Enter'&&e.target&&e.target.type==='password')cap()},true);
})();
"#;

/// Saves scroll position and typed text (never passwords, card or one-time-code fields) when Owl
/// locks, and puts them back when the page is reloaded after unlocking.
const FORM_JS: &str = r#"
(function(){
 if(top!==window||!/^https?:$/.test(location.protocol)||location.hostname==='owl.localhost')return;
 const skip=/pass|pwd|card|cc-|cvv|cvc|ssn|otp|one-time|token|secret|\bpin\b/i;
 const fields=()=>[...document.querySelectorAll('input,textarea,select')].filter(e=>{const t=(e.type||'').toLowerCase();
  return !['password','hidden','file','submit','button','reset','image','checkbox','radio'].includes(t)&&!skip.test((e.name||'')+' '+(e.id||'')+' '+(e.autocomplete||''))});
 const key=(e,i)=>(e.name||e.id||e.type||'')+'#'+i;
 window.__owlSnap=()=>{const f=[];fields().forEach((e,i)=>{if(e.value&&e.value.length<5000)f.push([key(e,i),e.value])});
  return JSON.stringify({y:Math.round(scrollY),x:Math.round(scrollX),f:f.slice(0,60)})};
 window.__owlRestore=s=>{try{const o=JSON.parse(s),m=new Map(o.f||[]);
  fields().forEach((e,i)=>{const k=key(e,i);if(m.has(k)&&!e.value){e.value=m.get(k);e.dispatchEvent(new Event('input',{bubbles:true}));e.dispatchEvent(new Event('change',{bubbles:true}))}});
  scrollTo(o.x||0,o.y||0)}catch(x){}};
})();
"#;

/// Fingerprint protection. Makes the values trackers read most often less unique: canvas and audio
/// read-backs get invisible per-session noise, WebGL reports a generic GPU, and hardware hints are
/// rounded to common values. It lowers passive fingerprinting; it is not Tor-level anonymity.
const FP_JS: &str = r#"
(function(){
 if(location.hostname==='owl.localhost')return;
 const seed=__SEED__;
 const def=(o,k,v)=>{try{Object.defineProperty(o,k,{get:()=>v,configurable:true})}catch(e){}};
 def(Navigator.prototype,'hardwareConcurrency',4);def(Navigator.prototype,'deviceMemory',8);
 try{delete Navigator.prototype.getBattery}catch(e){}
 // canvas: perturb a sparse set of pixels by +-1 in a copy, never the page's own canvas
 const noise=(d)=>{for(let i=0;i<d.length;i+=4*37){const n=((seed^(i*2654435761))>>>7)%3-1;d[i]=Math.max(0,Math.min(255,d[i]+n))}};
 const C=HTMLCanvasElement.prototype,X=CanvasRenderingContext2D.prototype,gid=X.getImageData;
 const clone=c=>{if(c.width*c.height<256)return null;try{const k=document.createElement('canvas');k.width=c.width;k.height=c.height;const x=k.getContext('2d');x.drawImage(c,0,0);const im=gid.call(x,0,0,k.width,k.height);noise(im.data);x.putImageData(im,0,0);return k}catch(e){return null}};
 const td=C.toDataURL,tb=C.toBlob;
 C.toDataURL=function(...a){const k=clone(this);return td.apply(k||this,a)};
 C.toBlob=function(...a){const k=clone(this);return tb.apply(k||this,a)};
 X.getImageData=function(...a){const r=gid.apply(this,a);if(r.width*r.height>=256)noise(r.data);return r};
 // WebGL: hide the exact GPU model
 for(const G of [window.WebGLRenderingContext,window.WebGL2RenderingContext]){if(!G)continue;const gp=G.prototype.getParameter;
  G.prototype.getParameter=function(p){if(p===37445)return 'Google Inc.';if(p===37446)return 'ANGLE (Generic)';return gp.call(this,p)}}
 // audio: tiny noise once per buffer
 if(window.AudioBuffer){const seen=new WeakSet(),g=AudioBuffer.prototype.getChannelData;
  AudioBuffer.prototype.getChannelData=function(c){const a=g.call(this,c);if(!seen.has(a)){seen.add(a);for(let i=0;i<a.length;i+=97)a[i]+=((seed^i)%5-2)*1e-7}return a}}
})();
"#;

/// Extra, stricter normalisation: the same screen, language and plugin list for everyone.
const FP_STRICT_JS: &str = r#"
(function(){
 if(location.hostname==='owl.localhost')return;
 const def=(o,k,v)=>{try{Object.defineProperty(o,k,{get:()=>v,configurable:true})}catch(e){}};
 def(Navigator.prototype,'languages',Object.freeze(['en-US','en']));def(Navigator.prototype,'language','en-US');
 def(Navigator.prototype,'plugins',[]);def(Navigator.prototype,'mimeTypes',[]);
 def(Screen.prototype,'width',1920);def(Screen.prototype,'height',1080);def(Screen.prototype,'availWidth',1920);
 def(Screen.prototype,'availHeight',1040);def(Screen.prototype,'colorDepth',24);def(Screen.prototype,'pixelDepth',24);
})();
"#;

const BLOCKED_HOSTS: &[&str] = &[
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
const BLOCKED_PATHS: &[&str] = &[
    "facebook.com/tr", "youtube.com/api/stats/ads", "youtube.com/pagead/", "youtube.com/ptracking",
    "/pagead/", "/adsbygoogle", "/ads/ga-audiences",
];

static BLOCK: AtomicBool = AtomicBool::new(true);
static HTTPS: AtomicBool = AtomicBool::new(false);
static BLOCKED: AtomicU32 = AtomicU32::new(0);
static BLOCKED_BASE: AtomicU64 = AtomicU64::new(0);
/// Hosts the user chose to open over plain http this session.
static ALLOW_HTTP: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn is_blocked(url: &str) -> bool {
    let rest = url.split("://").nth(1).unwrap_or("");
    let host_s = host_of(url);
    let host = host_s.as_str();
    if host == HOST || host.is_empty() {
        return false;
    }
    if BLOCKED_HOSTS.iter().any(|d| host == *d || host.ends_with(&format!(".{d}"))) {
        return true;
    }
    let r = rest.trim_start_matches("www.");
    BLOCKED_PATHS.iter().any(|p| if p.starts_with('/') { rest.contains(p) } else { r.starts_with(p) })
}

/// The host a browser would actually connect to: the authority ends at / ? # or \, anything before
/// the last '@' is userinfo (so https://bank.com:x@evil.com/ is evil.com), and the port is dropped.
fn host_of(url: &str) -> String {
    let rest = url.split("://").nth(1).unwrap_or("");
    let auth = rest.split(['/', '?', '#', '\\']).next().unwrap_or("");
    let auth = auth.rsplit('@').next().unwrap_or(auth);
    let h = if auth.starts_with('[') {
        auth.split(']').next().map(|x| format!("{x}]")).unwrap_or_default()
    } else {
        auth.split(':').next().unwrap_or("").to_string()
    };
    h.trim_start_matches("www.").to_lowercase()
}

/// Local means a real loopback or private-range address or localhost, never a look-alike name.
fn is_local(url: &str) -> bool {
    let h = host_of(url);
    if h == "localhost" || h == HOST || h.ends_with(".localhost") {
        return true;
    }
    match h.trim_matches(|c| c == '[' || c == ']').parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(a)) => a.is_loopback() || a.is_private() || a.is_link_local(),
        Ok(std::net::IpAddr::V6(a)) => a.is_loopback(),
        Err(_) => false,
    }
}

fn install_blocker(wv: &WebView) {
    use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
    use webview2_com::{take_pwstr, WebResourceRequestedEventHandler};
    use windows::core::{w, PWSTR};
    unsafe {
        let core = wv.webview();
        let env = wv.environment();
        let _ = core.AddWebResourceRequestedFilter(w!("*"), COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL);
        let mut token = Default::default();
        let _ = core.add_WebResourceRequested(
            &WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
                if !BLOCK.load(Relaxed) {
                    return Ok(());
                }
                if let Some(args) = args {
                    let req = args.Request()?;
                    let mut p = PWSTR::null();
                    req.Uri(&mut p)?;
                    let u = take_pwstr(p);
                    if is_blocked(&u) {
                        let r = env.CreateWebResourceResponse(None, 403, w!("Blocked"), w!(""))?;
                        args.SetResponse(&r)?;
                        BLOCKED.fetch_add(1, Relaxed);
                    } else if !u.starts_with("http://owl.localhost") {
                        // Global Privacy Control: tell sites not to sell or share data
                        if let Ok(h) = req.Headers() {
                            let _ = h.SetHeader(w!("Sec-GPC"), w!("1"));
                        }
                    }
                }
                Ok(())
            })),
            &mut token,
        );
    }
}

fn can_nav(wv: &WebView) -> (bool, bool) {
    unsafe {
        let c = wv.webview();
        let mut b = windows::core::BOOL(0);
        let mut f = windows::core::BOOL(0);
        let _ = c.CanGoBack(&mut b);
        let _ = c.CanGoForward(&mut f);
        (b.as_bool(), f.as_bool())
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Entry {
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub t: u64,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub group: String,
}
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Dl {
    #[serde(default)]
    pub id: u32,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub url: String,
    pub status: String,
    #[serde(default)]
    pub received: i64,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub speed: f64,
    #[serde(default)]
    pub t: u64,
}
#[derive(Serialize, Deserialize, Default)]
pub struct Store {
    #[serde(default)]
    pub history: Vec<Entry>,
    #[serde(default)]
    pub bookmarks: Vec<Entry>,
    #[serde(default)]
    pub session: Vec<String>,
    #[serde(default)]
    pub downloads: Vec<Dl>,
    /// seconds spent in the foreground per site, all time
    #[serde(default)]
    pub usage: HashMap<String, u64>,
    /// seconds per local calendar day (YYYY-MM-DD)
    #[serde(default)]
    pub daily: HashMap<String, u64>,
    #[serde(default)]
    pub blocked_total: u64,
    #[serde(default)]
    pub pins: Vec<String>,
    /// set at first-run setup; kept in the encrypted store, never sent anywhere
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
    /// typed text and scroll position saved when Owl locks, restored when the page reloads
    #[serde(default)]
    pub forms: HashMap<String, String>,
}
#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "yes")]
    pub block: bool,
    #[serde(default)]
    pub https_only: bool,
    #[serde(default)]
    pub sync_dir: String,
    #[serde(default = "sys")]
    pub theme: String,
    #[serde(default)]
    pub bookmark_bar: bool,
    #[serde(default)]
    pub pin_hash: String,
    #[serde(default)]
    pub pin_salt: String,
    #[serde(default = "yes")]
    pub organize: bool,
    #[serde(default = "yes")]
    pub pw_save: bool,
    #[serde(default)]
    pub pw_never: Vec<String>,
    /// wrong-key counter and escalating lockout, kept across restarts
    #[serde(default)]
    pub strikes: u32,
    #[serde(default)]
    pub fails: u32,
    #[serde(default)]
    pub locked_until: u64,
    /// hex(DPAPI(sync passphrase)); empty = sync file stays plain text
    #[serde(default)]
    pub sync_key: String,
    /// "pin" (4 digits) or "pass" (a passphrase of 10+ characters)
    #[serde(default = "pin_mode")]
    pub pin_mode: String,
    /// key-derivation generation: below 3 means data still uses the older iterated-SHA scheme
    #[serde(default)]
    pub kdf: u32,
    /// reduce browser fingerprinting (canvas, audio, WebGL, hardware hints)
    #[serde(default = "yes")]
    pub fp: bool,
    /// strict mode also normalises screen size, languages and plugins (may change site language)
    #[serde(default)]
    pub fp_strict: bool,
    /// lock (and seal all data) after this many idle minutes; 0 = never
    #[serde(default = "autolock_default")]
    pub autolock: u32,
    /// optional proxy, e.g. socks5://127.0.0.1:9050 (Tor) or http://host:port (VPN/proxy)
    #[serde(default)]
    pub proxy: String,
    /// public sealing key (hex) and its private half wrapped by the startup key (hex)
    #[serde(default)]
    pub seal_pub: String,
    #[serde(default)]
    pub seal_priv: String,
    /// lock (and seal) when Windows locks or the PC sleeps
    #[serde(default = "yes")]
    pub lock_on_winlock: bool,
}
fn autolock_default() -> u32 {
    15
}
fn pin_mode() -> String {
    "pin".into()
}
fn sys() -> String {
    "system".into()
}
fn yes() -> bool {
    true
}
impl Default for Config {
    fn default() -> Self {
        Config {
            block: true, https_only: false, sync_dir: String::new(), theme: sys(), bookmark_bar: false,
            pin_hash: String::new(), pin_salt: String::new(), organize: true, pw_save: true, pw_never: vec![],
            strikes: 0, fails: 0, locked_until: 0, sync_key: String::new(), pin_mode: pin_mode(), kdf: 0, fp: true,
            fp_strict: false, autolock: autolock_default(), proxy: String::new(),
            seal_pub: String::new(), seal_priv: String::new(), lock_on_winlock: true,
        }
    }
}
static THEME: Mutex<String> = Mutex::new(String::new());
static RANDOM: Mutex<String> = Mutex::new(String::new());
fn pick_random() {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos() as usize / 100).unwrap_or(0) + std::process::id() as usize;
    *RANDOM.lock().unwrap() = themes::POOL[n % themes::POOL.len()].to_string();
}
/// The concrete theme in effect ("random" resolves to the palette picked for this run).
fn theme_attr() -> String {
    let t = THEME.lock().unwrap().clone();
    let known = |x: &str| matches!(x, "system" | "light" | "dark" | "black" | "navy" | "invert" | "sand");
    if t == "random" {
        let r = RANDOM.lock().unwrap().clone();
        if known(&r) { r } else { "dark".into() }
    } else if known(&t) {
        t
    } else {
        "system".into() // never echo an unknown value into HTML
    }
}
fn today() -> String {
    use windows::Win32::System::SystemInformation::GetLocalTime;
    let t = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02}", t.wYear, t.wMonth, t.wDay)
}
/// Windows' app-mode preference (the "system" theme).
fn system_is_dark() -> bool {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("reg")
        .args(["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "/v", "AppsUseLightTheme"])
        .creation_flags(0x0800_0000)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("0x0"))
        .unwrap_or(false)
}
fn wry_theme(t: &str) -> wry::Theme {
    match themes::is_dark(t) {
        Some(true) => wry::Theme::Dark,
        Some(false) => wry::Theme::Light,
        None => if system_is_dark() { wry::Theme::Dark } else { wry::Theme::Light },
    }
}
fn raise(wv: &WebView) {
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_TOP, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE};
    unsafe {
        let _ = SetWindowPos(wv.hwnd(), Some(HWND_TOP), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

/// Legacy single-round hash, still accepted so existing keys keep working (upgraded at next unlock).
fn hash_pin_legacy(salt: &str, pin: &str) -> String {
    let mut h = Sha256::new();
    h.update(salt.as_bytes());
    h.update(b":");
    h.update(pin.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
/// Generation 2 (iterated SHA-256), still accepted so existing keys keep working.
fn hash_pin_v2(salt: &str, pin: &str) -> String {
    format!("v2${}", crypt::hex(&crypt::kdf(&format!("pin|{salt}"), pin, crypt::PIN_ROUNDS)))
}
/// The vault secret used by generation 2; needed once to migrate saved passwords.
fn vault_entropy_v2(salt: &str, pin: &str) -> Vec<u8> {
    crypt::kdf(&format!("vault|{salt}"), pin, crypt::PIN_ROUNDS).to_vec()
}
/// One Argon2id derivation per unlock; every other key comes from it.
fn master_key(salt: &str, secret: &str) -> [u8; 32] {
    crypt::argon(secret, &format!("master|{salt}"), 64 * 1024)
}
fn hash_pin(salt: &str, secret: &str) -> String {
    format!("v3${}", crypt::hex(&crypt::sub(&master_key(salt, secret), "verify")))
}

/// Keys that exist only in memory while Owl runs, all derived from the startup key.
#[derive(Clone)]
struct Keys {
    vault: Vec<u8>,
    store: [u8; 32],
    /// only for opening profile files sealed by earlier builds
    profile: [u8; 32],
    /// private half of the sealing key pair; opens sealed profile data
    privk: Option<[u8; 32]>,
}
fn keys_from(m: &[u8; 32], privk: Option<[u8; 32]>) -> Keys {
    Keys { vault: crypt::sub(m, "vault").to_vec(), store: crypt::sub(m, "store"), profile: crypt::sub(m, "profile"), privk }
}
/// The sealing private key is stored wrapped by the startup key; the public key sits in the clear.
fn wrap_priv(m: &[u8; 32], p: &[u8; 32]) -> String {
    crypt::hex(&crypt::seal(crypt::MAGIC_STORE, &crypt::sub(m, "seal-priv"), &[0u8; 16], p))
}
fn unwrap_priv(m: &[u8; 32], h: &str) -> Option<[u8; 32]> {
    crypt::open(&crypt::sub(m, "seal-priv"), &crypt::unhex(h)?)?.try_into().ok()
}

#[derive(Clone)]
struct Ctx {
    store: Arc<Mutex<Store>>,
    cfg: Arc<Mutex<Config>>,
    vault: Arc<Mutex<Vec<Cred>>>,
    /// keys derived from the startup key; None until the first successful unlock
    keys: Arc<Mutex<Option<Keys>>>,
    /// (salt, key) for the encrypted sync file
    sync: Arc<Mutex<Option<([u8; 16], [u8; 32])>>>,
    /// set when the sync file is encrypted and could not be opened: never overwrite it
    sync_ro: Arc<AtomicBool>,
    dir: PathBuf,
    private: bool,
    /// settings or vault were unreadable while encrypted data exists: refuse to start over
    recovery: bool,
}
/// Write a file so a crash or power cut can never leave it half-written: temp file, flushed to
/// disk, then renamed over the original, keeping the previous version as .bak.
fn write_atomic(path: &std::path::Path, data: &[u8]) {
    use std::io::Write;
    let tmp = path.with_extension("tmp");
    let ok = (|| -> std::io::Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()
    })();
    if ok.is_err() {
        let _ = fs::remove_file(&tmp);
        return;
    }
    if path.is_file() {
        let _ = fs::copy(path, path.with_extension("bak"));
    }
    let _ = fs::rename(&tmp, path);
}
/// Load JSON, falling back to the .bak. Returns (value, damaged) where damaged means a file
/// exists but neither it nor its backup could be read.
fn load_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> (Option<T>, bool) {
    let try_one = |p: &std::path::Path| fs::read_to_string(p).ok().and_then(|s| serde_json::from_str::<T>(&s).ok());
    if let Some(v) = try_one(path) {
        return (Some(v), false);
    }
    let existed = path.exists() || path.with_extension("bak").exists();
    match try_one(&path.with_extension("bak")) {
        Some(v) => (Some(v), false),
        None => (None, existed),
    }
}
/// config.json (or its backup) exists and parses.
fn cfg_exists_ok(dir: &std::path::Path) -> bool {
    load_json::<Config>(&dir.join("config.json")).0.is_some()
}
/// Drop anything in a loaded store that could be abused if the file was tampered with
/// (for example through a shared sync folder).
fn sanitize(s: &mut Store) {
    let clean = |t: &str, n: usize| -> String { t.chars().filter(|c| !c.is_control()).take(n).collect() };
    let url_ok = |u: &str| u.starts_with("http://") || u.starts_with("https://") || u.starts_with("owl://");
    let icon_ok = |u: &str| u.is_empty() || u.starts_with("http://") || u.starts_with("https://") || u.starts_with("data:image/");
    for list in [&mut s.history, &mut s.bookmarks] {
        list.retain(|e| url_ok(&e.url) && e.url.len() < 4096);
        for e in list.iter_mut() {
            e.title = clean(&e.title, 300);
            e.group = clean(&e.group, 60);
            if !icon_ok(&e.icon) || e.icon.len() > 20000 {
                e.icon.clear();
            }
        }
    }
    s.forms.retain(|k, v| url_ok(k) && v.len() < 60_000);
    while s.forms.len() > 25 {
        let k = s.forms.keys().next().cloned().unwrap();
        s.forms.remove(&k);
    }
    s.name = clean(&s.name, 60);
    s.email = clean(&s.email, 120);
    s.session.retain(|u| url_ok(u));
    s.pins.retain(|u| url_ok(u));
    for d in s.downloads.iter_mut() {
        d.name = clean(&d.name, 260);
        if !url_ok(&d.url) {
            d.url.clear();
        }
    }
}
impl Ctx {
    fn store_path(&self) -> PathBuf {
        let c = self.cfg.lock().unwrap();
        if c.sync_dir.is_empty() {
            self.dir.join("store.json")
        } else {
            let p = PathBuf::from(&c.sync_dir);
            let _ = fs::create_dir_all(&p);
            p.join("owl-store.json")
        }
    }
    fn synced(&self) -> bool {
        !self.cfg.lock().unwrap().sync_dir.is_empty()
    }
    fn save(&self) {
        if self.private {
            return;
        }
        let s = {
            let mut g = self.store.lock().unwrap();
            g.blocked_total = BLOCKED_BASE.load(Relaxed) + BLOCKED.load(Relaxed) as u64;
            serde_json::to_string(&*g).unwrap_or_default()
        };
        let path = self.store_path();
        if self.synced() {
            if self.sync_ro.load(Relaxed) {
                return; // an encrypted file we cannot open: leave it untouched
            }
            if let Some((salt, key)) = *self.sync.lock().unwrap() {
                let _ = fs::write(&path, crypt::seal(crypt::MAGIC_SYNC, &key, &salt, s.as_bytes()));
                return;
            }
            // no passphrase: never overwrite an existing encrypted file with plain text
            if fs::read(&path).map_or(false, |d| crypt::is_sealed(&d)) {
                return;
            }
            let _ = fs::write(path, s);
            return;
        }
        // local store: always sealed with a key derived from the startup key. Before the first
        // unlock there is no key, and nothing is written.
        let key = self.keys.lock().unwrap().as_ref().map(|k| k.store);
        if let Some(key) = key {
            let salt: [u8; 16] = crypt::random(16).try_into().unwrap();
            let tmp = path.with_extension("tmp");
            if fs::write(&tmp, crypt::seal(crypt::MAGIC_STORE, &key, &salt, s.as_bytes())).is_ok() {
                let _ = fs::rename(&tmp, &path);
            }
        }
    }
    /// Read the store (the synced file is opened with the sync passphrase, the local one with the
    /// startup key). Legacy plain-text files still load and are sealed on the next save.
    fn load_store(&self) -> Option<Store> {
        let data = fs::read(self.store_path()).ok()?;
        let synced = self.synced();
        let sync_pass = || {
            let c = self.cfg.lock().unwrap();
            if c.sync_key.is_empty() { None } else { vault::decrypt(&c.sync_key, &[]) }
        };
        let plain = match crypt::magic_of(&data) {
            Some(m) if m == *crypt::MAGIC_STORE => {
                let key = self.keys.lock().unwrap().as_ref().map(|k| k.store)?;
                match crypt::open(&key, &data) {
                    Some(p) => p,
                    None => {
                        // keep the unreadable file aside so a later save cannot destroy it
                        let p = self.store_path();
                        let _ = fs::copy(&p, p.with_extension("corrupt"));
                        return None;
                    }
                }
            }
            Some(m) => {
                // synced file: MAGIC_SYNC (Argon2id) or MAGIC_SYNC1 (older iterated SHA)
                let salt = crypt::header_salt(&data)?;
                let Some(pass) = sync_pass() else {
                    self.sync_ro.store(true, Relaxed);
                    return None;
                };
                let hexs = crypt::hex(&salt);
                let key = if m == *crypt::MAGIC_SYNC1 {
                    crypt::kdf(&hexs, &pass, crypt::SYNC_ROUNDS)
                } else {
                    crypt::argon(&pass, &hexs, 64 * 1024)
                };
                match crypt::open(&key, &data) {
                    Some(p) => {
                        // from now on always save with the Argon2id key for this salt
                        let k2 = if m == *crypt::MAGIC_SYNC1 { crypt::argon(&pass, &hexs, 64 * 1024) } else { key };
                        *self.sync.lock().unwrap() = Some((salt, k2));
                        p
                    }
                    None => {
                        self.sync_ro.store(true, Relaxed);
                        return None;
                    }
                }
            }
            None => {
                if synced {
                    // plain synced file but a passphrase is set: fresh salt so the next save encrypts
                    if let Some(pass) = sync_pass() {
                        let salt: [u8; 16] = crypt::random(16).try_into().unwrap();
                        *self.sync.lock().unwrap() = Some((salt, crypt::argon(&pass, &crypt::hex(&salt), 64 * 1024)));
                    }
                }
                data
            }
        };
        let Ok(mut s) = serde_json::from_slice::<Store>(&plain) else {
            // an unreadable store file is kept aside and never silently replaced by an empty one
            let p = self.store_path();
            let _ = fs::copy(&p, p.with_extension("corrupt"));
            if synced {
                self.sync_ro.store(true, Relaxed);
            }
            return None;
        };
        sanitize(&mut s);
        Some(s)
    }
    /// Current vault secret (empty while locked).
    fn entropy(&self) -> Vec<u8> {
        self.keys.lock().unwrap().as_ref().map(|k| k.vault.clone()).unwrap_or_default()
    }
    fn enc(&self, plain: &str) -> Option<String> {
        vault::encrypt(plain, &self.entropy())
    }
    fn dec(&self, h: &str) -> Option<String> {
        vault::decrypt(h, &self.entropy())
    }
    fn pub_key(&self) -> Option<[u8; 32]> {
        crypt::unhex(&self.cfg.lock().unwrap().seal_pub)?.try_into().ok()
    }
    /// Make sure a sealing key pair exists and return the private half (needs the master key).
    fn ensure_seal_keys(&self, m: &[u8; 32]) -> Option<[u8; 32]> {
        let (pubh, privh) = {
            let c = self.cfg.lock().unwrap();
            (c.seal_pub.clone(), c.seal_priv.clone())
        };
        if !pubh.is_empty() && !privh.is_empty() {
            return unwrap_priv(m, &privh);
        }
        let (p, q) = crypt::new_keypair();
        {
            let mut c = self.cfg.lock().unwrap();
            c.seal_pub = crypt::hex(&q);
            c.seal_priv = wrap_priv(m, &p);
        }
        self.save_cfg();
        Some(p)
    }
    fn salt(&self) -> String {
        let c = self.cfg.lock().unwrap();
        if c.pin_salt.is_empty() { "default".into() } else { c.pin_salt.clone() }
    }
    /// Re-encrypt every saved password: open with `old`, seal with `new`.
    fn reseal_vault(&self, old: &[u8], new: &[u8]) {
        {
            let mut v = self.vault.lock().unwrap();
            for c in v.iter_mut() {
                if let Some(p) = vault::decrypt(&c.pass, old) {
                    if let Some(e) = vault::encrypt(&p, new) {
                        c.pass = e;
                    }
                }
            }
        }
        self.save_vault();
    }
    fn save_cfg(&self) {
        if self.private {
            return; // private windows never change saved settings
        }
        let s = serde_json::to_string(&*self.cfg.lock().unwrap()).unwrap_or_default();
        write_atomic(&self.dir.join("config.json"), s.as_bytes());
    }
    fn save_vault(&self) {
        if self.private {
            return;
        }
        let s = serde_json::to_string(&*self.vault.lock().unwrap()).unwrap_or_default();
        write_atomic(&self.dir.join("vault.json"), s.as_bytes());
    }
    fn verify_pin(&self, pin: &str) -> bool {
        let c = self.cfg.lock().unwrap();
        if c.pin_hash.is_empty() {
            crypt::ct_eq(pin, DEFAULT_PIN)
        } else if c.pin_hash.starts_with("v3$") {
            crypt::ct_eq(&hash_pin(&c.pin_salt, pin), &c.pin_hash)
        } else if c.pin_hash.starts_with("v2$") {
            crypt::ct_eq(&hash_pin_v2(&c.pin_salt, pin), &c.pin_hash)
        } else {
            crypt::ct_eq(&hash_pin_legacy(&c.pin_salt, pin), &c.pin_hash)
        }
    }
    fn extensions(&self) -> Vec<(String, String)> {
        let mut v = vec![];
        if let Ok(rd) = fs::read_dir(self.dir.join("extensions")) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().map_or(false, |x| x == "js") {
                    if let Ok(src) = fs::read_to_string(&p) {
                        v.push((p.file_name().unwrap().to_string_lossy().to_string(), src));
                    }
                }
            }
        }
        v
    }
}

pub enum Ev {
    Ui(String),
    Tab(u32, String, bool),
    Nav(u32, String),
    Go(u32, String),
    Title(u32, String),
    Load(u32, bool, String),
    NewTab(String),
    DlAsk(u32, u32, String, String),
    DlProg(u32, i64, i64),
    DlState(u32, i32),
    Fullscreen(u32, bool),
    NavFail(u32, i32),
    NavOk(u32),
    EngineUpdate,
    Audible(u32, bool),
    Snap(u32, String),
    /// (dialog id, tab id, kind 0 alert / 1 confirm / 2 prompt / 3 leave-page, page address, message, default text)
    Dialog(u32, u32, i32, String, String, String),
    EndSession,
    SessionLock,
    Tick,
    Quit,
}

struct Tab {
    id: u32,
    wv: WebView,
    title: String,
    url: String,
    icon: String,
    loading: bool,
    zoom: f64,
    pinned: bool,
    muted: bool,
    audible: bool,
}

struct Media {
    tab: u32,
    title: String,
    artist: String,
    art: String,
    playing: bool,
    pos: u64,
    dur: u64,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
fn pct(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            o.push(b as char)
        } else {
            o.push_str(&format!("%{:02X}", b))
        }
    }
    o
}
fn internal(url: &str) -> String {
    match url.strip_prefix("owl://") {
        Some(p) => format!("http://{HOST}/{p}"),
        None => url.to_string(),
    }
}
fn display(url: &str) -> String {
    url.strip_prefix(&format!("http://{HOST}/")).map(|p| format!("owl://{p}")).unwrap_or(url.to_string())
}
fn normalize(input: &str) -> String {
    let s = input.trim();
    if s.is_empty() {
        return "owl://newtab".into();
    }
    if s.starts_with("owl://") || s.contains("://") || s.starts_with("about:") || s.starts_with("data:") {
        return s.to_string();
    }
    if !s.contains(' ') && (s.contains('.') || s.starts_with("localhost")) {
        let scheme = if s.starts_with("localhost") || s.starts_with("127.") { "http" } else { "https" };
        return format!("{scheme}://{s}");
    }
    format!("{SEARCH}{}", pct(s))
}

fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut o = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                o.push(v);
                i += 3;
                continue;
            }
        }
        o.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&o).to_string()
}

fn serve(path: &str, query: &str, ctx: &Ctx) -> (u16, &'static str, String) {
    let th = theme_attr();
    let html = "text/html; charset=utf-8";
    if path == "/httpsfail" {
        let u = query.split('&').find_map(|p| p.strip_prefix("u=")).map(pct_decode).unwrap_or_default();
        return (200, html, pages::httpsfail(&u, &th));
    }
    if path == "/downloads.json" {
        let mut d = ctx.store.lock().unwrap().downloads.clone();
        d.reverse();
        return (200, "application/json", serde_json::to_string(&d).unwrap_or("[]".into()));
    }
    let s = ctx.store.lock().unwrap();
    match path {
        "/newtab" | "/" => {
            let cfg = ctx.cfg.lock().unwrap();
            let n = ctx.vault.lock().unwrap().len();
            let total = BLOCKED_BASE.load(Relaxed) + BLOCKED.load(Relaxed) as u64;
            let day = s.daily.get(&today()).copied().unwrap_or(0);
            (200, html, pages::newtab(&s, &cfg, BLOCKED.load(Relaxed), total, n, day, &th))
        }
        "/history" => (200, html, pages::history(&s, &th)),
        "/bookmarks" => (200, html, pages::bookmarks(&s, &th)),
        "/downloads" => (200, html, pages::downloads(&th)),
        "/passwords" => {
            let l: Vec<(String, String)> = ctx.vault.lock().unwrap().iter().map(|c| (c.host.clone(), c.user.clone())).collect();
            (200, html, pages::passwords(&l, &th))
        }
        "/settings" => {
            let names: Vec<String> = ctx.extensions().into_iter().map(|e| e.0).collect();
            (200, html, pages::settings(&ctx.cfg.lock().unwrap(), &names, &ctx.dir.to_string_lossy(), (&s.name, &s.email), &th))
        }
        _ => (404, html, pages::notfound(&th)),
    }
}

struct App {
    window: Window,
    ui: WebView,
    wctx: WebContext,
    tabs: Vec<Tab>,
    active: u32,
    next: u32,
    fs: bool,
    extra: f64,
    locked: bool,
    fails: u32,
    lock_until: Option<Instant>,
    pending: Option<(String, String, String)>,
    speeds: HashMap<u32, (Instant, i64, f64)>,
    last_push: Instant,
    proxy: EventLoopProxy<Ev>,
    ctx: Ctx,
    ticks: u64,
    media: Option<Media>,
    mw: Option<WebView>,
    mcollapsed: bool,
    mshown: std::cell::Cell<bool>,
    /// tab id -> the http:// URL that HTTPS-only upgraded
    upg: HashMap<u32, String>,
    last_key: Instant,
    /// downloads waiting for the user's answer
    dl_ask: Vec<(u32, Instant)>,
    recent_dl: Vec<Instant>,
    engine_watch: bool,
    /// background seal started by Lock; unlock waits for it
    sealer: Option<std::thread::JoinHandle<Result<(), String>>>,
    sealed_at_lock: bool,
    last_active: Instant,
    /// pages are being asked for scroll/typed text before a lock completes
    snap_wait: Option<(Instant, usize)>,
    /// throttles key/passphrase attempts (unlock, reveal, replace, change)
    last_pin: Instant,
    /// last run ended without a clean exit
    unclean: bool,
    /// a Windows shutdown-block reason is active until sealing finishes
    block_reason: bool,
    /// saved page state waiting for a second try once the page has rendered
    restore2: Vec<(u32, String, Instant)>,
    /// JavaScript dialogs waiting for Owl's centered modal (shown one at a time)
    dlg_queue: std::collections::VecDeque<(u32, u32, i32, String, String, String)>,
    dlg_active: Option<(u32, u32)>,
    /// the modal is open: the chrome covers the whole window
    modal: bool,
    /// tabs whose further dialogs are refused because the user chose to block them
    no_dialog: std::collections::HashSet<u32>,
    /// address given on the command line (http, https or owl scheme only), opened once at start
    start_url: Option<String>,
}

impl App {
    fn base(&self) -> f64 {
        UI_BASE + if self.ctx.cfg.lock().unwrap().bookmark_bar { UI_BAR } else { 0.0 }
    }

    fn apply_theme(&self) {
        let t = wry_theme(&theme_attr());
        let _ = self.ui.set_theme(t);
        for tab in &self.tabs {
            let _ = tab.wv.set_theme(t);
        }
    }

    /// The browser chrome lives in a strip at the bottom; pages fill everything above it.
    fn layout(&self) {
        let sc = self.window.scale_factor();
        let s: LogicalSize<f64> = self.window.inner_size().to_logical(sc);
        let r = |y: f64, h: f64| Rect {
            position: LogicalPosition::new(0.0, y).into(),
            size: LogicalSize::new(s.width, h.max(1.0)).into(),
        };
        if self.locked || self.modal {
            let _ = self.ui.set_visible(true);
            let _ = self.ui.set_bounds(r(0.0, s.height));
            for t in &self.tabs {
                let _ = t.wv.set_visible(false);
            }
            if let Some(mw) = &self.mw {
                let _ = mw.set_visible(false);
                self.mshown.set(false);
            }
            raise(&self.ui);
            return;
        }
        let base = self.base();
        let ui_h = (base + self.extra).min(s.height);
        let page_h = if self.fs { s.height } else { (s.height - base).max(1.0) };
        let _ = self.ui.set_visible(!self.fs);
        let _ = self.ui.set_bounds(r((s.height - ui_h).max(0.0), ui_h));
        for t in &self.tabs {
            let _ = t.wv.set_bounds(r(0.0, page_h));
            let _ = t.wv.set_visible(t.id == self.active);
        }
        self.layout_media(s.width, page_h);
        if self.extra > 0.0 {
            raise(&self.ui);
        }
    }

    fn media_visible(&self) -> bool {
        !self.locked
            && !self.fs
            && self.media.as_ref().map_or(false, |m| m.tab != self.active && self.tabs.iter().any(|t| t.id == m.tab))
    }

    /// The now-playing widget floats at the bottom-right of the page area, on any tab but the one playing.
    fn layout_media(&self, w: f64, page_h: f64) {
        let Some(mw) = &self.mw else { return };
        if !self.media_visible() || page_h < 160.0 {
            let _ = mw.set_visible(false);
            self.mshown.set(false);
            return;
        }
        let (ww, hh) = if self.mcollapsed { (84.0, 84.0) } else { (340.0, 84.0) };
        let _ = mw.set_bounds(Rect {
            position: LogicalPosition::new((w - ww - 16.0).max(0.0), (page_h - hh - 16.0).max(0.0)).into(),
            size: LogicalSize::new(ww, hh).into(),
        });
        let _ = mw.set_visible(true);
        raise(mw);
        if !self.mshown.replace(true) {
            let _ = mw.evaluate_script("m.enter()");
        }
    }

    fn ensure_widget(&mut self) {
        if self.mw.is_some() {
            return;
        }
        let up = self.proxy.clone();
        let r = WebViewBuilder::new()
            .with_bounds(Rect { position: LogicalPosition::new(0.0, 0.0).into(), size: LogicalSize::new(340.0, 84.0).into() })
            .with_devtools(false)
            .with_html(secure_html(media::HTML, "img-src http: https: data:"))
            .with_ipc_handler(move |req| {
                let _ = up.send_event(Ev::Ui(req.body().clone()));
            })
            .build_as_child(&self.window);
        if let Ok(w) = r {
            let _ = w.set_visible(false);
            self.mw = Some(w);
        }
    }

    fn media_push(&self) {
        if let (Some(mw), Some(m)) = (&self.mw, &self.media) {
            let v = json!({"title":m.title,"artist":m.artist,"art":m.art,"playing":m.playing,"pos":m.pos,"dur":m.dur,"min":self.mcollapsed});
            let _ = mw.evaluate_script(&format!("m.state({v})"));
        }
    }

    fn foreground(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{GetAncestor, GetForegroundWindow, GA_ROOT};
        unsafe {
            let fg = GetForegroundWindow();
            !fg.0.is_null() && fg == GetAncestor(self.ui.hwnd(), GA_ROOT)
        }
    }

    /// Show the next queued JavaScript dialog in the centered modal.
    fn next_dialog(&mut self) {
        if self.dlg_active.is_some() || self.locked {
            return;
        }
        let Some((did, tab, kind, uri, msg, def)) = self.dlg_queue.pop_front() else {
            // nothing left: give the window back unless a chrome-side modal is still open
            if self.modal && self.dlg_active.is_none() {
                // chrome-side dialogs close themselves with {cmd:'modal',on:false}
            }
            return;
        };
        self.dlg_active = Some((did, tab));
        self.modal = true;
        self.layout();
        let title = if uri.contains(HOST) { "Owl".to_string() } else { host_of(&uri) };
        let o = json!({
            "id": did, "kind": match kind { 0 => "alert", 2 => "prompt", _ => "confirm" },
            "title": title, "text": if kind == 3 { "This page asks if you want to leave it. Changes you made may not be saved.".to_string() } else { msg },
            "def": def, "block": !uri.contains(HOST)
        });
        let _ = self.ui.focus();
        let _ = self.ui.evaluate_script(&format!("owl.dialog({o})"));
    }

    /// Cancel every dialog (used when Owl locks or a tab closes).
    fn drop_dialogs(&mut self) {
        for (did, ..) in self.dlg_queue.drain(..) {
            native::dialog_answer(did, false, "");
        }
        if let Some((did, _)) = self.dlg_active.take() {
            native::dialog_answer(did, false, "");
        }
        if self.modal {
            self.modal = false;
            let _ = self.ui.evaluate_script("owl.dialogClose()");
            self.layout();
        }
    }

    /// One key attempt every two seconds, for every path that checks the key.
    fn pin_gate(&mut self) -> bool {
        if self.last_pin.elapsed() < Duration::from_secs(2) {
            return false;
        }
        self.last_pin = Instant::now();
        true
    }

    fn hwnd(&self) -> windows::Win32::Foundation::HWND {
        use tao::platform::windows::WindowExtWindows;
        windows::Win32::Foundation::HWND(self.window.hwnd() as *mut _)
    }

    /// Lock: show the lock screen at once, ask each page for its scroll position and typed text,
    /// then (finish_lock) close every page and seal the profile on disk.
    fn begin_lock(&mut self) {
        if self.locked {
            return;
        }
        self.push(); // saves the current tab list for restore
        self.drop_dialogs();
        if let Some(k) = self.ctx.keys.lock().unwrap().as_mut() {
            k.vault.clear();
        }
        self.locked = true;
        self.layout();
        self.push();
        let _ = self.ui.focus();
        if self.ctx.private {
            return;
        }
        let mut n = 0;
        for t in &self.tabs {
            if t.url.starts_with("http") {
                let p = self.proxy.clone();
                let id = t.id;
                let _ = t.wv.evaluate_script_with_callback("window.__owlSnap?window.__owlSnap():''", move |r| {
                    let _ = p.send_event(Ev::Snap(id, r));
                });
                n += 1;
            }
        }
        if n == 0 {
            self.finish_lock();
        } else {
            self.snap_wait = Some((Instant::now(), n));
        }
    }

    fn finish_lock(&mut self) {
        self.snap_wait = None;
        for id in self.dl_ask.iter().map(|(i, _)| *i).collect::<Vec<_>>() {
            self.dl_answer(id, false);
        }
        {
            let mut st = self.ctx.store.lock().unwrap();
            for d in st.downloads.iter_mut().filter(|d| d.status == "downloading" || d.status == "paused") {
                d.status = "interrupted".into();
                d.speed = 0.0;
            }
        }
        self.ctx.save();
        self.tabs.clear();
        self.media = None;
        self.mw = None;
        self.active = 0;
        let privk = self.ctx.keys.lock().unwrap().as_ref().and_then(|k| k.privk);
        if let Some(pubk) = self.ctx.pub_key() {
            let dir = self.ctx.dir.clone();
            self.sealer = Some(std::thread::spawn(move || profile::seal_profile(&dir, &pubk, privk.as_ref())));
            self.sealed_at_lock = true;
        }
    }

    /// Once a second: decay stale download speeds, count time on the active site, save now and then.
    fn tick(&mut self) {
        self.ticks += 1;
        if self.snap_wait.map_or(false, |(t, _)| t.elapsed() > Duration::from_millis(1500)) {
            self.finish_lock();
        }
        if self.block_reason && self.snap_wait.is_none() && self.sealer.as_ref().map_or(true, |h| h.is_finished()) {
            self.block_reason = false;
            unsafe {
                let _ = windows::Win32::System::Shutdown::ShutdownBlockReasonDestroy(self.hwnd());
            }
        }
        // second attempt at restoring typed text once a page has had time to render
        let due: Vec<(u32, String)> =
            self.restore2.iter().filter(|(_, _, t)| t.elapsed() > Duration::from_millis(2500)).map(|(i, s, _)| (*i, s.clone())).collect();
        self.restore2.retain(|(_, _, t)| t.elapsed() <= Duration::from_millis(2500));
        for (id, snap) in due {
            self.eval_tab(id, &format!("window.__owlRestore&&window.__owlRestore({})", serde_json::to_string(&snap).unwrap()));
        }
        let mut any = false;
        {
            let mut st = self.ctx.store.lock().unwrap();
            for d in st.downloads.iter_mut().filter(|d| d.status == "downloading") {
                any = true;
                if let Some((at, _, ema)) = self.speeds.get_mut(&d.id) {
                    if at.elapsed() > Duration::from_millis(2500) {
                        *ema = 0.0;
                        d.speed = 0.0;
                    }
                }
            }
        }
        if any {
            self.push();
        }
        if !self.ctx.private && !self.locked && self.foreground() {
            if let Some(t) = self.tabs.iter().find(|t| t.id == self.active) {
                if t.url.starts_with("http") {
                    let h = host_of(&t.url);
                    if !h.is_empty() {
                        let mut st = self.ctx.store.lock().unwrap();
                        *st.usage.entry(h).or_insert(0) += 1;
                        *st.daily.entry(today()).or_insert(0) += 1;
                    }
                }
            }
        }
        // auto-lock: after N idle minutes Owl locks, which also seals cookies and history on disk.
        // Playing media, audible tabs and running downloads count as activity.
        if !self.locked && !self.ctx.private {
            // only what the engine itself reports as sound counts; a page's own "playing" claim
            // must not be able to keep Owl unlocked forever
            let busy = self.tabs.iter().any(|t| t.audible)
                || self.ctx.store.lock().unwrap().downloads.iter().any(|d| d.status == "downloading");
            if busy || (self.foreground() && system_idle_secs() < 60) {
                self.last_active = Instant::now();
            }
            let mins = self.ctx.cfg.lock().unwrap().autolock as u64;
            if mins > 0 && self.last_active.elapsed() > Duration::from_secs(mins * 60) {
                let a = self.active;
                self.command(json!({"cmd":"lock"}), true, a);
            }
        }
        // unanswered download questions are refused after a minute
        let stale: Vec<u32> = self.dl_ask.iter().filter(|(_, t)| t.elapsed() > Duration::from_secs(60)).map(|(i, _)| *i).collect();
        for id in stale {
            self.dl_answer(id, false);
        }
        if self.ticks % 30 == 0 {
            self.ctx.save();
        }
    }

    /// Settle a download that was held for a decision.
    fn dl_answer(&mut self, id: u32, allow: bool) {
        self.dl_ask.retain(|(i, _)| *i != id);
        dl::decide(id, allow);
        self.dl_mut(id, |d| {
            d.status = if allow { "downloading".into() } else { "cancelled".into() };
            d.speed = 0.0;
        });
        let _ = self.ui.evaluate_script(&format!("owl.dropNotice('dl{id}')"));
        if !allow {
            self.speeds.remove(&id);
        }
        self.ctx.save();
        self.push();
    }

    /// A page asked to download a file. Safe, expected downloads start at once; risky types,
    /// background-tab downloads and bursts are held until the user answers.
    fn dl_incoming(&mut self, id: u32, tab: u32, url: String, path: String) {
        let name = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        self.recent_dl.retain(|t| t.elapsed() < Duration::from_secs(10));
        self.recent_dl.push(Instant::now());
        let risky = dl::risky(&path);
        let background = tab != self.active;
        let burst = self.recent_dl.len() > 3;
        let ask = (risky || background || burst) && !self.locked;
        self.ctx.store.lock().unwrap().downloads.push(Dl {
            id, name: name.clone(), path, url: url.clone(),
            status: if ask { "pending".into() } else { "downloading".into() },
            received: 0, total: 0, speed: 0.0, t: now(),
        });
        self.speeds.insert(id, (Instant::now(), 0, 0.0));
        if self.locked {
            self.dl_answer(id, false);
            return;
        }
        if ask {
            let why = if risky { "This file type can run code." } else if background { "A background tab started it." } else { "Many downloads started at once." };
            let n = json!({
                "id": format!("dl{id}"), "label": "Download",
                "text": format!("{name}  \u{00b7}  {}  \u{00b7}  {why}", host_of(&url)),
                "actions": [
                    {"t":"Block","cmd":"dl_deny","args":{"id":id}},
                    {"t":"Allow","cmd":"dl_allow","args":{"id":id},"pri":true}
                ]
            });
            self.dl_ask.push((id, Instant::now()));
            let _ = self.ui.evaluate_script(&format!("owl.notice({n})"));
            self.ctx.save();
            self.push();
        } else {
            dl::decide(id, true);
            self.ctx.save();
            self.push();
        }
    }

    fn dl_summary(&self) -> Value {
        let s = self.ctx.store.lock().unwrap();
        let act: Vec<&Dl> = s.downloads.iter().filter(|d| d.status == "downloading" || d.status == "paused").collect();
        let (r, t): (i64, i64) = act.iter().fold((0, 0), |a, d| (a.0 + d.received, a.1 + d.total));
        let speed: f64 = act.iter().filter(|d| d.status == "downloading").map(|d| d.speed).sum();
        json!({"active": act.len(), "pct": if t > 0 { r * 100 / t } else { 0 }, "speed": speed})
    }

    fn push(&mut self) {
        self.last_push = Instant::now();
        let a = self.tabs.iter().find(|t| t.id == self.active);
        let (bm, bookmarks) = {
            let s = self.ctx.store.lock().unwrap();
            (a.map_or(false, |t| s.bookmarks.iter().any(|b| b.url == t.url)), serde_json::to_value(&s.bookmarks).unwrap())
        };
        let nav = a.map(|t| can_nav(&t.wv)).unwrap_or((false, false));
        let cfg = {
            let c = self.ctx.cfg.lock().unwrap();
            json!({"block":c.block,"https":c.https_only,"theme":c.theme,"eff":theme_attr(),"bar":c.bookmark_bar,"pw_save":c.pw_save,"lockmode":c.pin_mode,"setup":c.pin_hash.is_empty() && !self.ctx.private && !self.ctx.recovery,"recovery":self.ctx.recovery})
        };
        let today_secs = self.ctx.store.lock().unwrap().daily.get(&today()).copied().unwrap_or(0);
        // with a proxy set the chrome never loads page-chosen icon addresses (they would bypass it)
        let proxied = !self.ctx.cfg.lock().unwrap().proxy.is_empty();
        let mut bookmarks = bookmarks;
        if self.locked {
            bookmarks = json!([]);
        }
        if proxied {
            if let Some(a) = bookmarks.as_array_mut() {
                for b in a.iter_mut() {
                    b["icon"] = json!("");
                }
            }
        }
        let v = json!({
            "cfg": cfg, "locked": self.locked, "dl": self.dl_summary(),
            "active": self.active, "private": self.ctx.private, "bookmarked": bm, "bookmarks": bookmarks,
            "blocked": BLOCKED.load(Relaxed), "today": today_secs,
            // while locked the chrome gets no tab titles, addresses or bookmarks at all
            "tabs": self.tabs.iter().filter(|_| !self.locked).map(|t| json!({
                "id":t.id,"title":t.title,"url":t.url,"icon":if proxied { "" } else { t.icon.as_str() },"loading":t.loading,"zoom":t.zoom,
                "pinned":t.pinned,"muted":t.muted,"audible":t.audible,
                "back": t.id==self.active && nav.0, "fwd": t.id==self.active && nav.1
            })).collect::<Vec<_>>()
        });
        let _ = self.ui.evaluate_script(&format!("owl.state({v})"));
        if !self.ctx.private && !self.locked {
            let urls: Vec<String> =
                self.tabs.iter().map(|t| t.url.clone()).filter(|u| u.starts_with("http") || u.starts_with("owl://")).collect();
            let pins: Vec<String> = self.tabs.iter().filter(|t| t.pinned).map(|t| t.url.clone()).collect();
            let changed = {
                let mut s = self.ctx.store.lock().unwrap();
                let mut ch = false;
                if s.session != urls && !urls.is_empty() {
                    s.session = urls;
                    ch = true;
                }
                if s.pins != pins {
                    s.pins = pins;
                    ch = true;
                }
                ch
            };
            if changed {
                self.ctx.save();
            }
        }
    }

    fn new_tab(&mut self, url: &str) {
        let id = self.next;
        self.next += 1;
        let p = self.proxy.clone();
        let (p1, p2, p3, p4, p5, p6, p7, p8) =
            (p.clone(), p.clone(), p.clone(), p.clone(), p.clone(), p.clone(), p.clone(), p.clone());
        let ctx = self.ctx.clone();
        let c1 = ctx.clone();
        let cfgc = ctx.cfg.clone();
        let url = internal(&normalize(url));
        // every script is injected on its own, so one broken script cannot disable the others.
        // User scripts start with a guard that throws on Owl's own pages (which hold the browser's
        // privileges); code after it never runs there, even if the script is malformed.
        let mut scripts: Vec<String> = vec![];
        let ext_scripts: Vec<String> = ctx
            .extensions()
            .into_iter()
            .map(|(_, s)| format!("if(location.hostname==='owl.localhost')throw 0;\n{s}"))
            .collect();
        let mut gpc = "try{Object.defineProperty(Navigator.prototype,'globalPrivacyControl',{get:()=>true,configurable:true})}catch(e){}".to_string();
        if ctx.cfg.lock().unwrap().fp {
            // one seed per run: values stay stable inside a session but differ between runs
            static SEED: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
            let seed = *SEED.get_or_init(|| u32::from_le_bytes(crypt::random(4).try_into().unwrap()));
            gpc.push('\n');
            gpc.push_str(&FP_JS.replace("__SEED__", &seed.to_string()));
            if ctx.cfg.lock().unwrap().fp_strict {
                gpc.push('\n');
                gpc.push_str(FP_STRICT_JS);
            }
        }
        let proxy = ctx.cfg.lock().unwrap().proxy.clone();
        let proxy_args = if proxy.is_empty() {
            String::new()
        } else {
            // hide the IP behind the proxy and stop WebRTC from bypassing it
            format!(" --proxy-server={proxy} --force-webrtc-ip-handling-policy=disable_non_proxied_udp")
        };
        scripts.extend([gpc, KEYS_JS.to_string(), PW_JS.to_string(), FORM_JS.to_string(), media::PAGE_JS.to_string()]);
        scripts.extend(ext_scripts);
        let mut b0 = WebViewBuilder::new_with_web_context(&mut self.wctx)
            .with_bounds(Rect { position: LogicalPosition::new(0.0, 0.0).into(), size: LogicalSize::new(800.0, 600.0).into() })
            .with_url(&url);
        for sc in scripts {
            b0 = b0.with_initialization_script(sc);
        }
        let b = b0
            .with_autoplay(true)
            .with_hotkeys_zoom(true)
            .with_back_forward_navigation_gestures(true)
            .with_incognito(ctx.private)
            .with_additional_browser_args(format!(
                "--disable-features=msWebOOUI,msPdfOOUI \
                 --ignore-gpu-blocklist --enable-gpu-rasterization --enable-zero-copy \
                 --autoplay-policy=no-user-gesture-required{proxy_args}"
            ))
            .with_custom_protocol("owl".into(), move |_id, req| {
                let (code, ctype, body) = serve(req.uri().path(), req.uri().query().unwrap_or(""), &c1);
                Response::builder()
                    .status(code)
                    .header("Content-Type", ctype)
                    .header("Cache-Control", "no-store")
                    // no site may embed Owl's own pages (clickjacking of settings, passwords, ...)
                    .header("X-Frame-Options", "DENY")
                    .header("Content-Security-Policy", "frame-ancestors 'none'")
                    .header("X-Content-Type-Options", "nosniff")
                    .header("Referrer-Policy", "no-referrer")
                    .body(Cow::Owned(body.into_bytes()))
                    .unwrap()
            })
            .with_navigation_handler(move |u| {
                if HTTPS.load(Relaxed)
                    && u.starts_with("http://")
                    && !is_local(&u)
                    && !ALLOW_HTTP.lock().unwrap().contains(&host_of(&u))
                {
                    let _ = p1.send_event(Ev::Go(id, format!("https://{}", &u[7..])));
                    return false;
                }
                let _ = p1.send_event(Ev::Nav(id, u));
                true
            })
            .with_document_title_changed_handler(move |t| {
                let _ = p2.send_event(Ev::Title(id, t));
            })
            .with_on_page_load_handler(move |e, u| {
                let _ = p3.send_event(Ev::Load(id, matches!(e, wry::PageLoadEvent::Started), u));
            })
            .with_new_window_req_handler(move |u, _| {
                let _ = p4.send_event(Ev::NewTab(u));
                NewWindowResponse::Deny
            })
            .with_ipc_handler(move |req| {
                // only the exact portless owl.localhost origin served by Owl itself is trusted; a
                // local listener on owl.localhost:PORT must never gain browser privileges
                let trusted = req.uri().scheme_str() == Some("http") && req.uri().host() == Some(HOST) && req.uri().port().is_none();
                let _ = p5.send_event(Ev::Tab(id, req.body().clone(), trusted));
            });
        match b.build_as_child(&self.window) {
            Ok(wv) => {
                install_blocker(&wv);
                dl::install(&wv, p6, id, move || cfgc.lock().unwrap().organize);
                media::install_audio(&wv, p7, id);
                native::install(&wv, p8, id);
                if !self.engine_watch {
                    self.engine_watch = true;
                    native::watch_engine(&wv, self.proxy.clone());
                    native::watch_engine(&self.ui, self.proxy.clone());
                }
                let _ = wv.set_theme(wry_theme(&theme_attr()));
                self.tabs.push(Tab {
                    id, wv, title: String::new(), url: display(&url), icon: String::new(), loading: true, zoom: 1.0,
                    pinned: false, muted: false, audible: false,
                });
                self.active = id;
                self.layout();
                self.push();
            }
            Err(e) => {
                // never fail silently: tell the user instead of showing an empty window
                let n = json!({"id":"tabfail","label":"Error","text":format!("Could not open a tab: {e}"),"actions":[{"t":"OK"}]});
                let _ = self.ui.evaluate_script(&format!("owl.notice({n})"));
            }
        }
    }

    fn ensure_tabs(&mut self) {
        if !self.tabs.is_empty() {
            return;
        }
        let restore: Vec<String> = if self.ctx.private { vec![] } else { self.ctx.store.lock().unwrap().session.clone() };
        if restore.is_empty() {
            // with a start address, that page is the first tab instead of the home page
            if self.start_url.is_none() {
                self.new_tab("owl://newtab");
            }
        } else {
            let mut pins = self.ctx.store.lock().unwrap().pins.clone();
            for u in restore {
                self.new_tab(&u);
                if let Some(i) = pins.iter().position(|p| *p == u) {
                    pins.remove(i);
                    if let Some(t) = self.tabs.last_mut() {
                        t.pinned = true;
                    }
                }
            }
            self.tabs.sort_by_key(|t| !t.pinned);
            self.active = self.tabs[0].id;
        }
        // an address passed on the command line opens in its own tab, once
        if let Some(u) = self.start_url.take() {
            self.new_tab(&u);
        }
        self.layout();
        self.push();
    }

    fn cur(&mut self) -> Option<&mut Tab> {
        let a = self.active;
        self.tabs.iter_mut().find(|t| t.id == a)
    }

    fn eval_tab(&self, id: u32, js: &str) {
        if let Some(t) = self.tabs.iter().find(|t| t.id == id) {
            let _ = t.wv.evaluate_script(js);
        }
    }

    fn alert(&self, id: u32, msg: &str) {
        self.eval_tab(id, &format!("setTimeout(()=>alert({}),50)", serde_json::to_string(msg).unwrap()));
    }

    fn close(&mut self, id: u32) {
        if let Some(i) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(i);
            if self.media.as_ref().map_or(false, |m| m.tab == id) {
                self.media = None;
            }
            if self.tabs.is_empty() {
                self.new_tab("owl://newtab");
                return;
            }
            if self.active == id {
                self.active = self.tabs[i.min(self.tabs.len() - 1)].id;
            }
            self.layout();
            self.push();
        }
    }

    fn set_fs(&mut self, on: bool) {
        self.fs = on;
        self.window.set_fullscreen(if on { Some(Fullscreen::Borderless(None)) } else { None });
        self.layout();
    }

    fn zoom(&mut self, d: f64) {
        if let Some(t) = self.cur() {
            t.zoom = if d == 0.0 { 1.0 } else { (t.zoom + d).clamp(0.25, 5.0) };
            let _ = t.wv.zoom(t.zoom);
        }
        self.push();
    }

    fn toggle_bookmark(&mut self, group: &str) {
        if let Some((url, title, icon)) = self.cur().map(|t| (t.url.clone(), t.title.clone(), t.icon.clone())) {
            {
                let mut s = self.ctx.store.lock().unwrap();
                if let Some(i) = s.bookmarks.iter().position(|b| b.url == url) {
                    s.bookmarks.remove(i);
                } else {
                    s.bookmarks.push(Entry { url, title, t: now(), icon, group: group.to_string() });
                }
            }
            self.ctx.save();
            self.push();
        }
    }

    fn dl_mut<R>(&self, id: u32, f: impl FnOnce(&mut Dl) -> R) -> Option<R> {
        self.ctx.store.lock().unwrap().downloads.iter_mut().find(|d| d.id == id).map(f)
    }

    fn command(&mut self, v: Value, trusted: bool, id: u32) {
        let cmd = v["cmd"].as_str().unwrap_or("");
        // Web pages can reach window.ipc too, so they get a short allow-list. Shortcuts that could
        // be abused (devtools, lock) and fullscreen entry are trusted-UI only; fullscreen for a page
        // comes from WebView2's own event, which a page cannot forge.
        let kk = v["k"].as_str().unwrap_or("");
        let safe_key = matches!(
            kk,
            "addr" | "find" | "reload" | "back" | "forward" | "zin" | "zout" | "z0" | "new" | "close" | "bookmark" | "history" | "bar" | "fs"
        );
        if !trusted && !(matches!(cmd, "icon" | "pw_query" | "pw_capture" | "media") || (cmd == "key" && safe_key)) {
            return;
        }
        // a page message must come from a live tab, and shortcuts and password actions only from
        // the tab the user is looking at (a background tab or hidden frame cannot drive them)
        if !trusted {
            if !self.tabs.iter().any(|t| t.id == id) {
                return;
            }
            if matches!(cmd, "key" | "pw_query" | "pw_capture") && id != self.active {
                return;
            }
        }
        // private windows never change anything that is saved
        if self.ctx.private
            && matches!(
                cmd,
                "set" | "set_sync" | "set_sync_pass" | "set_num" | "set_proxy" | "set_profile" | "set_pin" | "setup" | "pw_add" | "pw_save"
                    | "pw_never" | "pw_delete" | "clear_usage"
            )
        {
            return;
        }
        if !trusted && cmd == "key" {
            match kk {
                "new" | "close" => {
                    if self.last_key.elapsed() < Duration::from_millis(400) || (kk == "new" && self.tabs.len() >= 40) {
                        return;
                    }
                    self.last_key = Instant::now();
                }
                "fs" if !self.fs => return, // a page may only leave fullscreen, never enter it
                _ => {}
            }
        }
        if self.locked && !matches!(cmd, "ready" | "unlock" | "setup" | "win_min" | "win_close" | "drag" | "resize" | "ui_extra") {
            return;
        }
        let s = |k: &str| v[k].as_str().unwrap_or("").to_string();
        let did = v["id"].as_u64().unwrap_or(0) as u32;
        match cmd {
            "ready" => self.push(),
            // ----- window chrome -----
            "win_min" => self.window.set_minimized(true),
            "win_max" => {
                let m = self.window.is_maximized();
                self.window.set_maximized(!m)
            }
            "win_close" => {
                let _ = self.proxy.send_event(Ev::Quit);
            }
            "drag" => {
                let _ = self.window.drag_window();
            }
            "resize" => {
                let d = match s("dir").as_str() {
                    "West" => ResizeDirection::West,
                    "East" => ResizeDirection::East,
                    _ => ResizeDirection::South,
                };
                let _ = self.window.drag_resize_window(d);
            }
            // ----- lock -----
            "unlock" => {
                let t = now();
                let wait = self.ctx.cfg.lock().unwrap().locked_until.saturating_sub(t);
                if wait > 0 {
                    let m = format!("Too many attempts. Try again in {wait} seconds.");
                    let _ = self.ui.evaluate_script(&format!("owl.lockFail({})", serde_json::to_string(&m).unwrap()));
                    return;
                }
                let pin = s("pin");
                if self.ctx.recovery {
                    let m = "Owl's settings file is damaged or missing, so it cannot safely open your encrypted data. Nothing was changed. Restore config.json from config.bak in %LOCALAPPDATA%\\Owl, then start Owl again.";
                    let _ = self.ui.evaluate_script(&format!("owl.lockFail({})", serde_json::to_string(m).unwrap()));
                    return;
                }
                if !v["internal"].as_bool().unwrap_or(false) && !self.pin_gate() {
                    let _ = self.ui.evaluate_script("owl.lockFail('Slow down: one attempt every two seconds.')");
                    return;
                }
                if self.ctx.verify_pin(&pin) {
                    let salt = self.ctx.salt();
                    let m = master_key(&salt, &pin);
                    let privk = self.ctx.ensure_seal_keys(&m);
                    let k = keys_from(&m, privk);
                    let migrate = {
                        let mut c = self.ctx.cfg.lock().unwrap();
                        // move older key hashes to the Argon2id generation
                        if !c.pin_hash.is_empty() && !c.pin_hash.starts_with("v3$") {
                            c.pin_hash = format!("v3${}", crypt::hex(&crypt::sub(&m, "verify")));
                        }
                        let mg = c.kdf < 3;
                        c.kdf = 3;
                        c.fails = 0;
                        c.strikes = 0;
                        c.locked_until = 0;
                        mg
                    };
                    self.ctx.save_cfg();
                    if migrate {
                        // saved passwords were sealed with the older derivation: re-seal once
                        self.ctx.reseal_vault(&vault_entropy_v2(&salt, &pin), &k.vault);
                    }
                    // an unlock during a pending lock simply cancels it
                    self.snap_wait = None;
                    // a seal started by Lock, shutdown or launch may still be running: let it finish
                    if let Some(h) = self.sealer.take() {
                        if let Ok(Err(e)) = h.join() {
                            let n = json!({"id":"sealfail","label":"Warning","text":format!("Your data could not be fully encrypted earlier: {e}. It is still intact."),"actions":[{"t":"OK"}]});
                            let _ = self.ui.evaluate_script(&format!("owl.notice({n})"));
                        }
                    }
                    let first = self.ctx.keys.lock().unwrap().is_none();
                    let reopen = first || self.sealed_at_lock;
                    let (legacy, privk) = (k.profile, k.privk);
                    if reopen && !self.ctx.private {
                        // restore cookies, logins and site data before any tab can open. If this
                        // fails the sealed file is kept untouched and Owl stays locked: opening
                        // tabs on an empty profile could lead to the real data being replaced.
                        if let Err(e) = profile::unseal_profile(&self.ctx.dir, &legacy, privk.as_ref()) {
                            let m = format!("Your saved data could not be restored ({e}). Nothing was changed or deleted. Try again, or contact support with the file sealed.owl kept in %LOCALAPPDATA%\\Owl\\profile.");
                            let _ = self.ui.evaluate_script(&format!("owl.lockFail({})", serde_json::to_string(&m).unwrap()));
                            return;
                        }
                    }
                    *self.ctx.keys.lock().unwrap() = Some(k);
                    if reopen && !self.ctx.private {
                        if first && !self.ctx.synced() {
                            open_store(&self.ctx);
                        }
                        self.sealed_at_lock = false;
                    }
                    if self.unclean {
                        self.unclean = false;
                        let n = json!({"id":"unclean","label":"Notice","text":"Owl did not close normally last time. Your browsing data was encrypted again as soon as Owl started.","actions":[{"t":"OK"}]});
                        let _ = self.ui.evaluate_script(&format!("owl.notice({n})"));
                    }
                    self.last_active = Instant::now();
                    self.locked = false;
                    self.ensure_tabs();
                    self.layout();
                    self.push();
                } else {
                    let msg = {
                        let mut c = self.ctx.cfg.lock().unwrap();
                        c.fails += 1;
                        if c.fails >= 5 {
                            c.fails = 0;
                            c.strikes += 1;
                            let secs = [30u64, 120, 600, 3600][(c.strikes as usize - 1).min(3)];
                            c.locked_until = t + secs;
                            format!("Too many attempts. Try again in {secs} seconds.")
                        } else {
                            "Wrong key.".to_string()
                        }
                    };
                    self.ctx.save_cfg();
                    let _ = self.ui.evaluate_script(&format!("owl.lockFail({})", serde_json::to_string(&msg).unwrap()));
                }
            }
            "lock" => self.begin_lock(),
            "set_pin" => {
                let (old, new) = (s("old"), s("new"));
                // only passphrases: a 4-digit key can be guessed offline in minutes
                let is_pin = false;
                let is_pass = new.chars().count() >= 10 && new.chars().count() <= 128;
                let internal = v["internal"].as_bool().unwrap_or(false);
                let msg = if !internal && !self.pin_gate() {
                    "Slow down: one attempt every two seconds."
                } else if !self.ctx.verify_pin(&old) {
                    "Your current key is wrong."
                } else if !is_pass {
                    "Use a passphrase of at least 10 characters."
                } else {
                    let salt = crypt::hex(&crypt::random(16));
                    let m = master_key(&salt, &new);
                    let old_priv = self.ctx.keys.lock().unwrap().as_ref().and_then(|k| k.privk);
                    let nk = keys_from(&m, old_priv);
                    // re-encrypt saved passwords for the new key before the old one is forgotten
                    let old_ent = self.ctx.entropy();
                    self.ctx.reseal_vault(&old_ent, &nk.vault);
                    *self.ctx.keys.lock().unwrap() = Some(nk);
                    {
                        let mut c = self.ctx.cfg.lock().unwrap();
                        c.pin_salt = salt;
                        c.pin_hash = format!("v3${}", crypt::hex(&crypt::sub(&m, "verify")));
                        c.pin_mode = if is_pin { "pin".into() } else { "pass".into() };
                        c.kdf = 3;
                        c.fails = 0;
                        c.strikes = 0;
                        c.locked_until = 0;
                        // the sealing private key must stay openable with the new key
                        if let Some(p) = old_priv {
                            c.seal_priv = wrap_priv(&m, &p);
                        }
                    }
                    self.ctx.save_cfg();
                    self.ctx.save(); // history and bookmarks are re-sealed with the new key
                    self.push();
                    if is_pin { "Key changed. A passphrase would be much stronger." } else { "Passphrase set." }
                };
                if !v["quiet"].as_bool().unwrap_or(false) {
                    self.alert(id, msg);
                }
            }
            // ----- first-run setup: replaces the default key with the user's own -----
            "setup" => {
                let fail = |app: &App, m: &str| {
                    let _ = app.ui.evaluate_script(&format!("owl.lockFail({})", serde_json::to_string(m).unwrap()));
                };
                let (name, email, pass) = (s("name").trim().to_string(), s("email").trim().to_string(), s("pass"));
                if !self.ctx.cfg.lock().unwrap().pin_hash.is_empty() {
                    return;
                }
                let at = email.find('@');
                if name.is_empty() || name.chars().count() > 60 {
                    return fail(self, "Enter your name.");
                }
                if email.len() > 120 || at.map_or(true, |i| i == 0 || !email[i + 1..].contains('.') || email.contains(' ')) {
                    return fail(self, "Enter a valid email address.");
                }
                if pass.chars().count() < 10 || pass.chars().count() > 128 {
                    return fail(self, "Choose a password of at least 10 characters.");
                }
                if pass == DEFAULT_PIN {
                    return fail(self, "Choose a new password, not the default.");
                }
                // open with the default key so sealed data and saved passwords load, then replace it
                self.command(json!({"cmd":"unlock","pin":DEFAULT_PIN,"internal":true}), true, id);
                if self.locked {
                    return fail(self, "Could not start setup. Try again.");
                }
                self.command(json!({"cmd":"set_pin","old":DEFAULT_PIN,"new":pass,"quiet":true,"internal":true}), true, id);
                {
                    let mut st = self.ctx.store.lock().unwrap();
                    st.name = name.chars().filter(|c| !c.is_control()).collect();
                    st.email = email.chars().filter(|c| !c.is_control()).collect();
                }
                self.ctx.save();
                self.push();
            }
            "set_profile" => {
                {
                    let mut st = self.ctx.store.lock().unwrap();
                    st.name = s("name").chars().filter(|c| !c.is_control()).take(60).collect();
                    st.email = s("email").chars().filter(|c| !c.is_control()).take(120).collect();
                }
                self.ctx.save();
            }
            "set_num" => {
                let n = v["val"].as_u64().unwrap_or(0) as u32;
                if s("key") == "autolock" && [0, 5, 15, 30, 60].contains(&n) {
                    self.ctx.cfg.lock().unwrap().autolock = n;
                    self.ctx.save_cfg();
                }
            }
            "set_proxy" => {
                let p = s("proxy").trim().to_string();
                let ok = p.is_empty()
                    || (["http://", "https://", "socks4://", "socks5://"].iter().any(|x| p.starts_with(x))
                        && p.len() < 100
                        && p.split("://").nth(1).map_or(false, |r| !r.is_empty() && r.chars().all(|c| c.is_ascii_alphanumeric() || ".-:[]".contains(c))));
                if ok {
                    self.ctx.cfg.lock().unwrap().proxy = p;
                    self.ctx.save_cfg();
                    self.alert(id, "Saved. Restart Owl to apply.");
                } else {
                    self.alert(id, "Use a proxy like socks5://127.0.0.1:9050 or http://host:8080.");
                }
            }
            // ----- passwords -----
            "pw_query" => {
                let url = self.tabs.iter().find(|t| t.id == id).map(|t| t.url.clone()).unwrap_or_default();
                let h = host_of(&url);
                let cred = self.ctx.vault.lock().unwrap().iter().filter(|c| c.host == h).max_by_key(|c| c.t).cloned();
                if let Some(c) = cred {
                    if let Some(p) = self.ctx.dec(&c.pass) {
                        let js = format!(
                            "window.__owlFill&&window.__owlFill({},{},{})",
                            serde_json::to_string(&c.user).unwrap(),
                            serde_json::to_string(&p).unwrap(),
                            serde_json::to_string(&h).unwrap()
                        );
                        self.eval_tab(id, &js);
                    }
                }
            }
            "pw_capture" => {
                let (user, pass) = (s("user"), s("pass"));
                let url = self.tabs.iter().find(|t| t.id == id).map(|t| t.url.clone()).unwrap_or_default();
                let h = host_of(&url);
                let (on, never) = {
                    let c = self.ctx.cfg.lock().unwrap();
                    (c.pw_save, c.pw_never.contains(&h))
                };
                if self.ctx.private || !on || never || h.is_empty() || pass.is_empty() || url.starts_with("owl://") {
                    return;
                }
                let same = self.ctx.vault.lock().unwrap().iter().any(|c| c.host == h && c.user == user && self.ctx.dec(&c.pass).as_deref() == Some(&pass));
                if same {
                    return;
                }
                self.pending = Some((h.clone(), user.clone(), pass));
                let _ = self.ui.evaluate_script(&format!("owl.savePrompt({})", json!({"host":h,"user":user})));
            }
            "pw_save" => {
                if let Some((h, u, p)) = self.pending.take() {
                    if let Some(enc) = self.ctx.enc(&p) {
                        let mut vlt = self.ctx.vault.lock().unwrap();
                        vlt.retain(|c| !(c.host == h && c.user == u));
                        vlt.push(Cred { host: h, user: u, pass: enc, t: now() });
                        drop(vlt);
                        self.ctx.save_vault();
                    }
                }
            }
            "pw_never" => {
                if let Some((h, _, _)) = self.pending.take() {
                    self.ctx.cfg.lock().unwrap().pw_never.push(h);
                    self.ctx.save_cfg();
                }
            }
            "pw_dismiss" => self.pending = None,
            "pw_reveal" => {
                let i = v["i"].as_u64().unwrap_or(0) as usize;
                if !self.pin_gate() {
                    self.alert(id, "Slow down: one attempt every two seconds.");
                    return;
                }
                if !self.ctx.verify_pin(&s("pin")) {
                    self.alert(id, "Wrong key.");
                    return;
                }
                let pw = self.ctx.vault.lock().unwrap().get(i).and_then(|c| self.ctx.dec(&c.pass));
                if let Some(pw) = pw {
                    self.eval_tab(id, &format!("revealed({i},{})", serde_json::to_string(&pw).unwrap()));
                }
            }
            "pw_delete" => {
                let i = v["i"].as_u64().unwrap_or(0) as usize;
                {
                    let mut vlt = self.ctx.vault.lock().unwrap();
                    if i < vlt.len() {
                        vlt.remove(i);
                    }
                }
                self.ctx.save_vault();
            }
            // ----- downloads -----
            "dl_pause" => {
                dl::pause(did);
                self.dl_mut(did, |d| d.status = "paused".into());
            }
            "dl_resume" => {
                let ok = dl::resume(did);
                self.dl_mut(did, |d| d.status = if ok { "downloading".into() } else { "failed".into() });
            }
            "dl_allow" => self.dl_answer(did, true),
            "dl_deny" => self.dl_answer(did, false),
            "http_allow" => {
                let u = s("url");
                if u.starts_with("http://") {
                    ALLOW_HTTP.lock().unwrap().push(host_of(&u));
                    self.upg.remove(&id);
                    if let Some(t) = self.cur() {
                        let _ = t.wv.load_url(&u);
                    }
                }
            }
            "restart" => {
                if let Ok(exe) = std::env::current_exe() {
                    let mut c = std::process::Command::new(exe);
                    if self.ctx.private {
                        c.arg("--private");
                    }
                    let _ = c.spawn();
                }
                let _ = self.proxy.send_event(Ev::Quit);
            }
            "set_sync_pass" => {
                let pass = s("pass");
                // removing the passphrase while a sync folder is set would write the store in plain text
                if pass.is_empty() && self.ctx.synced() {
                    self.alert(id, "Clear the sync folder first; a synced file is always encrypted.");
                    return;
                }
                {
                    let mut c = self.ctx.cfg.lock().unwrap();
                    c.sync_key = if pass.is_empty() { String::new() } else { vault::encrypt(&pass, &[]).unwrap_or_default() };
                }
                self.ctx.save_cfg();
                if pass.is_empty() {
                    *self.ctx.sync.lock().unwrap() = None;
                } else {
                    let salt: [u8; 16] = crypt::random(16).try_into().unwrap();
                    let key = crypt::argon(&pass, &crypt::hex(&salt), 64 * 1024);
                    *self.ctx.sync.lock().unwrap() = Some((salt, key));
                    self.ctx.sync_ro.store(false, Relaxed);
                }
                self.ctx.save();
            }
            "dl_cancel" => {
                dl::cancel(did);
                dl::decide(did, false);
                self.dl_ask.retain(|(i, _)| *i != did);
                self.dl_mut(did, |d| {
                    d.status = "cancelled".into();
                    d.speed = 0.0;
                });
                self.ctx.save();
                self.push();
            }
            "dl_open" | "dl_reveal" => {
                use windows::core::{w, HSTRING, PCWSTR};
                use windows::Win32::UI::Shell::ShellExecuteW;
                use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
                let p = self.dl_mut(did, |d| if d.status == "done" || cmd == "dl_reveal" { d.path.clone() } else { String::new() });
                // only files inside the Downloads folder, and never through a shell: a file name
                // like "a&calc.pdf" must not be able to inject a command. The resolved (canonical)
                // path is the one opened, so a swapped link cannot redirect it.
                let p = p.filter(|p| !p.is_empty()).and_then(|p| {
                    if dl::inside_root(&p) {
                        std::path::PathBuf::from(&p).canonicalize().ok().map(|c| c.to_string_lossy().trim_start_matches(r"\\?\").to_string())
                    } else {
                        None
                    }
                });
                if let Some(p) = p {
                    unsafe {
                        if cmd == "dl_open" && !dl::risky(&p) {
                            let f = HSTRING::from(p.as_str());
                            let _ = ShellExecuteW(None, w!("open"), &f, PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
                        } else {
                            let arg = HSTRING::from(format!("/select,\"{p}\""));
                            let _ = ShellExecuteW(None, w!("open"), w!("explorer.exe"), &arg, PCWSTR::null(), SW_SHOWNORMAL);
                        }
                    }
                }
            }
            "dl_retry" => {
                if let Some(u) = self.dl_mut(did, |d| d.url.clone()) {
                    self.new_tab(&u);
                }
            }
            "dl_remove" => {
                self.ctx.store.lock().unwrap().downloads.retain(|d| d.id != did || d.status == "downloading" || d.status == "paused" || d.status == "pending");
                self.ctx.save();
            }
            "dl_clear" => {
                self.ctx.store.lock().unwrap().downloads.retain(|d| d.status == "downloading" || d.status == "paused" || d.status == "pending");
                self.ctx.save();
                self.push();
            }
            "pw_add" => {
                let raw = s("host").to_lowercase();
                let h = host_of(&if raw.contains("://") { raw.clone() } else { format!("x://{raw}") });
                let (user, pass) = (s("user"), s("pass"));
                if self.ctx.private || h.is_empty() || pass.is_empty() {
                    return;
                }
                let exists = self.ctx.vault.lock().unwrap().iter().any(|c| c.host == h && c.user == user);
                if exists && (!self.pin_gate() || !self.ctx.verify_pin(&s("pin"))) {
                    self.alert(id, "Wrong key. The saved password was not replaced.");
                    return;
                }
                if let Some(enc) = self.ctx.enc(&pass) {
                    let mut vlt = self.ctx.vault.lock().unwrap();
                    vlt.retain(|c| !(c.host == h && c.user == user));
                    vlt.push(Cred { host: h, user, pass: enc, t: now() });
                    drop(vlt);
                    self.ctx.save_vault();
                }
            }
            "clear_usage" => {
                {
                    let mut st = self.ctx.store.lock().unwrap();
                    st.usage.clear();
                    st.daily.clear();
                }
                self.ctx.save();
            }
            // ----- tab context menu -----
            "tab_reload" => {
                if let Some(t) = self.tabs.iter().find(|t| t.id == did) {
                    let _ = t.wv.reload();
                }
            }
            "tab_dup" => {
                if let Some(i) = self.tabs.iter().position(|t| t.id == did) {
                    let u = self.tabs[i].url.clone();
                    self.new_tab(&u);
                    if self.tabs.len() > 1 {
                        let pinned = self.tabs.iter().filter(|t| t.pinned).count();
                        let at = (i + 1).max(pinned);
                        if at < self.tabs.len() - 1 {
                            let t = self.tabs.pop().unwrap();
                            self.tabs.insert(at, t);
                        }
                    }
                    self.layout();
                    self.push();
                }
            }
            "tab_pin" => {
                if let Some(i) = self.tabs.iter().position(|t| t.id == did) {
                    let mut t = self.tabs.remove(i);
                    t.pinned = !t.pinned;
                    let k = self.tabs.iter().filter(|x| x.pinned).count();
                    self.tabs.insert(k, t);
                    self.push();
                }
            }
            "tab_mute" => {
                if let Some(t) = self.tabs.iter_mut().find(|t| t.id == did) {
                    t.muted = !t.muted;
                    media::set_muted(&t.wv, t.muted);
                    self.push();
                }
            }
            "close_others" => {
                let gone: Vec<u32> = self.tabs.iter().filter(|t| t.id != did && !t.pinned).map(|t| t.id).collect();
                for g in gone {
                    if self.media.as_ref().map_or(false, |m| m.tab == g) {
                        self.media = None;
                    }
                    self.tabs.retain(|t| t.id != g);
                }
                if !self.tabs.iter().any(|t| t.id == self.active) {
                    self.active = did;
                }
                self.layout();
                self.push();
            }
            // ----- now playing -----
            "media" => {
                let title: String = s("title").chars().take(160).collect();
                if title.is_empty() {
                    if self.media.as_ref().map_or(false, |m| m.tab == id) {
                        self.media = None;
                        self.layout();
                    }
                    return;
                }
                let playing = v["playing"].as_bool().unwrap_or(false);
                if self.media.as_ref().map_or(false, |m| m.tab != id && m.playing && !playing) {
                    return;
                }
                let art = s("art");
                let proxied = !self.ctx.cfg.lock().unwrap().proxy.is_empty();
                let art = if !proxied && (art.starts_with("https://") || art.starts_with("http://")) && art.len() < 2000 { art } else { String::new() };
                self.media = Some(Media {
                    tab: id,
                    title,
                    artist: s("artist").chars().take(120).collect(),
                    art,
                    playing,
                    pos: v["pos"].as_u64().unwrap_or(0),
                    dur: v["dur"].as_u64().unwrap_or(0),
                });
                self.ensure_widget();
                self.media_push();
                self.layout();
            }
            "media_toggle" | "media_next" | "media_prev" => {
                if let Some(tab) = self.media.as_ref().map(|m| m.tab) {
                    let js = match cmd {
                        "media_toggle" => media::js_toggle(),
                        "media_next" => media::js_next(),
                        _ => media::js_prev(),
                    };
                    self.eval_tab(tab, &js);
                    if cmd == "media_toggle" {
                        if let Some(m) = self.media.as_mut() {
                            m.playing = !m.playing;
                        }
                        self.media_push();
                    }
                }
            }
            "media_goto" => {
                if let Some(tab) = self.media.as_ref().map(|m| m.tab) {
                    self.active = tab;
                    self.layout();
                    self.push();
                }
            }
            "media_collapse" => {
                self.mcollapsed = !self.mcollapsed;
                self.media_push();
                self.layout();
            }
            "media_close" => {
                self.media = None;
                self.layout();
            }
            // ----- browsing -----
            "navigate" => {
                let u = internal(&normalize(&s("url")));
                if let Some(t) = self.cur() {
                    let _ = t.wv.load_url(&u);
                }
            }
            "new_tab" => {
                let u = if s("url").is_empty() { "owl://newtab".to_string() } else { s("url") };
                self.new_tab(&u)
            }
            "close" => {
                let cid = v["id"].as_u64().map(|x| x as u32).unwrap_or(self.active);
                self.close(cid)
            }
            "switch" => {
                self.active = v["id"].as_u64().unwrap_or(0) as u32;
                self.layout();
                self.push();
            }
            "move" => {
                let (f, t) = (v["from"].as_u64().unwrap_or(0) as u32, v["to"].as_u64().unwrap_or(0) as u32);
                if let (Some(a), Some(b)) = (self.tabs.iter().position(|x| x.id == f), self.tabs.iter().position(|x| x.id == t)) {
                    let tab = self.tabs.remove(a);
                    self.tabs.insert(b, tab);
                    self.push();
                }
            }
            "back" => {
                if let Some(t) = self.cur() {
                    unsafe {
                        let _ = t.wv.webview().GoBack();
                    }
                }
            }
            "forward" => {
                if let Some(t) = self.cur() {
                    unsafe {
                        let _ = t.wv.webview().GoForward();
                    }
                }
            }
            "reload" => {
                if let Some(t) = self.cur() {
                    let _ = t.wv.reload();
                }
            }
            "stop" => {
                if let Some(t) = self.cur() {
                    unsafe {
                        let _ = t.wv.webview().Stop();
                    }
                }
            }
            "zoom" => self.zoom(v["d"].as_f64().unwrap_or(0.0)),
            "print" => {
                if let Some(t) = self.cur() {
                    let _ = t.wv.print();
                }
            }
            "devtools" => {
                if let Some(t) = self.cur() {
                    t.wv.open_devtools();
                }
            }
            "find" => {
                let q = s("q").replace('\\', "\\\\").replace('\'', "\\'");
                let back = v["back"].as_bool().unwrap_or(false);
                if let Some(t) = self.cur() {
                    let _ = t.wv.evaluate_script(&if q.is_empty() {
                        "getSelection().removeAllRanges()".to_string()
                    } else {
                        format!("window.find('{q}',false,{back},true)")
                    });
                }
            }
            "bookmark" => {
                let g = if s("group").is_empty() { "General".to_string() } else { s("group") };
                self.toggle_bookmark(&g)
            }
            "bm_group" => {
                let (u, g) = (s("url"), s("group"));
                if let Some(b) = self.ctx.store.lock().unwrap().bookmarks.iter_mut().find(|b| b.url == u) {
                    b.group = g;
                }
                self.ctx.save();
                self.push();
            }
            "bm_remove" => {
                let u = s("url");
                self.ctx.store.lock().unwrap().bookmarks.retain(|b| b.url != u);
                self.ctx.save();
                self.push();
            }
            "icon" => {
                let href = s("href");
                let same = self.tabs.iter().find(|t| t.id == id).map_or(true, |t| t.icon == href);
                // with a proxy set, never let the chrome fetch an address the page chose: that
                // request would leave directly and reveal the real IP
                let proxied = !self.ctx.cfg.lock().unwrap().proxy.is_empty();
                if !same && !proxied && (href.starts_with("https://") || href.starts_with("http://") || href.starts_with("data:image/")) && href.len() < 20_000 {
                    let mut url = String::new();
                    if let Some(t) = self.tabs.iter_mut().find(|t| t.id == id) {
                        t.icon = href.clone();
                        url = t.url.clone();
                    }
                    if !self.ctx.private {
                        let mut st = self.ctx.store.lock().unwrap();
                        for e in st.history.iter_mut().rev().take(5).filter(|e| e.url == url) {
                            e.icon = href.clone();
                        }
                        for b in st.bookmarks.iter_mut().filter(|b| b.url == url) {
                            b.icon = href.clone();
                        }
                        drop(st);
                        self.ctx.save();
                    }
                    self.push();
                }
            }
            "clear_history" => {
                self.ctx.store.lock().unwrap().history.clear();
                self.ctx.save();
            }
            "set" => {
                let on = v["val"].as_bool().unwrap_or(false);
                let key = s("key");
                {
                    let mut c = self.ctx.cfg.lock().unwrap();
                    match key.as_str() {
                        "block" => {
                            c.block = on;
                            BLOCK.store(on, Relaxed)
                        }
                        "https_only" => {
                            c.https_only = on;
                            HTTPS.store(on, Relaxed)
                        }
                        "bar" => c.bookmark_bar = on,
                        "organize" => c.organize = on,
                        "fp" => c.fp = on,
                        "fp_strict" => c.fp_strict = on,
                        "lock_win" => c.lock_on_winlock = on,
                        "pw_save" => c.pw_save = on,
                        "theme" if !matches!(s("val").as_str(), "system" | "light" | "dark" | "black" | "navy" | "invert" | "sand" | "random") => {}
                        "theme" => {
                            c.theme = s("val");
                            *THEME.lock().unwrap() = c.theme.clone();
                            if c.theme == "random" {
                                pick_random();
                            }
                        }
                        _ => {}
                    }
                }
                self.ctx.save_cfg();
                if key == "theme" {
                    self.apply_theme();
                    for t in &self.tabs {
                        // the settings page recolours itself live; other internal pages reload
                        if t.url.starts_with("owl://") && !(t.id == self.active && t.url == "owl://settings") {
                            let _ = t.wv.reload();
                        }
                    }
                }
                self.layout();
                self.push();
            }
            "dlg_answer" => {
                let ok = v["ok"].as_bool().unwrap_or(false);
                if let Some((did, tab)) = self.dlg_active {
                    if did == v["id"].as_u64().unwrap_or(0) as u32 {
                        native::dialog_answer(did, ok, &s("text"));
                        if v["block"].as_bool().unwrap_or(false) {
                            self.no_dialog.insert(tab);
                        }
                        self.dlg_active = None;
                        if self.dlg_queue.is_empty() {
                            self.modal = false;
                            self.layout();
                            self.push();
                            // hand the keyboard back to the page that asked, not the toolbar
                            if let Some(t) = self.tabs.iter().find(|t| t.id == tab) {
                                let _ = t.wv.focus();
                            }
                        } else {
                            self.next_dialog();
                        }
                    }
                }
            }
            // the chrome's own dialogs (bookmark group, ...) ask for the whole window while open
            "modal" => {
                if self.dlg_active.is_none() {
                    self.modal = v["on"].as_bool().unwrap_or(false);
                    self.layout();
                    self.push();
                }
            }
            "ui_extra" => {
                self.extra = v["h"].as_f64().unwrap_or(0.0);
                self.layout();
                // resizing the strip can drop keyboard focus; give it back to the address field if it had it
                let _ = self.ui.focus();
                let _ = self.ui.evaluate_script("owl.refocus()");
            }
            "suggest" => {
                let q = s("q").to_lowercase();
                let mut out: Vec<Value> = vec![];
                let mut seen = std::collections::HashSet::new();
                {
                    let st = self.ctx.store.lock().unwrap();
                    let it = st.bookmarks.iter().map(|e| ("Bookmark", e)).chain(st.history.iter().rev().map(|e| ("History", e)));
                    for (k, e) in it {
                        if out.len() >= 6 {
                            break;
                        }
                        if (e.url.to_lowercase().contains(&q) || e.title.to_lowercase().contains(&q)) && seen.insert(e.url.clone()) {
                            out.push(json!({"k":k,"url":e.url,"title":e.title,"icon":e.icon}));
                        }
                    }
                }
                let _ = self.ui.evaluate_script(&format!("owl.sugg({})", Value::Array(out)));
            }
            "set_sync" => {
                let path = s("path").trim().to_string();
                // a synced store must be encrypted: require the sync passphrase first
                if !path.is_empty() && self.ctx.cfg.lock().unwrap().sync_key.is_empty() {
                    self.alert(id, "Set a sync passphrase first (the row below). A synced file is always encrypted.");
                    return;
                }
                self.ctx.cfg.lock().unwrap().sync_dir = path;
                self.ctx.save_cfg();
                self.ctx.save();
            }
            "private" => {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).arg("--private").spawn();
                }
            }
            "fs" => self.set_fs(v["on"].as_bool().unwrap_or(false)),
            "key" => match s("k").as_str() {
                "new" => self.new_tab("owl://newtab"),
                "close" => self.close(self.active),
                "lock" => self.command(json!({"cmd":"lock"}), true, id),
                "addr" => {
                    let _ = self.ui.focus();
                    let _ = self.ui.evaluate_script("owl.focusAddr()");
                }
                "find" => {
                    let _ = self.ui.focus();
                    let _ = self.ui.evaluate_script("owl.toggleFind()");
                }
                "bar" => {
                    let on = !self.ctx.cfg.lock().unwrap().bookmark_bar;
                    self.command(json!({"cmd":"set","key":"bar","val":on}), true, id)
                }
                "reload" => self.command(json!({"cmd":"reload"}), true, id),
                "bookmark" => self.command(json!({"cmd":"bookmark"}), true, id),
                "history" => self.command(json!({"cmd":"navigate","url":"owl://history"}), true, id),
                "dev" => self.command(json!({"cmd":"devtools"}), true, id),
                "back" => self.command(json!({"cmd":"back"}), true, id),
                "forward" => self.command(json!({"cmd":"forward"}), true, id),
                "fs" => {
                    let f = !self.fs;
                    self.set_fs(f)
                }
                "zin" => self.zoom(0.1),
                "zout" => self.zoom(-0.1),
                "z0" => self.zoom(0.0),
                _ => {}
            },
            _ => {}
        }
    }
}

/// Show a real error box and stop. Without this a start-up failure just ends the process silently.
fn fatal(msg: &str) -> ! {
    use windows::core::{w, HSTRING};
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    unsafe {
        MessageBoxW(None, &HSTRING::from(msg), w!("Owl cannot start"), MB_OK | MB_ICONERROR);
    }
    std::process::exit(1)
}

static EVP: Mutex<Option<EventLoopProxy<Ev>>> = Mutex::new(None);
fn post(ev: Ev) {
    if let Some(p) = EVP.lock().unwrap().as_ref() {
        let _ = p.send_event(ev);
    }
}
/// Window hook: reacts to Windows shutdown, the PC locking, and sleep so data is sealed first.
unsafe extern "system" fn win_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wp: windows::Win32::Foundation::WPARAM,
    lp: windows::Win32::Foundation::LPARAM,
    _id: usize,
    _data: usize,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::System::Shutdown::ShutdownBlockReasonCreate;
    use windows::Win32::UI::Shell::DefSubclassProc;
    match msg {
        0x0011 => {
            // WM_QUERYENDSESSION: ask Windows to wait while the data is sealed
            let _ = ShutdownBlockReasonCreate(hwnd, windows::core::w!("Securing your browser data"));
            post(Ev::EndSession);
        }
        0x02B1 if wp.0 == 7 => post(Ev::SessionLock), // WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK
        0x0218 if wp.0 == 4 => post(Ev::SessionLock), // WM_POWERBROADCAST, PBT_APMSUSPEND
        _ => {}
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}

/// Seconds since the last keyboard or mouse input anywhere in Windows.
fn system_idle_secs() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    unsafe {
        let mut li = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        if GetLastInputInfo(&mut li).as_bool() {
            (GetTickCount().wrapping_sub(li.dwTime) / 1000) as u64
        } else {
            0
        }
    }
}

/// Load the history/bookmarks store into memory and tidy download state.
fn open_store(ctx: &Ctx) {
    if let Some(s) = ctx.load_store() {
        *ctx.store.lock().unwrap() = s;
    }
    // downloads that were running when Owl last closed cannot continue
    let mut st = ctx.store.lock().unwrap();
    BLOCKED_BASE.store(st.blocked_total, Relaxed);
    let mut maxid = 0;
    for d in st.downloads.iter_mut() {
        maxid = maxid.max(d.id);
        if d.status == "downloading" || d.status == "paused" || d.status == "pending" {
            d.status = "interrupted".into();
            d.speed = 0.0;
        }
    }
    dl::set_next(maxid + 1);
}

/// Adds a Content-Security-Policy and a per-load script nonce to Owl's own HTML (chrome and widget).
fn secure_html(html: &str, img: &str) -> String {
    let n = crypt::nonce_hex();
    let csp = format!("default-src 'none'; script-src 'nonce-{n}'; style-src 'unsafe-inline'; {img}; base-uri 'none'; form-action 'none'; object-src 'none'");
    html.replacen("<meta charset=\"utf-8\">", &format!("<meta http-equiv=\"Content-Security-Policy\" content=\"{csp}\"><meta charset=\"utf-8\">"), 1)
        .replacen("<meta charset=utf-8>", &format!("<meta http-equiv=\"Content-Security-Policy\" content=\"{csp}\"><meta charset=utf-8>"), 1)
        .replace("<script>", &format!("<script nonce={n}>"))
}

fn main() {
    let private = std::env::args().any(|a| a == "--private");
    let dir = PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into())).join("Owl");
    let _ = fs::create_dir_all(dir.join("extensions"));
    if !private {
        // one normal Owl at a time: two would fight over the same profile files
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows::Win32::System::Threading::CreateMutexW;
        unsafe {
            if let Ok(h) = CreateMutexW(None, false, windows::core::w!("Local\\OwlBrowserMain")) {
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    return;
                }
                std::mem::forget(h); // held until the process exits
            }
        }
    }
    // marker for "last run did not end cleanly": removed again only after a clean seal and exit
    let flag = dir.join("running.flag");
    let unclean = !private && flag.exists();
    if !private {
        let _ = fs::write(&flag, b"1");
    }
    // settings and the vault are written atomically with a .bak; a file that exists but cannot be
    // read must never be mistaken for a fresh install while encrypted data exists
    let (cfg_loaded, cfg_bad) = load_json::<Config>(&dir.join("config.json"));
    let cfg: Config = cfg_loaded.unwrap_or_default();
    let (vault_loaded, vault_bad) = load_json::<Vec<Cred>>(&dir.join("vault.json"));
    let recovery = !private
        && ((cfg_bad || vault_bad)
            || (cfg.seal_pub.is_empty() && cfg.pin_hash.is_empty() && profile::has_sealed(&dir) && !cfg_exists_ok(&dir)));
    BLOCK.store(cfg.block, Relaxed);
    HTTPS.store(cfg.https_only, Relaxed);
    *THEME.lock().unwrap() = cfg.theme.clone();
    pick_random();
    let vault_list: Vec<Cred> = if private { vec![] } else { vault_loaded.unwrap_or_default() };
    let ctx = Ctx {
        recovery,
        store: Arc::new(Mutex::new(Store::default())),
        cfg: Arc::new(Mutex::new(cfg)),
        vault: Arc::new(Mutex::new(vault_list)),
        keys: Arc::new(Mutex::new(None)),
        sync: Arc::new(Mutex::new(None)),
        sync_ro: Arc::new(AtomicBool::new(false)),
        dir: dir.clone(),
        private,
    };
    // a synced store opens now (it has its own passphrase); the local one opens after unlock
    if !private && ctx.synced() {
        open_store(&ctx);
    }

    let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let mut wb = WindowBuilder::new()
        .with_title(if private { "Owl — Private" } else { "Owl" })
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(LogicalSize::new(640.0, 400.0))
        .with_decorations(false);
    if let Ok(icon) = Icon::from_resource(1, None) {
        wb = wb.with_window_icon(Some(icon));
    }
    let window = match wb.build(&event_loop) {
        Ok(w) => w,
        Err(e) => fatal(&format!("Owl could not create its window.\n\n{e}")),
    };

    let up = proxy.clone();
    let tp = proxy.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        if tp.send_event(Ev::Tick).is_err() {
            break;
        }
    });
    // Private windows keep every engine folder in a temporary directory; old ones are removed.
    let private_dir = std::env::temp_dir().join(format!("owl-private-{}", std::process::id()));
    if private {
        if let Ok(rd) = fs::read_dir(std::env::temp_dir()) {
            for e in rd.flatten() {
                let stale = e.file_name().to_string_lossy().starts_with("owl-private-")
                    && e.metadata().and_then(|m| m.modified()).map_or(false, |t| t.elapsed().map_or(false, |d| d > Duration::from_secs(3600)));
                if stale {
                    let _ = fs::remove_dir_all(e.path());
                }
            }
        }
    }
    // The chrome gets an explicit engine folder under the user's data directory. The default is
    // next to Owl.exe, which fails (and used to abort silently) when Owl is installed where the
    // user cannot write, such as Program Files.
    let mut ui_ctx = WebContext::new(Some(if private { private_dir.join("chrome") } else { dir.join("chrome") }));
    let ui = match WebViewBuilder::new_with_web_context(&mut ui_ctx)
        .with_bounds(Rect { position: LogicalPosition::new(0.0, 0.0).into(), size: LogicalSize::new(1280.0, 800.0).into() })
        .with_devtools(false) // no inspector on the chrome or the lock screen
        .with_html(secure_html(&UI_HTML.replace("/*THEMES*/", themes::THEMES), "img-src http: https: data:"))
        .with_ipc_handler(move |req| {
            let _ = up.send_event(Ev::Ui(req.body().clone()));
        })
        .build_as_child(&window)
    {
        Ok(v) => v,
        Err(e) => fatal(&format!("Owl could not start its browser engine (Microsoft Edge WebView2).\n\n{e}\n\nInstall or repair the WebView2 Runtime from:\nhttps://developer.microsoft.com/microsoft-edge/webview2/")),
    };
    let wctx = WebContext::new(Some(if private { private_dir.join("tabs") } else { dir.join("profile") }));
    let mut app = App {
        window, ui, wctx, tabs: vec![], active: 0, next: 1, fs: false, extra: 0.0,
        locked: !private, fails: 0, lock_until: None, pending: None, speeds: HashMap::new(),
        last_push: Instant::now(), proxy, ctx, ticks: 0, media: None, mw: None, mcollapsed: false,
        mshown: std::cell::Cell::new(false),
        upg: HashMap::new(), last_key: Instant::now(), dl_ask: vec![], recent_dl: vec![], engine_watch: false,
        sealer: None, sealed_at_lock: false, last_active: Instant::now(),
        snap_wait: None, unclean, block_reason: false, restore2: vec![],
        dlg_queue: Default::default(), dlg_active: None, modal: false, no_dialog: Default::default(),
        start_url: std::env::args()
            .skip(1)
            .find(|a| !a.starts_with("--"))
            .filter(|u| u.len() < 2048 && (u.starts_with("http://") || u.starts_with("https://") || u.starts_with("owl://"))),
        last_pin: Instant::now().checked_sub(Duration::from_secs(10)).unwrap_or_else(Instant::now),
    };
    app.apply_theme();
    if private {
        app.ensure_tabs();
    } else {
        // after a crash or kill the profile may still be on disk in plain form: seal it right
        // away with the public key, before the lock screen is even answered
        if let Some(pubk) = app.ctx.pub_key() {
            if profile::has_plaintext(&dir) {
                let d = dir.clone();
                app.sealer = Some(std::thread::spawn(move || profile::seal_profile(&d, &pubk, None)));
            }
        }
        unsafe {
            use windows::core::PCWSTR;
            use windows::Win32::System::Recovery::{RegisterApplicationRestart, REGISTER_APPLICATION_RESTART_FLAGS};
            use windows::Win32::System::RemoteDesktop::WTSRegisterSessionNotification;
            use windows::Win32::UI::Shell::SetWindowSubclass;
            // Windows relaunches Owl after an update restart, but not after a crash or hang
            let _ = RegisterApplicationRestart(PCWSTR::null(), REGISTER_APPLICATION_RESTART_FLAGS(3));
            *EVP.lock().unwrap() = Some(app.proxy.clone());
            let h = app.hwnd();
            let _ = SetWindowSubclass(h, Some(win_proc), 1, 0);
            let _ = WTSRegisterSessionNotification(h, 0); // NOTIFY_FOR_THIS_SESSION
        }
    }
    app.layout();
    let _ = app.ui.focus();

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *flow = ControlFlow::Exit,
            Event::LoopDestroyed => {
                // close every page so the engine releases the profile, then seal it at rest
                if let Some(h) = app.sealer.take() {
                    let _ = h.join();
                }
                app.tabs.clear();
                app.mw = None;
                app.ctx.save();
                let privk = app.ctx.keys.lock().unwrap().as_ref().and_then(|k| k.privk);
                let mut sealed_ok = true;
                if let (Some(pubk), false) = (app.ctx.pub_key(), app.ctx.private) {
                    sealed_ok = profile::seal_profile(&app.ctx.dir, &pubk, privk.as_ref()).is_ok();
                }
                // mark this exit as clean only if everything really was sealed; otherwise the next
                // start repeats the seal and tells the user
                if !app.ctx.private && sealed_ok {
                    let _ = fs::remove_file(app.ctx.dir.join("running.flag"));
                }
            }
            Event::WindowEvent { event: WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }, .. } => app.layout(),
            Event::WindowEvent { event: WindowEvent::Focused(true), .. } => {
                if app.locked {
                    let _ = app.ui.focus();
                }
            }
            Event::UserEvent(ev) => match ev {
                Ev::Quit => *flow = ControlFlow::Exit,
                Ev::Ui(s) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&s) {
                        let a = app.active;
                        app.command(v, true, a)
                    }
                }
                Ev::Tab(id, s, trusted) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&s) {
                        app.command(v, trusted, id)
                    }
                }
                Ev::Go(id, u) => {
                    if let Some(t) = app.tabs.iter().find(|t| t.id == id) {
                        // remember the original http:// address so a failed upgrade can offer a fallback
                        app.upg.insert(id, format!("http://{}", u.strip_prefix("https://").unwrap_or(&u)));
                        let _ = t.wv.load_url(&u);
                    }
                }
                Ev::NavOk(id) => {
                    app.upg.remove(&id);
                }
                Ev::NavFail(id, status) => {
                    // 11 = the navigation was cancelled (user moved on), not a connection failure
                    if status != 11 {
                        if let Some(http) = app.upg.remove(&id) {
                            if let Some(t) = app.tabs.iter().find(|t| t.id == id) {
                                let _ = t.wv.load_url(&format!("http://{HOST}/httpsfail?u={}", pct(&http)));
                            }
                        }
                    }
                }
                Ev::Fullscreen(id, on) => {
                    // a real fullscreen request from the active page (video, games); never from background tabs
                    if !on || id == app.active {
                        app.set_fs(on);
                    }
                }
                Ev::EngineUpdate => {
                    let n = json!({
                        "id": "upd", "label": "Update",
                        "text": "A newer browser engine is installed. Restart Owl to use it and get the latest security fixes.",
                        "actions": [{"t":"Later"},{"t":"Restart now","cmd":"restart","pri":true}]
                    });
                    let _ = app.ui.evaluate_script(&format!("owl.notice({n})"));
                }
                Ev::Nav(id, u) => {
                    if let Some(t) = app.tabs.iter_mut().find(|t| t.id == id) {
                        t.url = display(&u);
                    }
                    app.push();
                }
                Ev::Title(id, title) => {
                    let mut url = String::new();
                    if let Some(t) = app.tabs.iter_mut().find(|t| t.id == id) {
                        t.title = title.clone();
                        url = t.url.clone();
                    }
                    if !app.ctx.private {
                        let mut s = app.ctx.store.lock().unwrap();
                        if let Some(h) = s.history.last_mut().filter(|h| h.url == url) {
                            h.title = title.clone();
                        }
                        if let Some(b) = s.bookmarks.iter_mut().find(|b| b.url == url && b.title.is_empty()) {
                            b.title = title;
                        }
                    }
                    app.push();
                }
                Ev::Audible(id, on) => {
                    if let Some(t) = app.tabs.iter_mut().find(|t| t.id == id) {
                        t.audible = on;
                    }
                    app.push();
                }
                Ev::Tick => app.tick(),
                Ev::Snap(id, r) => {
                    // a page answered the lock-time request for its scroll position and typed text
                    let inner = serde_json::from_str::<String>(&r).unwrap_or_default();
                    if !inner.is_empty() && inner.len() < 60_000 {
                        if let (Some(t), Ok(o)) = (app.tabs.iter().find(|t| t.id == id), serde_json::from_str::<Value>(&inner)) {
                            let has = o["f"].as_array().map_or(false, |a| !a.is_empty()) || o["y"].as_i64().unwrap_or(0) > 0;
                            if has && !app.ctx.private {
                                let mut s = app.ctx.store.lock().unwrap();
                                if s.forms.len() >= 25 {
                                    if let Some(k) = s.forms.keys().next().cloned() {
                                        s.forms.remove(&k);
                                    }
                                }
                                s.forms.insert(t.url.clone(), inner);
                            }
                        }
                    }
                    let done = match app.snap_wait.as_mut() {
                        Some((_, n)) => {
                            *n = n.saturating_sub(1);
                            *n == 0
                        }
                        None => false,
                    };
                    if done {
                        app.finish_lock();
                    }
                }
                Ev::Dialog(did, tab, kind, uri, msg, def) => {
                    if app.locked || app.no_dialog.contains(&tab) || app.dlg_queue.len() >= 5 {
                        native::dialog_answer(did, false, "");
                    } else {
                        app.dlg_queue.push_back((did, tab, kind, uri, msg, def));
                        app.next_dialog();
                    }
                }
                Ev::EndSession => {
                    // Windows is shutting down or signing out: lock (which seals) while the
                    // shutdown-block reason keeps the system waiting for us
                    app.block_reason = true;
                    app.begin_lock();
                }
                Ev::SessionLock => {
                    let on = app.ctx.cfg.lock().unwrap().lock_on_winlock;
                    if on {
                        app.begin_lock();
                    }
                }
                Ev::Load(id, started, u) => {
                    if started && app.media.as_ref().map_or(false, |m| m.tab == id) {
                        app.media = None;
                        app.layout();
                    }
                    let u = display(&u);
                    if let Some(t) = app.tabs.iter_mut().find(|t| t.id == id) {
                        t.loading = started;
                        t.url = u.clone();
                        if started {
                            t.title.clear();
                            t.icon.clear();
                        }
                    }
                    if !started && !app.ctx.private {
                        // put back scroll position and typed text saved when Owl locked
                        let snap = app.ctx.store.lock().unwrap().forms.remove(&u);
                        if let Some(snap) = snap {
                            app.eval_tab(id, &format!("window.__owlRestore&&window.__owlRestore({})", serde_json::to_string(&snap).unwrap()));
                            app.restore2.push((id, snap, Instant::now()));
                        }
                    }
                    if started && !app.ctx.private && !u.starts_with("owl://") && !u.starts_with("about:") {
                        {
                            let mut s = app.ctx.store.lock().unwrap();
                            s.history.push(Entry { url: u, t: now(), ..Default::default() });
                            let n = s.history.len();
                            if n > 3000 {
                                s.history.drain(0..n - 3000);
                            }
                        }
                        app.ctx.save();
                    }
                    app.push();
                }
                Ev::NewTab(u) => {
                    // popup blocker: only windows opened by a real user action are allowed
                    let user = native::POPUPS.lock().unwrap().pop_front().unwrap_or(true);
                    if (u.starts_with("http://") || u.starts_with("https://") || u.starts_with("owl://"))
                        && user
                        && !app.locked
                        && app.snap_wait.is_none()
                        && app.tabs.len() < 40
                    {
                        app.new_tab(&u);
                    }
                }
                Ev::DlAsk(id, tab, url, path) => app.dl_incoming(id, tab, url, path),
                Ev::DlProg(id, r, t) => {
                    let mut sp = 0.0;
                    if let Some((at, last, ema)) = app.speeds.get_mut(&id) {
                        let dt = at.elapsed().as_secs_f64();
                        if dt >= 0.4 {
                            let inst = (r - *last) as f64 / dt;
                            *ema = if *ema == 0.0 { inst } else { *ema * 0.6 + inst * 0.4 };
                            *at = Instant::now();
                            *last = r;
                        }
                        sp = *ema;
                    }
                    app.dl_mut(id, |d| {
                        d.received = r;
                        d.total = t;
                        if d.status == "downloading" {
                            d.speed = sp;
                        }
                    });
                    if app.last_push.elapsed() > Duration::from_millis(350) {
                        app.push();
                    }
                }
                Ev::DlState(id, st) => {
                    let paused = app.dl_mut(id, |d| d.status == "paused").unwrap_or(false);
                    match st {
                        2 => {
                            app.dl_mut(id, |d| {
                                d.status = "done".into();
                                d.speed = 0.0;
                                if d.total > 0 {
                                    d.received = d.total;
                                }
                            });
                            dl::forget(id);
                            app.speeds.remove(&id);
                        }
                        1 if !paused => {
                            app.dl_mut(id, |d| {
                                if d.status != "cancelled" {
                                    d.status = "failed".into();
                                }
                                d.speed = 0.0;
                            });
                            dl::forget(id);
                        }
                        _ => {}
                    }
                    app.ctx.save();
                    app.push();
                }
            },
            _ => {}
        }
    });
}
