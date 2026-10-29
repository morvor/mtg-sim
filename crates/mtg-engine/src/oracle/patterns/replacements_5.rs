//! Replacement grammar V (cards4-costs-mana-replacements): draw replacements.
//!
//! - "The next time you would draw a card this turn, instead [effect]." (Ring of Ma'rûf,
//!   Aladdin's Lamp): the "instead" first form of the Words cycle's one-use replacement
//!   (`replacements_next_draw.rs`, CR 614.1a, 614.11). An effect that ends in drawing a
//!   card ("..., then draw a card", Aladdin's Lamp) draws a new card: that draw is a new
//!   event this effect, used up, doesn't replace again (CR 614.5, 121.6).
//! - "If a permanent with a wind counter on it would untap during its controller's untap
//!   step, remove all wind counters from it instead." (Freyalise's Winds): a replacement of
//!   untapping during the untap step (CR 502.3, 614.1a; `kw/untap_replacement.rs`), "it"
//!   being the permanent.
//! - "look at the top X cards of your library, put all but one of them on the bottom of your
//!   library in a random order": the one left stays on top.

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;

/// "the next time you would draw a card this turn, instead [effect]".
fn next_draw_instead_first(l: &str, b: &mut Builder) -> Option<Effect> {
    let inner = end(l).strip_prefix("the next time you would draw a card this turn, instead ")?;
    let saved = b.targets.len();
    let e = parse_clause(inner, b);
    if b.targets.len() != saved {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::AddReplacement {
        def: ReplacementDef {
            event: ReplacementEvent::Draw(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(e?)),
            self_replacement: false,
            optional: false,
        },
        duration: Duration::EndOfTurn,
        uses: Some(1),
    })
}

inventory::submit! { EffectPattern { name: "replacements 5: the next time you would draw a card this turn, instead [effect]", priority: 70, parse: next_draw_instead_first } }

/// "look at the top N cards of your library, put all but one of them on the bottom of your
/// library in a random order[, then (instruction)]".
fn all_but_one_on_bottom(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("look at the top ")?;
    let (n, r) = parse_number(r)?;
    let r = r
        .trim_start()
        .strip_prefix("cards of your library, put all but one of them on the bottom of your library in a random order")?;
    let dig = Effect::Dig {
        who: PlayerRef::You,
        n,
        reveal: false,
        filter: Filter::Any,
        take: Value::c(1),
        take_up_to: false,
        take_to: Destination::library_top(),
        rest_to: Destination {
            position: LibraryPosition::BottomRandom,
            ..Destination::library_bottom()
        },
    };
    if r.is_empty() {
        return Some(dig);
    }
    let rest = r.strip_prefix(", then ")?;
    let then = parse_clause(rest, b)?;
    Some(Effect::seq(vec![dig, then]))
}

inventory::submit! { EffectPattern { name: "replacements 5: look at the top N cards, put all but one on the bottom in a random order", priority: 100, parse: all_but_one_on_bottom } }

/// "If [a permanent] would untap during its controller's untap step, [instructions]
/// instead." / "..., instead [instructions]."
fn untap_instead(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ")?;
    let (subj, r) = r.split_once(" would untap during its controller's untap step, ")?;
    let subj = subj
        .strip_prefix("a ")
        .or_else(|| subj.strip_prefix("an "))?;
    let (f, plural, rest) = crate::oracle::phrases::parse_object_phrase(subj)?;
    if plural || !rest.trim().is_empty() {
        return None;
    }
    let inner = r
        .strip_prefix("instead ")
        .or_else(|| r.strip_suffix(" instead"))?;
    let e = super::replacement_grammar::instructions(inner, ctx, Sel::TriggerObject)?;
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::UntapDuringUntapStep(Filter::and(vec![
                    f,
                    Filter::Permanent,
                ])),
                action: ReplacementAction::Instead(Box::new(e)),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

inventory::submit! { StaticPattern { name: "replacements 5: if [a permanent] would untap during its controller's untap step, [instructions] instead", priority: 100, parse: untap_instead } }
