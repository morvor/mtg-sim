//! "As long as ~ is on the stack, [static ability]": a static ability of an instant or
//! sorcery that functions while it's on the stack (CR 113.6, 113.6b exceptions: an
//! instant's or sorcery's static abilities function on the stack only when they say so).
//! "As long as ~ is on the stack, spells that target it cost {2} more to cast." (Kaervek's
//! Torch); "... each creature has \"{T}: ...\"" (Torrent of Lava).

use crate::ability::*;
use crate::oracle::CompileContext;

const PREFIX: &str = "as long as ~ is on the stack, ";

/// The static abilities of `rest` (the text after the "as long as" clause), functioning
/// on the stack. In a quoted ability the grant grammar reads "dealt to ~ by ~" as the
/// object that has the ability, then the granting spell (Torrent of Lava, CR 113.7).
pub fn on_stack(rest: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let mut rest = rest.trim().to_string();
    // "spells that target it": "it" is the spell itself.
    rest = rest.replace(" that target it ", " that target ~ ");
    let mut c = rest.chars();
    let first = c.next()?;
    let rest = first.to_uppercase().collect::<String>() + c.as_str();
    // The static grammar is the permanents' one: the instant or sorcery is compiled
    // without its spell types, so its text isn't read as a spell ability.
    let mut tl = ctx.type_line.clone();
    tl.card_types.remove(crate::types::CardType::Instant);
    tl.card_types.remove(crate::types::CardType::Sorcery);
    let as_static = CompileContext {
        type_line: &tl,
        ..*ctx
    };
    let parsed = crate::oracle::statics::parse_static(&rest, &as_static)?;
    let mut out = Vec::new();
    for a in parsed {
        let AbilityKind::Static(s) = &a.kind else {
            return None;
        };
        if s.condition.is_some() || s.zone != FunctionZone::Battlefield {
            return None;
        }
        let mut s = s.clone();
        s.zone = FunctionZone::Stack;
        out.push(AbilityDef::with_link(AbilityKind::Static(s), a.text.clone(), a.link));
    }
    if out.is_empty() {
        return None;
    }
    Some(out)
}

/// "As long as ~ is on the stack, [static]." on an instant or sorcery (from
/// `statics::parse_spell_static`).
pub fn as_long_as_on_stack(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = text.trim();
    t.to_lowercase().strip_prefix(PREFIX)?;
    let rest = t.get(PREFIX.len()..)?;
    let v = on_stack(rest, ctx)?;
    Some(
        v.into_iter()
            .map(|a| AbilityDef::with_link(a.kind.clone(), t, a.link))
            .collect(),
    )
}
