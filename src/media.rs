//! Now-playing: a page reporter injected into every tab, the floating widget, and per-tab audio hooks.
use crate::Ev;
use tao::event_loop::EventLoopProxy;
use webview2_com::{
    Microsoft::Web::WebView2::Win32::ICoreWebView2_8, IsDocumentPlayingAudioChangedEventHandler,
};
use windows::core::{Interface, BOOL};
use wry::{WebView, WebViewExtWindows};

/// Injected into every top-level page. Reports what is playing (YouTube, YouTube Music, any page
/// that sets navigator.mediaSession) about once a second while it changes.
pub const PAGE_JS: &str = r#"
(function(){
 if(top!==window||!/^https?:$/.test(location.protocol))return;
 const post=o=>window.ipc.postMessage(JSON.stringify(o));
 let sent='',had=false;
 const yt=()=>{try{const v=new URL(location.href).searchParams.get('v');return v?'https://i.ytimg.com/vi/'+v+'/hqdefault.jpg':''}catch(e){return ''}};
 function tick(){
  const ms=[...document.querySelectorAll('video,audio')].filter(m=>m.readyState>1&&m.duration>25&&m.currentTime>0&&!m.ended);
  const m=ms.find(x=>!x.paused)||ms[0];
  const md=navigator.mediaSession&&navigator.mediaSession.metadata;
  if(!m||(!md&&!/youtube\.com$/.test(location.hostname))){if(had){had=false;sent='';post({cmd:'media',title:''})}return}
  let art='';if(md&&md.artwork&&md.artwork.length)art=md.artwork[md.artwork.length-1].src;
  if(!art)art=yt();
  const o={cmd:'media',title:(md&&md.title)||document.title.replace(/ - YouTube.*$/,''),artist:(md&&md.artist)||'',art,playing:!m.paused,pos:Math.floor(m.currentTime),dur:isFinite(m.duration)?Math.floor(m.duration):0};
  const k=JSON.stringify(o);if(k===sent)return;sent=k;had=true;post(o)}
 const seen=new Set();
 function scan(){document.querySelectorAll('video,audio').forEach(m=>{const u=m.currentSrc||m.src;if(u&&/^https?:/.test(u)&&!seen.has(u)){seen.add(u);post({cmd:'media_found',url:u})}})}
 setInterval(scan,1500);
 setInterval(tick,1000);
})();
"#;

const FIND: &str = "const q=s=>document.querySelector(s);const m=[...document.querySelectorAll('video,audio')].find(x=>x.currentTime>0);";
pub fn js_toggle() -> String {
    format!("(function(){{{FIND}const b=q('ytmusic-player-bar .play-pause-button')||q('.ytp-play-button');if(b){{b.click();return}}if(m){{m.paused?m.play():m.pause()}}}})()")
}
pub fn js_next() -> String {
    format!("(function(){{{FIND}const b=q('ytmusic-player-bar .next-button')||q('.ytp-next-button');if(b)b.click();else if(m&&isFinite(m.duration))m.currentTime=m.duration}})()")
}
pub fn js_prev() -> String {
    format!("(function(){{{FIND}const b=q('ytmusic-player-bar .previous-button')||q('.ytp-prev-button');if(b)b.click();else if(m)m.currentTime=0}})()")
}

/// The floating widget (Lucide icons, ISC licence).
pub const HTML: &str = r##"<!doctype html><meta charset=utf-8><style>
*{box-sizing:border-box;margin:0;padding:0}
html,body{width:100%;height:100%;overflow:hidden;background:#111;color:#fff;font:12px/1.2 "Helvetica Neue",Helvetica,Arial,sans-serif;-webkit-font-smoothing:antialiased;user-select:none}
body{display:flex;border-top:3px solid #e30613;box-shadow:inset 0 0 0 1px #ffffff26;animation:in .3s cubic-bezier(.2,.8,.2,1)}
@keyframes in{from{opacity:0;transform:translateY(18px)}to{opacity:1;transform:none}}
#art{width:80px;flex:none;position:relative;background:#262626;cursor:pointer;overflow:hidden}
#art img{width:100%;height:100%;object-fit:cover;display:block;transition:transform .5s}
#art:hover img{transform:scale(1.07)}
#note{position:absolute;inset:0;display:flex;align-items:center;justify-content:center;font:italic 34px Georgia,serif;color:#e30613}
#eq{position:absolute;left:0;bottom:0;display:flex;gap:2px;align-items:flex-end;height:20px;background:#e30613;padding:4px 5px}
#eq i{width:3px;height:3px;background:#fff;transition:height .2s}
body.play #eq i{animation:eq .9s ease-in-out infinite}
#eq i:nth-child(2){animation-delay:.25s}#eq i:nth-child(3){animation-delay:.5s}#eq i:nth-child(4){animation-delay:.12s}
@keyframes eq{0%,100%{height:3px}50%{height:12px}}
#info{flex:1;min-width:0;display:flex;flex-direction:column;padding:7px 0 0 12px}
body.min #info{display:none}
#h{display:flex;align-items:center;padding-right:2px}
#t{flex:1;min-width:0;font-weight:700;font-size:13px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
#a{color:#a8a8a8;margin-top:3px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;padding-right:8px}
#c{display:flex;align-items:center;margin-top:auto;margin-bottom:4px}
button{background:none;border:0;color:#fff;cursor:pointer;width:32px;height:28px;display:flex;align-items:center;justify-content:center;transition:background .12s,color .12s,transform .1s}
button:hover{background:#fff;color:#111}button:active{transform:scale(.9)}
button svg{width:16px;height:16px;stroke:currentColor;fill:none;stroke-width:1.8;stroke-linejoin:miter}
#h button{width:22px;height:22px}#h button svg{width:13px;height:13px}
#pp{background:#e30613;width:38px}#pp:hover{background:#fff;color:#e30613}
#pp svg{fill:currentColor}
#c .sp{flex:1}
#pg{height:3px;background:#383838;margin-left:-12px}
#pg i{display:block;height:100%;width:0;background:#e30613;transition:width 1s linear}
</style><body>
<div id=art><div id=note>&#9834;</div><img id=img alt=""><div id=eq><i></i><i></i><i></i><i></i></div></div>
<div id=info>
 <div id=h><span id=t></span><button id=mn title="Minimise" data-i=min></button><button id=cl title="Close" data-i=x></button></div>
 <div id=a></div>
 <div id=c><button id=pv title="Previous" data-i=prev></button><button id=pp title="Play / pause" data-i=play></button><button id=nx title="Next" data-i=next></button><span class=sp></span></div>
 <div id=pg><i></i></div>
</div>
<script>
const $=i=>document.getElementById(i),send=o=>ipc.postMessage(JSON.stringify(o));
const P={min:'<path d="M5 12h14"/>',x:'<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
play:'<polygon points="6 3 20 12 6 21 6 3"/>',pause:'<rect x="14" y="4" width="4" height="16"/><rect x="6" y="4" width="4" height="16"/>',
next:'<polygon points="5 4 15 12 5 20 5 4"/><line x1="19" x2="19" y1="5" y2="19"/>',prev:'<polygon points="19 20 9 12 19 4 19 20"/><line x1="5" x2="5" y1="19" y2="5"/>'};
const ic=n=>'<svg viewBox="0 0 24 24">'+P[n]+'</svg>';
document.querySelectorAll('[data-i]').forEach(b=>b.innerHTML=ic(b.dataset.i));
let art='',pl=null,min=false;
const img=$('img');img.onerror=()=>{img.style.display='none'};
window.m={state(o){min=!!o.min;document.body.classList.toggle('min',min);document.body.classList.toggle('play',!!o.playing);
 $('t').textContent=o.title||'';$('a').textContent=o.artist||'';
 if(o.art!==art){art=o.art;if(art){img.style.display='';img.src=art}else img.style.display='none'}
 if(pl!==!!o.playing){pl=!!o.playing;$('pp').innerHTML=ic(pl?'pause':'play')}
 $('pg').firstChild.style.width=o.dur>0?Math.min(100,o.pos*100/o.dur)+'%':'0'},
 enter(){document.body.style.animation='none';void document.body.offsetWidth;document.body.style.animation=''}};
$('pp').onclick=()=>send({cmd:'media_toggle'});$('nx').onclick=()=>send({cmd:'media_next'});$('pv').onclick=()=>send({cmd:'media_prev'});
$('mn').onclick=()=>send({cmd:'media_collapse'});$('cl').onclick=()=>send({cmd:'media_close'});
$('art').onclick=()=>send({cmd:min?'media_collapse':'media_goto'});
</script>"##;

/// Fires `Ev::Audible` whenever a tab starts or stops making sound.
pub fn install_audio(wv: &WebView, proxy: EventLoopProxy<Ev>, id: u32) {
    unsafe {
        let Ok(c8) = wv.webview().cast::<ICoreWebView2_8>() else { return };
        let mut token = Default::default();
        let _ = c8.add_IsDocumentPlayingAudioChanged(
            &IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |s, _| {
                if let Some(c) = s.and_then(|s| s.cast::<ICoreWebView2_8>().ok()) {
                    let mut b = BOOL(0);
                    c.IsDocumentPlayingAudio(&mut b)?;
                    let _ = proxy.send_event(Ev::Audible(id, b.as_bool()));
                }
                Ok(())
            })),
            &mut token,
        );
    }
}

pub fn set_muted(wv: &WebView, on: bool) {
    unsafe {
        if let Ok(c8) = wv.webview().cast::<ICoreWebView2_8>() {
            let _ = c8.SetIsMuted(on);
        }
    }
}
