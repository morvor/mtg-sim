//! "Create [a token]. When that token leaves the battlefield, [effect]." (Ugin, the
//! Ineffable: "+1: Exile the top card of your library face down and look at it. Create a
//! 2/2 colorless Spirit creature token. When that token leaves the battlefield, put the
//! exiled card into your hand."): a delayed triggered ability created by the resolving
//! ability (CR 603.7), about the tokens just created, that triggers once (CR 603.7c) — the
//! first time one of them leaves the battlefield, even if the ability's source has left
//! the battlefield by then. "The exiled card" is the card this resolution exiled, so each
//! activation's token returns its own card.
//!
//! Also "exile the top card of your library face down and look at it" (the player who
//! exiled it may look at it while it remains exiled, CR 406.3) and "permanent that's one
//! or more colors" (not colorless, CR 105.2).

use super::{EffectPattern, FilterSuffixPattern, FollowupPattern};
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, parse_simple, Builder};
use crate::oracle::phrases::end;
use crate::zones::MAY_LOOK_AT_EXILED;

/// The tokens the delayed trigger is about, captured as it's created.
const TOKENS: Var = vars::USER + 6071;
/// The cards the resolving ability exiled, captured as the delayed trigger is created.
const EXILED: Var = vars::USER + 6072;

/// Whether the effect ends by creating tokens.
fn creates_tokens(e: &Effect) -> bool {
    match e {
        Effect::CreateToken { .. } | Effect::CreateTokenWithPT { .. } => true,
        Effect::Seq(v) => v.last().is_some_and(creates_tokens),
        _ => false,
    }
}

/// Whether the paragraph with the delayed trigger exiles cards before it ("Exile the top
/// card of your library face down ... When that token leaves the battlefield, put the
/// exiled card into your hand").
fn exiles_before(l: &str) -> bool {
    let raw = crate::oracle::raw_text().to_lowercase();
    raw.lines().any(|p| {
        p.find(l.trim_end_matches('.'))
            .is_some_and(|i| p[..i].contains("exile "))
    })
}

fn when_that_token_leaves(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(text) = end(l).strip_prefix("when that token leaves the battlefield, ") else {
        return false;
    };
    if !creates_tokens(prev) {
        return false;
    }
    // "the exiled card": the card an earlier instruction of this ability exiled, still
    // recorded as "it" (creating the token doesn't change that).
    let refers_to_exiled = text.contains("the exiled card");
    if refers_to_exiled && !exiles_before(end(l)) {
        return false;
    }
    let mut sub = Builder::new(b.ctx);
    sub.in_trigger = true;
    sub.it = Sel::Var(EXILED);
    sub.it_player = b.it_player.clone();
    let text = text.replace("the exiled card", "it");
    let Some(effect) = parse_effect_text(&text, &mut sub) else {
        return false;
    };
    if !sub.targets.is_empty() {
        return false;
    }
    let mut seq = vec![std::mem::take(prev)];
    if refers_to_exiled {
        seq.push(Effect::Store {
            var: EXILED,
            sel: Sel::Var(vars::IT),
        });
    }
    seq.push(Effect::Store {
        var: TOKENS,
        sel: Sel::Var(vars::CREATED),
    });
    seq.push(Effect::DelayedTrigger {
        trigger: TriggerCond::LeavesBattlefield(Filter::In(Box::new(Sel::Var(TOKENS)))),
        body: Box::new(Body::effect(effect)),
        once: true,
    });
    *prev = Effect::seq(seq);
    true
}

inventory::submit! { FollowupPattern { name: "r603.7 when that token leaves the battlefield, [effect]", priority: 80, apply: when_that_token_leaves } }

/// "exile the top card of your library face down and look at it".
fn exile_face_down_and_look(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_suffix(" and look at it")?;
    if !r.starts_with("exile ") || !r.ends_with(" face down") {
        return None;
    }
    let e = parse_simple(r, b)?;
    if !matches!(
        e,
        Effect::Exile {
            face_down: true,
            ..
        }
    ) {
        return None;
    }
    Some(Effect::seq(vec![e, Effect::Custom(MAY_LOOK_AT_EXILED.into())]))
}

inventory::submit! { EffectPattern { name: "r406 exile face down and look at it", priority: 80, parse: exile_face_down_and_look } }

/// "[permanent] that's one or more colors": not colorless.
fn one_or_more_colors<'a>(t: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let r = t
        .strip_prefix("that's one or more colors")
        .or_else(|| t.strip_prefix("that are one or more colors"))?;
    (r.is_empty() || r.starts_with([' ', ',', '.'])).then(|| (Filter::not(Filter::Colorless), r))
}

inventory::submit! { FilterSuffixPattern { name: "that's one or more colors", priority: 100, parse: one_or_more_colors } }
