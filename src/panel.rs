//! The small floating panel that opens from the toolbar: Data saver, Secure DNS, OwlKick and site
//! information. It is its own little webview, like an extension popup, so it floats over the page
//! instead of pushing a strip across the window. It talks to the app with the same trusted
//! commands as the toolbar.

pub const HTML: &str = r#"<!doctype html><html data-theme="system"><meta charset="utf-8"><style>
/*THEMES*/
*{box-sizing:border-box;margin:0;padding:0}
html,body{background:var(--bg);color:var(--ink);font:13px/1.45 "Segoe UI","Helvetica Neue",Helvetica,Arial,sans-serif;overflow:hidden;user-select:none}
#p{border:1px solid var(--ink);border-top:3px solid var(--red);padding:14px 16px 16px;opacity:0;transform:translateY(6px);transition:opacity .16s,transform .2s cubic-bezier(.2,.8,.2,1)}
#p.in{opacity:1;transform:none}
header{display:flex;align-items:center;gap:10px;margin-bottom:12px}
header svg{width:18px;height:18px;fill:none;stroke:currentColor;stroke-width:1.9;stroke-linecap:round;stroke-linejoin:round;color:var(--red);flex:none}
header b{font-size:15px;letter-spacing:-.01em;flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.x{background:none;border:0;color:var(--mute);width:24px;height:24px;cursor:pointer;font-size:16px;line-height:1}.x:hover{color:var(--ink)}
.big{font-size:26px;font-weight:700;letter-spacing:-.03em;line-height:1.05;font-family:"Space Grotesk","Segoe UI",sans-serif}
.cap{font-size:11px;letter-spacing:.12em;text-transform:uppercase;color:var(--mute);font-weight:700;margin-top:3px}
.row{display:flex;justify-content:space-between;align-items:center;gap:12px;padding:9px 0;border-top:1px solid var(--line)}
.row span{color:var(--mute)}.row b{font-weight:600;text-align:right;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:62%}
.row b.bad{color:var(--red)}
.nums{display:grid;grid-template-columns:1fr 1fr;gap:12px;margin:4px 0 10px}
.seg{display:grid;grid-template-columns:repeat(4,1fr);border:1px solid var(--ink);margin:6px 0 10px}
.seg button{background:var(--bg);color:var(--ink);border:0;border-right:1px solid var(--ink);min-height:32px;font:700 10px "Segoe UI",sans-serif;letter-spacing:.12em;text-transform:uppercase;cursor:pointer}
.seg button:last-child{border-right:0}.seg button:hover{background:var(--ink);color:var(--on)}.seg button.on{background:var(--red);color:var(--onred)}
.sw{display:flex;justify-content:space-between;align-items:center;gap:12px;padding:10px 0;border-top:1px solid var(--line)}
.sw b{font-weight:600}.sw p{color:var(--mute);font-size:12px}
.tg{position:relative;width:38px;height:20px;border:1px solid var(--ink);background:var(--bg);cursor:pointer;flex:none}
.tg::after{content:"";position:absolute;top:3px;left:3px;width:12px;height:12px;background:var(--ink);transition:left .15s,background .15s}
.tg[aria-checked=true]{background:var(--red);border-color:var(--red)}.tg[aria-checked=true]::after{left:21px;background:var(--onred)}
.list{margin:2px 0 8px}.list div{display:flex;justify-content:space-between;gap:10px;padding:5px 0;font-size:12px;border-top:1px solid var(--line)}
.list div span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.list div i{font-style:normal;font-weight:700;font-variant-numeric:tabular-nums}
.note{font-size:12px;color:var(--mute);margin:6px 0 8px}
.acts{display:flex;gap:8px;flex-wrap:wrap;margin-top:12px}
.btn{border:0;background:var(--ink);color:var(--on);min-height:32px;padding:0 14px;font:700 10px "Segoe UI",sans-serif;letter-spacing:.14em;text-transform:uppercase;cursor:pointer}
.btn:hover{background:var(--red);color:var(--onred)}.btn.g{background:var(--soft);color:var(--ink)}.btn.g:hover{background:var(--ink);color:var(--on)}
.btn.sm{min-height:24px;padding:0 8px;font-size:9px}
.ld{color:var(--mute);font-size:12px}
:focus-visible{outline:2px solid var(--red);outline-offset:2px}
@media(prefers-reduced-motion:reduce){*{transition:none!important}}
</style><body><div id="p" role="dialog" aria-labelledby="pt"><header><span id="pi"></span><b id="pt"></b><button class="x" id="px" aria-label="Close" title="Close (Esc)">&#215;</button></header><div id="pb"></div></div>
<script>
const send=o=>window.ipc.postMessage(JSON.stringify(o));
const $=i=>document.getElementById(i);
function el(t,c,x){const e=document.createElement(t);if(c)e.className=c;if(x!=null)e.textContent=x;return e}
const I={saver:'<path d="m12 14 4-4"/><path d="M3.34 19a10 10 0 1 1 17.32 0"/>',dns:'<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/>',kick:'<circle cx="12" cy="12" r="9"/><path d="m5.7 5.7 12.6 12.6"/>',lock:'<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',open:'<path d="M2 12h20"/><circle cx="12" cy="12" r="10"/>',info:'<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>'};
const svg=n=>'<svg viewBox="0 0 24 24" aria-hidden="true">'+I[n]+'</svg>';
let D={};
const close=()=>send({cmd:'panel_close'});
const go=u=>{send({cmd:'open_owl',url:u});close()};
const fit=()=>requestAnimationFrame(()=>send({cmd:'panel_h',h:Math.ceil($('p').getBoundingClientRect().height)}));
function row(k,v,bad){const r=el('div','row');r.appendChild(el('span','',k));r.appendChild(el('b',bad?'bad':'',v));return r}
function tg(on,label,fn){const b=el('button','tg');b.setAttribute('role','switch');b.setAttribute('aria-checked',on);b.setAttribute('aria-label',label);b.onclick=fn;return b}
function sw(title,desc,on,fn){const w=el('div','sw'),t=el('div');t.appendChild(el('b','',title));if(desc)t.appendChild(el('p','',desc));w.append(t,tg(on,title,fn));return w}
function acts(list){const a=el('div','acts');list.forEach(([t,fn,g])=>{const b=el('button','btn'+(g?' g':''),t);b.onclick=fn;a.appendChild(b)});return a}
const P={saver(b){const v=+D.compress||0,k=v===0?0:v<38?1:v<68?2:3,names=['Off','Light','Balanced','Ultra'];
  b.appendChild(el('div','big',v?names[k]+' · '+v:'Off'));b.appendChild(el('div','cap','Data saver'));
  const s=el('div','seg');[[0,'Off'],[25,'Light'],[50,'Balanced'],[85,'Ultra']].forEach(([n,t],i)=>{const x=el('button',i===k?'on':'',t);x.setAttribute('aria-pressed',i===k);x.onclick=()=>send({cmd:'set_num',key:'compress',val:n,panel:'saver'});s.appendChild(x)});b.appendChild(s);
  b.appendChild(el('p','note',v?'Sites are asked for lighter pages, images below the fold load later'+(D.imgs?', and big photos are cut down.':'. Image shrinking pauses while '+(D.proxied?'a proxy':'Secure DNS')+' is on.'):'Pages load exactly as the site sends them.'));
  b.appendChild(el('p','note','On a photo-heavy test page Ultra loaded 3 times faster with 64% less data. New pages and reloads use the new level.'));
  b.appendChild(acts([['Reload page',()=>{send({cmd:'key',k:'reload'});close()}],['Settings',()=>go('owl://settings#speed'),1]]))},
 dns(b){const on=!!D.doh,run=!!D.doh_on;
  b.appendChild(el('div','big',D.proxied?'Proxy in use':on?(run?'On':'On after restart'):(run?'Off after restart':'Off')));b.appendChild(el('div','cap','Secure DNS · Cloudflare 1.1.1.1'));
  b.appendChild(sw('Secure DNS','Look up site names over HTTPS',on,()=>send({cmd:'quick_dns',panel:'dns'})));
  b.appendChild(row('Resolver','1.1.1.1 (DNS over HTTPS)'));b.appendChild(row('Status',D.proxied?'not used with a proxy':run?'running':'not running',!run&&on));
  b.appendChild(el('p','note','Your network no longer sees the names you look up. It is not a VPN: sites still see your IP address.'));
  const a=[];if(on!==run&&!D.proxied)a.push(['Restart now',()=>send({cmd:'restart'})]);a.push(['Test it',()=>go('https://1.1.1.1/help'),a.length>0]);a.push(['Settings',()=>go('owl://settings#network'),1]);b.appendChild(acts(a))},
 kick(b){const h=D.host||'';
  const n=el('div','nums');[[D.here,'on '+(h||'this page')],[D.session,'this session']].forEach(([v,c])=>{const d=el('div');d.appendChild(el('div','big',(+v||0).toLocaleString()));d.appendChild(el('div','cap',c));n.appendChild(d)});b.appendChild(n);
  b.appendChild(sw('OwlKick','Block ads and trackers everywhere',!!D.block,()=>send({cmd:'set',key:'block',val:!D.block,panel:'kick'})));
  if(h)b.appendChild(sw('On '+h,D.off?'Switched off for this site':'Blocking on this site',D.block&&!D.off,()=>{const l=(D.offlist||[]).filter(x=>x!==h);if(!D.off)l.push(h);send({cmd:'set_kick_off',sites:l.join('\n'),panel:'kick'})}));
  if((D.top||[]).length){b.appendChild(el('div','cap','Blocked on this site'));const l=el('div','list');D.top.slice(0,5).forEach(t=>{const r=el('div');r.appendChild(el('span','',t.h));r.appendChild(el('i','',t.n));l.appendChild(r)});b.appendChild(l)}
  b.appendChild(acts([['Activity',()=>go('owl://owlkick')],['Update lists',()=>{send({cmd:'filters_update'});close()},1]]))},
 site(b){const u=D.url||'';
  if(D.owl){b.appendChild(el('p','note','This is one of Owl’s own pages. It runs on your PC and never touches the network.'));return}
  b.appendChild(row('Connection',D.https?'Secure (HTTPS)':'Not secure (HTTP)',!D.https));
  const c=el('div');c.id='cert';c.appendChild(row('Certificate',D.https?'checking...':'none',!D.https));b.appendChild(c);
  const k=el('div');k.id='cook';k.appendChild(row('Cookies','checking...'));b.appendChild(k);
  const p=el('div');p.id='perm';b.appendChild(p);
  b.appendChild(acts([['Clear site data',()=>{send({cmd:'site_clear',origin:D.origin,panel:'site'});const x=$('cook');x.textContent='';x.appendChild(row('Cookies','cleared'))}],['Permissions',()=>go('owl://permissions'),1]]))}};
window.Pn={show(d){D=d;document.documentElement.dataset.theme=d.theme||'system';const p=$('p');
  $('pi').innerHTML=svg(d.kind==='site'?(d.owl?'info':d.https?'lock':'open'):d.kind);$('pt').textContent=d.title||'';
  const b=$('pb');b.textContent='';(P[d.kind]||(()=>{}))(b);if(!d.keep){p.classList.remove('in');void p.offsetWidth}p.classList.add('in');fit();if(!d.keep)setTimeout(()=>{const f=b.querySelector('button');if(f)f.focus({preventScroll:true});else $('px').focus()},60)},
 part(k,v){if(k==='cert'){const c=$('cert');if(!c)return;c.textContent='';if(!v||!v.subject){c.appendChild(row('Certificate',D.https?'verified by Windows':'none',!D.https))}else{c.appendChild(row('Issued to',v.subject));c.appendChild(row('Issued by',v.issuer));c.appendChild(row('Valid until',v.until))}}
  if(k==='cook'){const c=$('cook');if(!c)return;c.textContent='';c.appendChild(row('Cookies',v.n?v.n+' for this site':'none'))}
  if(k==='perm'){const c=$('perm');if(!c)return;c.textContent='';const K={1:'Microphone',2:'Camera',3:'Location',4:'Notifications',5:'Motion sensors',6:'Clipboard read',7:'Multiple downloads',8:'File access',9:'Autoplay',10:'Local fonts',11:'MIDI',12:'Window management'};
   (v||[]).forEach(x=>{const r=row(K[x.k]||'Permission',x.s===1?'allowed':'blocked');const z=el('button','btn g sm','Reset');z.onclick=()=>{send({cmd:'perm_reset',kind:x.k,origin:x.o,panel:'site'});r.remove();fit()};r.appendChild(z);c.appendChild(r)});
   if(!(v||[]).length)c.appendChild(row('Permissions','none changed'))}
  fit()}};
$('px').onclick=close;
addEventListener('keydown',e=>{if(e.key==='Escape')close()});
addEventListener('blur',()=>setTimeout(()=>{if(!document.hasFocus())close()},120));
new ResizeObserver(fit).observe($('p'));
</script></body></html>"#;
