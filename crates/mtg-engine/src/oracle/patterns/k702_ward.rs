//! Oracle patterns for ward (CR 702.21) granted with a defined X: "~ has ward {X}, where
//! X is the number of experience counters you have." The value of X is determined each
//! time the ward ability resolves, not locked in as it triggers (CR 702.21b).

use super::StaticPattern;
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "the number of [kind] counters you have", or a value phrase the core understands.
fn ward_x_value(s: &str, ctx: &CompileContext) -> Option<Value> {
    if let Some(kind) = s
        .strip_prefix("the number of ")
        .and_then(|r| r.strip_suffix(" counters you have"))
        .filter(|k| !k.contains(' '))
    {
        return Some(Value::PlayerCounters(PlayerRef::You, SmolStr::new(kind)));
    }
    let (v, tail) = crate::oracle::statics::parse_value_phrase(s, &mut Builder::new(ctx))?;
    end(&tail).is_empty().then_some(v)
}

/// "~ has ward {x}, where x is [value]".
fn has_ward_x(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("~ has ward {x}, where x is ")?;
    let x = ward_x_value(r, ctx)?;
    let kw = Keyword {
        cost: Some(Cost::mana(ManaCost::parse("{X}")?)),
        x: Some(x),
        text: Some(SmolStr::new("Ward {X}")),
        ..Keyword::new(KeywordKind::Ward)
    };
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::AddKeyword(kw)],
        })),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "k702.21 has ward {x}", priority: 60, parse: has_ward_x } }
