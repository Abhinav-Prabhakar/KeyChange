// KeyChange — vanilla JS instrument (piano typer + free-forever tail)
const $=s=>document.querySelector(s),$$=s=>[...document.querySelectorAll(s)];
// loader — KeyChange soundcheck sequence
(()=>{const words=['TUNING…','A = 440Hz ✓','VELOCITY ✓','PEDAL ✓','FORTISSIMO!'];let i=0;const el=$('#loaderWord'),fill=$('#loaderFill');if(!el||!fill){setTimeout(()=>{const l=$('#loader');if(l)l.classList.add('done')},800);return;}const t=setInterval(()=>{el.textContent=words[i];fill.style.width=((i+1)/words.length*100)+'%';i++;if(i>=words.length){clearInterval(t);setTimeout(()=>$('#loader').classList.add('done'),450);}},260);})();
setTimeout(()=>{const l=$('#loader');if(l)l.classList.add('done')},3000);

// cursor
const cur=$('#cursor'),ring=$('#cursorRing');let mx=0,my=0,rx=0,ry=0;
addEventListener('mousemove',e=>{mx=e.clientX;my=e.clientY;if(cur){cur.style.left=mx+'px';cur.style.top=my+'px';}});
(function loop(){rx+=(mx-rx)*.16;ry+=(my-ry)*.16;if(ring){ring.style.left=rx+'px';ring.style.top=ry+'px';}requestAnimationFrame(loop)})();

// dust notes bg
const dust=$('#dust');if(dust){const dctx=dust.getContext('2d');let parts=[];
function dustSize(){dust.width=innerWidth;dust.height=innerHeight} dustSize();addEventListener('resize',dustSize);
const glyphs=['♪','♫','♩','♬','𝄞','·','a','e','o','C','G'];
for(let i=0;i<70;i++)parts.push({x:Math.random()*innerWidth,y:Math.random()*innerHeight,s:Math.random()*18+8,v:Math.random()*.4+.1,o:Math.random()*.35+.05,g:glyphs[i%glyphs.length]});
(function dustLoop(){dctx.clearRect(0,0,dust.width,dust.height);parts.forEach(p=>{p.y-=p.v;p.x+=Math.sin(p.y/90)*.2;if(p.y<-30){p.y=innerHeight+20;p.x=Math.random()*innerWidth}dctx.globalAlpha=p.o;dctx.fillStyle='#17130c';dctx.font='900 '+p.s+'px serif';dctx.fillText(p.g,p.x,p.y)});dctx.globalAlpha=1;requestAnimationFrame(dustLoop)})();}

// ticker
const items=['C MAJOR = COPY ⌘C','G MAJOR = PASTE ⌘V','F MAJOR = SAVE ⌘S','SUSTAIN PEDAL = SPACEBAR','VELOCITY = BOLD','OCTAVE = CAPS LOCK','ARPEGGIO = AUTOCOMPLETE','TEMPO = WPM','QUANTIZE = AUTOCORRECT','BLACK KEYS = SYMBOLS','GLISSANDO = DELETE SENTENCE','88 KEYS • ZERO CLACK • FREE FOREVER •'];
const tt=$('#tickerTrack');if(tt)tt.innerHTML=(items.concat(items).map(t=>`&nbsp;&nbsp;✦&nbsp;&nbsp;${t}`).join(''));

// audio
let AC=null,master=null,soundOn=true;
function ac(){if(!AC){AC=new (window.AudioContext||window.webkitAudioContext)();master=AC.createGain();master.gain.value=.5;master.connect(AC.destination)}if(AC.state==='suspended')AC.resume();return AC}
function tone(midi,vel=90,dur=.5,type='triangle'){if(!soundOn)return;try{ac();const f=440*Math.pow(2,(midi-69)/12);const o=AC.createOscillator(),g=AC.createGain();o.type=type;o.frequency.value=f;const v=vel/127;g.gain.setValueAtTime(0,AC.currentTime);g.gain.linearRampToValueAtTime(.35*v+.05,AC.currentTime+.015);g.gain.exponentialRampToValueAtTime(.0001,AC.currentTime+dur);o.connect(g);g.connect(master);o.start();o.stop(AC.currentTime+dur+.05);}catch(e){}}
function toneF(freq,vel=80,dur=.35,type='triangle'){if(!soundOn)return;try{const c=ac(),o=c.createOscillator(),g=c.createGain();o.type=type;o.frequency.value=freq;const v=vel/127;g.gain.setValueAtTime(.0001,c.currentTime);g.gain.exponentialRampToValueAtTime(.25*v+.02,c.currentTime+.015);g.gain.exponentialRampToValueAtTime(.0001,c.currentTime+dur);o.connect(g).connect(c.destination);o.start();o.stop(c.currentTime+dur+.05);}catch(e){}}

// toast (transplanted)
let toastT;
function toast(msg){const t=$('#toast');if(!t)return;t.textContent=msg;t.classList.add('show');clearTimeout(toastT);toastT=setTimeout(()=>t.classList.remove('show'),1900)}

// ---------- PIANO — re-skinned for KeyChange ----------
const piano=$('#piano');
const layers=['abcdefghijklmno'.split(''),'ABCDEFGHIJKLMNO'.split(''),'1234567890!?.,@'.split('')];
let layer=0,octave=4,velocity=90,sustain=false,startedAt=Date.now(),charTimes=[];
const baseFreq=261.63;
const whiteIdx=[0,2,4,5,7,9,11,12,14,16,17,19,21,23,24];
const blackSyms=['!','?','@','#','.',':',';',',','…','‽'];
const blackSlots=[0,1,3,4,5,7,8,10,11,12];
const NOTE_NAMES=['C','C#','D','D#','E','F','F#','G','G#','A','A#','B'];
const startMidi=60,endMidi=84;
const keyEls={};const activeNotes=new Set();let activeTimer=null;

function setVel(v){velocity=v;const f=$('#velFill');if(f)f.style.width=(v/127*100)+'%';const n2=$('#velNum2');if(n2)n2.textContent=v;const vv=$('#velVal');if(vv)vv.textContent=v>105?'ff':v>85?'mf':v>55?'mp':'pp';}
function midiToLetter(m){for(const k in keyEls){if(+k===m){const el=keyEls[k];const b=el.querySelector('.lbl');return (b?b.textContent:el.textContent).toLowerCase();}}return '?';}

function typeChar(ch,vel){
  const out=$('#typedOutput');if(!out)return;
  const hard=vel>100;
  const s=document.createElement('span');
  if(hard)s.className='nk';else s.className='nk';
  let c=(layer===1||hard)&&/[a-z]/.test(ch)?ch.toUpperCase():ch;
  s.textContent=(sustain||c===' ')?'␣':c;
  if(String(ch).match(/[!?@#.,:;…‽]/))s.classList.add('black-k');
  if(sustain)s.classList.add('sus');
  const w=.9+(vel/127)*.9;s.style.fontSize=(22*w)+'px';
  s.style.fontWeight=vel>105?'900':vel>75?'700':'400';
  s.style.fontStyle=vel<45?'italic':'normal';
  if(vel>105){s.style.color='#ff2e1f';s.style.background='#ffc700';s.style.border='2px solid #17130c';s.style.padding='0 4px'}
  const caret=out.querySelector('.caret-blink');
  out.insertBefore(s,caret);
  while(out.children.length>42)out.removeChild(out.firstChild);
  charTimes.push(Date.now());updateWpm();
  // quantize: sloppy hands, perfect spelling — teh → the
  const txt=[...out.querySelectorAll('.nk')].map(n=>n.textContent==='␣'?' ':n.textContent).join('').toLowerCase();
  if(txt.endsWith('teh')){for(let k=0;k<3;k++){const l=out.querySelectorAll('.nk');l[l.length-1-k].remove();}['t','h','e'].forEach(ch2=>{const ns=document.createElement('span');ns.className='nk';ns.textContent=ch2;ns.style.background='#0e7a4f';ns.style.color='#fff';ns.style.border='2px solid #17130c';ns.style.padding='0 3px';out.insertBefore(ns,caret);});toast('𝄢 QUANTIZED: teh → the ✓');}
  setVel(vel);
}

function flash(k){if(!k)return;k.classList.add('active');setTimeout(()=>k.classList.remove('active'),170);}
function spawnBurstByEl(el,letter){
  if(!el||!piano)return;const r=piano.getBoundingClientRect(),er=el.getBoundingClientRect();
  const b=document.createElement('div');b.className='burst';b.textContent='♪ '+(letter||'').toUpperCase();
  b.style.left=(er.left-r.left+er.width/2-10)+'px';b.style.top='-10px';
  b.style.color=Math.random()>.5?'#ff2e1f':'#2233ff';b.style.background='#ffc700';b.style.border='2px solid #17130c';b.style.padding='0 6px';
  piano.appendChild(b);setTimeout(()=>b.remove(),900);
}
function pressWhite(i,vel,el){
  const semis=whiteIdx[i];const midi=60+semis;
  tone(midi,vel,.45);
  if(el)flash(el);
  typeChar(layers[layer][i]||'?',vel);
  spawnBurstByEl(el,layers[layer][i]);spawnRoll(midi);eqKick();waveKick(vel);
  activeNotes.add(midi);clearTimeout(activeTimer);activeTimer=setTimeout(()=>{if(activeNotes.size>=3)chordFired();activeNotes.clear()},380);
}
function pressBlack(b,vel,el){
  toneF(baseFreq*Math.pow(2,(b*2+1)/12)*2,vel,.3,'square');
  if(el)flash(el);
  typeChar(blackSyms[b%blackSyms.length],vel);
  spawnBurstByEl(el,blackSyms[b%blackSyms.length]);spawnRoll(60+b*2+1);eqKick();waveKick(vel);
}
function pressKey(midi,vel=90){
  ac();
  const el=keyEls[midi];
  if(el){if(el.classList.contains('white')){const i=+el.dataset.i;pressWhite(i,vel,el);}else{pressBlack(+el.dataset.b,vel,el);}return;}
  const semi=(midi-60+120)%24;let best=0,bd=99;whiteIdx.forEach((s,i)=>{const d=Math.abs(s-semi);if(d<bd){bd=d;best=i;}});
  const wel=piano?piano.querySelector(`.white[data-i="${best%15}"]`):null;
  pressWhite(best%15,vel,wel);
}
function releaseKey(m){activeNotes.delete(m);}

function buildPiano(){
  if(!piano)return;
  piano.innerHTML='';for(const k in keyEls)delete keyEls[k];
  for(let i=0;i<15;i++){
    const k=document.createElement('div');
    k.className='key white';k.dataset.i=i;
    k.innerHTML=`<span class="lbl">${layers[layer][i]||''}</span><span class="sub">${'AWSEDFTGYHUJK'.split('')[i]||''}</span>`;
    const down=e=>{e.preventDefault();const r=k.getBoundingClientRect();const y=(e.clientY||(r.top+r.height*.8))-r.top;const vel=Math.round(35+(y/r.height)*92);pressWhite(i,vel,k);};
    k.addEventListener('pointerdown',down);
    piano.appendChild(k);
    keyEls[60+whiteIdx[i]]=k;
  }
  blackSlots.forEach((slot,b)=>{
    const k=document.createElement('div');
    k.className='key black';k.dataset.b=b;
    k.style.left=`calc(${(slot+1)} * (100% / 15) - 2.7%)`;
    k.textContent=blackSyms[b];
    k.addEventListener('pointerdown',e=>{e.preventDefault();e.stopPropagation();pressBlack(b,105,k);});
    piano.appendChild(k);
    keyEls[61+b*2]=k;
  });
}
function setLayer(n){
  layer=(n+3)%3;
  $$('.oct').forEach(b=>b.classList.toggle('active',+b.dataset.oct===layer));
  const ov=$('#octVal');if(ov)ov.textContent='C'+(layer+1);
  buildPiano();toast(['layer: abc','layer: ABC','layer: 123!?'][layer]);
}
$$('.oct').forEach(b=>{b.onclick=()=>setLayer(+b.dataset.oct);});
buildPiano();

// computer keyboard plays piano
const qmap={a:0,w:'b0',s:1,e:'b1',d:2,f:3,t:'b4',g:4,y:'b5',h:5,u:'b6',j:6,k:7};
document.addEventListener('keydown',e=>{
  if(e.repeat)return;
  if(e.code==='Space'){e.preventDefault();setSustain(true);pressWhite(4,100,piano?piano.querySelector('.white[data-i="4"]'):null);return;}
  if(e.code==='Backspace'){const o=$('#typedOutput');if(o&&o.children.length>1)o.removeChild(o.children[o.children.length-2]);tone(40,60,.2,'sawtooth');return;}
  if(e.key==='z'||e.key==='ArrowLeft')return setLayer(layer-1);
  if(e.key==='x'||e.key==='ArrowRight')return setLayer(layer+1);
  const map={KeyA:0,KeyS:1,KeyD:2,KeyF:3,KeyG:4,KeyH:5,KeyJ:6,KeyK:7,KeyL:8,Semicolon:9};
  const m=map[e.code];
  if(m!==undefined){const el=piano?piano.querySelector(`.white[data-i="${m}"]`):null;pressWhite(m,70+Math.floor(Math.random()*40),el);}
});
addEventListener('keyup',e=>{if(e.code==='Space')setSustain(false)});
function setSustain(v){sustain=v;const pv=$('#pedalVis');if(pv)pv.classList.toggle('hold',v);const sb=$('#susBtn');if(sb)sb.classList.toggle('on',v);}
const susBtn=$('#susBtn');if(susBtn)susBtn.onclick=()=>setSustain(!sustain);
const octDown=$('#octDown');if(octDown)octDown.onclick=()=>setLayer(layer-1);
const octUp=$('#octUp');if(octUp)octUp.onclick=()=>setLayer(layer+1);
const clearBtn=$('#clearBtn');if(clearBtn)clearBtn.onclick=()=>{const o=$('#typedOutput');if(o)o.innerHTML='<span class="caret-blink">|</span>';toast('cleared');};
const backBtn=$('#backBtn');if(backBtn)backBtn.onclick=()=>{const o=$('#typedOutput');if(o&&o.children.length>1)o.removeChild(o.children[o.children.length-2]);tone(140,110,.3,'sawtooth');toast('♪ BEND—DELETED');};
const soundBtn=$('#soundBtn');if(soundBtn)soundBtn.onclick=e=>{soundOn=!soundOn;e.target.textContent=soundOn?'♪ ON':'♪ OFF';e.target.classList.toggle('on',soundOn);};
const vsl=$('#velSlider');if(vsl)vsl.oninput=e=>{const v=+e.target.value;setVel(v);const d=$('#velDemo');if(d){d.textContent=v>105?'FORTE!':v>75?'mezzo':'piano…';d.style.fontWeight=v>90?'900':'400';d.style.fontSize=(20+v/127*26)+'px';d.style.background=v>90?'#ff2e1f':'#ffc700';d.style.color=v>90?'#fff':'#17130c';d.style.display='inline-block';d.style.padding='2px 10px';d.style.border='3px solid #17130c';}};

// wpm
function updateWpm(){const now=Date.now();charTimes=charTimes.filter(t=>now-t<60000);const mins=(now-startedAt)/60000||1/60;const wpm=Math.round((charTimes.length/5)/Math.max(mins,.05));const wv=$('#wpmVal');if(wv)wv.textContent=Math.min(wpm,240);const tr=$('#tempoRead');if(tr)tr.textContent=`♪ ${bpm} BPM • ${Math.min(wpm,240)} WPM`;}
setInterval(updateWpm,1000);

// waveform
const wave=$('#wave');let waveKick=v=>{},eqKick=()=>{};
if(wave){const wctx=wave.getContext('2d');let amps=new Array(90).fill(4);
function waveSize(){wave.width=wave.clientWidth||800;wave.height=64;const w2=$('#wave2');if(w2){w2.width=w2.clientWidth||300;w2.height=60;}}setTimeout(waveSize,100);addEventListener('resize',waveSize);
waveKick=v=>{amps[Math.floor(Math.random()*amps.length)]=10+v/127*50};
(function waveLoop(){if(!wave.width)return;wctx.clearRect(0,0,wave.width,wave.height);wctx.fillStyle='#17130c';wctx.fillRect(0,0,wave.width,wave.height);const bw=wave.width/amps.length;amps.forEach((a,i)=>{amps[i]*=.94;const h=Math.max(3,a);wctx.fillStyle=i%3===0?'#ffc700':i%3===1?'#ff2e1f':'#fff8e8';wctx.fillRect(i*bw+1,32-h/2,bw-2,h)});const w2=$('#wave2');if(w2&&w2.width){const c=w2.getContext('2d');c.clearRect(0,0,w2.width,w2.height);c.fillStyle='#17130c';c.fillRect(0,0,w2.width,w2.height);c.strokeStyle='#ffc700';c.lineWidth=3;c.beginPath();amps.forEach((a,i)=>{const x=i/amps.length*w2.width,y=30+Math.sin(i/4+Date.now()/300)*a/2;c[i?'lineTo':'moveTo'](x,y)});c.stroke()}requestAnimationFrame(waveLoop)})();}

// piano roll
const roll=$('#roll');let notes=[];
function spawnRoll(midi){if(!roll)return;const pal=['#ff2e1f','#2233ff','#ffc700','#0e7a4f'];notes.push({m:midi,y:-20,x:((midi-startMidi)/(endMidi-startMidi))*roll.width,letter:midiToLetter(midi),c:pal[Math.abs(midi)%pal.length]});if(notes.length>60)notes.shift();}
if(roll){const rctx=roll.getContext('2d');
function rollSize(){roll.width=roll.clientWidth||800;roll.height=380}setTimeout(rollSize,100);addEventListener('resize',rollSize);
const cr=$('#clearRoll');if(cr)cr.onclick=()=>notes=[];
(function rollLoop(){if(!roll.width){requestAnimationFrame(rollLoop);return;}rctx.clearRect(0,0,roll.width,roll.height);rctx.fillStyle='#fff8e8';rctx.fillRect(0,0,roll.width,roll.height);rctx.strokeStyle='#17130c';rctx.lineWidth=1;for(let i=0;i<8;i++){rctx.globalAlpha=.18;rctx.beginPath();rctx.moveTo(0,i*48);rctx.lineTo(roll.width,i*48);rctx.stroke()}for(let i=0;i<12;i++){rctx.globalAlpha=.18;rctx.beginPath();rctx.moveTo(i*roll.width/12,0);rctx.lineTo(i*roll.width/12,roll.height);rctx.stroke()}rctx.globalAlpha=1;
notes.forEach(n=>{n.y+=2.2;rctx.fillStyle=n.c;const w=54,h=26;rctx.lineWidth=2.5;rctx.strokeStyle='#17130c';rctx.beginPath();if(rctx.roundRect)rctx.roundRect(n.x,n.y,w,h,6);else rctx.rect(n.x,n.y,w,h);rctx.fill();rctx.stroke();rctx.fillStyle='#fff';rctx.font='900 13px monospace';rctx.fillText(String(n.letter).toUpperCase(),n.x+20,n.y+18)});notes=notes.filter(n=>n.y<roll.height+30);requestAnimationFrame(rollLoop)})();}

// eq
const eq=$('#eq');if(eq){for(let i=0;i<24;i++){const b=document.createElement('i');eq.appendChild(b)}
eqKick=()=>{[...eq.children].forEach(b=>{if(Math.random()>.5)b.style.height=(15+Math.random()*85)+'%'})};
setInterval(()=>{[...eq.children].forEach(b=>{const h=parseFloat(b.style.height||'20');b.style.height=Math.max(8,h*.92)+'%'});},300);}

// metronome
let bpm=120,metroOn=false,metroInt=null;
const metroBtn=$('#metroBtn');if(metroBtn)metroBtn.onclick=e=>{metroOn=!metroOn;e.target.textContent=metroOn?'ON':'OFF';const d=$('#metroDot');if(metroOn){let b=false;metroInt=setInterval(()=>{b=!b;if(d)d.classList.toggle('beat',b);if(b){tone(88,50,.08,'square');const bt=$('#bpmTop');if(bt)bt.textContent=`♪ ${bpm} BPM — ${($('#wpmVal')||{textContent:0}).textContent} WPM`}},60000/bpm)}else{clearInterval(metroInt);if(d)d.classList.remove('beat')}};

// chords (kept)
const chordInfo={'C,E,G':['C major','⌘C • COPY — copied'],'G,B,D':['G major','⌘V • PASTE — pasted'],'F,A,C':['F major','⌘S • SAVE — saved'],'A,C,E':['A minor','⌘Z • UNDO — undone'],'D,F#,A':['D major','⏎ ENTER — new line'],'E,G#,B':['E major','␣ SPACE — sustained']};
$$('.chord').forEach(ch=>ch.onclick=()=>{ch.classList.add('playing');[60,64,67].forEach((m,i)=>setTimeout(()=>tone(m+[0,4,7][i],100,.6),i*70));chordFired();setTimeout(()=>ch.classList.remove('playing'),600)});
function chordToMidi(){return [60,64,67]}
function chordFired(){const keys=Object.keys(chordInfo);const pick=keys[Math.floor(Math.random()*keys.length)];const[name,action]=chordInfo[pick];const t=$('#chordToast');if(t){t.textContent=`♫ ${name} → ${action}`;t.style.background='#17130c';t.style.color='#ffc700';}[60,64,67].forEach((m,i)=>setTimeout(()=>tone(m+[0,4,7][i],100,.6),i*70));updateWpm();}

// wheel
const wheel=$('#wheel');if(wheel){const shortcuts=['COPY','PASTE','SAVE','UNDO','BOLD','ITALIC','FIND','ENTER','SPACE','TAB','DELETE','SEND'];
shortcuts.forEach((s,i)=>{const a=(i/12)*Math.PI*2;const el=document.createElement('i');el.textContent=s;el.style.transform=`translate(${Math.cos(a)*122}px,${Math.sin(a)*122}px)`;el.style.background=`hsl(${i*30} 85% 60%)`;el.style.color='#111';wheel.appendChild(el)});}

// demos
const arpDemo=$('#arpDemo');if(arpDemo)arpDemo.onclick=()=>{const word='hello';[...word].forEach((ch,i)=>setTimeout(()=>{tone(60+i*2,90,.4);typeChar(ch,90)},i*160))};
const susDemo=$('#susDemo');if(susDemo){susDemo.onpointerdown=e=>{e.target.textContent='SUSTAINING…';setSustain(true)};susDemo.onpointerup=e=>{e.target.textContent='HOLD TO SUSTAIN';setSustain(false)};}
const ctaHear=$('#ctaHear');if(ctaHear)ctaHear.onclick=()=>{ac();'play your words loud'.split('').forEach((ch,i)=>setTimeout(()=>{if(ch!==' ')pressKey(60+(i*2)%15,95)},i*130))};

// MIDI — hero + plug section share it
async function connectMIDI(){
  try{
    const m=await navigator.requestMIDIAccess();
    const dot=$('#midiDot');if(dot){dot.textContent='● MIDI LIVE';dot.classList.add('live');}
    const st=$('#midiStatus');if(st)st.textContent=m.inputs.size?`🎹 ${m.inputs.size} KEYBOARD(S) LIVE — play anything, it types.`:'No keyboards found — but the on-screen one rips anyway.';
    const cp=$('#cablePulse');if(cp)cp.textContent='♪ ♪ ♪';
    m.inputs.forEach(inp=>inp.onmidimessage=msg=>{const[cmd,note,vel]=msg.data;if(cmd>=144&&cmd<160&&vel>0){const i=note%15;const el=piano?piano.querySelector(`.white[data-i="${i}"]`):null;pressWhite(i,vel,el);}if(cmd===128||(cmd>=144&&vel===0))releaseKey(note);});
    if(!m.inputs.size&&st)st.textContent='No keyboards found — but the on-screen one rips anyway. (Free forever, no account.)';
    toast('🎹 MIDI CONNECTED — FREE FOREVER');
  }catch(e){const st=$('#midiStatus');if(st)st.textContent='Web MIDI needs Chrome/Edge + HTTPS. On-screen piano works everywhere — still free.';const dot=$('#midiDot');if(dot){dot.textContent='○ NO MIDI';}}
}
const ctaConnect=$('#ctaConnect');if(ctaConnect)ctaConnect.onclick=connectMIDI;
const midiBtn=$('#midiBtn');if(midiBtn)midiBtn.onclick=connectMIDI;

// reveal + progress
const io=new IntersectionObserver(es=>es.forEach(e=>{if(e.isIntersecting){e.target.classList.add('in');}}),{threshold:.15});
$$('.reveal').forEach(el=>io.observe(el));
addEventListener('scroll',()=>{const h=document.documentElement;const p=h.scrollTop/Math.max(1,(h.scrollHeight-h.clientHeight));const pp=$('#pitchProgress');if(pp)pp.style.setProperty('--p',(p*100)+'%')},{passive:true});

// encore ticket tilt + free download + confetti
const ticket=$('#ticket');if(ticket)ticket.addEventListener('pointermove',e=>{const r=e.currentTarget.getBoundingClientRect();e.currentTarget.style.transform=`perspective(900px) rotateY(${((e.clientX-r.left)/r.width-.5)*4}deg)`;});
const dlBtn=$('#dlBtn');if(dlBtn)dlBtn.onclick=e=>{e.preventDefault();[60,64,67,72].forEach((mm,i)=>setTimeout(()=>tone(mm,100,.5),i*110));burst2();toast('★ ENCORE! download started — FREE FOREVER ★');};
function burst2(){const c=$('#confetti');if(!c)return;for(let i=0;i<26;i++){const s=document.createElement('i');s.textContent=['♪','♫','A','K','E','Y','S','★'][i%8];s.style.left=Math.random()*100+'%';s.style.color=['#ff2e1f','#2233ff','#17130c'][i%3];s.style.background='#fff8e8';s.style.border='2px solid #17130c';s.style.padding='0 5px';s.style.fontSize=(14+Math.random()*22)+'px';s.style.animationDuration=(1.4+Math.random()*1.6)+'s';c.appendChild(s);setTimeout(()=>s.remove(),3200)}}
// hero interactive headline
(function(){const h=$('#headline');if(!h)return;const scale=[60,62,64,65,67,69,71,72,74,76,77,79,81,84];let i=0;function split(node){[...node.childNodes].forEach(ch=>{if(ch.nodeType===3){const frag=document.createDocumentFragment();[...ch.textContent].forEach(c=>{if(c.trim()===''){frag.appendChild(document.createTextNode(' '))}else{const s=document.createElement('span');s.className='ltr';s.textContent=c;s.addEventListener('mouseenter',()=>{tone(scale[i++%scale.length],90,.4);eqKick()});frag.appendChild(s)}});node.replaceChild(frag,ch)}else if(ch.nodeType===1&&!ch.classList.contains('note-chip'))split(ch)})}split(h)})();
// spotlight follows mouse
addEventListener('mousemove',e=>{const s=document.querySelector('.stage-light');if(s)s.style.transform=`translateX(-50%) translateX(${(e.clientX/innerWidth-.5)*120}px)`});
// manifesto highlight
const mani=$('#maniText');if(mani){const words=mani.textContent.split(' ');mani.innerHTML=words.map(w=>`<span style="opacity:.18">${w}</span>`).join(' ');
addEventListener('scroll',()=>{const r=mani.getBoundingClientRect();const p=Math.min(1,Math.max(0,(innerHeight*.8-r.top)/(r.height+200)));const n=Math.floor(p*words.length);[...mani.children].forEach((s,i)=>{s.style.opacity=i<n?1:.18;s.className=i<n?'hl':''})},{passive:true});}
