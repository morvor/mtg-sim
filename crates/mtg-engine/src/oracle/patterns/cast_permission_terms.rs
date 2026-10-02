//! Permissions to play cards that come with terms (see `permissions.rs`):
//!
//! * "If you cast a spell this way, pay life equal to its mana value rather than pay its
//!   mana cost." after an effect's permission to play cards (Inside Information, Xander's
//!   Pact): the alternative cost every spell cast with that permission is cast for
//!   (CR 118.9b), so no other alternative cost can be used for it (CR 118.9a);
//! * "Each opponent exiles the top card of their library. You may cast spells from among
//!   those cards this turn." (Xander's Pact): permissions to cast those cards, not to play
//!   them as lands (CR 305.9);
//! * "You may play lands and cast spells from the top of your library. If you cast a spell
//!   this way, pay life equal to its mana value rather than pay its mana cost." (Bolas's
//!   Citadel): a static permission whose spells are cast for that alternative cost; "You
//!   may cast noncreature spells from the top of your library. If you cast a spell this
//!   way, you may cast it as though it had flash." (Elsha of the Infinite): one whose
//!   spells may be cast any time their player could cast an instant (CR 702.8a);
//! * "During each of your turns, you may play a land and cast a permanent spell of each
//!   permanent type from your graveyard." (Muldrotha, the Gravetide): one once-each-turn
//!   permission per permanent type (CR 110.4), each used up by the card played with it
//!   (see `kw/once_each_turn_cast.rs`).

use super::{EffectPattern, FollowupPattern, StaticPattern};
use crate::ability::*;
use crate::kw::once_each_turn_cast::once_unused;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::{end, parse_number};
use crate::oracle::CompileContext;
use crate::types::CardType;

/// The alternative cost "pay life equal to its mana value": the mana value of the spell
/// the card would become (see `permissions::spell_relative_cost`).
fn life_equal_to_mana_value() -> Cost {
    Cost::free().with(CostPart::PayLife(Value::ManaValueOf(Box::new(Sel::This))))
}

/// "If you cast a spell this way, pay life equal to its mana value rather than pay its
/// mana cost."
fn is_pay_life_instead(l: &str) -> bool {
    matches!(
        end(l),
        "if you cast a spell this way, pay life equal to its mana value rather than pay its mana cost"
            | "if you cast a spell this way, pay life equal to that spell's mana value rather than pay its mana cost"
    )
}

/// Whether the effect gives permissions to play cards.
fn grants_play(e: &Effect) -> bool {
    match e {
        Effect::GrantPlayPermission { .. } => true,
        Effect::Seq(v) => v.iter().any(grants_play),
        Effect::If {
            then, otherwise, ..
        } => grants_play(then) || grants_play(otherwise),
        Effect::ForEach { effect, .. }
        | Effect::ForEachPlayer { effect, .. }
        | Effect::WithPlayTerms { effect, .. } => grants_play(effect),
        _ => false,
    }
}

/// Wraps the effect so that the permissions it gives come with `terms`.
fn with_terms(prev: &mut Effect, terms: PlayTerms) {
    *prev = Effect::WithPlayTerms {
        terms,
        effect: Box::new(std::mem::take(prev)),
    };
}

fn pay_life_instead(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    if !is_pay_life_instead(l) || !grants_play(prev) {
        return false;
    }
    with_terms(
        prev,
        PlayTerms {
            alt_cost: Some(life_equal_to_mana_value()),
            ..Default::default()
        },
    );
    true
}

/// "Each opponent exiles the top card of their library", "each player exiles the top two
/// cards of their library".
fn each_exiles_top(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, r) = if let Some(r) = l.strip_prefix("each opponent exiles the top ") {
        (PlayerRef::EachOpponent, r)
    } else {
        (
            PlayerRef::EachPlayer,
            l.strip_prefix("each player exiles the top ")?,
        )
    };
    let n = if let Some(r) = r.strip_prefix("card of their library") {
        if !r.is_empty() {
            return None;
        }
        Value::c(1)
    } else {
        let (n, r) = parse_number(r)?;
        n.as_const()?;
        if r.trim() != "cards of their library" {
            return None;
        }
        n
    };
    b.it = Sel::Var(vars::IT);
    Some(Effect::Exile {
        what: Sel::TopOfLibrary(who, n),
        face_down: false,
        link: false,
    })
}

/// Whether the effect ends by exiling the top cards of libraries.
fn ends_exiling_top_cards(e: &Effect) -> bool {
    match e {
        Effect::Exile {
            what: Sel::TopOfLibrary(..),
            face_down: false,
            ..
        } => true,
        Effect::Seq(v) => v.last().is_some_and(ends_exiling_top_cards),
        _ => false,
    }
}

/// "You may cast spells from among those cards this turn." after exiling the top cards of
/// libraries: a permission to cast each of them (not to play a land, CR 305.9).
fn may_cast_spells_from_among(l: &str, prev: &mut Effect, _b: &mut Builder) -> bool {
    let duration = match end(l) {
        "you may cast spells from among those cards this turn"
        | "you may cast spells from among them this turn"
        | "until end of turn, you may cast spells from among those cards" => Duration::EndOfTurn,
        _ => return false,
    };
    if !ends_exiling_top_cards(prev) {
        return false;
    }
    let grant = Effect::GrantPlayPermission {
        who: PlayerRef::You,
        what: Sel::Var(vars::IT),
        duration,
        free: false,
    }
    .cast_only();
    *prev = Effect::seq(vec![std::mem::take(prev), grant]);
    true
}

/// "[A permission to play cards from a zone]. If you cast a spell this way, pay life equal
/// to its mana value rather than pay its mana cost." (Bolas's Citadel), "... If you cast a
/// spell this way, you may cast it as though it had flash." (Elsha of the Infinite).
fn static_with_terms(l: &str, text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let (first, second) = end(l).split_once(". ")?;
    let flash = end(second) == "if you cast a spell this way, you may cast it as though it had flash";
    if !flash && !is_pay_life_instead(second) {
        return None;
    }
    let abilities = crate::oracle::statics::parse_static(first, ctx)?;
    let [a] = abilities.as_slice() else {
        return None;
    };
    let mut kind = a.kind.clone();
    let AbilityKind::Static(s) = &mut kind else {
        return None;
    };
    let StaticEffect::PlayPermission(p) = &mut s.effect else {
        return None;
    };
    if !p.spells || p.cost.is_some() || p.flash || p.zone == ZoneKind::Hand {
        return None;
    }
    if flash {
        p.flash = true;
    } else {
        p.cost = Some(life_equal_to_mana_value());
    }
    Some(vec![AbilityDef::new(kind, text)])
}

/// "During each of your turns, you may play a land and cast a permanent spell of each
/// permanent type from your graveyard."
fn land_and_spell_of_each_permanent_type(
    l: &str,
    text: &str,
    ctx: &CompileContext,
) -> Option<Vec<Ability>> {
    if ctx.is_spell()
        || end(l)
            != "during each of your turns, you may play a land and cast a permanent spell of each permanent type from your graveyard"
    {
        return None;
    }
    let permission = |t: CardType, land: bool, slot: &str| {
        let mut s = StaticAbility::new(StaticEffect::PlayPermission(PlayPermission {
            who: PlayerRel::You,
            zone: ZoneKind::Graveyard,
            top_only: false,
            what: Filter::Type(t),
            lands: land,
            spells: !land,
            cost: None,
            flash: false,
        }));
        s.condition = Some(Condition::And(vec![Condition::YourTurn, once_unused(slot)]));
        AbilityDef::new(AbilityKind::Static(s), text)
    };
    // CR 110.4: the permanent types.
    let mut v = vec![permission(CardType::Land, true, "land")];
    for (t, slot) in [
        (CardType::Artifact, "artifact spell"),
        (CardType::Battle, "battle spell"),
        (CardType::Creature, "creature spell"),
        (CardType::Enchantment, "enchantment spell"),
        (CardType::Planeswalker, "planeswalker spell"),
    ] {
        v.push(permission(t, false, slot));
    }
    Some(v)
}

inventory::submit! { FollowupPattern { name: "cast permission terms: if you cast a spell this way, pay life equal to its mana value rather than pay its mana cost", priority: 85, apply: pay_life_instead } }
inventory::submit! { EffectPattern { name: "cast permission terms: each opponent exiles the top card of their library", priority: 90, parse: each_exiles_top } }
inventory::submit! { FollowupPattern { name: "cast permission terms: you may cast spells from among those cards this turn", priority: 90, apply: may_cast_spells_from_among } }
inventory::submit! { StaticPattern { name: "cast permission terms: play from a zone, paying life equal to its mana value or with flash", priority: 60, parse: static_with_terms } }
inventory::submit! { StaticPattern { name: "cast permission terms: a land and a permanent spell of each permanent type from your graveyard", priority: 100, parse: land_and_spell_of_each_permanent_type } }
