//! "{1}: The next time you would draw a card this turn, [effect] instead." (the Words
//! cycle; CR 614.1a, 614.11): a one-use replacement effect on the player's next card draw
//! this turn. When several apply to the same draw, the player drawing chooses one, which
//! replaces the draw and is used up (CR 616.1); the others wait for later draws.
//!
//! Also "Until end of turn, if target player would draw a card, instead that player skips
//! that draw and you draw a card." (Plagiarize): the target player is locked in as the
//! effect is created (`prevention::lock_def`).
//!
//! The replacing effect's choices are made as it replaces the draw. An effect with a
//! target ("~ deals 2 damage to any target instead") would need its target chosen then,
//! which isn't supported, so such text is left uncompiled.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::{end, parse_target};

fn next_draw_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("the next time you would draw a card this turn, ")?;
    let inner = r.strip_suffix(" instead")?;
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

inventory::submit! { EffectPattern { name: "replacements: the next time you would draw a card this turn, [effect] instead", priority: 70, parse: next_draw_instead } }

/// "until end of turn, if target player would draw a card, instead that player skips that
/// draw and [effect]".
fn until_eot_draw_instead(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("until end of turn, if ")?;
    let (who, r) = r.split_once(" would draw a card, instead ")?;
    let inner = r.strip_prefix("that player skips that draw and ")?;
    let saved = b.targets.len();
    let parsed = (|| {
        let (spec, rest) = parse_target(who)?;
        if !rest.trim().is_empty()
            || !matches!(spec.what, TargetKind::Player(_))
            || spec.fixed_min() != Some(1)
            || !matches!(spec.max, Value::Const(1))
        {
            return None;
        }
        let slot = b.add_target(spec, who);
        let before = b.targets.len();
        let e = parse_clause(inner, b)?;
        if b.targets.len() != before {
            return None;
        }
        Some(Effect::AddReplacement {
            def: ReplacementDef {
                event: ReplacementEvent::Draw(PlayerFilter::Ref(Box::new(PlayerRef::Target(
                    slot,
                )))),
                action: ReplacementAction::Instead(Box::new(e)),
                self_replacement: false,
                optional: false,
            },
            duration: Duration::EndOfTurn,
            uses: None,
        })
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "replacements: until end of turn, if target player would draw a card, instead [effect]", priority: 70, parse: until_eot_draw_instead } }
