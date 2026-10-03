//! Replacement effects on objects being put into graveyards (CR 614.1a, 614.6):
//!
//! - "If ~ would be put into a graveyard from anywhere, exile it instead." (the back faces
//!   of disturb cards) and "... reveal ~ and shuffle it into its owner's library instead."
//!   (Darksteel Colossus). Such an ability of the object itself functions in every zone
//!   the object can be put into a graveyard from — the battlefield, the stack, a hand, a
//!   library, exile — so it's a static ability that functions anywhere.
//! - "If a card [or token] would be put into [a / your / an opponent's] graveyard from
//!   anywhere, exile it instead." (Rest in Peace, Leyline of the Void) from a permanent.
//! - "If a card would be put into your graveyard from anywhere this turn, exile that card
//!   instead." as a one-shot effect that lasts for the turn (Yawgmoth's Will).

use super::{EffectPattern, StaticPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

inventory::submit! {
    StaticPattern { name: "control_exile: would be put into a graveyard from anywhere", priority: 60, parse: s_graveyard_instead }
}
inventory::submit! {
    EffectPattern { name: "control_exile: would be put into a graveyard this turn", priority: 60, parse: p_graveyard_instead_this_turn }
}

/// "[objects] would be put into [a / your / an opponent's] graveyard from anywhere
/// [this turn], [instead]" → (filter, whether the objects are the source itself, rest).
fn event(r: &str) -> Option<(Filter, bool, &str)> {
    let (who, rest) = r.split_once(" would be put into ")?;
    let (owner, rest) = if let Some(x) = rest.strip_prefix("a graveyard from anywhere") {
        (None, x)
    } else if let Some(x) = rest.strip_prefix("your graveyard from anywhere") {
        (Some(PlayerRel::You), x)
    } else if let Some(x) = rest.strip_prefix("an opponent's graveyard from anywhere") {
        (Some(PlayerRel::Opponent), x)
    } else {
        return None;
    };
    let (mut filter, is_self) = if who == "~" {
        (Filter::Source, true)
    } else {
        let x = who.strip_prefix("a ").or_else(|| who.strip_prefix("an "))?;
        let (f, plural, tail) = parse_object_phrase(x)?;
        if plural || !tail.trim().is_empty() {
            return None;
        }
        // Only cards and tokens are put into graveyards (CR 111.7, 404.1): a phrase
        // naming another kind of object ("a spell", "a permanent") has a meaning of its
        // own not modeled here.
        if !names_cards_or_tokens(&f) {
            return None;
        }
        (f, false)
    };
    if let Some(rel) = owner {
        // CR 400.3: a card is put into its owner's graveyard.
        filter = Filter::and(vec![filter, Filter::OwnedBy(rel)]);
    }
    Some((filter, is_self, rest))
}

fn names_cards_or_tokens(f: &Filter) -> bool {
    match f {
        Filter::Card | Filter::Token => true,
        Filter::Or(v) => v.iter().all(names_cards_or_tokens),
        Filter::And(v) => v.iter().any(names_cards_or_tokens),
        _ => false,
    }
}

/// What happens instead: "exile it", "exile that card", "reveal ~ and shuffle it into its
/// owner's library".
fn instead(s: &str, is_self: bool) -> Option<Destination> {
    match s {
        "exile it instead" | "exile that card instead" | "instead exile it" => {
            Some(Destination::zone(ZoneKind::Exile))
        }
        "reveal ~ and shuffle it into its owner's library instead"
        | "shuffle it into its owner's library instead"
            if is_self =>
        {
            let mut d = Destination::zone(ZoneKind::Library);
            d.position = LibraryPosition::Shuffled;
            Some(d)
        }
        _ => None,
    }
}

fn def(filter: Filter, to: Destination) -> ReplacementDef {
    ReplacementDef {
        event: ReplacementEvent::ZoneChange {
            filter,
            from: None,
            to: Some(ZoneKind::Graveyard),
        },
        action: ReplacementAction::MoveInstead(to),
        self_replacement: false,
        optional: false,
    }
}

fn s_graveyard_instead(l: &str, text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let r = end(l).strip_prefix("if ")?;
    let (filter, is_self, rest) = event(r)?;
    let to = instead(rest.strip_prefix(", ")?, is_self)?;
    let mut st = StaticAbility::new(StaticEffect::Replacement(def(filter, to)));
    if is_self {
        // The object's own ability applies wherever it would be put into a graveyard
        // from (CR 113.6: the ability says it functions "from anywhere").
        st.zone = FunctionZone::Anywhere;
    }
    Some(vec![AbilityDef::new(AbilityKind::Static(st), text)])
}

/// "if a card would be put into your graveyard from anywhere this turn, exile that card
/// instead".
fn p_graveyard_instead_this_turn(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("if ")?;
    let (filter, is_self, rest) = event(r)?;
    if is_self {
        return None;
    }
    let rest = rest.strip_prefix(" this turn, ")?;
    let to = instead(rest, false)?;
    Some(Effect::AddReplacement {
        def: def(filter, to),
        duration: Duration::EndOfTurn,
        uses: None,
    })
}
