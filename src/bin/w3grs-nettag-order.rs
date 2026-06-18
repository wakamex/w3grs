//! Ground-truth NetTag b-ordering for one replay: the engine-recorded (a,b) per
//! referenced unit, sorted by b (= engine creation order), with type from
//! SelectSubgroup item_id where available. This is the table w3sim diffs its own
//! creation sequence against. Framing copied from game_data.rs.
use std::collections::HashMap;
use w3grs::buffer::StatefulBufferParser;
use w3grs::{MetadataParser, RawParser};

fn zstr(s:&[u8],o:&mut usize)->bool{let st=*o;let mut i=st;while i<s.len()&&s[i]!=0{i+=1;}if i>=s.len(){return false;}*o=i+1;true}
fn take(s:&[u8],o:&mut usize,n:usize)->bool{if *o+n>s.len(){return false;}*o+=n;true}
fn u32le(s:&[u8],o:usize)->Option<u32>{if o+4>s.len(){return None;}Some(u32::from_le_bytes([s[o],s[o+1],s[o+2],s[o+3]]))}
fn cd(s:&[u8],o:&mut usize)->bool{zstr(s,o)&&zstr(s,o)&&zstr(s,o)}
fn cu(s:&[u8],o:&mut usize)->bool{if !take(s,o,4){return false} let Some(it)=u32le(s,*o) else{return false};*o+=4;for _ in 0..it{if !take(s,o,12){return false}} if !take(s,o,48){return false} let Some(a)=u32le(s,*o) else{return false};*o+=4;for _ in 0..a{if !take(s,o,8){return false}} if !take(s,o,12){return false} let Some(d)=u32le(s,*o) else{return false};*o+=4;let Some(db)=(d as usize).checked_mul(4) else{return false};if !take(s,o,db){return false} take(s,o,6)}
fn norm(id:u8,p:bool)->u8{if p&&id>0x77{id.saturating_add(1)}else{id}}
fn fourcc(v:u32)->String{let b=[(v>>24) as u8,(v>>16) as u8,(v>>8) as u8,v as u8];if b.iter().all(|c|c.is_ascii_graphic()){b.iter().map(|c|*c as char).collect()}else{format!("0x{v:08x}")}}

struct Acc{ first_b: HashMap<u32, u32>, ty: HashMap<u32,u32>, a_of: HashMap<u32,u32> } // keyed by b (unique)
fn note(acc:&mut Acc, a:u32, b:u32, ty:Option<u32>){ if a==u32::MAX&&b==u32::MAX{return;} acc.a_of.entry(b).or_insert(a); acc.first_b.entry(b).or_insert(b); if let Some(t)=ty{acc.ty.entry(b).or_insert(t);} }

fn consume(id:u8,s:&[u8],ps:usize,acc:&mut Acc)->Option<usize>{
    let mut o=ps;
    macro_rules! fx{($n:expr)=>{{if !take(s,&mut o,$n){return None}return Some(o);}}}
    macro_rules! tag{($off:expr)=>{match(u32le(s,$off),u32le(s,$off+4)){(Some(a),Some(b))=>Some((a,b)),_=>None}}}
    match id{
        0x01=>fx!(1),0x02=>return Some(o),0x03=>fx!(1),0x04|0x05=>return Some(o),
        0x06=>{if zstr(s,&mut o)&&zstr(s,&mut o)&&take(s,&mut o,1){return Some(o)}return None}
        0x07=>fx!(4),0x10=>fx!(14),0x11=>fx!(22),
        0x12=>{if let Some((a,b))=tag!(ps+22){note(acc,a,b,None)} fx!(30)}
        0x13=>{if let Some((a,b))=tag!(ps+22){note(acc,a,b,None)} fx!(38)}
        0x14=>fx!(43),0x15=>fx!(51),
        0x16=>{if ps+3>s.len(){return None} let n=u16::from_le_bytes([s[ps+1],s[ps+2]]) as usize; for k in 0..n{if let Some((a,b))=tag!(ps+3+k*8){note(acc,a,b,None)}} if !take(s,&mut o,3+n*8){return None} return Some(o)}
        0x17=>{if ps+3>s.len(){return None} let n=u16::from_le_bytes([s[ps+1],s[ps+2]]) as usize; for k in 0..n{if let Some((a,b))=tag!(ps+3+k*8){note(acc,a,b,None)}} if !take(s,&mut o,3+n*8){return None} return Some(o)}
        0x18=>fx!(2),
        0x19=>{ let ity=u32le(s,ps); if let Some((a,b))=tag!(ps+4){note(acc,a,b,ity)} fx!(12)} // SelectSubgroup: item_id(type)+object
        0x1a=>return Some(o),
        0x1b=>{if let Some((a,b))=tag!(ps+1){note(acc,a,b,None)} fx!(9)}
        0x1c=>fx!(9),0x1d=>fx!(8),0x1e|0x1f=>fx!(5),0x20=>return Some(o),0x21=>fx!(8),0x22..=0x26=>return Some(o),
        0x27|0x28=>fx!(5),0x29..=0x2c=>return Some(o),0x2d=>fx!(5),0x2e=>fx!(4),0x2f=>return Some(o),
        0x50=>fx!(5),0x51=>fx!(9),0x60=>{if take(s,&mut o,8)&&zstr(s,&mut o){return Some(o)}return None}
        0x61=>return Some(o),0x62=>fx!(12),0x63=>fx!(8),0x64|0x65=>fx!(8),0x66|0x67=>return Some(o),0x68=>fx!(12),0x69=>fx!(16),0x6a=>fx!(16),
        0x6b|0x6c=>{if cd(s,&mut o)&&take(s,&mut o,4){return Some(o)}return None}0x6d=>{if cd(s,&mut o)&&take(s,&mut o,1){return Some(o)}return None}0x6e=>{if cd(s,&mut o)&&cu(s,&mut o){return Some(o)}return None}0x70..=0x73=>{if cd(s,&mut o){return Some(o)}return None}
        0x75=>fx!(1),0x76=>fx!(10),0x77=>{if !take(s,&mut o,8){return None}let Some(l)=u32le(s,o) else{return None};o+=4;if take(s,&mut o,l as usize){return Some(o)}return None}0x78=>{if zstr(s,&mut o)&&zstr(s,&mut o)&&take(s,&mut o,4){return Some(o)}return None}0x79=>{if take(s,&mut o,16)&&zstr(s,&mut o){return Some(o)}return None}0x7a=>fx!(20),
        0x7b=>{if let Some((a,b))=tag!(ps){note(acc,a,b,None)} fx!(16)} // CommandCardSource source tag
        0xa0=>fx!(14),0xa1=>fx!(9),_=>None,
    }
}
fn analyze(slice:&[u8],post:bool,acc:&mut Acc){let mut off=0;while off<slice.len(){let id=norm(slice[off],post);off+=1;match consume(id,slice,off,acc){Some(n)=>off=n,None=>return}}}
fn walk(gd:&[u8],post:bool,acc:&mut Acc){let mut p=StatefulBufferParser::new(gd);while !p.is_done(){let Ok(id)=p.read_u8() else{break};match id{
    0x17=>{if p.skip(13).is_err(){break}}0x1a..=0x1c=>{if p.skip(4).is_err(){break}}
    0x1f|0x1e=>{let Ok(bc)=p.read_u16_le() else{break};let bc=bc as usize;let Ok(_)=p.read_u16_le() else{break};if bc<2{break}let end=p.offset()+(bc-2);while p.offset()<end{let Ok(_)=p.read_u8() else{break};let Ok(al)=p.read_u16_le() else{break};let al=al as usize;let st=p.offset();let sp=(st+al).min(p.buffer().len());analyze(&p.buffer()[st..sp],post,acc);p.set_offset(st+al);}}
    0x20=>{let Ok(_)=p.read_u8() else{break};let Ok(_)=p.read_u16_le() else{break};let Ok(f)=p.read_u8() else{break};if f==0x20&&p.skip(4).is_err(){break}if p.read_zero_term_string().is_err(){break}}
    0x22=>{let Ok(l)=p.read_u8() else{break};if p.skip(l as isize).is_err(){break}}0x23=>{if p.skip(10).is_err(){break}}0x2f=>{if p.skip(8).is_err(){break}}_=>{}
}}}
fn main(){
    let path=std::env::args().nth(1).expect("usage: <replay>");
    let limit:usize=std::env::args().nth(2).and_then(|s|s.parse().ok()).unwrap_or(40);
    let b=std::fs::read(&path).unwrap();
    let raw=RawParser::new().parse(&b).unwrap();
    let md=MetadataParser::new().parse(&raw.blocks).unwrap();
    let mut acc=Acc{first_b:HashMap::new(),ty:HashMap::new(),a_of:HashMap::new()};
    walk(&md.game_data, md.is_post_202_replay_format, &mut acc);
    let mut bs:Vec<u32>=acc.first_b.keys().copied().collect(); bs.sort();
    println!("{}  post202={}  distinct referenced units (by b)={}", path, md.is_post_202_replay_format, bs.len());
    println!("{:>5}  {:>10}  {:>10}  {:>8}","rank","b(serial)","a(slot)","type");
    for (i,b) in bs.iter().take(limit).enumerate(){
        let ty=acc.ty.get(b).map(|t|fourcc(*t)).unwrap_or_else(||"?".into());
        println!("{:>5}  {:>10}  {:>#10x}  {:>8}", i+1, b, acc.a_of[b], ty);
    }
    if bs.len()>limit { println!("... (+{} more)", bs.len()-limit); }
}
