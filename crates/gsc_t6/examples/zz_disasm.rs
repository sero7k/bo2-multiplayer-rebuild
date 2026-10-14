// scratch: delete before handing back
use gsc_t6::object::{Insn, ScriptObject, op_name};
fn main() {
    let only = std::env::var("FN").ok();
    for p in std::env::args().skip(1) {
        let b = std::fs::read(&p).unwrap();
        let o = ScriptObject::parse(&b).unwrap();
        println!("#### {} includes {:?}", o.name, o.includes);
        for f in &o.functions {
            if only.as_ref().is_some_and(|n| !f.name.eq_ignore_ascii_case(n)) { println!("fn {}({})", f.name, f.params); continue; }
            println!("fn {}({}) @{:#x}", f.name, f.params, f.address);
            for (a, i) in &f.code {
                let s = match i {
                    Insn::Plain(c) => op_name(*c).to_string(),
                    Insn::Int(c, v) => format!("{} {v}", op_name(*c)),
                    Insn::Float(v) => format!("float {v}"),
                    Insn::Vector(v) => format!("vec {v:?}"),
                    Insn::Str(c, s) => format!("{} {s:?}", op_name(*c)),
                    Insn::Locals(l) => format!("locals {l:?}"),
                    Insn::Hash(h) => format!("hash {h:#x}"),
                    Insn::Call(c, imp) => format!("{} {}({})", op_name(*c), imp.full(), imp.params),
                    Insn::Jump(c, t) => format!("{} -> {t:#x}", op_name(*c)),
                    Insn::Switch { cases, end } => format!("switch {:?} end {end:#x}", cases.iter().map(|c| format!("{:?}->{:#x}", c.value, c.target)).collect::<Vec<_>>()),
                    Insn::EndSwitch { end } => format!("endswitch {end:#x}"),
                    Insn::Anim { tree, anim } => format!("anim %{tree}::{anim}"),
                    Insn::AnimTree(t) => format!("animtree {t}"),
                    Insn::DevBlock { to } => format!("devblock -> {to:#x}"),
                };
                println!("  {a:#06x} {s}");
            }
        }
    }
}
