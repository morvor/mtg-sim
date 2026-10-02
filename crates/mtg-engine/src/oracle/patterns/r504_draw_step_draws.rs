//! Card draws and "the first one they draw in each of their draw steps" (CR 504.1, 121).
//!
//! * "Whenever an opponent draws a card except the first one they draw in each of their
//!   draw steps, ..." (Orcish Bowmasters, Leela, Sevateem Warrior; Xyris, the Writhing
//!   Storm) and "If [player] would draw a card except the first one [they] draw in each of
//!   [their] draw steps, ..." (Notion Thief, Hullbreacher, Alhammarret's Archive, Chains of
//!   Mephistopheles, ...): the ability without the exception, with the draw qualified —
//!   triggers by `kw::draw_step_draws::NOT_FIRST_DRAW_IN_DRAW_STEP`, replacement effects
//!   by `Not(PlayerFilter::FirstDrawInDrawStep)`.
//! * "If [player] would draw a card, instead [effect]" / "..., [effect] instead": a
//!   replacement effect for each card draw (CR 614.1a, 121.6), "that player" being the
//!   player who would draw. "Instead that player skips that draw and [effect]" says the
//!   same thing (the draw is replaced). "That player discards a card instead. If the
//!   player discards a card this way, they draw a card. If the player doesn't discard a
//!   card this way, they mill a card." (Chains of Mephistopheles): the follow-ups depend
//!   on whether the discard happened.

use super::{AbilityPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// The exception, as each card words it (after the "draws a card"/"would draw a card" it
/// qualifies).
const EXCEPT_FIRST: [&str; 2] = [
    " except the first one they draw in each of their draw steps",
    " except the first one you draw in each of your draw steps",
];

/// An ability with "except the first one they draw in each of their draw steps": the
/// ability without it, with its card draw qualified.
fn except_first_draw(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let phrase = EXCEPT_FIRST.iter().find(|p| block.contains(*p))?;
    if block.matches(phrase).count() != 1 {
        return None;
    }
    let (before, after) = block.split_once(phrase)?;
    if !(before.ends_with(" would draw a card") || before.ends_with(" draws a card")) {
        return None;
    }
    let rest = format!("{before}{after}");
    let parsed = crate::oracle::parse_ability(&rest, ctx)?;
    let mut qualified = 0;
    let mut out = Vec::new();
    for a in parsed {
        let mut kind = a.kind.clone();
        qualified += qualify(&mut kind);
        out.push(AbilityDef::with_link(kind, block, a.link));
    }
    (qualified == 1).then_some(out)
}

/// Qualifies the card draw an ability is about; returns how many it qualified.
fn qualify(kind: &mut AbilityKind) -> usize {
    match kind {
        AbilityKind::Triggered(t) => qualify_trigger(&mut t.trigger),
        AbilityKind::Static(s) => match &mut s.effect {
            StaticEffect::Replacement(ReplacementDef {
                event: ReplacementEvent::Draw(pf),
                ..
            }) => {
                *pf = PlayerFilter::And(vec![
                    pf.clone(),
                    PlayerFilter::Not(Box::new(PlayerFilter::FirstDrawInDrawStep)),
                ]);
                1
            }
            _ => 0,
        },
        _ => 0,
    }
}

fn qualify_trigger(t: &mut TriggerCond) -> usize {
    match t {
        TriggerCond::Draws { .. } => {
            *t = TriggerCond::Where {
                trigger: Box::new(t.clone()),
                cond: Condition::Custom(
                    crate::kw::draw_step_draws::NOT_FIRST_DRAW_IN_DRAW_STEP.into(),
                ),
            };
            1
        }
        TriggerCond::AnyOf(v) => v.iter_mut().map(qualify_trigger).sum(),
        _ => 0,
    }
}

inventory::submit! { AbilityPattern { name: "r504: except the first one they draw in each of their draw steps", priority: 50, parse: except_first_draw } }

/// "If [player] would draw a card, instead [effect]." / "..., [effect] instead." (with the
/// discard follow-ups of Chains of Mephistopheles).
fn would_draw_instead(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ")?;
    let (who, r) = r.split_once(" would draw a card, ")?;
    let who = match who {
        "you" => PlayerFilter::You,
        "an opponent" => PlayerFilter::Opponent,
        "a player" => PlayerFilter::Any,
        _ => return None,
    };
    let mut sentences = r.split(". ");
    let first = sentences.next()?;
    let clause = first
        .strip_prefix("instead ")
        .or_else(|| first.strip_suffix(" instead"))?;
    // The draw is replaced: "skips that draw" says so again.
    let clause = clause
        .strip_prefix("that player skips that draw and ")
        .unwrap_or(clause);
    if clause.starts_with("you may ") || clause.contains(" may ") {
        return None;
    }
    let mut b = Builder::new(ctx);
    // "That player" is the player who would draw; "it" has no antecedent ("that card"
    // isn't a card yet).
    b.it = crate::oracle::patterns::oracle_hardening_referents::no_referent();
    b.it_player = PlayerRef::TriggerPlayer;
    let mut effect = if clause == "that player skips that draw" {
        Effect::Noop
    } else {
        parse_clause(clause, &mut b)?
    };
    // "If the player discards a card this way, they draw a card. If the player doesn't
    // discard a card this way, they mill a card."
    let rest: Vec<&str> = sentences.collect();
    match rest.as_slice() {
        [] => {}
        [yes, no] => {
            let then = discarded_this_way(yes, "if the player discards a card this way, ", ctx)?;
            let otherwise = discarded_this_way(
                no,
                "if the player doesn't discard a card this way, ",
                ctx,
            )?;
            if !matches!(effect, Effect::Discard { .. }) {
                return None;
            }
            effect = Effect::Seq(vec![
                effect,
                Effect::If {
                    cond: Condition::PrevHappened,
                    then: Box::new(then),
                    otherwise: Box::new(otherwise),
                },
            ]);
        }
        _ => return None,
    }
    if !b.targets.is_empty() {
        return None;
    }
    Some(vec![AbilityDef::new(
        AbilityKind::Static(StaticAbility::new(StaticEffect::Replacement(
            ReplacementDef {
                event: ReplacementEvent::Draw(who),
                action: ReplacementAction::Instead(Box::new(effect)),
                self_replacement: false,
                optional: false,
            },
        ))),
        text,
    )])
}

/// "[if the player ... this way, ]they [effect]": the player who would have drawn does it.
fn discarded_this_way(s: &str, prefix: &str, ctx: &CompileContext) -> Option<Effect> {
    let clause = s.strip_prefix(prefix)?.strip_prefix("they ")?;
    let mut b = Builder::new(ctx);
    b.it = crate::oracle::patterns::oracle_hardening_referents::no_referent();
    let effect = parse_clause(clause, &mut b)?;
    b.targets.is_empty().then_some(Effect::AsPlayer {
        who: PlayerRef::TriggerPlayer,
        effect: Box::new(effect),
    })
}

inventory::submit! { StaticPattern { name: "r121: if a player would draw a card, instead [effect]", priority: 110, parse: would_draw_instead } }
