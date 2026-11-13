use crate::{Config, Store};
use std::collections::HashSet;

const CSS: &str = r#"*{box-sizing:border-box;margin:0;padding:0}
main{animation:pagein .42s cubic-bezier(.2,.8,.2,1) both}@keyframes pagein{from{opacity:0;transform:translateY(10px)}to{opacity:1;transform:none}}@media(prefers-reduced-motion:reduce){main{animation:none}}
/*THEMES*/
html{scroll-behavior:smooth}@media(prefers-reduced-motion:reduce){html{scroll-behavior:auto}}
body{font:16px/1.45 "Helvetica Neue",Helvetica,Arial,sans-serif;color:var(--ink);background:var(--bg);padding:0 40px 40px;min-height:100vh;-webkit-font-smoothing:antialiased;transition:background .35s,color .35s}
a{color:inherit;text-decoration:none}
@keyframes rise{from{opacity:0;transform:translateY(26px)}to{opacity:1;transform:none}}
@keyframes grow{from{transform:scaleX(0)}to{transform:scaleX(1)}}
.grid{display:grid;grid-template-columns:repeat(12,1fr);column-gap:24px}
header{border-top:6px solid var(--ink);padding:10px 0 0;display:flex;justify-content:space-between;font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;margin-bottom:28px}
header i{font-style:normal;color:var(--red)}
h1{font-size:clamp(36px,4.2vw,60px);line-height:.95;letter-spacing:-.035em;font-weight:700;margin-bottom:18px;animation:rise .7s cubic-bezier(.2,.7,.2,1) both}
h1 em{font-style:normal;color:var(--red)}
h2{font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;padding-top:10px;border-top:2px solid var(--ink);margin-bottom:12px;display:flex;justify-content:space-between}
h2 span{color:var(--mute)}
button,.btn{border:0;background:var(--ink);color:var(--on);padding:0 22px;font:700 11px "Helvetica Neue",Helvetica,Arial,sans-serif;letter-spacing:.14em;text-transform:uppercase;cursor:pointer;display:inline-flex;align-items:center;min-height:36px;transition:background .15s,color .15s,transform .12s}
button:hover,.btn:hover{background:var(--red);color:var(--onred)}
button:active,.btn:active{transform:scale(.96)}
button.on{background:var(--red);color:var(--onred)}
button.ghost,.btn.ghost{background:var(--soft);color:var(--ink)}button.ghost:hover,.btn.ghost:hover{background:var(--ink);color:var(--on)}
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
.cf{display:inline-flex;flex-direction:column;gap:5px;font-size:12px;font-weight:700;color:var(--ink)}.cf span{letter-spacing:.02em}
.set input,.af input,.pf{font:500 14px "Helvetica Neue",Helvetica,Arial,sans-serif;padding:10px;border:1px solid var(--ink);background:var(--bg);color:var(--ink);width:380px;max-width:100%;outline:0;transition:box-shadow .15s}
.set input:focus,.af input:focus,.pf:focus{box-shadow:0 0 0 2px var(--red)}
input::placeholder{color:var(--mute);opacity:1}
.set input.pin{width:90px;letter-spacing:.3em;text-align:center}
.set input.pk,.set select.pk{width:210px}
select{-webkit-appearance:none;appearance:none;font:700 11px "Helvetica Neue",Helvetica,Arial,sans-serif;letter-spacing:.12em;text-transform:uppercase;min-height:38px;padding:0 36px 0 14px;border:1px solid var(--ink);border-radius:0;color:var(--ink);cursor:pointer;outline:0;background:var(--bg) no-repeat;background-image:linear-gradient(45deg,transparent 50%,currentColor 50%),linear-gradient(135deg,currentColor 50%,transparent 50%);background-position:calc(100% - 19px) 52%,calc(100% - 14px) 52%;background-size:5px 5px;transition:background-color .15s,color .15s,box-shadow .15s}select:hover{background-color:var(--ink);color:var(--on)}select:focus-visible{box-shadow:0 0 0 2px var(--red)}option{background:var(--bg);color:var(--ink);font:500 14px "Helvetica Neue",Helvetica,Arial,sans-serif;text-transform:none;letter-spacing:0}
/* Data saver: segmented presets over a flat scale with a square thumb */
.sv{display:grid;gap:12px;width:380px;max-width:100%}.sv .seg{display:grid;grid-template-columns:repeat(4,1fr);border:1px solid var(--ink)}.sv .seg button{background:var(--bg);color:var(--ink);justify-content:center;padding:0 6px;min-height:36px;border-right:1px solid var(--ink)}.sv .seg button:last-child{border-right:0}.sv .seg button:hover{background:var(--ink);color:var(--on)}.sv .seg button.on{background:var(--red);color:var(--onred)}.sv .rr{display:grid;grid-template-columns:1fr 70px;gap:16px;align-items:center}.sv output{font:700 30px/1 "Space Grotesk","Helvetica Neue",Helvetica,Arial,sans-serif;letter-spacing:-.03em;text-align:right;font-variant-numeric:tabular-nums}.sv .sc{position:relative;height:26px}.sv .sc::before{content:"";position:absolute;left:0;right:0;top:12px;height:2px;background:var(--ink)}.sv .sc::after{content:"";position:absolute;left:0;top:10px;height:6px;width:var(--v,0%);background:var(--red)}.sv .sc i{position:absolute;top:4px;width:1px;height:18px;background:var(--ink)}.set input.sw{-webkit-appearance:none;appearance:none;position:absolute;inset:0;width:100%;height:26px;margin:0;padding:0;border:0;background:transparent;z-index:1;cursor:pointer;box-shadow:none}input.sw::-webkit-slider-runnable-track{height:26px;background:transparent}input.sw::-webkit-slider-thumb{-webkit-appearance:none;width:12px;height:26px;border:0;border-radius:0;background:var(--ink);transition:background .15s,transform .12s}input.sw:hover::-webkit-slider-thumb,input.sw:active::-webkit-slider-thumb{background:var(--red)}input.sw:active::-webkit-slider-thumb{transform:scaleX(1.4)}input.sw:focus-visible::-webkit-slider-thumb{background:var(--red);box-shadow:0 0 0 3px var(--bg),0 0 0 5px var(--red)}.sv .lbl{display:flex;justify-content:space-between;font-size:11px;font-weight:700;letter-spacing:.12em;text-transform:uppercase;color:var(--mute)}
.soon{display:inline-block;vertical-align:2px;margin-left:8px;padding:2px 6px;font-size:10px;font-weight:700;letter-spacing:.12em;text-transform:uppercase;border:1px solid var(--red);color:var(--red)}
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
@font-face{font-family:"Space Grotesk";src:url(/font/space-grotesk.woff2) format("woff2");font-weight:300 700;font-display:swap}
.clock{font:600 clamp(96px,22vh,280px)/.8 "Space Grotesk","Helvetica Neue",Helvetica,Arial,sans-serif;letter-spacing:-.065em;white-space:nowrap;font-variant-numeric:tabular-nums;display:flex;align-items:flex-start}
.clock .co{font-style:normal;color:var(--red);margin:0 .02em;transform:translateY(-.04em);transition:opacity .35s}
.clock.tick .co{opacity:.25}
.clock small{font:500 .16em/1 "Space Grotesk","Helvetica Neue",Arial,sans-serif;letter-spacing:0;color:var(--red);margin-left:.35em;padding-top:.32em;font-variant-numeric:tabular-nums}
.clk{display:inline-flex;flex-direction:column;align-self:flex-start;gap:10px}
.secs{height:3px;background:var(--line);position:relative}
.secs i{position:absolute;left:0;top:0;bottom:0;background:var(--red);width:0;transition:width .95s linear}
.stat b,.kst b,.stats b{font-family:"Space Grotesk","Helvetica Neue",Arial,sans-serif}
@media(prefers-reduced-motion:reduce){.clock .co,.secs i{transition:none}.clock.tick .co{opacity:1}}
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
 // links out of Owl's own pages open in a new tab instead of replacing the page
 document.addEventListener('click',e=>{const l=e.target.closest('a[href]');if(!l||e.defaultPrevented)return;const h=l.getAttribute('href')||'';if(/^https?:\/\//i.test(h)&&!/^https?:\/\/owl\.localhost/i.test(h)){e.preventDefault();send({cmd:'open_owl',url:h})}},true);
 document.addEventListener('click',e=>{const b=e.target.closest('[data-act]');if(!b)return;const a=b.dataset.act;
  if(a==='cmpset'){const r=document.querySelector('[data-sel=compress]');if(r){r.value=b.dataset.v;r.dispatchEvent(new Event('input',{bubbles:true}));r.dispatchEvent(new Event('change',{bubbles:true}))}return}
  if(a==='warp'){send({cmd:'warp',a:b.dataset.w});return}
  if(a==='setko'){send({cmd:'set_kick_off',sites:document.getElementById('ko').value});return}
  if(a==='kickupd'){send({cmd:'filters_update'});return}
  if(a==='updcheck'){send({cmd:'update_check'});return}
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
 document.addEventListener('input',e=>{const t=e.target;if(t&&t.dataset&&t.dataset.out){const o=document.getElementById(t.dataset.out);if(o)o.textContent=+t.value?t.value:'Off'}if(t&&t.classList&&t.classList.contains('sw'))svSync(t)});
function svSync(t){const w=t.closest('.sv');if(!w)return;const v=+t.value;w.style.setProperty('--v',(v/99*100)+'%');const k=v===0?0:v<38?1:v<68?2:3;w.querySelectorAll('.seg button').forEach((b,i)=>{b.classList.toggle('on',i===k);b.setAttribute('aria-pressed',i===k)})}
{const svInit=()=>document.querySelectorAll('input.sw').forEach(svSync);if(document.readyState==='loading')addEventListener('DOMContentLoaded',svInit);else svInit()}
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
        "default-src 'none'; script-src 'nonce-{n}'; style-src 'unsafe-inline'; img-src http: https: data:; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; object-src 'none'"
    );
    let html = format!(
        "<!doctype html><html data-theme=\"{theme}\"><meta http-equiv=\"Content-Security-Policy\" content=\"{csp}\"><meta charset=utf-8><title>{title}</title>\
<script>document.documentElement.classList.add('js')</script>{SHARED_JS}{head}<style>{css}</style><body class=\"{cls}\">{body}</body></html>"
    );
    // everything dynamic was escaped, so every literal "<script>" here is Owl's own code
    html.replace("<script>", &format!("<script nonce={n}>"))
}
fn hdr(title: &str) -> String {
    format!("<header><span>Owl<i>.</i> <i>/</i> {title}</span><span>The private browser</span></header>")
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

const NEWTAB_JS: &str = r#"<script>const p=n=>String(n).padStart(2,'0');function t(){const d=new Date(),h=d.getHours(),sec=d.getSeconds();hh.textContent=p(h);mm.textContent=p(d.getMinutes());ss.textContent=p(sec);c.classList.toggle('tick',sec%2===1);sb.style.transition=sec===0?'none':'';sb.style.width=(sec/59*100)+'%';
const wk=(()=>{const x=new Date(Date.UTC(d.getFullYear(),d.getMonth(),d.getDate()));const n=x.getUTCDay()||7;x.setUTCDate(x.getUTCDate()+4-n);const y=new Date(Date.UTC(x.getUTCFullYear(),0,1));return Math.ceil(((x-y)/864e5+1)/7)})();
const nm=document.querySelector('.hello').dataset.name;gr.textContent=(h<5?'Good night':h<12?'Good morning':h<18?'Good afternoon':'Good evening')+(nm?', '+nm:'');
wd.textContent=d.toLocaleDateString('en',{weekday:'long'});dt.textContent=d.toLocaleDateString('en',{day:'numeric',month:'long',year:'numeric'})+' \u00b7 Week '+wk}t();setInterval(t,1000);
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
<div class=clk><div class=clock id=c aria-live=off><span id=hh>00</span><i class=co aria-hidden=true>:</i><span id=mm>00</span><small id=ss>00</small></div><div class=secs aria-hidden=true><i id=sb></i></div></div>\
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

const REPO: &str = "https://github.com/fardinahosancse/owl-browser";

/// Inline Markdown: escaping first, then `code`, **bold**, _em_ and [links](url). Relative links
/// (CHANGELOG.md, LICENSE) point at the repository.
fn md_inline(t: &str) -> String {
    let mut out = String::new();
    let mut rest = esc(t);
    // links
    while let Some(a) = rest.find('[') {
        let Some(b) = rest[a..].find("](").map(|i| a + i) else { break };
        let Some(c) = rest[b..].find(')').map(|i| b + i) else { break };
        out.push_str(&rest[..a]);
        let (label, url) = (&rest[a + 1..b], &rest[b + 2..c]);
        let href = if url.starts_with("http") { url.to_string() } else { format!("{REPO}/blob/main/{url}") };
        out.push_str(&format!("<a href=\"{href}\">{label}</a>"));
        rest = rest[c + 1..].to_string();
    }
    out.push_str(&rest);
    let pair = |s: String, m: &str, open: &str, close: &str| {
        let parts: Vec<&str> = s.split(m).collect();
        if parts.len() < 3 {
            return s;
        }
        let mut r = String::new();
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                // an unmatched last marker stays as it was
                let unmatched = i == parts.len() - 1 && (parts.len() - 1) % 2 == 1;
                r.push_str(if unmatched { m } else if i % 2 == 1 { open } else { close });
            }
            r.push_str(p);
        }
        r
    };
    let out = pair(out, "`", "<code>", "</code>");
    let out = pair(out, "**", "<b>", "</b>");
    pair(out, "_", "<em>", "</em>")
}

/// The small Markdown subset PRIVACY.md and TERMS.md use: headings, paragraphs, lists, tables.
fn md_to_html(md: &str) -> String {
    let mut h = String::new();
    let mut para: Vec<String> = vec![];
    let mut list: Vec<String> = vec![];
    let mut table: Vec<Vec<String>> = vec![];
    let flush = |h: &mut String, para: &mut Vec<String>, list: &mut Vec<String>, table: &mut Vec<Vec<String>>| {
        if !para.is_empty() {
            h.push_str(&format!("<p>{}</p>", md_inline(&para.join(" "))));
            para.clear();
        }
        if !list.is_empty() {
            h.push_str("<ul>");
            for i in list.iter() {
                h.push_str(&format!("<li>{}</li>", md_inline(i)));
            }
            h.push_str("</ul>");
            list.clear();
        }
        if !table.is_empty() {
            h.push_str("<table>");
            for (r, row) in table.iter().enumerate() {
                let tag = if r == 0 { "th" } else { "td" };
                h.push_str("<tr>");
                for c in row {
                    h.push_str(&format!("<{tag}>{}</{tag}>", md_inline(c)));
                }
                h.push_str("</tr>");
            }
            h.push_str("</table>");
            table.clear();
        }
    };
    for line in md.replace("\r\n", "\n").lines().skip(1) {
        let l = line.trim_end();
        if l.is_empty() {
            flush(&mut h, &mut para, &mut list, &mut table);
        } else if let Some(t) = l.strip_prefix("## ") {
            flush(&mut h, &mut para, &mut list, &mut table);
            h.push_str(&format!("<h2>{}</h2>", md_inline(t)));
        } else if l.starts_with('|') {
            let cells: Vec<String> = l.trim_matches('|').split('|').map(|c| c.trim().to_string()).collect();
            if !cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':')) {
                table.push(cells);
            }
        } else if let Some(t) = l.strip_prefix("- ") {
            list.push(t.to_string());
        } else if l.starts_with("  ") && !list.is_empty() {
            let last = list.last_mut().unwrap();
            last.push(' ');
            last.push_str(l.trim());
        } else {
            para.push(l.trim().to_string());
        }
    }
    flush(&mut h, &mut para, &mut list, &mut table);
    h
}

const DOC_CSS: &str = r#"<style>
.doc{max-width:76ch}
.doc h2{font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;border-top:2px solid var(--ink);padding-top:12px;margin:36px 0 12px}
.doc p,.doc li{font-size:16px;line-height:1.6;margin-bottom:12px}
.doc ul{padding-left:20px;margin-bottom:12px}
.doc table{border-collapse:collapse;width:100%;margin:8px 0 16px;font-size:14px}
.doc th{text-align:left;font-size:11px;letter-spacing:.12em;text-transform:uppercase;border-bottom:2px solid var(--ink);padding:8px 12px 8px 0}
.doc td{border-bottom:1px solid var(--line);padding:10px 12px 10px 0;vertical-align:top}
.doc a{color:var(--ink);text-decoration-color:var(--red);text-underline-offset:3px}
.docnav{display:flex;gap:0;flex-wrap:wrap;margin:0 0 40px;border:1px solid var(--ink);width:max-content}.docnav .btn{text-decoration:none}.docnav .btn+.btn{border-left:1px solid var(--ink)}.docnav .btn[aria-current]{background:var(--red);color:var(--onred)}
</style>"#;

fn doc_nav(cur: &str) -> String {
    [("owl://about", "About"), ("owl://privacy", "Privacy"), ("owl://terms", "Terms")]
        .iter()
        .map(|(u, l)| format!("<a class=\"btn{}\" href=\"{u}\"{}>{l}</a>", if *l == cur { "" } else { " ghost" }, if *l == cur { " aria-current=page" } else { "" }))
        .collect::<Vec<_>>()
        .join("")
}

pub fn privacy(theme: &str) -> String {
    let body = format!("<div class=docnav>{}</div><div class=doc>{}</div>", doc_nav("Privacy"), md_to_html(include_str!("../PRIVACY.md")));
    list_page("Privacy", &body, "", DOC_CSS, theme)
}

pub fn terms(theme: &str) -> String {
    let body = format!("<div class=docnav>{}</div><div class=doc>{}</div>", doc_nav("Terms"), md_to_html(include_str!("../TERMS.md")));
    list_page("Terms", &body, "", DOC_CSS, theme)
}

pub fn about(version: &str, theme: &str) -> String {
    let css = r#"<style>
.ab{display:grid;grid-template-columns:minmax(0,1.2fr) minmax(0,1fr);gap:56px;align-items:start}
.ab .ver{font-size:clamp(56px,8vw,120px);font-weight:700;letter-spacing:-.05em;line-height:.9;font-family:"Space Grotesk","Helvetica Neue",Helvetica,Arial,sans-serif}
.ab .ver em{color:var(--red);font-style:normal}
.ab p{font-size:16px;line-height:1.6;max-width:60ch;margin-bottom:14px}
.who{border-top:2px solid var(--ink);padding-top:14px}
.who .k{font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;color:var(--mute)}
.who .n{font-size:28px;font-weight:700;letter-spacing:-.02em;margin:6px 0 4px}
.who dl{display:grid;grid-template-columns:120px 1fr;margin-top:18px;border-top:1px solid var(--line)}
.who dt,.who dd{padding:10px 0;border-bottom:1px solid var(--line);font-size:14px}.who dt{color:var(--mute)}
.who a{color:var(--ink);text-decoration-color:var(--red);text-underline-offset:3px}
.cr{margin-top:28px}.cr li{font-size:14px;line-height:1.6;list-style:none;border-bottom:1px solid var(--line);padding:8px 0}
@media(max-width:900px){.ab{grid-template-columns:1fr}}
</style>"#;
    let body = format!(
        "<div class=docnav>{nav}</div><div class=ab><div><div class=ver>Owl<em>.</em> {v}</div>\
<p style=\"margin-top:24px\">A small private browser for Windows, written in Rust around the Edge WebView2 engine. Free and open source under the MIT licence. No account, no telemetry.</p>\
<p><a class=btn href=\"{REPO}/releases/tag/v{v}\">Release notes</a> <a class=\"btn ghost\" href=\"{REPO}\">Source code</a> <a class=\"btn ghost\" href=\"{REPO}/issues\">Report a problem</a></p>\
<ul class=cr><li><b>Page engine</b>: Microsoft Edge WebView2</li><li><b>OwlKick</b>: adblock-rust by Brave (MPL-2.0), EasyList and EasyPrivacy</li><li><b>BitTorrent</b>: librqbit (Apache-2.0)</li><li><b>Type</b>: Space Grotesk (SIL Open Font License)</li><li><b>Icons</b>: Lucide (ISC)</li><li>Not made or endorsed by Microsoft, Brave or Cloudflare.</li></ul></div>\
<div class=who><div class=k>Created by</div><div class=n>fardinahosancse</div><p>Designer and developer of Owl.</p>\
<dl><dt>GitHub</dt><dd><a href=\"https://github.com/fardinahosancse\">github.com/fardinahosancse</a></dd>\
<dt>Project</dt><dd><a href=\"{REPO}\">owl-browser</a></dd><dt>Licence</dt><dd><a href=\"{REPO}/blob/main/LICENSE\">MIT</a>, copyright 2026</dd>\
<dt>Security</dt><dd><a href=\"{REPO}/blob/main/SECURITY.md\">Report a vulnerability privately</a></dd></dl></div></div>",
        nav = doc_nav("About"),
        v = esc(version)
    );
    list_page("About", &body, "", &format!("{DOC_CSS}{css}"), theme)
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
.dtop{display:grid;grid-template-columns:1fr auto;gap:12px;margin-bottom:14px}
.dtop .pf{width:100%}
.tb{display:flex;gap:8px;align-items:center;margin-bottom:14px;flex-wrap:wrap}.tb .sp{flex:1}
.tb select,.tb .pf{width:auto;min-width:0}
.stats{display:grid;grid-template-columns:repeat(4,1fr);border-top:2px solid var(--ink);margin-bottom:8px}
.stats div{padding:12px 0 14px;border-right:1px solid var(--line)}.stats div:last-child{border-right:0}
.stats b{display:block;font-size:28px;font-weight:700;letter-spacing:-.02em;font-variant-numeric:tabular-nums}
.stats span{font-size:11px;letter-spacing:.12em;text-transform:uppercase;color:var(--mute)}
.stats div+div{padding-left:16px}
.vh{position:absolute!important;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}
.dr>div{min-width:0}
.dr{display:grid;grid-template-columns:44px minmax(0,1fr) 230px 240px;gap:18px;padding:16px 0;border-top:1px solid var(--line);align-items:center}
.dr .ic{width:40px;height:40px;display:flex;align-items:center;justify-content:center;background:var(--soft);color:var(--ink)}
.dr.torrent .ic{background:var(--ink);color:var(--on)}
.dr .ic svg{width:20px;height:20px;fill:none;stroke:currentColor;stroke-width:1.8;stroke-linecap:round;stroke-linejoin:round}
.dr .nm{font-size:17px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.dr .pa{font-size:12px;color:var(--mute);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;margin-top:2px}
.dr .tag{display:inline-block;font-size:10px;font-weight:700;letter-spacing:.12em;text-transform:uppercase;padding:2px 6px;margin-right:6px;border:1px solid var(--line);color:var(--mute);vertical-align:2px}
.dr .tag.t{border-color:var(--ink);color:var(--ink)}
.dr .bar{margin-top:8px}.dr .bar i{transition:width .45s ease}
.parts{display:flex;gap:2px;margin-top:8px;height:6px}.parts i{flex:1;background:var(--soft);position:relative;overflow:hidden}
.parts i::after{content:"";position:absolute;left:0;top:0;bottom:0;width:var(--p);background:var(--red);transition:width .45s ease}
.dr.paused .parts i::after,.dr.paused .bar i{background:var(--mute)}
.dr .inf{font-size:12px;font-variant-numeric:tabular-nums;line-height:1.5}.dr .inf b{display:block;font-size:11px;letter-spacing:.12em;text-transform:uppercase;margin-bottom:2px}
.dr .inf .m{color:var(--mute)}
.dr .ac{display:flex;gap:6px;justify-content:flex-end;flex-wrap:wrap}.dr .ac button{min-height:34px;width:34px;padding:0;justify-content:center}.dr .ac button svg{width:16px;height:16px;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}.dr .ac button.ok{background:var(--red);color:var(--onred)}.dr{cursor:pointer;transition:background .15s,box-shadow .15s}.dr:hover{background:var(--soft)}.dr.sel{box-shadow:inset 3px 0 0 var(--red);background:var(--soft)}.selnote{font-size:12px;color:var(--mute);margin:-2px 0 10px}.selnote button{min-height:24px;padding:0 8px;margin-left:8px;font-size:10px}
.dr.done .bar i{background:var(--ink)}.dr.failed .bar i,.dr.cancelled .bar i{background:var(--mute)}
button[disabled]{opacity:.45;cursor:default}
@media(max-width:900px){.dr{grid-template-columns:44px 1fr}.dr .inf,.dr .ac{grid-column:2}.dr .ac{justify-content:flex-start}.stats{grid-template-columns:repeat(2,1fr)}.dtop{grid-template-columns:1fr}}
</style>"#;
    let extra = r#"<div class=dtop><label class=vh for=addu>Add a download</label><input class=pf id=addu placeholder="Paste a link or a magnet link to download it" autocomplete=off spellcheck=false><button id=addb>Add download</button></div>
<div class=stats aria-live=polite><div><b id=s1>0</b><span id=l1>Active</span></div><div><b id=s2>0 B/s</b><span id=l2>Download speed</span></div><div><b id=s3>0</b><span id=l3>Connections</span></div><div><b id=s4>0 B/s</b><span id=l4>Torrent upload</span></div></div><p class=selnote id=selnote hidden></p>
<div class=tb role=group aria-label="Filter downloads"><button class=on data-f=all aria-pressed=true>All</button><button class=ghost data-f=active aria-pressed=false>Active</button><button class=ghost data-f=done aria-pressed=false>Completed</button><button class=ghost data-f=torrent aria-pressed=false>Torrents</button>
<label class=vh for=cat>Type</label><select class=pk id=cat aria-label="File type"><option value="">All types</option><option>Video</option><option>Music</option><option>Documents</option><option>Images</option><option>Archives</option><option>Programs</option><option>Other</option></select>
<label class=vh for=dq>Search downloads</label><input class=pf id=dq placeholder="Search" style="width:180px"><span class=sp></span>
<label class=vh for=lim>Speed limit</label><select class=pk id=lim data-sel=dl_limit aria-label="Speed limit"><option value=0>No speed limit</option><option value=512>512 KB/s</option><option value=1024>1 MB/s</option><option value=2048>2 MB/s</option><option value=5120>5 MB/s</option><option value=10240>10 MB/s</option><option value=20480>20 MB/s</option></select>
<label class=vh for=par>Downloads at once</label><select class=pk id=par data-sel=dl_active aria-label="Downloads at once"><option value=1>1 at once</option><option value=2>2 at once</option><option value=3>3 at once</option><option value=5>5 at once</option><option value=8>8 at once</option></select>
<button class=ghost id=fold>Open folder</button><button class=ghost id=clr>Clear finished</button></div>
<div id=list></div>
<script>
const send=o=>ipc.postMessage(JSON.stringify(o));let F='all',C='',Q='',D=[],sig='',meta={};
const sz=b=>b>=1e9?(b/1e9).toFixed(2)+' GB':b>=1e6?(b/1e6).toFixed(1)+' MB':b>=1e3?Math.round(b/1e3)+' KB':Math.round(b)+' B';
const eta=s=>{if(!isFinite(s)||s<=0)return '';s=Math.round(s);return s>3600?Math.floor(s/3600)+'h '+Math.floor(s%3600/60)+'m':s>60?Math.floor(s/60)+'m '+s%60+'s':s+'s'};
const act=d=>d.status==='downloading'||d.status==='paused'||d.status==='pending';
const EXT={Video:['mp4','mkv','avi','mov','webm','wmv','m4v'],Music:['mp3','wav','flac','m4a','ogg','aac','opus'],Documents:['pdf','doc','docx','xls','xlsx','ppt','pptx','txt','csv','epub','odt','rtf'],Images:['jpg','jpeg','png','gif','webp','bmp','svg','avif','heic'],Archives:['zip','rar','7z','tar','gz','bz2','xz','iso'],Programs:['exe','msi','msix','apk','dmg']};
function cat(d){if(d.kind==='torrent')return 'Torrent';const e=(d.name.split('.').pop()||'').toLowerCase();for(const k in EXT)if(EXT[k].includes(e))return k;return 'Other'}
const P={Torrent:'<path d="m6 15-4-4 6.75-6.77a7.79 7.79 0 0 1 11 11L13 22l-4-4 6.39-6.36a2.14 2.14 0 0 0-3-3L6 15"/><path d="m5 8 4 4"/><path d="m12 15 4 4"/>',Video:'<path d="m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.87a.5.5 0 0 0-.752-.432L16 10.5"/><rect x="2" y="6" width="14" height="12" rx="2"/>',Music:'<path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>',Documents:'<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M10 13H8"/><path d="M16 17H8"/>',Images:'<rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/>',Archives:'<rect width="20" height="5" x="2" y="3" rx="1"/><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8"/><path d="M10 12h4"/>',Programs:'<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/>',Other:'<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/>'};
function el(t,c,x){const e=document.createElement(t);if(c)e.className=c;if(x!=null)e.textContent=x;return e}
const AI={'Start now':'<polygon points="6 3 20 12 6 21 6 3"/>','Pause':'<rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/>','Stop':'<rect x="5" y="5" width="14" height="14"/>','Resume':'<polygon points="6 3 20 12 6 21 6 3"/>','Stop and delete':'<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>','Cancel':'<circle cx="12" cy="12" r="10"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/>','Open':'<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>','Show':'<path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>','Stop seeding':'<path d="m17 11-5-5-5 5"/><path d="M12 6v12"/><path d="M4 4l16 16"/>','Copy link':'<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>','Retry':'<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>','Remove':'<path d="M18 6 6 18"/><path d="m6 6 12 12"/>','Copied':'<path d="M20 6 9 17l-5-5"/>'};
const TIP={'Start now':'Start now, even if the downloads-at-once limit is reached','Pause':'Pause; resumes from where it stopped','Stop':'Stop the torrent','Resume':'Resume','Open':'Open the file','Show':'Show the file in its folder','Copy link':'Copy the address this file came from','Remove':'Remove from the list; the file stays on disk','Cancel':'Stop and delete the partial file','Retry':'Resume from where it stopped; finished parts are kept','Stop and delete':'Stop the torrent and delete what was downloaded','Stop seeding':'Stop uploading this finished torrent to other peers'};
const svg=n=>'<svg viewBox="0 0 24 24" aria-hidden="true">'+(AI[n]||'')+'</svg>';
function btn(label,cmd,d){const b=el('button','ghost');b.innerHTML=svg(label);b.dataset.a=label;b.title=TIP[label]||label;b.setAttribute('aria-label',label+': '+d.name);b.onclick=e=>{e.stopPropagation();if(cmd==='copy'){try{navigator.clipboard.writeText(d.url)}catch(_){}b.innerHTML=svg('Copied');b.classList.add('ok');b.title='Copied';setTimeout(()=>{b.innerHTML=svg(label);b.classList.remove('ok');b.title=TIP[label]},1200);return}send({cmd,id:d.id});setTimeout(poll,150)};return b}
let SEL=0;
function row(d){const L=d.live||{},tor=d.kind==='torrent',k=cat(d);const r=el('div','dr '+d.status+(tor?' torrent':'')+(SEL===d.id?' sel':''));r.tabIndex=0;r.setAttribute('aria-selected',SEL===d.id);r.title='Click to show this download in the totals above';r.onclick=()=>{SEL=SEL===d.id?0:d.id;sig='';stats();draw()};r.onkeydown=e=>{if((e.key==='Enter'||e.key===' ')&&e.target===r){e.preventDefault();r.click()}};
 const ic=el('div','ic');ic.setAttribute('aria-hidden','true');ic.innerHTML='<svg viewBox="0 0 24 24">'+(P[k]||P.Other)+'</svg>';r.appendChild(ic);
 const mid=el('div');const nm=el('div','nm');const tg=el('span','tag'+(tor?' t':''),tor?'Torrent':d.kind==='fast'?'Fast':d.kind==='media'?'Media':k);nm.appendChild(tg);nm.appendChild(document.createTextNode(d.name));nm.title=d.url||d.name;mid.appendChild(nm);
 mid.appendChild(el('div','pa',d.path));
 const pct=d.total>0?Math.min(100,d.received*100/d.total):(d.status==='done'?100:0);
 if(Array.isArray(L.parts)&&L.parts.length>1&&act(d)){const ps=el('div','parts');ps.setAttribute('role','progressbar');ps.setAttribute('aria-valuenow',Math.round(pct));ps.setAttribute('aria-label',L.parts.length+' parts');L.parts.forEach(p=>{const i=el('i');i.style.setProperty('--p',(+p||0)+'%');ps.appendChild(i)});mid.appendChild(ps)}
 else{const b=el('div','bar');b.setAttribute('role','progressbar');b.setAttribute('aria-valuenow',Math.round(pct));const i=el('i');i.style.width=pct+'%';b.appendChild(i);mid.appendChild(b)}
 r.appendChild(mid);
 const inf=el('div','inf');let label=d.status;if(L.queued)label='queued';if(tor&&L.state&&act(d))label=L.state;if(tor&&L.state==='seeding')label='seeding';inf.appendChild(el('b','',label));
 const sp=d.status==='downloading'?d.speed:0,e=sp>0&&d.total>0?eta((d.total-d.received)/sp):'';
 inf.appendChild(el('div','',sz(d.received)+(d.total>0?' of '+sz(d.total):'')+(sp?' · '+sz(sp)+'/s':'')+(e?' · '+e+' left':'')));
 if(tor&&d.live)inf.appendChild(el('div','m',L.peers+' peers · '+L.seeders+' seeders · ↑ '+sz(L.up||0)+'/s · uploaded '+sz(L.uploaded||0)));
 else if(L.conns>0)inf.appendChild(el('div','m',L.conns+' connections · resumable'));
 else if(d.kind==='fast'&&act(d))inf.appendChild(el('div','m','resumable'));
 r.appendChild(inf);
 const ac=el('div','ac');
 if(L.queued)ac.appendChild(btn('Start now','dl_force',d));
 if(d.status==='downloading'&&!L.queued)ac.appendChild(btn(tor?'Stop':'Pause','dl_pause',d));
 if(d.status==='paused')ac.appendChild(btn('Resume','dl_resume',d));
 if(tor&&(act(d)||L.state==='seeding'))ac.appendChild(btn('Stop and delete','dl_delete',d));
 else if(act(d))ac.appendChild(btn('Cancel','dl_cancel',d));
 if(d.status==='done'){ac.appendChild(btn('Open','dl_open',d));ac.appendChild(btn('Show','dl_reveal',d))}
 if(tor&&L.state==='seeding')ac.appendChild(btn('Stop seeding','dl_seedstop',d));
 if(d.url&&!tor)ac.appendChild(btn('Copy link','copy',d));
 if(d.status==='failed'||d.status==='cancelled'||d.status==='interrupted')ac.appendChild(btn('Retry','dl_retry',d));
 if(!act(d))ac.appendChild(btn('Remove','dl_remove',d));
 r.appendChild(ac);return r}
const R=new Map();
function draw(){const L=document.getElementById('list');
 const rows=D.filter(d=>(F==='all'||(F==='active'?act(d):F==='done'?d.status==='done':d.kind==='torrent'))&&(!C||cat(d)===C)&&(!Q||(d.name+' '+d.url).toLowerCase().includes(Q)));
 if(!rows.length){L.textContent='';R.clear();L.appendChild(el('div','empty',F==='active'?'Nothing is downloading right now.':F==='torrent'?'No torrents yet. Open a magnet link or a .torrent file, or paste one above.':D.length?'Nothing matches.':'Files you download appear here. Paste a link above to download it with up to 8 connections.'));return}
 const em=L.querySelector('.empty');if(em)em.remove();
 // update rows in place: only a row whose data changed is rebuilt, so the list does not flicker
 const keep=new Set(),fa=document.activeElement,ft=fa&&fa.tagName==='BUTTON'?fa.dataset.a:null;
 rows.forEach(d=>{const s=JSON.stringify(d);let x=R.get(d.id);if(!x||x.s!==s){const n=row(d);const had=x&&x.el.contains(fa);if(x)x.el.replaceWith(n);x={el:n,s};R.set(d.id,x);if(had&&ft){const b=[...n.querySelectorAll('button')].find(b=>b.dataset.a===ft);if(b)b.focus()}}keep.add(d.id);L.appendChild(x.el)});
 for(const [id,x] of R)if(!keep.has(id)){x.el.remove();R.delete(id)}}
const ST=['downloading','paused','pending','done','failed','cancelled','interrupted'],KD=['','fast','torrent','media','image','add'];
function setS(i,v,l){document.getElementById('s'+i).textContent=v;document.getElementById('l'+i).textContent=l}
// the totals bar: every download, or only the selected one
function stats(){const x=SEL&&D.find(d=>d.id===SEL),n=document.getElementById('selnote');
 if(!x){SEL=0;n.hidden=true;const a=D.filter(d=>d.status==='downloading');
  setS(1,a.length,'Active');setS(2,sz(a.reduce((s,d)=>s+d.speed,0))+'/s','Download speed');
  setS(3,D.reduce((s,d)=>s+((d.live&&+d.live.conns)||0)+((d.live&&+d.live.peers)||0),0),'Connections');
  setS(4,sz(D.reduce((s,d)=>s+((d.live&&+d.live.up)||0),0))+'/s','Torrent upload');return}
 const L=x.live||{},tor=x.kind==='torrent',pct=x.total>0?Math.min(100,x.received*100/x.total):(x.status==='done'?100:0),sp=x.status==='downloading'?x.speed:0;
 setS(1,Math.floor(pct)+'%',x.total>0?sz(x.received)+' of '+sz(x.total):x.status);
 setS(2,sz(sp)+'/s',sp>0&&x.total>0?eta((x.total-x.received)/sp)+' left':'Download speed');
 if(tor){setS(3,(+L.peers||0)+' / '+(+L.seeders||0),'Peers / seeders');setS(4,sz(+L.up||0)+'/s','Upload · '+sz(+L.uploaded||0)+' sent')}
 else{setS(3,+L.conns||0,(L.parts&&L.parts.length>1)?L.parts.length+' parts':'Connections');setS(4,x.kind==='fast'?'Fast':x.kind==='media'?'Media':'Browser','Engine')}
 n.hidden=false;n.textContent='Showing '+x.name+'.';const c=el('button','ghost','Show all');c.onclick=()=>{SEL=0;sig='';stats();draw()};n.appendChild(c)}
document.addEventListener('keydown',e=>{if(e.key==='Escape'&&SEL){SEL=0;sig='';stats();draw()}});
async function poll(){try{const j=await (await fetch('/downloads.json',{cache:'no-store'})).json();meta=j||{};
 D=(j.items||[]).map(d=>({id:+d.id||0,name:String(d.name||''),path:String(d.path||''),url:String(d.url||''),status:ST.includes(d.status)?d.status:'failed',kind:KD.includes(d.kind)?d.kind:'',received:+d.received||0,total:+d.total||0,speed:+d.speed||0,live:d.live&&typeof d.live==='object'?d.live:null}));
 stats();
 const lim=document.getElementById('lim'),par=document.getElementById('par');if(document.activeElement!==lim)lim.value=String(+meta.limit||0);if(document.activeElement!==par)par.value=String(+meta.active||3);
 document.getElementById('clr').disabled=!D.some(d=>!act(d));
 const k=JSON.stringify(D)+F+C+Q+SEL;if(k!==sig){sig=k;draw()}}catch(e){}}
document.querySelectorAll('.tb button[data-f]').forEach(b=>b.onclick=()=>{F=b.dataset.f;document.querySelectorAll('.tb button[data-f]').forEach(x=>{x.className=x===b?'on':'ghost';x.setAttribute('aria-pressed',x===b)});sig='';poll()});
document.getElementById('cat').onchange=e=>{C=e.target.value;sig='';draw()};
document.getElementById('dq').oninput=e=>{Q=e.target.value.trim().toLowerCase();sig='';draw()};
function add(){const i=document.getElementById('addu'),u=i.value.trim();if(!u)return;send({cmd:'dl_add',url:u});i.value='';setTimeout(poll,300)}
document.getElementById('addb').onclick=add;document.getElementById('addu').onkeydown=e=>{if(e.key==='Enter')add()};
document.getElementById('fold').onclick=()=>send({cmd:'dl_folder'});
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
    ("sand", "Sand", "#efe9dc", "#171717", "#b8290f"),
    ("random", "Random", "conic-gradient(#e30613,#0b1a3a,#efe9dc,#000,#e30613)", "#fff", "#111"),
];

pub fn settings(c: &Config, exts: &[String], dir: &str, prof: (&str, &str), theme: &str) -> String {
    // a switch: the state is exposed to assistive technology and shown as On / Off
    let tog = |key: &str, on: bool, title: &str, desc: &str| {
        format!(
            "<div class=set><div><b id=l-{key}>{title}</b><p>{desc}</p></div><button class=\"tg ghost{}\" role=switch aria-checked={on} aria-labelledby=l-{key} data-act=tog data-key=\"{key}\">{}</button></div>",
            if on { " on" } else { "" }, if on { "On" } else { "Off" }
        )
    };
    let sel = |key: &str, label: &str, cur: u32, opts: &[(u32, &str)]| {
        let o: String = opts.iter().map(|(v, l)| format!("<option value={v}{}>{l}</option>", if cur == *v { " selected" } else { "" })).collect();
        format!("<select class=pk data-sel={key} aria-label=\"{label}\">{o}</select>")
    };
    let mut cards = String::new();
    for (i, (id, label, bg, ink, acc)) in THEME_CARDS.iter().enumerate() {
        let on = c.theme == *id;
        cards += &format!(
            "<button class=\"th{}\" data-t=\"{id}\" aria-pressed={on} aria-label=\"{label} theme\" style=\"--i:{i};--b:{bg};--i2:{ink};--a:{acc}\"><span class=pv style=\"--i:{ink}\"><u></u></span><span class=l>{label}</span></button>",
            if on { " on" } else { "" }
        );
    }
    let ex = if exts.is_empty() { "none".to_string() } else { exts.join(", ") };
    let js = r#"<script>
document.querySelectorAll('.th').forEach(b=>b.onclick=()=>{const t=b.dataset.t,live=!['system','random'].includes(t);
 document.querySelectorAll('.th').forEach(x=>{x.classList.toggle('on',x===b);x.setAttribute('aria-pressed',x===b)});
 if(live)document.documentElement.dataset.theme=t;
 ipc.postMessage(JSON.stringify({cmd:'set',key:'theme',val:t}));
 if(!live)setTimeout(()=>location.reload(),300)});
(function(){const links=[...document.querySelectorAll('.snav a')];const secs=links.map(a=>document.getElementById(a.getAttribute('href').slice(1)));
 const mark=id=>links.forEach(a=>{const on=a.getAttribute('href')==='#'+id;a.classList.toggle('on',on);if(on)a.setAttribute('aria-current','true');else a.removeAttribute('aria-current')});
 links.forEach(a=>a.onclick=e=>{e.preventDefault();const t=document.getElementById(a.getAttribute('href').slice(1));if(t){t.scrollIntoView({block:'start'});mark(t.id);hold=Date.now()+900;history.replaceState(null,'','#'+t.id)}});
 let hold=location.hash?Date.now()+1200:0;const io=new IntersectionObserver(es=>{if(Date.now()<hold)return;const v=es.filter(e=>e.isIntersecting).sort((a,b)=>a.boundingClientRect.top-b.boundingClientRect.top)[0];if(v)mark(v.target.id)},{rootMargin:'-10% 0px -70% 0px'});
 secs.forEach(s=>s&&io.observe(s));if(location.hash)mark(location.hash.slice(1));else if(secs[0])mark(secs[0].id)})();
</script>"#;
    let css = "<style>.sg{display:grid;grid-template-columns:210px minmax(0,1fr);gap:48px;align-items:start}\
.snav{position:sticky;top:20px;display:flex;flex-direction:column;border-top:2px solid var(--ink)}\
.snav a{display:flex;align-items:center;gap:10px;padding:9px 0 9px 12px;border-bottom:1px solid var(--line);font-size:13px;font-weight:500;color:var(--ink);text-decoration:none;border-left:3px solid transparent;transition:border-color .15s,background .15s}\
.snav a:hover{background:var(--soft)}.snav a.on{border-left-color:var(--red);font-weight:700}.snav svg{width:16px;height:16px;flex:none;fill:none;stroke:currentColor;stroke-width:1.8;stroke-linecap:round;stroke-linejoin:round;opacity:.75}.snav a.on svg{opacity:1;stroke:var(--red)}\
.sgp{scroll-margin-top:16px;margin-bottom:40px}.sgp>.gh:first-child{margin-top:0}\
@media(max-width:900px){.sg{grid-template-columns:1fr;gap:16px}.snav{position:sticky;top:0;z-index:2;background:var(--bg);flex-direction:row;overflow-x:auto;border-top:0;border-bottom:2px solid var(--ink)}.snav a{white-space:nowrap;border-left:0;border-bottom:3px solid transparent;padding:10px 12px}.snav a.on{border-bottom-color:var(--red)}}</style>";

    let groups: Vec<(&str, &str, String)> = vec![
        ("appearance", "Appearance", format!(
            "<div class=themes role=group aria-label=\"Theme\">{cards}</div>{}{}{}",
            tog("home_calm", c.home_calm, "Calm home page", "A quieter home page built from your own day: time online, what OwlKick stopped, running downloads and the pages you were on. Off shows the classic home page."),
            tog("bar_top", c.bar_top, "Toolbar at the top", "Moves the address bar and tabs to the top of the window instead of the bottom."),
            tog("bar", c.bookmark_bar, "Bookmark bar", "Show bookmarks next to the tabs (Ctrl+B).")
        )),
        ("security", "Security", format!(
            "<div class=set><div><b>Startup key</b><p><b>Use a passphrase of 12 or more characters.</b> It protects your saved passwords, history, bookmarks and cookies. A 4-digit key can be guessed offline.</p>\
<details><summary>How it works</summary><p>The key is stretched with Argon2id and wrong guesses lock you out for longer each time. Data is sealed when Owl locks or closes cleanly; if Owl is killed it stays readable until the next lock or start. The default key before setup is 1966.</p></details></div>\
<div class=ctl><label class=cf><span>Current key</span><input class=pk type=password id=op maxlength=128 autocomplete=off aria-label=\"Current key\" placeholder=\"Current key\"></label><label class=cf><span>New key or passphrase</span><input class=pk type=password id=np maxlength=128 autocomplete=off aria-label=\"New key or passphrase\" placeholder=\"New key or passphrase\"></label><button data-act=setpin>Change</button></div></div>\
<div class=set><div><b>Auto-lock</b><p>After this long without use, Owl locks, closes every page and encrypts your data on disk. Playing media and downloads count as use. Unlocking reopens your tabs.</p></div>{}</div>{}{}{}\
<div class=set><div><b>Site permissions</b><p>Review and reset the camera, microphone, location and other permissions you gave sites.</p></div><a class=btn href=\"owl://permissions\">Review</a></div>",
            sel("autolock", "Auto-lock after", c.autolock, &[(0, "Never"), (5, "5 minutes"), (15, "15 minutes"), (30, "30 minutes"), (60, "1 hour")]),
            tog("lock_win", c.lock_on_winlock, "Lock when Windows locks or sleeps", "Locks Owl and encrypts your data when you lock the PC (Win+L), sleep or shut down. Typed text and scroll position are restored on unlock."),
            tog("pw_save", c.pw_save, "Offer to save passwords", "Ask to save a password after you sign in to a site."),
            tog("perm_block", c.perm_block, "Block camera, microphone, location and notifications", "Refuses these requests without asking. Sensors, local fonts and MIDI system access are always refused. Turn this off to be asked per site.")
        )),
        ("privacy", "Privacy", format!(
            "{}{}{}{}{}",
            tog("https_only", c.https_only, "HTTPS-only mode", "Upgrades http:// pages to https:// and warns before using plain http."),
            tog("track_strict", c.track_strict, "Strict tracking prevention", "Blocks third-party trackers and their stored cookies more aggressively. Some embedded logins and videos may stop working. Applies to new pages."),
            tog("fp", c.fp, "Fingerprint protection", "Adds invisible noise to canvas and audio reads, hides the exact GPU model and rounds hardware hints. Applies to new pages."),
            tog("fp_strict", c.fp_strict, "Strict fingerprint protection", "Also reports the same screen size, language and plugin list to every site. Sites may show English or odd layouts."),
            tog("update_check", c.update_check, "Check for new versions", "Asks GitHub once a day whether a newer Owl exists. Nothing is downloaded or installed, and no identifier is sent. Skipped when a proxy is set.")
        )),
        ("owlkick", "OwlKick blocking", format!(
            "{}\
<div class=set><div><b>Activity</b><p>See what OwlKick blocked: counts, the hosts it stopped most and the latest requests.</p></div><a class=btn href=\"owl://owlkick\">Open activity</a></div>\
<div class=set><div><b>Filter lists</b><p>OwlKick is built on adblock-rust, the engine Brave uses (MPL-2.0; not made or endorsed by Brave). It always uses Owl's built-in tracker list. Download EasyList and EasyPrivacy for much wider coverage and clutter hiding; once downloaded they refresh weekly. Lists on disk: {}.</p></div><button class=ghost data-act=kickupd>Update lists</button></div>\
<div class=set><div><b>Off for these sites</b><p>Comma-separated sites where blocking and clutter hiding are switched off, for example a bank or a site that breaks.</p></div><div class=ctl><label class=cf><span>Sites where OwlKick is off</span><input class=pk id=ko maxlength=800 aria-label=\"Sites where OwlKick is off\" placeholder=\"example.com, shop.example.org\" value=\"{}\"></label><button class=ghost data-act=setko>Save</button></div></div>",
            tog("block", c.block, "Block trackers and ads", "OwlKick blocks requests to ads, analytics and trackers, and hides page clutter."),
            crate::owlkick::list_count(),
            esc(&c.kick_off.join(", "))
        )),
        ("network", "Network", format!(
            "{}\
<div class=set><div><b>Cloudflare WARP (installed client)</b><p>WARP is a real VPN from Cloudflare. Owl cannot include it, but if you installed the free client it can switch it on and off here. While it is on, all your traffic goes through Cloudflare.</p></div><div class=ctl><button class=ghost data-act=warp data-w=connect>Connect</button><button class=ghost data-act=warp data-w=disconnect>Disconnect</button><button class=ghost data-act=warp data-w=status>Status</button></div></div>\
<div class=set><div><b>Proxy or VPN</b><p>Send all page traffic through a proxy to hide your IP address, for example <code>socks5://127.0.0.1:9050</code> (Tor). WebRTC is forced through it too. Empty means direct. Restart to apply.</p></div>\
<div class=ctl><label class=cf><span>Proxy address</span><input class=pk id=px maxlength=100 aria-label=\"Proxy address\" placeholder=\"socks5://127.0.0.1:9050\" value=\"{}\"></label><button class=ghost data-act=setproxy>Save</button></div></div>",
            tog("doh", c.doh, "Secure DNS (Cloudflare 1.1.1.1)", "Looks up site names over HTTPS at 1.1.1.1 so your network cannot see which sites you open. This is not a VPN: your IP address is unchanged. If 1.1.1.1 is blocked, pages will not load. Needs a restart. Not used while a proxy is set."),
            esc(&c.proxy)
        )),
        ("speed", "Speed and data", format!(
            "<div class=set><div><b>Data saver</b><p>Asks sites for lighter pages (Save-Data), loads off-screen images later, and cuts very large images down: big progressive JPEGs are only partly downloaded, other huge images are shrunk. Higher means smaller images and lower quality; Off at 0. Small images are never touched. The image part is off while a proxy or secure DNS is on. Applies to new pages.</p>\
</div>\
<div class=ctl><div class=sv style=\"--v:{pct}%\"><div class=seg role=group aria-label=\"Data saver presets\"><button data-act=cmpset data-v=0 title=\"No changes to pages\">Off</button><button data-act=cmpset data-v=25 title=\"Save-Data and lazy images, photos barely touched\">Light</button><button data-act=cmpset data-v=50 title=\"Big photos partly downloaded, a good middle\">Balanced</button><button data-act=cmpset data-v=85 title=\"Smallest pages, softer photos\">Ultra</button></div><div class=rr><div class=sc><i style=\"left:25.25%\"></i><i style=\"left:50.5%\"></i><i style=\"left:85.85%\"></i><input type=range class=\"pk sw\" min=0 max=99 value={v} data-sel=compress data-out=cmpv aria-label=\"Data saver strength\" aria-describedby=cmpv></div><output id=cmpv>{vt}</output></div><div class=lbl><span>Full quality</span><span>Smallest</span></div></div></div></div>",
            v = c.compress,
            pct = c.compress as f64 / 99.0 * 100.0,
            vt = if c.compress == 0 { "Off".to_string() } else { c.compress.to_string() }
        )),
        ("downloads", "Downloads", format!(
            "<div class=set><div><b>Download speed limit</b><p>Caps the total speed of Owl's fast download engine. Useful on shared connections.</p></div>{}</div>\
<div class=set><div><b>Downloads at once</b><p>More fast downloads wait in a queue until one finishes. You can start a queued one at once from the download list.</p></div>{}</div>{}{}\
<div class=set><div><b>Choose files in a torrent <span class=soon>Coming soon</span></b><p>Pick which files of a multi-file torrent to download. Today Owl downloads all of them.</p></div><button class=ghost disabled title=\"Not available yet\">Choose</button></div>\n<div class=set><div><b>Download list</b><p>Add links, watch parts and connections, and manage torrents.</p></div><a class=btn href=\"owl://downloads\">Open downloads</a></div>",
            sel("dl_limit", "Download speed limit", c.dl_limit, &[(0, "Unlimited"), (512, "512 KB/s"), (1024, "1 MB/s"), (2048, "2 MB/s"), (5120, "5 MB/s"), (10240, "10 MB/s"), (20480, "20 MB/s")]),
            sel("dl_active", "Downloads at once", c.dl_active, &[(1, "1"), (2, "2"), (3, "3"), (5, "5"), (8, "8")]),
            tog("organize", c.organize, "Sort downloads into folders", "Documents, Images, Video, Music, Archives and Programs inside your Downloads folder."),
            tog("torrent_seed", c.torrent_seed, "Keep seeding finished torrents", "Off by default: the torrent engine shuts down as soon as a download ends. When on, finished torrents keep uploading to peers Owl is connected to until you stop them. Owl opens no listening port.")
        )),
        ("sync", "Sync", format!(
            "<div class=set><div><b>Sync folder</b><p>Keep bookmarks, history and open tabs in a folder synced by OneDrive, Dropbox or Syncthing. Set the passphrase below first. Restart Owl after changing.</p></div>\
<div class=ctl><label class=cf><span>Sync folder</span><input class=pk id=sy aria-label=\"Sync folder\" value=\"{}\" placeholder=\"C:\\Users\\you\\OneDrive\\Owl\"></label><button data-act=setsync>Save</button></div></div>\
<div class=set><div><b>Sync passphrase</b><p>Encrypts the sync file (ChaCha20-Poly1305). {} Use the same passphrase on each PC. Restart Owl to apply.</p></div>\
<div class=ctl><label class=cf><span>Sync passphrase</span><input class=pk id=sp type=password autocomplete=off aria-label=\"Sync passphrase\" placeholder=\"Passphrase\"></label><button data-act=setsyncpass>Save</button></div></div>",
            esc(&c.sync_dir),
            if c.sync_key.is_empty() { "Not set yet." } else { "A passphrase is set." }
        )),
        ("advanced", "Profile and advanced", format!(
            "<div class=set><div><b>Your profile</b><p>Set at first run. Kept in the encrypted store on this PC and used only for the home-page greeting.</p></div>\
<div class=ctl><label class=cf><span>Name</span><input class=pk id=pn maxlength=60 aria-label=\"Name\" placeholder=Name value=\"{}\"></label><label class=cf><span>Email</span><input class=pk id=pe maxlength=120 aria-label=\"Email\" placeholder=Email value=\"{}\"></label><button class=ghost data-act=setprofile>Save</button></div></div>\
<div class=set><div><b>Extensions (user scripts)</b><p>Put .js files in <code>{}\\extensions</code>. They run on every web page. Loaded: {}</p></div><button class=ghost data-act=opendata>Open Owl folder</button></div>\
<div class=set><div><b>Time tracking</b><p>Minutes per site are kept on this PC for the home page. Clear them any time.</p></div><button class=ghost data-act=clearusage>Clear</button></div>\
<div class=set><div><b>Check for updates now</b><p>Asks GitHub for the latest release. Owl never updates itself.</p></div><button class=ghost data-act=updcheck>Check now</button></div>\
<div class=set><div><b>Restart Owl</b><p>Some settings (proxy, secure DNS, sync) apply after a restart.</p></div><button class=ghost data-act=restart>Restart now</button></div>\
<div class=set><div><b>About and credits</b><p>Icons are from Lucide (ISC licence). The page engine is Microsoft Edge WebView2.</p></div><a class=btn href=\"owl://about\">About Owl</a></div>",
            esc(prof.0), esc(prof.1), esc(dir), esc(&ex)
        )),
    ];
    let side = |id: &str| -> (&str, &str) {
        match id {
        "appearance" => (r#"<circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/>"#, "Theme, home page style, toolbar position, bookmark bar"),
        "security" => (r#"<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>"#, "Startup key, auto-lock, saved passwords"),
        "privacy" => (r#"<path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/><circle cx="12" cy="12" r="3"/>"#, "Tracking protection, fingerprinting, permissions"),
        "owlkick" => (r#"<circle cx="12" cy="12" r="9"/><path d="m5.7 5.7 12.6 12.6"/>"#, "The content blocker and its filter lists"),
        "network" => (r#"<circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/>"#, "Secure DNS, Cloudflare WARP, proxy"),
        "speed" => (r#"<path d="m12 14 4-4"/><path d="M3.34 19a10 10 0 1 1 17.32 0"/>"#, "Data saver"),
        "downloads" => (r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/>"#, "Speed limit, queue, folders, torrents"),
        "sync" => (r#"<path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/><path d="M16 16h5v5"/>"#, "Keep bookmarks and history in a synced folder"),
        "advanced" => (r#"<path d="M20 7h-9"/><path d="M14 17H5"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>"#, "Your profile, time tracking, restart, about"),
        _ => ("", ""),
        }
    };
    let nav: String = groups
        .iter()
        .map(|(id, t, _)| {
            let (i, tip) = side(id);
            format!("<a href=\"#{id}\" title=\"{tip}\"><svg viewBox=\"0 0 24 24\" aria-hidden=true>{i}</svg><span>{t}</span></a>")
        })
        .collect();
    let body: String = groups.iter().map(|(id, t, h)| format!("<section class=sgp id={id} aria-labelledby=h-{id}><h2 class=gh id=h-{id}>{t}</h2>{h}</section>")).collect();
    list_page("Settings", &format!("<div class=sg><nav class=snav aria-label=\"Settings sections\">{nav}</nav><div>{body}</div></div>{js}"), "", css, theme)
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

pub fn permissions(theme: &str) -> String {
    let js = r#"<script>
(function(){const send=o=>ipc.postMessage(JSON.stringify(o));
const K={1:'Microphone',2:'Camera',3:'Location',4:'Notifications',5:'Motion sensors',6:'Clipboard read',7:'Multiple downloads',8:'File access',9:'Autoplay',10:'Local fonts',11:'MIDI',12:'Window management'};
const S={1:'Allowed',2:'Blocked'};
function draw(list){const L=document.getElementById('pl');L.textContent='';
 if(!list.length){const e=document.createElement('div');e.className='empty';e.textContent='No site has been allowed or blocked yet. When a site asks for the camera, location and so on, your choice appears here.';L.appendChild(e);return}
 list.forEach(p=>{const r=document.createElement('div');r.className='row';
  const n=document.createElement('span');n.className='n';n.textContent=S[p.s]||'';
  const t=document.createElement('span');t.className='t';t.textContent=(K[p.k]||'Permission')+' \u00b7 '+p.o;
  const d=document.createElement('span');d.className='d';
  const b=document.createElement('button');b.className='ghost sm';b.textContent='Reset';b.setAttribute('aria-label','Reset '+(K[p.k]||'permission')+' for '+p.o);
  b.onclick=()=>{send({cmd:'perm_reset',kind:p.k,origin:p.o});r.remove()};
  d.appendChild(b);r.append(n,t,d);L.appendChild(r)})}
window.__perms=function(j){try{draw(JSON.parse(j))}catch(e){draw([])}};
send({cmd:'perm_list'});
})();
</script>"#;
    list_page("Permissions", "<div id=pl><div class=empty>Loading...</div></div>", "<p class=lead>Sites you have allowed or blocked. Reset puts a site back to asking.</p>", js, theme)
}

pub fn owlkick(theme: &str) -> String {
    let css = r#"<style>
.kst{display:grid;grid-template-columns:repeat(4,1fr);border-top:2px solid var(--ink);margin-bottom:36px}
.kst div{padding:14px 16px 16px 0;border-right:1px solid var(--line)}.kst div+div{padding-left:16px}.kst div:last-child{border-right:0}
.kst b{display:block;font-size:clamp(28px,4vw,48px);font-weight:700;letter-spacing:-.03em;font-variant-numeric:tabular-nums;line-height:1}
.kst span{display:block;margin-top:6px;font-size:11px;letter-spacing:.12em;text-transform:uppercase;color:var(--mute)}
.kg{display:grid;grid-template-columns:1fr 1fr;gap:48px;margin-bottom:36px}
.kh{font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;padding-bottom:10px;border-bottom:2px solid var(--ink);margin-bottom:4px;display:flex;justify-content:space-between}
.kb{display:grid;grid-template-columns:minmax(0,1fr) 56px;gap:4px 12px;align-items:center;padding:8px 0;border-bottom:1px solid var(--line);font-size:13px}
.kb .h{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.kb .c{text-align:right;font-variant-numeric:tabular-nums;font-weight:700}
.kb .bar{grid-column:1/-1;height:3px}
.kr{display:grid;grid-template-columns:72px minmax(0,1fr) 90px minmax(0,1fr);gap:12px;padding:8px 0;border-bottom:1px solid var(--line);font-size:13px;align-items:center}
.kr span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.kr .t,.kr .k{color:var(--mute);font-variant-numeric:tabular-nums;font-size:12px}
.kr .k{text-transform:uppercase;letter-spacing:.08em;font-size:10px;font-weight:700}
.ktb{display:flex;gap:8px;flex-wrap:wrap;margin-bottom:28px;align-items:center}.ktb .sp{flex:1}.ktb .st{font-size:13px;color:var(--mute)}
@media(max-width:900px){.kst{grid-template-columns:repeat(2,1fr)}.kg{grid-template-columns:1fr}.kr{grid-template-columns:60px minmax(0,1fr)}.kr .k,.kr .p{display:none}}
</style>"#;
    let body = r#"<div class=ktb><span class=st id=kon></span><span class=sp></span><span class=st id=kmsg role=status></span><button class=ghost id=kupd>Update lists</button><a class=btn href="owl://settings#owlkick">OwlKick settings</a></div>
<div class=kst aria-live=polite><div><b id=k1>0</b><span>Blocked this session</span></div><div><b id=k2>0</b><span>Blocked all time</span></div><div><b id=k3>0</b><span>Filter lists</span></div><div><b id=k4>-</b><span>Lists updated</span></div></div>
<div class=kg><div><div class=kh>Most blocked hosts <span>session</span></div><div id=kh></div></div><div><div class=kh>Pages with most blocking <span>session</span></div><div id=kp></div><div class=kh style="margin-top:28px">By kind</div><div id=kk></div></div></div>
<div class=kh>Latest blocked requests <span id=kc></span></div><div id=kl></div>
<script>
(function(){const send=o=>ipc.postMessage(JSON.stringify(o));
function el(t,c,x){const e=document.createElement(t);if(c)e.className=c;if(x!=null)e.textContent=x;return e}
function bars(id,list,empty){const L=document.getElementById(id);L.textContent='';if(!list.length){L.appendChild(el('div','empty',empty));return}
 const max=Math.max(...list.map(x=>+x.n||0),1);list.forEach(x=>{const r=el('div','kb');r.appendChild(el('span','h',String(x.h||'')));r.appendChild(el('span','c',String(+x.n||0)));const b=el('div','bar');const i=el('i');i.style.width=((+x.n||0)*100/max)+'%';b.appendChild(i);r.appendChild(b);L.appendChild(r)})}
const ago=t=>{const s=Math.max(0,Math.floor(Date.now()/1000)-(+t||0));return s<60?s+'s ago':s<3600?Math.floor(s/60)+'m ago':Math.floor(s/3600)+'h ago'};
let sig='';
async function poll(){try{const j=await (await fetch('/owlkick.json',{cache:'no-store'})).json();
 document.getElementById('k1').textContent=(+j.session||0).toLocaleString();document.getElementById('k2').textContent=(+j.total||0).toLocaleString();
 {const b=document.getElementById('kupd'),u=j.upd||{};if(u.state==='running'){b.textContent='Updating...';b.disabled=true}else{b.disabled=false;b.textContent='Update lists';const m=document.getElementById('kmsg');if(m)m.textContent=u.msg||''}}
 document.getElementById('k3').textContent=String(+j.lists||0);document.getElementById('k4').textContent=j.age==null?'never':(+j.age===0?'today':j.age+' d ago');
 document.getElementById('kon').textContent=j.on?'OwlKick is on'+(Array.isArray(j.off)&&j.off.length?' · off on '+j.off.length+' site'+(j.off.length>1?'s':''):''):'OwlKick is off. Turn it on in settings.';
 const k=JSON.stringify([j.recent&&j.recent.slice(0,40),j.hosts,j.pages,j.kinds]);if(k===sig)return;sig=k;
 bars('kh',j.hosts||[],'Nothing blocked yet in this session.');bars('kp',j.pages||[],'No pages yet.');
 bars('kk',Object.entries(j.kinds||{}).map(([h,n])=>({h,n})).sort((a,b)=>b.n-a.n),'No requests yet.');
 const L=document.getElementById('kl');L.textContent='';const rec=(j.recent||[]).slice(0,60);document.getElementById('kc').textContent=rec.length?'newest first':'';
 if(!rec.length){L.appendChild(el('div','empty','Blocked requests show up here as you browse.'));return}
 rec.forEach(x=>{const r=el('div','kr');r.appendChild(el('span','t',ago(x.t)));r.appendChild(el('span','h',String(x.h||'')));r.appendChild(el('span','k',String(x.k||'')));r.appendChild(el('span','p',x.p?'on '+x.p:''));L.appendChild(r)})}catch(e){}}
document.getElementById('kupd').onclick=()=>{send({cmd:'filters_update'});const b=document.getElementById('kupd');b.textContent='Updating...';b.disabled=true;setTimeout(poll,300)};
poll();setInterval(poll,2000)})();
</script>"#;
    list_page("OwlKick", body, "<p class=lead style=\"max-width:62ch;margin-bottom:24px\">What Owl's content blocker stopped while you browsed. Nothing here leaves this PC, and it is cleared when Owl closes.</p>", css, theme)
}

/// The calm home page: one screen, built from the user's own day rather than fixed sections.
pub fn home_calm(s: &Store, c: &Config, sess: u32, total: u64, today: u64, theme: &str) -> String {
    // recent distinct pages, newest first
    let mut seen = HashSet::new();
    let mut recent = String::new();
    for e in s.history.iter().rev() {
        let h = host(&e.url);
        if h.is_empty() || !seen.insert(h.clone()) {
            continue;
        }
        let t = if e.title.trim().is_empty() { h.clone() } else { e.title.chars().take(90).collect() };
        recent += &format!(
            "<a class=rc href=\"{}\" style=\"--d:{}\"><span class=rn>{:02}</span><span class=rt>{}</span><span class=rh>{}</span><time data-t=\"{}\"></time></a>",
            esc(&safe_href(&e.url)), seen.len(), seen.len(), esc(&t), esc(&h), e.t
        );
        if seen.len() >= 6 {
            break;
        }
    }
    if recent.is_empty() {
        recent = "<p class=quiet>Pages you open will gather here, newest first.</p>".into();
    }
    // where the time went (all days), top three
    let mut use_: Vec<(&String, &u64)> = s.usage.iter().collect();
    use_.sort_by(|a, b| b.1.cmp(a.1));
    let max = use_.first().map(|x| *x.1).unwrap_or(1).max(1);
    let spent: String = use_
        .iter()
        .take(3)
        .map(|(h, v)| format!("<div class=sp><span>{}</span><b>{}</b><i style=\"--w:{}%\"></i></div>", esc(h), dur_txt(**v), (**v * 100 / max).max(4)))
        .collect();
    // short, honest hints that change with the user's settings
    let mut hints: Vec<String> = vec![];
    if crate::owlkick::list_count() == 0 && c.block {
        hints.push("<a href=\"owl://settings#owlkick\">Turn on wider blocking</a> with EasyList and EasyPrivacy.".into());
    }
    if c.autolock == 0 {
        hints.push("Auto-lock is off. <a href=\"owl://settings#security\">Lock after a while</a> to keep your data sealed.".into());
    }
    if c.compress > 0 {
        hints.push(format!("Data saver is on at strength {}.", c.compress));
    }
    if c.doh {
        hints.push("Site names are looked up privately through 1.1.1.1.".into());
    }
    let hint = hints.into_iter().next().unwrap_or_else(|| "Everything you do here stays on this PC.".into());
    let name = esc(s.name.split_whitespace().next().unwrap_or(""));
    let css = r#"<style>
@font-face{font-family:"Space Grotesk";src:url(/font/space-grotesk.woff2) format("woff2");font-weight:300 700;font-display:swap}
body.calm{padding:0;overflow-x:hidden}
.cw{position:relative;min-height:100vh;padding:28px 48px 40px;display:grid;grid-template-rows:auto 1fr auto;isolation:isolate}
.cw::before{content:"";position:absolute;inset:-20vmax;z-index:-1;background:radial-gradient(40vmax 40vmax at var(--sx,70%) var(--sy,30%),color-mix(in srgb,var(--red) 14%,transparent),transparent 70%);animation:drift 38s ease-in-out infinite alternate}
@keyframes drift{to{transform:translate(-6vmax,4vmax) scale(1.08)}}
.ct{display:flex;justify-content:space-between;align-items:baseline;font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase}
.ct span:last-child{color:var(--mute)}
.cm{display:grid;grid-template-columns:minmax(0,1.25fr) minmax(0,1fr);gap:clamp(32px,6vw,96px);align-items:center;padding:4vh 0}
.gr{font-size:clamp(20px,2.4vw,30px);font-weight:500;letter-spacing:-.01em;margin-bottom:14px;opacity:0;animation:up .8s .05s cubic-bezier(.2,.7,.2,1) forwards}
.gr em{font-style:normal;color:var(--red)}
.ck{font:300 clamp(88px,15vw,210px)/.82 "Space Grotesk","Helvetica Neue",Arial,sans-serif;letter-spacing:-.06em;font-variant-numeric:tabular-nums;opacity:0;animation:up .9s .12s cubic-bezier(.2,.7,.2,1) forwards}
.ck b{font-weight:600}
.day{position:relative;height:28px;margin:22px 0 30px;opacity:0;animation:up .9s .2s cubic-bezier(.2,.7,.2,1) forwards}
.day .tr{position:absolute;left:0;right:0;top:12px;height:2px;background:var(--line)}
.day .fl{position:absolute;left:0;top:12px;height:2px;background:var(--ink);width:var(--p,0%);transition:width 1s linear}
.day .nw{position:absolute;top:6px;width:14px;height:14px;margin-left:-7px;border-radius:50%;background:var(--red);left:var(--p,0%);box-shadow:0 0 0 6px color-mix(in srgb,var(--red) 20%,transparent);transition:left 1s linear}
.day .hr{position:absolute;top:22px;font-size:10px;color:var(--mute);font-variant-numeric:tabular-nums;transform:translateX(-50%)}
.cs{display:flex;border-bottom:2px solid var(--ink);opacity:0;animation:up .9s .28s cubic-bezier(.2,.7,.2,1) forwards}
.cs input{flex:1;min-width:0;border:0;outline:0;background:none;color:var(--ink);font:400 clamp(20px,2.2vw,28px) "Helvetica Neue",Helvetica,Arial,sans-serif;padding:12px 0}
.cs input::placeholder{color:var(--mute)}
.cs:focus-within{border-color:var(--red)}
.story{font-size:clamp(16px,1.5vw,19px);line-height:1.55;max-width:46ch;margin-top:26px;opacity:0;animation:up .9s .36s cubic-bezier(.2,.7,.2,1) forwards}
.story b{font-weight:700;font-variant-numeric:tabular-nums}
.hint{margin-top:12px;font-size:13px;color:var(--mute)}.hint a{color:var(--ink)}
.side h2{font-size:11px;font-weight:700;letter-spacing:.14em;text-transform:uppercase;padding-bottom:10px;border-bottom:2px solid var(--ink);margin:0 0 4px}
.side section{margin-bottom:34px}
.rc{display:grid;grid-template-columns:30px minmax(0,1fr) auto;gap:2px 12px;padding:11px 0;border-bottom:1px solid var(--line);color:var(--ink);text-decoration:none;opacity:0;animation:up .7s calc(.3s + var(--d)*.06s) cubic-bezier(.2,.7,.2,1) forwards;transition:padding .2s,background .2s}
.rc:hover{padding-left:8px;background:var(--soft)}
.rn{font-size:11px;color:var(--mute);font-variant-numeric:tabular-nums;padding-top:3px}
.rt{font-size:15px;font-weight:500;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.rh{grid-column:2;font-size:12px;color:var(--mute);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.rc time{grid-row:1;grid-column:3;font-size:11px;color:var(--mute);font-variant-numeric:tabular-nums;white-space:nowrap;padding-top:3px}
.sp{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:6px 12px;padding:10px 0;border-bottom:1px solid var(--line);font-size:14px}
.sp span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.sp b{font-variant-numeric:tabular-nums}
.sp i{grid-column:1/-1;height:3px;background:var(--soft);position:relative}.sp i::after{content:"";position:absolute;left:0;top:0;bottom:0;width:var(--w);background:var(--red)}
.quiet{color:var(--mute);font-size:14px;padding:12px 0}
.cf{display:flex;justify-content:space-between;gap:16px;flex-wrap:wrap;font-size:11px;letter-spacing:.12em;text-transform:uppercase}
.cf a{color:var(--ink);text-decoration:none;border-bottom:1px solid var(--line);padding-bottom:2px}.cf a:hover{border-color:var(--red)}
.cf nav{display:flex;gap:18px;flex-wrap:wrap}
@keyframes up{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:none}}
@media(max-width:900px){.cw{padding:20px}.cm{grid-template-columns:1fr}}
@media(prefers-reduced-motion:reduce){.cw::before{animation:none}.gr,.ck,.day,.cs,.story,.rc{animation:none;opacity:1}}
</style>"#;
    let body = format!(
        "<div class=cw><div class=ct><span>Owl</span><span id=cd></span></div>\
<div class=cm><div>\
<div class=gr><span id=cg>Hello</span>{nm}<em>.</em></div>\
<div class=ck id=ck aria-label=\"Time\"><b id=chh>00</b>:<span id=cmm>00</span></div>\
<div class=day aria-hidden=true><div class=tr></div><div class=fl id=dfl></div><div class=nw id=dnw></div>\
<span class=hr style=\"left:0\">00</span><span class=hr style=\"left:25%\">06</span><span class=hr style=\"left:50%\">12</span><span class=hr style=\"left:75%\">18</span><span class=hr style=\"left:100%\">24</span></div>\
<form class=cs id=cf2><input id=cq autofocus autocomplete=off spellcheck=false aria-label=\"Search or address\" placeholder=\"Search privately or type an address\"></form>\
<p class=story id=st aria-live=polite>You have spent <b>{today_t}</b> online today. OwlKick stopped <b id=sbk>{sess}</b> requests this session and <b>{total}</b> in all.<span id=sdl></span></p>\
<p class=hint>{hint}</p></div>\
<div class=side><section><h2>Pick up where you left off</h2>{recent}</section>{spent_block}</div></div>\
<div class=cf><span id=cpart></span><nav><a href=\"owl://downloads\">Downloads</a><a href=\"owl://history\">History</a><a href=\"owl://bookmarks\">Bookmarks</a><a href=\"owl://owlkick\">OwlKick</a><a href=\"owl://settings\">Settings</a></nav></div></div>",
        nm = if name.is_empty() { String::new() } else { format!(", {name}") },
        today_t = dur_txt(today),
        sess = sess,
        total = total,
        hint = hint,
        recent = recent,
        spent_block = if spent.is_empty() { String::new() } else { format!("<section><h2>Where your time goes</h2>{spent}</section>") }
    );
    let js = r#"<script>
(function(){const p=n=>String(n).padStart(2,'0'),$=i=>document.getElementById(i);
const part=h=>h<5?'Night':h<12?'Morning':h<17?'Afternoon':h<21?'Evening':'Night';
const greet=h=>h<5?'Still up':h<12?'Good morning':h<17?'Good afternoon':h<21?'Good evening':'Good night';
const ago=t=>{const s=Math.max(0,Math.floor(Date.now()/1000)-t);return s<60?'just now':s<3600?Math.floor(s/60)+' min ago':s<86400?Math.floor(s/3600)+' h ago':Math.floor(s/86400)+' d ago'};
function tick(){const d=new Date(),h=d.getHours(),m=d.getMinutes();$('chh').textContent=p(h);$('cmm').textContent=p(m);
 $('cg').textContent=greet(h);$('cd').textContent=d.toLocaleDateString('en',{weekday:'long',day:'numeric',month:'long'});
 const pc=((h*60+m)/1440*100).toFixed(2)+'%';$('dfl').style.setProperty('--p',pc);$('dnw').style.setProperty('--p',pc);
 $('cpart').textContent=part(h)+' · '+Math.round((1440-h*60-m)/60)+' hours left in the day';
 // the light drifts across the page with the time of day
 document.querySelector('.cw').style.setProperty('--sx',(15+70*(h*60+m)/1440)+'%');document.querySelector('.cw').style.setProperty('--sy',(20+30*Math.abs(12-h)/12)+'%');
 document.querySelectorAll('time[data-t]').forEach(x=>x.textContent=ago(+x.dataset.t))}
tick();setInterval(tick,15000);
async function live(){try{const k=await (await fetch('/owlkick.json',{cache:'no-store'})).json();$('sbk').textContent=(+k.session||0).toLocaleString();
 const d=await (await fetch('/downloads.json',{cache:'no-store'})).json();const a=(d.items||[]).filter(x=>x.status==='downloading');
 $('sdl').textContent=a.length?' '+a.length+(a.length===1?' download is':' downloads are')+' running.':''}catch(e){}}
live();setInterval(live,4000);
$('cf2').onsubmit=e=>{e.preventDefault();const v=$('cq').value.trim();if(!v)return;
 location.href=/^[a-z]+:\/\//i.test(v)?v:(!/\s/.test(v)&&/\./.test(v))?'https://'+v:'https://duckduckgo.com/?q='+encodeURIComponent(v)};
document.addEventListener('keydown',e=>{if(e.key==='/'&&document.activeElement!==$('cq')){e.preventDefault();$('cq').focus()}});
})();
</script>"#;
    page("Home", &format!("{body}{js}"), css, theme, "calm")
}

#[cfg(test)]
mod preview {
    /// Writes preview HTML of Owl pages to OWL_PREVIEW_DIR (run with --ignored).
    #[test]
    #[ignore]
    fn dump_pages() {
        let Ok(dir) = std::env::var("OWL_PREVIEW_DIR") else { return };
        let mut s = crate::Store::default();
        s.name = "Alex Example".into();
        for (i, (u, t)) in [("https://news.ycombinator.com/", "Hacker News"), ("https://github.com/fardinahosancse/owl-browser", "owl-browser on GitHub"), ("https://en.wikipedia.org/wiki/Swiss_Style_(design)", "Swiss Style (design) - Wikipedia"), ("https://www.bbc.co.uk/news", "BBC News")].iter().enumerate() {
            s.history.push(crate::Entry { url: u.to_string(), title: t.to_string(), t: 1_794_500_000 + i as u64 * 600, ..Default::default() });
        }
        s.usage.insert("github.com".into(), 5400);
        s.usage.insert("news.ycombinator.com".into(), 2600);
        s.usage.insert("wikipedia.org".into(), 1200);
        let mut c = crate::Config::default();
        c.home_calm = true;
        let html = super::home_calm(&s, &c, 42, 1337, 4380, "light").replace("/font/space-grotesk.woff2", "../../Desktop/Owlbrowser/assets/fonts/SpaceGrotesk-latin.woff2");
        std::fs::write(format!("{dir}/calm-light.html"), html).unwrap();
        let html = super::home_calm(&s, &c, 42, 1337, 4380, "navy");
        std::fs::write(format!("{dir}/calm-navy.html"), html).unwrap();
    }
}
