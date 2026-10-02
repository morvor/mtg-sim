//! "~ can be attached only to a [quality] creature" (Gate Smasher, Konda's Banner): the
//! Equipment can't legally equip other creatures. Attempts to attach it to one do nothing
//! (CR 301.5b, 701.3b), and it becomes unattached from one as a state-based action
//! (CR 301.5c, 704.5n). Its equip ability may still target any creature you control.

use crate::ability::*;
use crate::oracle::patterns::StaticPattern;
use crate::oracle::phrases::{end, parse_object_phrase};
use crate::oracle::CompileContext;

fn attach_only_to(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = l.strip_prefix("~ can be attached only to ")?;
    let r = r.strip_prefix("a ").or_else(|| r.strip_prefix("an "))?;
    let (f, plural, tail) = parse_object_phrase(r)?;
    if plural || !end(tail).trim().is_empty() {
        return None;
    }
    let s = StaticAbility::new(StaticEffect::AttachOnlyTo(f));
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text)])
}

inventory::submit! { StaticPattern { name: "~ can be attached only to a [filter]", priority: 100, parse: attach_only_to } }
