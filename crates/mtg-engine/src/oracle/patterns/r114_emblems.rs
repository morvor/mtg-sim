//! Oracle patterns for emblems (CR 114): "[player] gets an emblem with "[ability]"" and
//! "... with "[ability]" and "[ability]"".

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_player};
use crate::oracle::CompileContext;
use crate::types::TypeLine;

/// The quoted abilities of "with "A" and "B"" (the text after "with ").
fn quoted_abilities(s: &str, b: &Builder) -> Option<Vec<Ability>> {
    // An emblem has no types (CR 114.3): its abilities aren't spell abilities even when
    // an instant or sorcery creates it.
    let no_types = TypeLine::default();
    let ctx = CompileContext {
        card_name: "",
        full_name: "",
        type_line: &no_types,
        keywords: &[],
        power: None,
        toughness: None,
        ..*b.ctx
    };
    let mut out = Vec::new();
    let mut rest = s.trim();
    loop {
        let r = rest.strip_prefix('"')?;
        let close = r.find('"')?;
        // Inside an emblem, "this emblem" is the emblem itself.
        let inner = r[..close].replace("this emblem", "~");
        let inner = inner.trim().trim_end_matches('.');
        for a in crate::oracle::parse_ability(inner, &ctx)? {
            if matches!(a.kind, AbilityKind::Unsupported(_)) {
                return None;
            }
            out.push(a);
        }
        rest = end(&r[close + 1..]);
        if rest.is_empty() {
            return Some(out);
        }
        rest = rest
            .strip_prefix("and ")
            .or_else(|| rest.strip_prefix(", and "))
            .or_else(|| rest.strip_prefix(", "))?
            .trim();
    }
}

/// "you get an emblem with "..."", "target opponent gets an emblem with "..."", "each
/// opponent gets an emblem with "..."" (CR 114.2).
fn gets_an_emblem(l: &str, b: &mut Builder) -> Option<Effect> {
    let (who, rest) = l.split_once(" an emblem with ")?;
    let who = who
        .strip_suffix(" gets")
        .or_else(|| who.strip_suffix(" get"))?;
    let who_sp = format!("{who} ");
    let (r, spec, tail) = parse_player(&who_sp)?;
    if !tail.trim().is_empty() {
        return None;
    }
    let abilities = quoted_abilities(rest, b)?;
    let who = match spec {
        Some(spec) => {
            let text = spec.text.clone();
            let slot = b.add_target(spec, &text);
            b.it_player = PlayerRef::Target(slot);
            PlayerRef::Target(slot)
        }
        None => r,
    };
    Some(Effect::CreateEmblem { who, abilities })
}

inventory::submit! { EffectPattern { name: "gets an emblem", priority: 0, parse: gets_an_emblem } }
