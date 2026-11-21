//! Notes kept on objects (CR 607.2e; the engine side is `kw/noted_info.rs`):
//!
//! * "Note a creature type that hasn't been noted for ~." and, after it, "that type"
//!   (Long List of the Ents);
//! * "note one of its creature types that hasn't been noted for ~'s Journal" in the
//!   ability of the token named "~'s Journal" (Volo, Itinerant Scholar): the token's name
//!   is how its own ability refers to it (CR 111.4, 201.4);
//! * "Draw a card for each creature type noted for target permanent you control named
//!   ~'s Journal.";
//! * "Note the mana value of each card as it's put into exile." and "the greatest number
//!   noted for it this turn" (Bell Borca, Spectral Sergeant).

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::kw::noted_info as notes;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// Whether `who` names the object whose ability this is: "~", or the name a token the
/// card creates gives it ("~'s Journal" in the ability of "create ~'s Journal, a ...
/// token").
fn names_self(who: &str) -> bool {
    if who == "~" {
        return true;
    }
    // In the token's own ability its name may also be spelled out ("volo's journal").
    let suffix = who
        .strip_prefix("~'s ")
        .or_else(|| who.rsplit_once("'s ").map(|(_, s)| s));
    suffix
        .and_then(token_name)
        .is_some_and(|n| who.starts_with("~'s ") || n.to_lowercase() == who)
}

/// The name, as printed, of the token the card creates named "[the card's name]'s
/// [suffix]" ("create Volo's Journal, a legendary colorless Book artifact token ...").
fn token_name(suffix: &str) -> Option<String> {
    if suffix.is_empty() || suffix.contains(' ') {
        return None;
    }
    let raw = crate::oracle::raw_text();
    let lower = raw.to_lowercase();
    let full = crate::oracle::card_name();
    let short = full.split(',').next().unwrap_or(&full).trim().to_string();
    for cand in [full.as_str(), short.as_str(), "~"] {
        if cand.is_empty() {
            continue;
        }
        let printed = format!("{}'s {suffix}", cand.to_lowercase());
        let pat = format!("create {printed}, a ");
        if let Some(i) = lower.find(&pat) {
            let start = i + "create ".len();
            return Some(raw[start..start + printed.len()].to_string());
        }
    }
    None
}

fn note_creature_type(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    if let Some(who) = l.strip_prefix("note a creature type that hasn't been noted for ") {
        return names_self(who).then(|| Effect::Custom(notes::NOTE_CREATURE_TYPE.into()));
    }
    let who = l.strip_prefix("note one of its creature types that hasn't been noted for ")?;
    // "its": the spell the ability triggered on.
    if !names_self(who) || !matches!(b.it, Sel::TriggerSpell) {
        return None;
    }
    Some(Effect::Custom(notes::NOTE_ITS_CREATURE_TYPE.into()))
}

inventory::submit! { EffectPattern { name: "notes 5: note a creature type that hasn't been noted for ~", priority: 50, parse: note_creature_type } }

/// "When you next cast a creature spell of that type this turn, ..." after noting a
/// creature type: the type noted (the linked choice, as "the chosen type").
fn of_that_type(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    fn notes_type(e: &Effect) -> bool {
        match e {
            Effect::Seq(v) => v.last().is_some_and(notes_type),
            Effect::Custom(n) => n.as_str() == notes::NOTE_CREATURE_TYPE,
            _ => false,
        }
    }
    if !notes_type(prev) || !l.contains(" of that type") {
        return false;
    }
    let text = l.replacen(" of that type", " of the chosen type", 1);
    let Some(e) = crate::oracle::effects::parse_sentence(&text, b) else {
        return false;
    };
    let old = std::mem::take(prev);
    *prev = Effect::seq(vec![old, e]);
    true
}

inventory::submit! { FollowupPattern { name: "notes 5: of that type (the noted creature type)", priority: 50, apply: of_that_type } }

/// "Draw a card for each creature type noted for target permanent you control named ~'s
/// Journal."
fn draw_for_each_noted_type(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let r = l.strip_prefix("draw a card for each creature type noted for target permanent you control named ")?;
    let suffix = r.strip_prefix("~'s ")?;
    // The token's name as printed ("Volo's Journal").
    let name = token_name(suffix)?;
    let filter = Filter::and(vec![
        Filter::Permanent,
        Filter::ControlledBy(PlayerRel::You),
        Filter::Named(name.as_str().into()),
    ]);
    let text = format!("target permanent you control named {name}");
    let slot = b.add_target(TargetSpec::object(filter, text.clone()), &text);
    Some(Effect::Draw {
        who: PlayerRef::You,
        n: Value::Custom(format!("{}{slot}", notes::NOTED_TYPES_OF_TARGET).into()),
    })
}

inventory::submit! { EffectPattern { name: "notes 5: draw a card for each creature type noted for target permanent named ...", priority: 50, parse: draw_for_each_noted_type } }

/// "Note the mana value of each card as it's put into exile."
fn note_exiled_mana_values(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if end(l) != "note the mana value of each card as it's put into exile" || !ctx.is_permanent() {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Custom(
            notes::NOTE_EXILED_MANA_VALUES.into(),
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "notes 5: note the mana value of each card as it's put into exile", priority: 50, parse: note_exiled_mana_values } }

/// "the greatest number noted for it this turn" (the value grammar's atom), on a card
/// that notes numbers.
pub(crate) fn value(s: &str) -> Option<(Value, String)> {
    let rest = s
        .strip_prefix("the greatest number noted for it this turn")
        .or_else(|| s.strip_prefix("the greatest number noted for ~ this turn"))?;
    let raw = crate::oracle::raw_text().to_lowercase();
    if !raw.contains("note the mana value of each card as it's put into exile") {
        return None;
    }
    Some((
        Value::Custom(notes::GREATEST_NOTED_THIS_TURN.into()),
        rest.to_string(),
    ))
}
