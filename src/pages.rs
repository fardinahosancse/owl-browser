use crate::{Config, Store};
use std::collections::HashSet;

const CSS: &str = r#"*{box-sizing:border-box;margin:0;padding:0}
/*THEMES*/
html{scroll-behavior:smooth}@media(prefers-reduced-motion:reduce){html{scroll-behavior:auto}}
body{font:16px/1.45 "Helvetica Neue",Helvetica,Arial,sans-serif;color:var(--ink);background:var(--bg);padding:0 40px 40px;min-height:100vh;-webkit-font-smoothing:antialiased;transition:background .35s,color .35s}
a{color:inherit;text-decoration:none}
@keyframes rise{from{opacity:0;transform:translateY(26px)}to{opacity:1;transform:none}}
@keyframes grow{from{transform:scaleX(0)}to{transform:scaleX(1)}}
.grid{display:grid;grid-template-columns:repeat(12,1fr);column-gap:24px}
header{border-top:6px solid var(--ink);padding:10px 0 0;display:flex;justify-content:space-between;font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;margin-bottom:28px}
header i{font-style:normal;color:var(--red)}
h1{font-size:clamp(44px,6vw,88px);line-height:.92;letter-spacing:-.04em;font-weight:700;margin-bottom:24px;animation:rise .7s cubic-bezier(.2,.7,.2,1) both}
h1 em{font-style:normal;color:var(--red)}
h2{font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;padding-top:10px;border-top:2px solid var(--ink);margin-bottom:12px;display:flex;justify-content:space-between}
h2 span{color:var(--mute)}
button,.btn{border:0;background:var(--ink);color:var(--on);padding:0 22px;font:700 11px "Helvetica Neue",Helvetica,Arial,sans-serif;letter-spacing:.14em;text-transform:uppercase;cursor:pointer;display:inline-flex;align-items:center;min-height:36px;transition:background .15s,color .15s,transform .12s}
button:hover,.btn:hover{background:var(--red);color:var(--onred)}
button:active,.btn:active{transform:scale(.96)}
button.on{background:var(--red);color:var(--onred)}
button.ghost{background:var(--soft);color:var(--ink)}button.ghost:hover{background:var(--ink);color:var(--on)}
:focus-visible{outline:2px solid var(--red);outline-offset:2px}
.sec{grid-column:1/13;margin-bottom:40px}
.row{display:grid;grid-template-columns:56px 1fr 2fr 140px;gap:24px;padding:13px 0;border-top:1px solid var(--line);align-items:baseline;animation:rise .45s cubic-bezier(.2,.7,.2,1) both;animation-delay:calc(var(--i,0)*28ms);transition:background .15s,color .15s,padding .15s}
.row.hv:hover{background:var(--ink);color:var(--on);padding-left:8px}.row.hv:hover .n,.row.hv:hover .u,.row.hv:hover .d,.tile:hover .k{color:var(--mute-on)}.row.hv .t{min-width:0}
.row.hv .d{display:flex;gap:8px;justify-content:flex-end;align-items:center;overflow:visible}.row.hv .d time{font-size:13px;font-variant-numeric:tabular-nums}
.n{font-size:11px;font-weight:700;color:var(--mute)}.u,.d{font-size:13px;color:var(--mute);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.t{font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.empty{border-top:1px solid var(--line);padding:16px 0;color:var(--mute)}.empty a{color:var(--ink);text-decoration:underline}
button.sm{min-height:32px;padding:0 12px;font-size:11px}.tb{display:flex;gap:8px;align-items:center;margin-bottom:20px;flex-wrap:wrap}
.ctl{display:flex;gap:8px;align-items:center;justify-content:flex-end;flex-wrap:wrap}
.gh{margin:34px 0 6px}.gh:first-of-type{margin-top:6px}
details summary{cursor:pointer;color:var(--mute);font-size:13px;margin-top:6px;text-decoration:underline}details p{margin-top:6px}
button[disabled]{opacity:.45;cursor:default}button[disabled]:hover{background:var(--ink);color:var(--on)}button.ghost[disabled]:hover{background:var(--soft);color:var(--ink)}
.set{display:grid;grid-template-columns:1fr auto;gap:24px;align-items:center;padding:18px 0;border-top:1px solid var(--line);animation:rise .45s cubic-bezier(.2,.7,.2,1) both;animation-delay:calc(var(--i,0)*35ms)}
.set p{color:var(--mute);font-size:14px;margin-top:4px}
.set input,.af input,.pf{font:500 14px "Helvetica Neue",Helvetica,Arial,sans-serif;padding:10px;border:1px solid var(--ink);background:var(--bg);color:var(--ink);width:380px;max-width:100%;outline:0;transition:box-shadow .15s}
.set input:focus,.af input:focus,.pf:focus{box-shadow:0 0 0 2px var(--red)}
input::placeholder{color:var(--mute);opacity:1}
.set input.pin{width:90px;letter-spacing:.3em;text-align:center}
.set input.pk,.set select.pk{width:210px}
.set select.pk{font:500 14px "Helvetica Neue",Helvetica,Arial,sans-serif;padding:10px;border:1px solid var(--ink);background:var(--bg);color:var(--ink)}
.pill{font-size:11px;font-weight:700;letter-spacing:.12em;text-transform:uppercase;cursor:pointer}
code{font-size:13px;background:var(--soft);padding:2px 6px}
.themes{display:grid;grid-template-columns:repeat(auto-fill,minmax(132px,1fr));gap:14px;margin:4px 0 34px}
.th{display:block;padding:0;background:none;color:var(--ink);border:2px solid var(--line);text-align:left;min-height:0;letter-spacing:0;animation:rise .45s cubic-bezier(.2,.7,.2,1) both;animation-delay:calc(var(--i,0)*45ms)}
.th:hover{background:none;color:var(--ink);border-color:var(--ink)}
.th.on{border-color:var(--red)}
.th .pv{display:block;height:88px;background:var(--b);position:relative}
.th .pv::before{content:"";position:absolute;left:12px;top:14px;width:52%;height:10px;background:var(--i)}
.th .pv::after{content:"";position:absolute;right:12px;bottom:12px;width:26px;height:26px;background:var(--a)}
.th .pv u{position:absolute;left:12px;top:32px;width:30%;height:4px;background:var(--i);opacity:.5}
.th .l{display:flex;justify-content:space-between;padding:9px 10px;font:700 11px "Helvetica Neue",Arial,sans-serif;letter-spacing:.14em;text-transform:uppercase}
.th.on .l::after{content:"\25CF";color:var(--onred)}
.tg{min-width:64px;justify-content:center}.tg.on{background:var(--red);color:var(--onred)}.tg.on:hover{background:var(--ink);color:var(--on)}
/* ===== landing home ===== */
body.landing{padding:0}
.land>header{padding:18px 40px 0}.tiles .empty{border-top:0;padding:16px}
.land section{padding:clamp(64px,11vh,120px) 40px}
.hero{min-height:56vh;display:grid;grid-template-rows:auto 1fr auto;padding-top:0!important;padding-bottom:36px!important}
.hero header{margin-bottom:0}
.hcore{display:flex;flex-direction:column;justify-content:center;gap:clamp(18px,4vh,44px)}
.hcore>*{animation:rise .9s cubic-bezier(.2,.7,.2,1) both}.hcore>:nth-child(2){animation-delay:.08s}.hcore>:nth-child(3){animation-delay:.16s}
.clock{font:italic 400 clamp(80px,19vh,240px)/.82 Georgia,"Palatino Linotype","Book Antiqua",serif;letter-spacing:-.045em;white-space:nowrap;font-variant-numeric:oldstyle-nums}
.clock small{font:700 .17em/1 "Helvetica Neue",Helvetica,Arial,sans-serif;font-style:normal;letter-spacing:0;color:var(--red);margin-left:.3em;vertical-align:top;display:inline-block;padding-top:.35em}
.hello{display:flex;justify-content:space-between;align-items:baseline;font-size:clamp(18px,3vh,28px);font-weight:500}
.hello b{font-size:11px;letter-spacing:.14em;text-transform:uppercase;color:var(--red);margin-right:14px}
.hello span:last-child{color:var(--mute)}
form.s{display:flex;border-top:2px solid var(--ink);border-bottom:2px solid var(--ink);transition:border-color .2s}
form.s:focus-within{border-color:var(--red)}
form.s input{flex:1;min-width:0;border:0;outline:0;font:500 clamp(24px,5vh,40px) "Helvetica Neue",Helvetica,Arial,sans-serif;padding:clamp(12px,2.4vh,24px) 0;background:none;color:var(--ink)}
form.s input::placeholder{color:var(--mute)}
form.s kbd{align-self:center;font:700 12px "Helvetica Neue",Arial;color:var(--mute);padding-right:6px}
.scroll{display:flex;justify-content:space-between;font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;color:var(--mute);padding-top:14px}
.scroll span:first-child::after{content:"\2193";display:inline-block;margin-left:10px}
.huge.sm{font-size:clamp(32px,4vw,56px);margin-bottom:22px;max-width:none}.land section.q{padding-top:28px}
.scroll a{text-decoration:underline;text-underline-offset:3px}.scroll a:hover{color:var(--red)}
.huge{font-size:clamp(48px,9vw,140px);line-height:.88;letter-spacing:-.05em;font-weight:700;margin-bottom:clamp(28px,6vh,64px);max-width:12ch}
.huge em{font-style:normal;color:var(--red)}
.tiles{display:grid;grid-template-columns:repeat(4,1fr);border-left:1px solid var(--line);margin-bottom:56px}
.tile{border-right:1px solid var(--line);border-bottom:1px solid var(--line);border-top:1px solid var(--line);padding:16px;min-height:150px;display:flex;flex-direction:column;justify-content:space-between;min-width:0}
.tile:hover{background:var(--ink);color:var(--on)}
.tile .k{font-size:11px;font-weight:700;color:var(--mute);letter-spacing:.1em}
.tile img,.tile .l{width:32px;height:32px}
.tile .l{background:var(--ink);color:var(--on);display:flex;align-items:center;justify-content:center;font-weight:700;font-size:16px}
.tile:hover .l{background:var(--red);color:var(--onred)}
.tile span.h{font-size:15px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.groups{display:grid;grid-template-columns:repeat(4,1fr);gap:24px}
.grp h3{font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;color:var(--red);padding:9px 0 6px;border-top:2px solid var(--ink)}
.grp a{display:flex;align-items:center;gap:10px;padding:9px 0;border-top:1px solid var(--line);font-size:15px;transition:color .15s,padding .15s}
.grp a:hover{color:var(--red);padding-left:6px}.grp img{width:16px;height:16px}
.inv{background:var(--ink);color:var(--on)}
.inv .huge em{color:var(--rink)}
.stats{display:grid;grid-template-columns:repeat(4,1fr);gap:24px}
.stat{border-top:2px solid var(--on);padding-top:14px;display:block}
.stat b{display:block;font-size:clamp(48px,7vw,110px);line-height:.9;letter-spacing:-.05em;font-weight:700;font-variant-numeric:tabular-nums;transition:color .2s;white-space:nowrap}
.stat b.r{color:var(--rink)}
.stat span{display:block;margin-top:12px;font-size:14px;opacity:.75;max-width:24ch}
.stat:hover b{color:var(--rink)}
.time{display:grid;grid-template-columns:1fr 2fr;gap:64px;align-items:start}
.tbig b{display:block;font:italic 400 clamp(56px,8vw,120px)/.9 Georgia,serif;letter-spacing:-.04em;font-variant-numeric:oldstyle-nums;white-space:nowrap}
.tbig>span{display:block;font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;color:var(--mute);margin:10px 0 34px}
.bars .br{display:grid;grid-template-columns:36px 190px 1fr 84px;gap:16px;align-items:center;padding:13px 0;border-top:1px solid var(--line)}
.bars .bn{font-size:17px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.bars .bb{height:10px;background:var(--soft)}
.bars .bb i{display:block;height:100%;background:var(--red);transform-origin:left;transform:scaleX(0);transition:transform 1.1s cubic-bezier(.2,.7,.2,1);width:var(--w)}
.in .bars .bb i,.bars.in .bb i{transform:scaleX(1)}
.bars .bt{text-align:right;font-size:14px;font-weight:700;font-variant-numeric:tabular-nums}
.dls .drow{display:grid;grid-template-columns:1fr 140px 90px;gap:24px;padding:16px 0;border-top:1px solid var(--line);align-items:center}
.dls .nm{font-size:20px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.bar{height:6px;background:var(--soft);position:relative}.bar i{position:absolute;left:0;top:0;bottom:0;background:var(--red)}
.dls .st{font-size:11px;font-weight:700;letter-spacing:.12em;text-transform:uppercase;color:var(--mute);text-align:right}
.keys{display:grid;grid-template-columns:repeat(4,1fr);border-left:1px solid var(--line)}
.key{border-right:1px solid var(--line);border-bottom:1px solid var(--line);border-top:1px solid var(--line);padding:20px 16px;min-height:130px;display:flex;flex-direction:column;justify-content:space-between;transition:background .2s,color .2s}
.key:hover{background:var(--ink);color:var(--on)}
.key kbd{font:italic 400 clamp(26px,3vw,44px)/1 Georgia,serif;letter-spacing:-.02em}
.key span{font-size:13px;color:var(--mute)}
.fin{background:var(--red);color:var(--onred);padding-bottom:36px!important}
.fin .word{font:italic 400 clamp(120px,30vw,520px)/.8 Georgia,"Palatino Linotype",serif;letter-spacing:-.06em;margin:0 0 40px -.04em}
.fin .facts{display:grid;grid-template-columns:repeat(4,1fr);gap:24px;border-top:2px solid var(--onred);padding-top:14px}
.fin .facts b{display:block;font-size:clamp(24px,3vw,44px);letter-spacing:-.03em}
.fin .facts span{font-size:13px;opacity:.85}
.fin .foot{margin-top:56px;font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;display:flex;justify-content:space-between;opacity:.9}
/* reveal on scroll */
.js .rv{opacity:0;transform:translateY(34px);transition:opacity .8s cubic-bezier(.2,.7,.2,1),transform .8s cubic-bezier(.2,.7,.2,1)}
.js .rv.in{opacity:1;transform:none}
.js .rv .tile,.js .rv .stat,.js .rv .key,.js .rv .grp,.js .rv .br{opacity:0;transform:translateY(20px);transition:opacity .6s cubic-bezier(.2,.7,.2,1) var(--d,0s),transform .6s cubic-bezier(.2,.7,.2,1) var(--d,0s),background .2s,color .2s,padding .15s}
.js .rv.in .tile,.js .rv.in .stat,.js .rv.in .key,.js .rv.in .grp,.js .rv.in .br{opacity:1;transform:none}
@media(prefers-reduced-motion:reduce){*,*::before,*::after{animation:none!important;transition:none!important}.js .rv,.js .rv *{opacity:1!important;transform:none!important}}
@media(max-width:900px){.tiles,.groups,.stats,.keys,.fin .facts{grid-template-columns:repeat(2,1fr)}.row{grid-template-columns:40px 1fr}.row .u{display:none}.row .d{grid-column:2}.row.hv .d{justify-content:flex-start}.set{grid-template-columns:1fr}.ctl{justify-content:flex-start}.set input,.set select,.set .pk{width:100%!important}.hello{display:block}.time{grid-template-columns:1fr;gap:24px}.bars .br{grid-template-columns:28px 110px 1fr 70px}}"#;

const SHARED_JS: &str = r#"<script>
(function(){
 const send=o=>ipc.postMessage(JSON.stringify(o)),$=i=>document.getElementById(i);
 document.addEventListener('click',e=>{const b=e.target.closest('[data-act]');if(!b)return;const a=b.dataset.act;
  if(a==='clear_history'){if(!confirm('Delete all '+(b.dataset.n||'')+' history entries, including any not shown here? This cannot be undone.'))return;send({cmd:'clear_history'});location.reload()}
  else if(a==='hrm'){send({cmd:'hist_remove',t:+b.dataset.t,url:b.dataset.u});const r=b.closest('.row');if(r)r.remove()}
  else if(a==='bmrm'){if(!confirm('Remove this bookmark?'))return;send({cmd:'bm_remove',url:b.dataset.url});setTimeout(()=>location.reload(),150)}
  else if(a==='restart'){send({cmd:'restart'})}
  else if(a==='opendata'){send({cmd:'open_data'})}
  else if(a==='tog'){b.classList.toggle('on');const o=b.classList.contains('on');b.textContent=o?'On':'Off';b.setAttribute('aria-checked',o);send({cmd:'set',key:b.dataset.key,val:o})}
  else if(a==='setpin'){send({cmd:'set_pin',old:$('op').value,new:$('np').value})}
  else if(a==='clearusage'){send({cmd:'clear_usage'});b.textContent='Cleared'}
  else if(a==='setsync'){send({cmd:'set_sync',path:$('sy').value});alert('Saved. Restart Owl to apply.')}
  else if(a==='setsyncpass'){send({cmd:'set_sync_pass',pass:$('sp').value});alert('Saved. Restart Owl to apply.')}
  else if(a==='bmedit'){const g=prompt('Move this bookmark to which group?',b.dataset.group||'General');if(g===null||!g.trim())return;send({cmd:'bm_group',url:b.dataset.url,group:g.trim()});setTimeout(()=>location.reload(),150)}
  else if(a==='http_allow'){send({cmd:'http_allow',url:b.dataset.url})}
  else if(a==='setprofile'){send({cmd:'set_profile',name:$('pn').value,email:$('pe').value});b.textContent='Saved'}
  else if(a==='setproxy'){send({cmd:'set_proxy',proxy:$('px').value})}
  else if(a==='back'){history.length>1?history.back():location.href='owl://newtab'}});
 document.addEventListener('change',e=>{const t=e.target;if(t&&t.dataset&&t.dataset.sel)send({cmd:'set_num',key:t.dataset.sel,val:+t.value})});
 document.addEventListener('error',e=>{const t=e.target;if(t&&t.tagName==='IMG'){if(t.dataset.hide!==undefined)t.style.visibility='hidden';else if(t.dataset.fb){const d=document.createElement('div');d.className='l';d.textContent=t.dataset.fb;t.replaceWith(d)}}},true);
 function cu(e){const n=+e.dataset.n;if(!n)return;if(matchMedia('(prefers-reduced-motion:reduce)').matches)return;const t0=performance.now();(function f(t){const k=Math.min(1,(t-t0)/1000);e.textContent=Math.round(n*(1-Math.pow(1-k,3)));if(k<1)requestAnimationFrame(f)})(t0)}
 const io=new IntersectionObserver(es=>es.forEach(e=>{if(e.isIntersecting){e.target.classList.add('in');e.target.querySelectorAll('[data-n]').forEach(cu);io.unobserve(e.target)}}),{threshold:.14});
 addEventListener('DOMContentLoaded',()=>document.querySelectorAll('.rv').forEach(e=>io.observe(e)));
})();
</script>"#;

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
/// Only http(s) and owl:// links are ever emitted as hrefs (never javascript: or data:).
fn safe_href(u: &str) -> String {
    if u.starts_with("http://") || u.starts_with("https://") || u.starts_with("owl://") {
        u.to_string()
    } else {
        "owl://newtab".to_string()
    }
}
fn host(u: &str) -> String {
    u.split("://").nth(1).unwrap_or(u).split('/').next().unwrap_or(u).trim_start_matches("www.").to_string()
}
fn page(title: &str, body: &str, head: &str, theme: &str, cls: &str) -> String {
    let css = CSS.replace("/*THEMES*/", crate::themes::THEMES);
    let n = crate::crypt::nonce_hex();
    // Strict CSP: only scripts carrying this per-response nonce run, so injected markup or
    // inline handlers can never execute inside these privileged pages.
    let csp = format!(
        "default-src 'none'; script-src 'nonce-{n}'; style-src 'unsafe-inline'; img-src http: https: data:; connect-src 'self'; base-uri 'none'; form-action 'none'; object-src 'none'"
    );
    let html = format!(
        "<!doctype html><html data-theme=\"{theme}\"><meta http-equiv=\"Content-Security-Policy\" content=\"{csp}\"><meta charset=utf-8><title>{title}</title>\
<script>document.documentElement.classList.add('js')</script>{SHARED_JS}{head}<style>{css}</style><body class=\"{cls}\">{body}</body></html>"
    );
    // everything dynamic was escaped, so every literal "<script>" here is Owl's own code
    html.replace("<script>", &format!("<script nonce={n}>"))
}
fn hdr(title: &str) -> String {
    format!("<header><span>Owl <i>/</i> {title}</span><span>Ultra-light browser</span></header>")
}
fn icon(i: &str, u: &str) -> String {
    let h = host(u);
    let l = h.chars().next().unwrap_or('?').to_uppercase().to_string();
    if i.is_empty() {
        format!("<div class=l>{}</div>", esc(&l))
    } else {
        format!("<img src=\"{}\" data-fb=\"{}\">", esc(i), esc(&l))
    }
}
fn fmt_size(b: i64) -> String {
    let b = b as f64;
    if b >= 1e9 { format!("{:.2} GB", b / 1e9) } else if b >= 1e6 { format!("{:.1} MB", b / 1e6) } else if b >= 1e3 { format!("{:.0} KB", b / 1e3) } else { format!("{b:.0} B") }
}
fn dur_txt(s: u64) -> String {
    let (h, m) = (s / 3600, (s % 3600) / 60);
    if h > 0 { format!("{h}h {m:02}m") } else if m > 0 { format!("{m}m") } else if s > 0 { "<1m".into() } else { "0m".into() }
}

const NEWTAB_JS: &str = r#"<script>const p=n=>String(n).padStart(2,'0');function t(){const d=new Date(),h=d.getHours();c.innerHTML=p(h)+':'+p(d.getMinutes())+'<small>'+p(d.getSeconds())+'</small>';
const nm=document.querySelector('.hello').dataset.name;gr.textContent=(h<5?'Good night':h<12?'Good morning':h<18?'Good afternoon':'Good evening')+(nm?', '+nm:'');
wd.textContent=d.toLocaleDateString('en',{weekday:'long'});dt.textContent=d.toLocaleDateString('en',{day:'numeric',month:'long',year:'numeric'})}t();setInterval(t,1000);
f.onsubmit=e=>{e.preventDefault();const v=q.value.trim();if(!v)return;
location.href=/^[a-z]+:\/\//i.test(v)?v:(!/\s/.test(v)&&/\./.test(v))?'https://'+v:'https://duckduckgo.com/?q='+encodeURIComponent(v)};
addEventListener('keydown',e=>{if(document.activeElement===q||e.ctrlKey||e.altKey||e.metaKey)return;
if(e.key==='/'){e.preventDefault();scrollTo(0,0);q.focus()}else if(/^[1-8]$/.test(e.key)){const a=document.querySelectorAll('.tile')[+e.key-1];if(a)location.href=a.href}});
document.querySelectorAll('.tile,.stat,.key,.grp,.br').forEach((e,i)=>e.style.setProperty('--d',(i%8)*55+'ms'));</script>"#;

pub fn newtab(s: &Store, c: &Config, sess: u32, total: u64, pw_count: usize, today: u64, theme: &str) -> String {
    // recent sites
    let mut seen = HashSet::new();
    let mut tiles = String::new();
    for e in s.history.iter().rev() {
        let h = host(&e.url);
        if !seen.insert(h.clone()) {
            continue;
        }
        let src = if e.icon.is_empty() {
            s.bookmarks.iter().find(|b| host(&b.url) == h).map(|b| b.icon.clone()).unwrap_or_default()
        } else {
            e.icon.clone()
        };
        tiles += &format!(
            "<a class=tile href=\"{}\"><span class=k>{:02}</span><div>{}</div><span class=h>{}</span></a>",
            esc(&safe_href(&e.url)), seen.len(), icon(&src, &e.url), esc(&h)
        );
        if seen.len() >= 8 {
            break;
        }
    }
    if tiles.is_empty() {
        tiles = "<div class=empty style=\"grid-column:1/5\">Sites you visit will appear here. Press 1\u{2013}8 to open one.</div>".into();
    }
    // bookmark groups
    let mut groups: Vec<(String, Vec<&crate::Entry>)> = vec![];
    for b in &s.bookmarks {
        let g = if b.group.is_empty() { "General".to_string() } else { b.group.clone() };
        match groups.iter_mut().find(|(n, _)| *n == g) {
            Some((_, v)) => v.push(b),
            None => groups.push((g, vec![b])),
        }
    }
    groups.sort_by(|a, b| a.0.cmp(&b.0));
    let mut gh = String::new();
    for (g, v) in &groups {
        gh += &format!("<div class=grp><h3>{}</h3>", esc(g));
        for b in v {
            let t = if b.title.is_empty() { host(&b.url) } else { b.title.clone() };
            gh += &format!("<a href=\"{}\"><img src=\"{}\" data-hide>{}</a>", esc(&safe_href(&b.url)), esc(&b.icon), esc(&t));
        }
        gh += "</div>";
    }
    if gh.is_empty() {
        gh = "<div class=empty style=\"grid-column:1/5\">Press the star in the address bar to bookmark a page into a group.</div>".into();
    }
    // time spent per site
    let mut sites: Vec<(&String, &u64)> = s.usage.iter().collect();
    sites.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let all: u64 = s.usage.values().sum();
    let max = sites.first().map(|x| *x.1).unwrap_or(1).max(1);
    let mut bars = String::new();
    for (i, (h, secs)) in sites.iter().take(6).enumerate() {
        bars += &format!(
            "<div class=br><span class=n>{:02}</span><span class=bn>{}</span><div class=bb><i style=\"--w:{}%\"></i></div><span class=bt>{}</span></div>",
            i + 1, esc(h), (**secs * 100 / max).max(2), dur_txt(**secs)
        );
    }
    if bars.is_empty() {
        bars = "<div class=empty>Owl counts the minutes each site is open in the foreground. It stays on this PC, and private windows are never counted.</div>".into();
    }
    let (th, tm) = (today / 3600, (today % 3600) / 60);
    let (ah, am) = (all / 3600, (all % 3600) / 60);
    // downloads
    let mut dl = String::new();
    for d in s.downloads.iter().rev().take(4) {
        let pct = if d.total > 0 { (d.received * 100 / d.total.max(1)).min(100) } else if d.status == "done" { 100 } else { 0 };
        dl += &format!(
            "<div class=drow><div><div class=nm>{}</div><div class=bar><i style=\"width:{pct}%\"></i></div></div><span class=u>{}</span><span class=st>{}</span></div>",
            esc(&d.name), if d.total > 0 { fmt_size(d.total) } else { String::new() }, esc(&d.status)
        );
    }
    if dl.is_empty() {
        dl = "<div class=empty>Nothing downloaded yet. Pause, resume and sort files from the manager.</div>".into();
    }
    let on = |b: bool| if b { "On" } else { "Off" };
    let mut body = format!(
        "<div class=land>\
<section class=hero>{hdr}<div class=hcore>\
<div class=hello data-name=\"{name}\"><span><b id=gr></b><span id=wd></span></span><span id=dt></span></div>\
<div class=clock id=c>00:00<small>00</small></div>\
<form class=s id=f><input id=q autofocus autocomplete=off spellcheck=false placeholder=\"Search privately or type an address\"><kbd>/</kbd></form></div>\
<div class=scroll><span>Scroll</span><span><a href=\"#quick\">Jump back in</a> \u{00b7} <a href=\"#priv\">Privacy</a> \u{00b7} <a href=\"#time\">Time</a> \u{00b7} <a href=\"#dl\">Downloads</a> \u{00b7} <a href=\"owl://about\">About</a></span></div></section>\
<section class=\"rv q\" id=quick><h2 style=\"margin-bottom:20px\">Quick access <span>press 1\u{2013}8</span></h2><div class=\"huge sm\">Jump back in<em>.</em></div><div class=tiles>{tiles}</div><h2>Bookmarks</h2><div class=groups>{gh}</div></section>\
<section class=\"inv rv\" id=priv><div class=huge>Private by default<em>.</em></div><div class=stats>\
<a class=stat href=\"owl://settings\"><b class=r data-n=\"{total}\">{total}</b><span>trackers and ads blocked, all time</span></a>\
<a class=stat href=\"owl://settings\"><b data-n=\"{sess}\">{sess}</b><span>blocked this session. Blocking is {}</span></a>\
<a class=stat href=\"owl://settings\"><b>{}</b><span>HTTPS-only mode, refuses plain http</span></a>\
<a class=stat href=\"owl://passwords\"><b data-n=\"{pw_count}\">{pw_count}</b><span>passwords, encrypted by Windows for your account</span></a></div></section>\
<section class=rv id=time><h2 style=\"margin-bottom:28px\">Time online <span>stays on this PC</span></h2><div class=huge>Where your time goes<em>.</em></div>\
<div class=time><div class=tbig><b><span data-n=\"{th}\">{th}</span>h <span data-n=\"{tm}\">{tm}</span>m</b><span>today</span>\
<b><span data-n=\"{ah}\">{ah}</span>h <span data-n=\"{am}\">{am}</span>m</b><span>all time</span></div><div class=bars>{bars}</div></div></section>\
<section class=\"dls rv\" id=dl><div class=huge>Downloads<em>.</em></div>{dl}<p style=\"margin-top:24px\"><a class=btn href=\"owl://downloads\">Open manager</a></p></section>\
</div>",
        on(c.block), on(c.https_only),
        name = esc(&s.name),
        hdr = hdr("Home")
    );
    body.push_str(NEWTAB_JS);
    page("Home", &body, "", theme, "landing")
}

fn list_page(title: &str, rows: &str, extra: &str, head: &str, theme: &str) -> String {
    page(title, &format!("{}<main class=grid><div class=sec><h1>{title}<em>.</em></h1>{extra}{rows}</div></main>", hdr(title)), head, theme, "")
}

/// One empty-state pattern everywhere: a short sentence and one way forward.
fn empty(msg: &str, link: Option<(&str, &str)>) -> String {
    match link {
        Some((href, label)) => format!("<div class=empty>{msg} <a href=\"{href}\">{label}</a></div>"),
        None => format!("<div class=empty>{msg}</div>"),
    }
}

pub fn notfound(path: &str, theme: &str) -> String {
    let shown: String = path.chars().take(120).collect();
    let body = format!(
        "{}<main class=grid><div class=sec><h1>Not found<em>.</em></h1>\
<p style=\"max-width:60ch;margin-bottom:24px\">There is no Owl page at <code>owl://{}</code>.</p>\
<p><a class=btn href=\"owl://newtab\">Go home</a></p></div></main>",
        hdr("404"), esc(shown.trim_start_matches('/'))
    );
    page("Not found", &body, "", theme, "")
}

pub fn history(s: &Store, theme: &str) -> String {
    let mut o = String::new();
    let total = s.history.len();
    for (i, e) in s.history.iter().rev().take(500).enumerate() {
        let t = if e.title.is_empty() { &e.url } else { &e.title };
        o += &format!(
            "<div class=\"row hv h\" style=\"--i:{}\" data-t=\"{}\" data-q=\"{}\"><span class=n>{:02}</span><a class=t href=\"{h}\">{}</a><span class=u>{u}</span>\
<span class=d><time></time><button class=\"ghost sm\" data-act=hrm data-t=\"{}\" data-u=\"{u}\" aria-label=\"Remove {} from history\">Remove</button></span></div>",
            i.min(14), e.t, esc(&format!("{} {}", t, e.url).to_lowercase()), i + 1, esc(t), e.t, esc(t),
            u = esc(&e.url), h = esc(&safe_href(&e.url))
        );
    }
    let list = if o.is_empty() { empty("No history yet. Pages you visit appear here.", None) } else { format!("<div id=hl>{o}</div>") };
    let extra = format!(
        "<div class=tb><label class=vh for=hq>Search history</label><input class=pf id=hq placeholder=\"Search history\" style=\"flex:1;min-width:200px\"{}><button class=ghost data-act=clear_history data-n=\"{total}\"{}>Clear history</button></div>",
        if total == 0 { " disabled" } else { "" },
        if total == 0 { " disabled" } else { "" }
    );
    let js = r#"<script>
(function(){const L=document.getElementById('hl');if(!L)return;
 const rows=[...L.querySelectorAll('.row')],fmt=d=>d.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'});
 const day=d=>{const n=new Date(),a=new Date(n.getFullYear(),n.getMonth(),n.getDate()),b=new Date(d.getFullYear(),d.getMonth(),d.getDate()),k=Math.round((a-b)/864e5);
  return k===0?'Today':k===1?'Yesterday':d.toLocaleDateString([], {weekday:'long',day:'numeric',month:'long'})};
 let last='';rows.forEach(r=>{const d=new Date(+r.dataset.t*1000);r.querySelector('time').textContent=fmt(d);const g=day(d);
  if(g!==last){last=g;const h=document.createElement('h2');h.className='dh';h.textContent=g;L.insertBefore(h,r)}});
 document.getElementById('hq').oninput=e=>{const v=e.target.value.toLowerCase();rows.forEach(r=>r.style.display=r.dataset.q.includes(v)?'':'none');
  L.querySelectorAll('.dh').forEach(h=>{let n=h.nextElementSibling,any=false;while(n&&!n.classList.contains('dh')){if(n.style.display!=='none')any=true;n=n.nextElementSibling}h.style.display=any?'':'none'})};
})();
</script>"#;
    list_page("History", &format!("{list}{js}"), &extra, "<style>.dh{margin-top:26px}.vh{position:absolute;left:-9999px}</style>", theme)
}

pub fn bookmarks(s: &Store, theme: &str) -> String {
    let mut o = String::new();
    let mut v: Vec<&crate::Entry> = s.bookmarks.iter().collect();
    v.sort_by(|a, b| a.group.cmp(&b.group));
    let mut cur = String::from("\u{0}");
    for (i, e) in v.into_iter().enumerate() {
        if e.group != cur {
            cur = e.group.clone();
            o += &format!("<h2 style=\"margin-top:28px\">{}</h2>", esc(if cur.is_empty() { "General" } else { &cur }));
        }
        let t = if e.title.is_empty() { &e.url } else { &e.title };
        o += &format!(
            "<div class=\"row hv\" style=\"--i:{}\"><span class=n>\u{2605}</span><a class=t href=\"{h}\">{}</a><span class=u>{u}</span>\
<span class=d><button class=\"ghost sm\" data-act=bmedit data-url=\"{u}\" data-group=\"{}\" aria-label=\"Move {} to another group\">Move</button>\
<button class=\"ghost sm\" data-act=bmrm data-url=\"{u}\" aria-label=\"Remove {} from bookmarks\">Remove</button></span></div>",
            i.min(14), esc(t), esc(&e.group), esc(t), esc(t), u = esc(&e.url), h = esc(&safe_href(&e.url)),
        );
    }
    if o.is_empty() {
        o = empty("No bookmarks yet. Press the star in the address bar to save a page.", None);
    }
    list_page("Bookmarks", &o, "", "", theme)
}

pub fn downloads(theme: &str) -> String {
    let css = r#"<style>
.tb{display:flex;gap:8px;align-items:center;margin-bottom:20px;flex-wrap:wrap}.tb .sp{flex:1}
.agg{font-size:13px;color:var(--mute);font-variant-numeric:tabular-nums}
.dr{display:grid;grid-template-columns:48px 1fr 190px 230px;gap:20px;padding:16px 0;border-top:1px solid var(--line);align-items:center;animation:rise .4s cubic-bezier(.2,.7,.2,1) both}
.dr .nm{font-size:18px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.dr .pa{font-size:12px;color:var(--mute);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;margin-top:2px}
.dr .bar{margin-top:8px}.dr .bar i{transition:width .5s ease}.dr .inf{font-size:12px;font-variant-numeric:tabular-nums}.dr .inf b{display:block;font-size:11px;letter-spacing:.12em;text-transform:uppercase;margin-bottom:3px}
.dr .ac{display:flex;gap:6px;justify-content:flex-end;flex-wrap:wrap}.dr .ac button{min-height:32px;padding:0 12px;font-size:11px}
.dr.done .bar i{background:var(--ink)}.dr.failed .bar i,.dr.cancelled .bar i{background:var(--mute)}.dr.paused .bar i{background:var(--mute)}
button[disabled]{opacity:.45;cursor:default}
@media(max-width:900px){.dr{grid-template-columns:1fr}.dr .ac{justify-content:flex-start}}
</style>"#;
    let extra = r#"<div class=tb role=group aria-label="Filter downloads"><button class=on data-f=all aria-pressed=true>All</button><button class=ghost data-f=active aria-pressed=false>Active</button><button class=ghost data-f=done aria-pressed=false>Completed</button><span class=sp></span><span class=agg id=agg aria-live=polite></span><button class=ghost id=clr disabled>Clear finished</button></div><div id=list><div class=empty>Files you download appear here.</div></div>
<script>
const send=o=>ipc.postMessage(JSON.stringify(o));let F='all',D=[],sig='';
const sz=b=>b>=1e9?(b/1e9).toFixed(2)+' GB':b>=1e6?(b/1e6).toFixed(1)+' MB':b>=1e3?Math.round(b/1e3)+' KB':b+' B';
const eta=s=>{if(!isFinite(s)||s<=0)return '';s=Math.round(s);return s>3600?Math.floor(s/3600)+'h '+Math.floor(s%3600/60)+'m':s>60?Math.floor(s/60)+'m '+s%60+'s':s+'s'};
const act=d=>d.status==='downloading'||d.status==='paused'||d.status==='pending';
function draw(){const L=document.getElementById('list');const rows=D.filter(d=>F==='all'||(F==='active'?act(d):d.status==='done'));
 if(!rows.length){L.innerHTML='<div class=empty>'+(F==='active'?'Nothing is downloading right now.':F==='done'?'No completed downloads yet.':'Files you download appear here.')+'</div>';return}
 const first=!L.querySelector('.dr');
 L.innerHTML=rows.map((d,i)=>{const pct=d.total>0?Math.min(100,d.received*100/d.total):(d.status==='done'?100:0);
  const sp=d.status==='downloading'?sz(d.speed)+'/s':'';const e=d.status==='downloading'&&d.total>0&&d.speed>0?eta((d.total-d.received)/d.speed):'';
  const b=[];if(d.status==='downloading')b.push(['Pause','dl_pause']);if(d.status==='paused')b.push(['Resume','dl_resume']);
  if(act(d))b.push(['Cancel','dl_cancel']);if(d.status==='done'){b.push(['Open','dl_open']);b.push(['Show','dl_reveal'])}
  if(d.status==='failed'||d.status==='cancelled'||d.status==='interrupted')b.push(['Retry','dl_retry']);if(!act(d))b.push(['Remove','dl_remove']);
  return '<div class="dr '+d.status+'" style="'+(first?'animation-delay:'+Math.min(i,10)*30+'ms':'animation:none')+'"><span class=n>'+String(i+1).padStart(2,'0')+'</span><div><div class=nm></div><div class=pa></div><div class=bar role=progressbar aria-valuemin=0 aria-valuemax=100 aria-valuenow='+Math.round(pct)+'><i style="width:'+pct+'%"></i></div></div>'+
  '<div class=inf><b>'+d.status+'</b>'+sz(d.received)+(d.total>0?' of '+sz(d.total):'')+(sp?' &middot; '+sp:'')+(e?' &middot; '+e:'')+'</div><div class=ac>'+b.map(x=>'<button class=ghost data-c='+x[1]+' data-i='+d.id+'>'+x[0]+'</button>').join('')+'</div></div>'}).join('');
 L.querySelectorAll('.dr').forEach((el,i)=>{el.querySelector('.nm').textContent=rows[i].name;el.querySelector('.pa').textContent=rows[i].path;el.querySelector('.nm').title=rows[i].url});
 L.querySelectorAll('button[data-c]').forEach(x=>x.onclick=()=>send({cmd:x.dataset.c,id:+x.dataset.i}))}
async function poll(){try{D=await (await fetch('/downloads.json',{cache:'no-store'})).json();const a=D.filter(d=>d.status==='downloading');
 document.getElementById('agg').textContent=a.length?a.length+' active \u00b7 '+sz(a.reduce((s,d)=>s+d.speed,0))+'/s':D.length+(D.length===1?' file':' files');
 document.getElementById('clr').disabled=!D.some(d=>!act(d));
 const k=JSON.stringify(D)+F;if(k!==sig){sig=k;draw()}}catch(e){}}
document.querySelectorAll('.tb button[data-f]').forEach(b=>b.onclick=()=>{F=b.dataset.f;document.querySelectorAll('.tb button[data-f]').forEach(x=>{x.className=x===b?'on':'ghost';x.setAttribute('aria-pressed',x===b)});sig='';poll()});
document.getElementById('clr').onclick=()=>{send({cmd:'dl_clear'});setTimeout(poll,150)};poll();setInterval(poll,700);
</script>"#;
    list_page("Downloads", "", extra, css, theme)
}

pub fn passwords(list: &[(String, String)], theme: &str) -> String {
    let mut o = String::new();
    for (i, (h, u)) in list.iter().enumerate() {
        o += &format!(
            "<div class=\"row pw\" data-h=\"{h}\" data-u=\"{u}\" data-q=\"{q}\" style=\"--i:{}\"><span class=n>{:02}</span><span class=t>{h}</span><span class=u id=u{i}>{u}</span><span class=d>\
<button class=\"ghost sm\" data-a=cu data-i={i} aria-label=\"Copy username for {h}\">Copy user</button>\
<button class=\"ghost sm\" data-a=show data-i={i} aria-label=\"Show password for {h}\">Show</button>\
<button class=\"ghost sm\" data-a=del data-i={i} aria-label=\"Delete password for {h}\">Delete</button></span></div>",
            i.min(14), i + 1, h = esc(h), u = esc(u), q = esc(&format!("{h} {u}").to_lowercase())
        );
    }
    if o.is_empty() {
        o = empty("No saved passwords yet. Owl offers to save one when you sign in to a site, or add one above.", None);
    }
    let extra = "<p style=\"color:var(--mute);margin-bottom:22px;max-width:62ch\">Passwords are encrypted with Windows (DPAPI) plus a secret derived from your startup key or passphrase. Showing or replacing one asks for it again. Saved logins fill in when you click a sign-in field.</p>\
<form class=af id=af><input id=ah aria-label=\"Website\" placeholder=\"Website, e.g. example.com\" required autocomplete=off><input id=au aria-label=\"Username or email\" placeholder=\"Username or email\" autocomplete=off><input id=ap aria-label=\"Password\" type=password placeholder=Password required autocomplete=off>\
<button type=button class=ghost id=gen>Generate</button><button type=button class=ghost id=vw>Show</button><button type=submit>Save</button></form>\
<input class=pf id=q aria-label=\"Search saved passwords\" placeholder=\"Search saved passwords\" style=\"width:100%;margin-bottom:14px\">";
    let css = "<style>.af{display:grid;grid-template-columns:1.2fr 1fr 1fr auto auto auto;gap:8px;margin-bottom:22px}.af input{width:100%!important;min-width:0}\
.row.pw{grid-template-columns:56px 1.2fr 1fr auto}.row.pw .d{display:flex;gap:6px;justify-content:flex-end;flex-wrap:wrap;overflow:visible}\
@media(max-width:900px){.af{grid-template-columns:1fr 1fr}.row.pw{grid-template-columns:40px 1fr}.row.pw .u{display:block}.row.pw .d{grid-column:2;justify-content:flex-start}}</style>";
    let js = r#"<script>
const send=o=>ipc.postMessage(JSON.stringify(o));
const rows=[...document.querySelectorAll('.row[data-h]')];
const norm=h=>h.toLowerCase().replace(/^[a-z]+:\/\//,'').replace(/^www\./,'').split('/')[0];
q.oninput=()=>{const v=q.value.toLowerCase();rows.forEach(r=>r.style.display=r.dataset.q.includes(v)?'':'none')};
gen.onclick=()=>{const c='ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%^&*-_=+';const a=new Uint32Array(20);crypto.getRandomValues(a);ap.type='text';ap.value=[...a].map(x=>c[x%c.length]).join('')};
vw.onclick=()=>{ap.type=ap.type==='password'?'text':'password'};
af.onsubmit=e=>{e.preventDefault();const h=norm(ah.value.trim()),u=au.value.trim();let pin='';
 if(rows.some(r=>r.dataset.h===h&&r.dataset.u===u)){pin=prompt('This replaces a saved password. Enter your startup key or passphrase')||'';if(!pin)return}
 send({cmd:'pw_add',host:h,user:u,pass:ap.value,pin});setTimeout(()=>location.reload(),350)};
rows.forEach((r,i)=>{r.querySelectorAll('button').forEach(b=>b.onclick=()=>{const a=b.dataset.a;
 if(a==='cu'){navigator.clipboard.writeText(r.dataset.u).then(()=>{b.textContent='Copied';setTimeout(()=>b.textContent='Copy user',1200)})}
 else if(a==='show'){const p=prompt('Enter your startup key or passphrase');if(p)send({cmd:'pw_reveal',i,pin:p})}
 else if(a==='del'&&confirm('Delete this password?')){send({cmd:'pw_delete',i});setTimeout(()=>location.reload(),250)}})});
window.revealed=(i,pw)=>{const e=document.getElementById('u'+i);const u=rows[i].dataset.u;e.textContent=u+'  \u00b7  '+pw;setTimeout(()=>e.textContent=u,10000)};
</script>"#;
    list_page("Passwords", &format!("{o}{js}"), extra, css, theme)
}

const THEME_CARDS: &[(&str, &str, &str, &str, &str)] = &[
    ("system", "System", "linear-gradient(90deg,#fff 50%,#101010 50%)", "#777", "#e30613"),
    ("light", "Light", "#fff", "#111", "#e30613"),
    ("dark", "Dark", "#101010", "#f4f4f4", "#ff3b30"),
    ("black", "Black", "#000", "#fff", "#ff2b1f"),
    ("navy", "Navy", "#0b1a3a", "#eef3ff", "#ff5a36"),
    ("invert", "Invert", "#e30613", "#fff", "#111"),
    ("sand", "Sand", "#efe9dc", "#171717", "#b8290f"),
    ("random", "Random", "conic-gradient(#e30613,#0b1a3a,#efe9dc,#000,#e30613)", "#fff", "#111"),
];

pub fn settings(c: &Config, exts: &[String], dir: &str, prof: (&str, &str), theme: &str) -> String {
    // a switch: the state is exposed to assistive technology and shown as On / Off
    let tog = |i: usize, key: &str, on: bool, title: &str, desc: &str| {
        format!(
            "<div class=set style=\"--i:{i}\"><div><b id=l-{key}>{title}</b><p>{desc}</p></div><button class=\"tg ghost{}\" role=switch aria-checked={on} aria-labelledby=l-{key} data-act=tog data-key=\"{key}\">{}</button></div>",
            if on { " on" } else { "" }, if on { "On" } else { "Off" }
        )
    };
    let opts: String = [(0u32, "Never"), (5, "5 minutes"), (15, "15 minutes"), (30, "30 minutes"), (60, "1 hour")]
        .iter()
        .map(|(v, l)| format!("<option value={v}{}>{l}</option>", if c.autolock == *v { " selected" } else { "" }))
        .collect();
    let mut cards = String::new();
    for (i, (id, label, bg, ink, acc)) in THEME_CARDS.iter().enumerate() {
        let on = c.theme == *id;
        cards += &format!(
            "<button class=\"th{}\" data-t=\"{id}\" aria-pressed={on} aria-label=\"{label} theme\" style=\"--i:{i};--b:{bg};--i2:{ink};--a:{acc}\"><span class=pv style=\"--i:{ink}\"><u></u></span><span class=l>{label}</span></button>",
            if on { " on" } else { "" }
        );
    }
    let ex = if exts.is_empty() { "none".to_string() } else { exts.join(", ") };
    let theme_js = r#"<script>
document.querySelectorAll('.th').forEach(b=>b.onclick=()=>{const t=b.dataset.t,live=!['system','random'].includes(t);
 document.querySelectorAll('.th').forEach(x=>{x.classList.toggle('on',x===b);x.setAttribute('aria-pressed',x===b)});
 if(live)document.documentElement.dataset.theme=t;
 ipc.postMessage(JSON.stringify({cmd:'set',key:'theme',val:t}));
 if(!live)setTimeout(()=>location.reload(),300)});
</script>"#;
    let appearance = format!("<h2 class=gh>Appearance <span>Swiss palettes. Applies instantly.</span></h2><div class=themes role=group aria-label=\"Theme\">{cards}</div>{}", tog(8, "bar", c.bookmark_bar, "Bookmark bar", "Show bookmarks above the tabs (Ctrl+B)."));
    let security = format!(
        "<h2 class=gh>Security</h2>\
<div class=set><div><b>Startup key</b><p><b>Use a passphrase of 10 or more characters.</b> It protects your saved passwords, history, bookmarks and cookies. A 4-digit key can be guessed offline.</p>\
<details><summary>How it works</summary><p>The key is stretched with Argon2id and wrong guesses lock you out for longer each time. Data is sealed when Owl locks or closes cleanly; if Owl is killed it stays readable until the next lock or start. The default key before setup is 1966.</p></details></div>\
<div class=ctl><input class=pk type=password id=op maxlength=128 autocomplete=off aria-label=\"Current key\" placeholder=\"Current key\"><input class=pk type=password id=np maxlength=128 autocomplete=off aria-label=\"New key or passphrase\" placeholder=\"New key or passphrase\"><button data-act=setpin>Change</button></div></div>\
<div class=set><div><b>Auto-lock</b><p>After this long without use, Owl locks, closes every page and encrypts your data on disk. Playing media and downloads count as use. Unlocking reopens your tabs.</p></div><select class=pk data-sel=autolock aria-label=\"Auto-lock after\">{opts}</select></div>{}{}",
        tog(15, "lock_win", c.lock_on_winlock, "Lock when Windows locks or sleeps", "Locks Owl and encrypts your data when you lock the PC (Win+L), sleep or shut down. Typed text and scroll position are restored on unlock."),
        tog(12, "pw_save", c.pw_save, "Offer to save passwords", "Ask to save a password after you sign in to a site.")
    );
    let privacy = format!(
        "<h2 class=gh>Privacy</h2>{}{}{}{}{}{}\
<div class=set><div><b>Proxy or VPN</b><p>Send all page traffic through a proxy to hide your IP address, for example <code>socks5://127.0.0.1:9050</code> (Tor). WebRTC is forced through it too. Empty means direct.</p></div>\
<div class=ctl><input class=pk id=px maxlength=100 aria-label=\"Proxy address\" placeholder=\"socks5://127.0.0.1:9050\" value=\"{}\"><button class=ghost data-act=setproxy>Save</button></div></div>",
        tog(9, "block", c.block, "Block trackers and ads", "Blocks requests to known ad, analytics and tracking hosts."),
        tog(10, "https_only", c.https_only, "HTTPS-only mode", "Upgrades http:// pages to https:// and warns before using plain http."),
        tog(13, "fp", c.fp, "Fingerprint protection", "Adds invisible noise to canvas and audio reads, hides the exact GPU model and rounds hardware hints. Applies to new pages."),
        tog(17, "track_strict", c.track_strict, "Strict tracking prevention", "Blocks third-party trackers and their stored cookies more aggressively. Some embedded logins and videos may stop working. Applies to new pages."),
        tog(16, "perm_block", c.perm_block, "Block camera, microphone, location and notifications", "Refuses these requests without asking. Sensors, local fonts and MIDI system access are always refused. Turn this off to be asked per site."),
        tog(14, "fp_strict", c.fp_strict, "Strict fingerprint protection", "Also reports the same screen size, language and plugin list to every site. Sites may show English or odd layouts."),
        esc(&c.proxy)
    );
    let data = format!(
        "<h2 class=gh>Downloads and data</h2>{}\
<div class=set><div><b>Time tracking</b><p>Minutes per site are kept on this PC for the home page. Clear them any time.</p></div><button class=ghost data-act=clearusage>Clear</button></div>",
        tog(11, "organize", c.organize, "Sort downloads into folders", "Documents, Images, Video, Music, Archives and Programs inside your Downloads folder.")
    );
    let sync = format!(
        "<h2 class=gh>Sync</h2>\
<div class=set><div><b>Sync folder</b><p>Keep bookmarks, history and open tabs in a folder synced by OneDrive, Dropbox or Syncthing. Set the passphrase below first. Restart Owl after changing.</p></div>\
<div class=ctl><input class=pk id=sy aria-label=\"Sync folder\" value=\"{}\" placeholder=\"C:\\Users\\you\\OneDrive\\Owl\"><button data-act=setsync>Save</button></div></div>\
<div class=set><div><b>Sync passphrase</b><p>Encrypts the sync file (ChaCha20-Poly1305). {} Use the same passphrase on each PC. Restart Owl to apply.</p></div>\
<div class=ctl><input class=pk id=sp type=password autocomplete=off aria-label=\"Sync passphrase\" placeholder=\"Passphrase\"><button data-act=setsyncpass>Save</button></div></div>",
        esc(&c.sync_dir),
        if c.sync_key.is_empty() { "Not set yet." } else { "A passphrase is set." }
    );
    let advanced = format!(
        "<h2 class=gh>Profile and advanced</h2>\
<div class=set><div><b>Your profile</b><p>Set at first run. Kept in the encrypted store on this PC and used only for the home-page greeting.</p></div>\
<div class=ctl><input class=pk id=pn maxlength=60 aria-label=\"Name\" placeholder=Name value=\"{}\"><input class=pk id=pe maxlength=120 aria-label=\"Email\" placeholder=Email value=\"{}\"><button class=ghost data-act=setprofile>Save</button></div></div>\
<div class=set><div><b>Extensions (user scripts)</b><p>Put .js files in <code>{}\\extensions</code>. They run on every web page. Loaded: {}</p></div><button class=ghost data-act=opendata>Open Owl folder</button></div>\
<div class=set><div><b>Restart Owl</b><p>Some settings (proxy, sync) apply after a restart.</p></div><button class=ghost data-act=restart>Restart now</button></div>\
<div class=set><div><b>About and credits</b><p>Icons are from Lucide (ISC licence). The page engine is Microsoft Edge WebView2.</p></div><a class=btn href=\"owl://about\">About Owl</a></div>",
        esc(prof.0), esc(prof.1), esc(dir), esc(&ex)
    );
    list_page("Settings", &format!("{appearance}{security}{privacy}{data}{sync}{advanced}{theme_js}"), "", "", theme)
}

pub fn httpsfail(url: &str, theme: &str) -> String {
    let safe = if url.starts_with("http://") { url } else { "http://invalid" };
    let body = format!(
        "{}<main class=grid><div class=sec><h1>Not secure<em>.</em></h1>\
<p style=\"max-width:60ch;margin-bottom:24px\"><b class=warn>Warning.</b> HTTPS-only mode is on, but <b>{}</b> did not load over https. The site may not support it, or someone may be blocking the secure connection.</p>\
<p style=\"display:flex;gap:18px;align-items:center;flex-wrap:wrap\"><button data-act=back>Go back</button><a class=lnk href=\"#\" data-act=http_allow data-url=\"{}\" role=button>Continue over http anyway (not secure)</a></p></div></main>",
        hdr("HTTPS"), esc(safe), esc(safe)
    );
    page("Not secure", &body, "<style>.warn{color:var(--red)}.lnk{font-size:13px;color:var(--mute);text-decoration:underline;display:inline-block;padding:6px 2px;min-height:24px}.lnk:hover{color:var(--red)}</style>", theme, "")
}

pub fn about(theme: &str) -> String {
    let body = format!(
        "<div class=land>{}\
<section class=\"rv\"><div class=huge>Keyboard first<em>.</em></div><div class=keys>\
<div class=key><kbd>Ctrl T</kbd><span>New tab</span></div><div class=key><kbd>Ctrl L</kbd><span>Address bar</span></div>\
<div class=key><kbd>Ctrl D</kbd><span>Bookmark</span></div><div class=key><kbd>Ctrl F</kbd><span>Find in page</span></div>\
<div class=key><kbd>Ctrl Tab</kbd><span>Next tab (Ctrl 1-9 jump)</span></div><div class=key><kbd>Ctrl H</kbd><span>History</span></div>\
<div class=key><kbd>Alt \u{2190} \u{2192}</kbd><span>Back and forward</span></div><div class=key><kbd>Ctrl Shift L</kbd><span>Lock Owl</span></div></div></section>\
<section class=\"fin rv\"><div class=word>Owl.</div><div class=facts>\
<div><b>About 1 MB</b><span>the whole browser</span></div><div><b>Rust</b><span>built with tao and wry</span></div>\
<div><b>No account</b><span>nothing to sign in to</span></div><div><b>Encrypted</b><span>your data is sealed when Owl locks</span></div></div>\
<div class=foot><span>Owl</span><span>Ultra-light browser</span></div></section></div>",
        hdr("About")
    );
    page("About", &body, "", theme, "landing")
}
