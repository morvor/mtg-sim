//! North Star: "{4}, {T}: For one spell this turn, you may spend mana as though it were
//! mana of any type to pay that spell's mana cost. (Additional costs are still paid
//! normally.)"
//!
//! The cost is compiled; the effect gives its controller a use-once permission until end
//! of turn (`PlayerModification::Custom(`[`ONE_SPELL`]`)`). As they cast a spell this turn
//! paying its mana cost, they may use it for that spell (the `use_spend_permission`
//! hook): each mana symbol of the spell's mana cost (colored, hybrid or colorless) may
//! then be paid with one mana of any type (CR 609.4b: how the cost is paid, not what it
//! is), and the permission is used up. Mana symbols the total cost has beyond those of
//! the mana cost (additional costs, cost increases) are paid normally (reminder text);
//! a spell cast for an alternative cost isn't paying its mana cost (CR 118.9), so the
//! permission doesn't apply to it (interpretation `one-spell-spend-permission`).

use super::{parse, ManualAbility};
use crate::ability::*;
use crate::game::Game;
use crate::keywords::KeywordKind;
use crate::kw::{KeywordRegistration, KeywordRules};
use crate::mana::{ManaCost, ManaSymbol};
use crate::types::{ObjectId, PlayerId};

/// `PlayerModification::Custom` (until end of turn): the use-once permission.
pub const ONE_SPELL: &str =
    "card:North Star:for one spell this turn, spend mana as though any type for its mana cost";

const TEXT: &str = "{4}, {T}: For one spell this turn, you may spend mana as though it were mana of any type to pay that spell's mana cost.";

fn build(ctx: &crate::oracle::CompileContext) -> Vec<Ability> {
    // The compiled cost, with a stand-in effect replaced by the permission.
    let (cost, _) = TEXT.split_once(": ").unwrap_or_default();
    let abilities = parse(ctx, &format!("{cost}: Draw a card."));
    match abilities.as_slice() {
        [a] if matches!(a.kind, AbilityKind::Activated(_)) => vec![super::map_effect(a, TEXT, |_| {
            Effect::AddPlayerEffect {
                who: PlayerRef::You,
                effect: PlayerModification::Custom(ONE_SPELL.into()),
                duration: Duration::EndOfTurn,
            }
        })],
        _ => vec![AbilityDef::new(AbilityKind::Unsupported(TEXT.into()), TEXT)],
    }
}

inventory::submit! { ManualAbility {
    card: "North Star",
    face: 0,
    text: TEXT,
    build,
    reason: "a use-once permission to spend mana as though any type for one spell of its controller's choice this turn: the only such card",
} }

fn is_permission(e: &crate::game::PlayerEffect, p: PlayerId) -> bool {
    e.players.contains(&p) && matches!(&e.effect, PlayerModification::Custom(n) if n == ONE_SPELL)
}

/// Whether `s` is a symbol mana of any type may pay with the permission.
fn convertible(s: &ManaSymbol) -> bool {
    matches!(
        s,
        ManaSymbol::Colored(_)
            | ManaSymbol::Hybrid(..)
            | ManaSymbol::TwoHybrid(_)
            | ManaSymbol::ColorlessHybrid(_)
            | ManaSymbol::Colorless
    )
}

/// `cost` with each symbol of `mana_cost` mana of any type may pay turned into one
/// generic mana (paid with mana of any type). Returns whether anything changed.
fn apply(mana_cost: &ManaCost, cost: &mut Cost) -> bool {
    let Some(m) = cost.mana.as_mut() else {
        return false;
    };
    let mut changed = false;
    for s in mana_cost.symbols.iter().filter(|s| convertible(s)) {
        if let Some(t) = m.symbols.iter_mut().find(|t| *t == s) {
            *t = ManaSymbol::Generic(1);
            changed = true;
        }
    }
    if changed {
        let generic = m.generic_amount();
        m.symbols.retain(|s| !matches!(s, ManaSymbol::Generic(_)));
        if generic > 0 {
            m.symbols.insert(0, ManaSymbol::Generic(generic));
        }
    }
    changed
}

struct Rules;

impl KeywordRules for Rules {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[]
    }
    fn spend_permission(
        &self,
        g: &Game,
        p: PlayerId,
        _card: ObjectId,
        mana_cost: Option<&ManaCost>,
        cost: &mut Cost,
    ) {
        if let Some(mc) = mana_cost {
            if g.player_effects.iter().any(|e| is_permission(e, p)) {
                apply(mc, cost);
            }
        }
    }
    fn use_spend_permission(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        mana_cost: Option<&ManaCost>,
        cost: &mut Cost,
    ) {
        let Some(mc) = mana_cost else {
            return;
        };
        // The earliest permission (they're all alike).
        let Some(first) = g
            .player_effects
            .iter()
            .filter(|e| is_permission(e, p))
            .min_by_key(|e| e.timestamp)
            .map(|e| (e.id, e.source))
        else {
            return;
        };
        let mut with = cost.clone();
        if !apply(mc, &mut with) {
            return; // nothing for it to do
        }
        // By default, used only when the spell can't be paid for without it.
        let src = first.1;
        let chars = g.obj(spell).chars.clone();
        let needed = !g.can_pay_cost_optimistic(p, cost, Some(spell), &chars);
        if !g.ask_yes_no(
            p,
            src,
            "For this spell, spend mana as though it were mana of any type to pay its mana cost (using up the permission)?",
            needed,
        ) {
            return;
        }
        g.player_effects.retain(|e| e.id != first.0);
        g.dirty = true;
        *cost = with;
        g.log(|_| format!("{p} uses a permission to spend mana as though any type"));
    }
}

inventory::submit! { KeywordRegistration(&Rules) }
