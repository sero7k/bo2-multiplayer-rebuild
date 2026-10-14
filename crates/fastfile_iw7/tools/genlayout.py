import re,sys
KW={'type','loop','move','ref','match','fn','mod','use','self','super','crate','impl','trait','struct','enum','where','while','for','in','if','else','let','mut','const','static','return','break','continue','as','box','dyn','extern','false','true','pub','unsafe','async','await','yield','macro','override','priv','virtual','final','abstract','become','do','try','typeof','unsized'}
txt=open(sys.argv[1]).read()
blocks=txt.split('*** Dumping AST Record Layout')
recs={}
for b in blocks:
    lines=[l for l in b.split('\n') if '|' in l]
    if not lines: continue
    m=re.match(r'\s*0 \| (struct|union) zonetool::iw7::([A-Za-z0-9_]+)$', lines[0])
    if not m: continue
    name=m.group(2)
    size=int(re.search(r'sizeof=(\d+)', b).group(1))
    align=int(re.search(r'align=(\d+)', b).group(1))
    fields=[]
    stack=[]  # (depth, prefix)
    for l in lines[1:]:
        off_s, rest = l.split('|',1)
        if 'sizeof=' in rest or 'nvsize' in rest: continue
        off_s=off_s.strip()
        if not off_s: continue
        depth=(len(rest)-len(rest.lstrip(' '))-1)//2
        decl=rest.strip()
        if ':' in off_s:  # bitfield
            off=int(off_s.split(':')[0]); bit=True
        else:
            off=int(off_s); bit=False
        # member name = last identifier
        mm=re.match(r'(.*?)([A-Za-z_][A-Za-z0-9_]*)\s*$', decl)
        if not mm: continue
        ty, fname = mm.group(1).strip(), mm.group(2)
        anon = '(anonymous' in decl
        while stack and stack[-1][0]>=depth: stack.pop()
        prefix=''.join(p for d,p in stack)
        if anon:
            stack.append((depth,''))
            continue
        if ty.startswith('struct ') or ty.startswith('union '):
            stack.append((depth, fname+'__'))
        fields.append((prefix+fname, off, ty, bit))
    recs[name]=(size,align,fields)
out=['//! Generated: MSVC x64 record layouts of the x64-zt IW7 headers','//! (`clang++ --target=x86_64-pc-windows-msvc -Xclang -fdump-record-layouts`).','//! Offsets in bytes; `SIZE` is `sizeof`. Do not edit by hand.','#![allow(non_snake_case, non_upper_case_globals, dead_code)]','']
for name in sorted(recs):
    size,align,fields=recs[name]
    out.append(f'pub mod {name} {{')
    out.append(f'    pub const SIZE: usize = {size};')
    out.append(f'    pub const ALIGN: usize = {align};')
    seen=set()
    for fname,off,ty,bit in fields:
        if fname in seen or fname in ('SIZE','ALIGN'): continue
        seen.add(fname)
        rid = 'r#'+fname if fname in KW else fname
        out.append(f'    pub const {rid}: usize = {off};')
    out.append('}')
open(sys.argv[2],'w').write('\n'.join(out)+'\n')
print(len(recs))
