//! Activated abilities' total costs (CR 602.2b, 601.2f, 118.7, 118.9) and the permissions
//! that relax when or how often abilities may be activated (CR 602.5d, 606.3, 302.6 with
//! 609.4).
//!
//! * [`CostChanges`] is the CR 601.2f ordering shared by spells and abilities: the total
//!   cost is the mana cost, activation cost or alternative cost plus every additional cost
//!   and cost increase, minus every cost reduction, so reductions are collected while the
//!   increases are added and applied after all of them.
//! * "This effect can't reduce the mana in that cost to less than one mana"
//!   ([`CostChange::ReduceGenericMinOne`]) reduces only generic mana, and only while the
//!   cost keeps at least one mana; it never adds mana to a cost.
//! * Cost changes for abilities that target something ("Abilities your opponents activate
//!   that target a Merfolk you control cost {2} more") look at the ability's targets,
//!   which are chosen before its total cost is determined (CR 601.2c, 601.2f). Before
//!   that (while checking whether it could be activated) a reduction is assumed to apply
//!   and an increase not to.
//! * Alternative activation costs ("you may pay {0} rather than pay the equip cost of the
//!   first equip ability you activate each turn") are announced as the ability is
//!   activated (CR 601.2b, 118.9a: at most one); the rest of the total cost applies to
//!   them (CR 118.9d). For a keyword ability, the alternative replaces the keyword's cost
//!   only: a cycling ability's "Discard this card" is still paid (CR 702.29a).
//! * [`ActivationPermission`]s: instant timing for abilities that would need sorcery
//!   timing (CR 602.5d, 606.3), loyalty abilities more than once each turn (CR 606.3), and
//!   {T}/{Q} abilities of creatures as though they had haste (CR 302.6, 602.5a, 609.4: it
//!   doesn't grant haste or let them attack).

use crate::ability::*;
use crate::cost_rules::Half;
use crate::eval::Ctx;
use crate::game::Game;
use crate::mana::{ManaCost, ManaSymbol};
use crate::types::*;

/// A cost reduction waiting for every increase to be applied (CR 601.2f).
#[derive(Clone, Debug)]
pub enum Reduction {
    /// {N} less: generic mana (and generic-payable hybrid symbols), then the generic mana
    /// of a waterbend cost (CR 118.7a).
    Generic(u32),
    /// {N} less, keeping at least one mana in the cost.
    GenericMinOne(u32),
    /// Colored mana less; any excess reduces generic mana (CR 118.7b–c).
    Colored(Color, u32),
    /// The given mana symbols less (CR 118.7a–g); `bool`: colored mana only.
    Mana(ManaCost, bool),
}

/// CR 601.2f: increases apply as they're found, reductions after all of them.
#[derive(Clone, Debug, Default)]
pub struct CostChanges {
    pub reductions: Vec<Reduction>,
}

impl CostChanges {
    /// Applies `change` (its values evaluated in `ctx`) to `cost`: an increase or an
    /// additional cost at once, a reduction once [`CostChanges::apply`] is called. Returns
    /// false for changes that are announced choices instead (alternative and optional
    /// costs), which this leaves alone.
    pub fn add(&mut self, g: &Game, cost: &mut Cost, change: &CostChange, ctx: &Ctx) -> bool {
        let n = |v: &Value| g.eval_value(v, ctx).max(0) as u32;
        match change {
            CostChange::IncreaseGeneric(v) => {
                crate::casting::add_cost(cost, &Cost::mana(ManaCost::generic(n(v))))
            }
            CostChange::IncreaseMana(m) => crate::casting::add_cost(cost, &Cost::mana(m.clone())),
            CostChange::AdditionalCost(c) => crate::casting::add_cost(cost, c),
            CostChange::ReduceGeneric(v) => self.reductions.push(Reduction::Generic(n(v))),
            CostChange::ReduceGenericMinOne(v) => {
                self.reductions.push(Reduction::GenericMinOne(n(v)))
            }
            CostChange::ReduceColored(c, v) => self.reductions.push(Reduction::Colored(*c, n(v))),
            CostChange::ReduceMana { mana, colored_only } => self
                .reductions
                .push(Reduction::Mana(mana.clone(), *colored_only)),
            CostChange::AlternativeCost(_)
            | CostChange::FlashForAdditionalCost(_)
            | CostChange::OptionalAdditionalCost { .. }
            | CostChange::AdditionalCostChoice(_) => return false,
            // How the cost is paid, not what it is (see `cost_rules::spend_any_type`).
            CostChange::SpendAnyType => return false,
        }
        true
    }

    /// Applies the collected reductions to `cost` (CR 118.7): reductions by an amount of
    /// mana first, in the order found, then reductions by mana symbols, where `half`
    /// chooses the half of a hybrid symbol each one uses (CR 118.7e; its index counts the
    /// hybrid symbols reduced so far).
    pub fn apply(
        self,
        cost: &mut Cost,
        mut half: impl FnMut(usize, &ManaCost, ManaSymbol) -> Half,
    ) {
        let mut by_symbols = Vec::new();
        for r in self.reductions {
            match r {
                Reduction::Generic(n) => {
                    let left = match cost.mana.as_mut() {
                        Some(m) => crate::cost_rules::reduce_generic_and_hybrid(m, n),
                        None => n,
                    };
                    // CR 601.2f: then the generic mana of a waterbend cost, part of the
                    // total cost too.
                    crate::kwa::bending::reduce_waterbend_generic(cost, left);
                }
                Reduction::GenericMinOne(n) => reduce_keeping_one_mana(cost, n),
                Reduction::Colored(c, n) => {
                    if let Some(m) = cost.mana.as_mut() {
                        for _ in 0..n {
                            crate::cost_rules::reduce_one_colored(m, c, false);
                        }
                    }
                }
                Reduction::Mana(m, colored_only) => by_symbols.push((m, colored_only)),
            }
        }
        let mut hybrid = 0;
        for (by, colored_only) in by_symbols {
            if let Some(m) = cost.mana.as_mut() {
                crate::cost_rules::reduce_by(m, &by, colored_only, |_, cur, s| {
                    let h = half(hybrid, cur, s);
                    hybrid += 1;
                    h
                });
            }
        }
    }
}

/// The mana `cost` asks for: its mana component (with X already announced) and the
/// generic mana of a waterbend cost.
fn mana_amount(cost: &Cost) -> u32 {
    cost.mana.as_ref().map_or(0, |m| m.mana_value()) + waterbend_generic(cost)
}

fn waterbend_generic(cost: &Cost) -> u32 {
    cost.parts
        .iter()
        .map(|p| match p {
            CostPart::Effect(e) => match &**e {
                Effect::KeywordAction {
                    action: KeywordAction::Waterbend,
                    n: Value::Const(k),
                    ..
                } => (*k).max(0) as u32,
                _ => 0,
            },
            _ => 0,
        })
        .sum()
}

/// "Costs {N} less ... This effect can't reduce the mana in that cost to less than one
/// mana": generic mana only (never colored or snow mana), and only as far as the cost
/// keeps one mana; a cost with no mana is left alone.
pub fn reduce_keeping_one_mana(cost: &mut Cost, n: u32) {
    let total = mana_amount(cost);
    let n = n.min(total.saturating_sub(1));
    if n == 0 {
        return;
    }
    let left = match cost.mana.as_mut() {
        Some(m) => {
            let before = m.generic_amount();
            m.reduce_generic(n);
            n - (before - m.generic_amount())
        }
        None => n,
    };
    crate::kwa::bending::reduce_waterbend_generic(cost, left);
}

/// Whether an activated ability is of the kind `class` names.
pub fn class_matches(class: AbilityClass, a: &Ability, act: &ActivatedAbility) -> bool {
    match class {
        AbilityClass::Any => true,
        AbilityClass::Loyalty => act.is_loyalty,
        AbilityClass::Keyword(k) => crate::keyword_impls::ability_from_keyword(a) == Some(k),
    }
}

/// Whether `p` activated an ability of the kind `class` describes (of a source matching
/// `sources`, relative to `ctx`) earlier this turn.
fn activated_one_this_turn(g: &Game, p: PlayerId, scope: &AbilityScope, ctx: &Ctx) -> bool {
    g.history.activated.iter().any(|(who, src, uid)| {
        *who == p
            && g.obj(*src).chars.abilities.iter().any(|a| {
                a.uid == *uid
                    && matches!(&a.kind, AbilityKind::Activated(act)
                        if class_matches(scope.class, a, act)
                            && (!scope.nonmana || !act.is_mana_ability))
            })
            && (matches!(scope.sources, Filter::Any) || g.matches(*src, &scope.sources, ctx))
    })
}

/// Whether the ability `a` of `src` that `p` activates (or would activate) is one of the
/// abilities `scope` describes, relative to `ctx` (the effect's source and controller).
/// `stack`: the ability on the stack, once its targets are chosen; before that, a
/// requirement on its targets is assumed met when `optimistic` and not met otherwise.
#[allow(clippy::too_many_arguments)]
pub fn scope_matches(
    g: &Game,
    scope: &AbilityScope,
    p: PlayerId,
    src: ObjectId,
    a: &Ability,
    act: &ActivatedAbility,
    stack: Option<ObjectId>,
    optimistic: bool,
    ctx: &Ctx,
) -> bool {
    if !class_matches(scope.class, a, act) || (scope.nonmana && act.is_mana_ability) {
        return false;
    }
    if !g.matches(src, &scope.sources, ctx) {
        return false;
    }
    if let Some(t) = &scope.targeting {
        let met = match stack {
            Some(id) => g.matches(id, &Filter::Targets(Box::new(t.clone())), ctx),
            None => optimistic,
        };
        if !met {
            return false;
        }
    }
    !(scope.first_each_turn && activated_one_this_turn(g, p, scope, ctx))
}

fn is_reduction(c: &CostChange) -> bool {
    matches!(
        c,
        CostChange::ReduceGeneric(_)
            | CostChange::ReduceGenericMinOne(_)
            | CostChange::ReduceColored(..)
            | CostChange::ReduceMana { .. }
    )
}

/// Whether the cost modifier `cm` (of a static or effect from `ctx.source`) applies to
/// the ability `a` of `src` that `p` activates. `stack`: as for [`scope_matches`].
#[allow(clippy::too_many_arguments)]
pub fn modifier_applies(
    g: &Game,
    cm: &CostModifier,
    p: PlayerId,
    src: ObjectId,
    a: &Ability,
    act: &ActivatedAbility,
    stack: Option<ObjectId>,
    ctx: &Ctx,
) -> bool {
    if !g.player_rel_matches(cm.who, p, ctx) {
        return false;
    }
    match &cm.applies_to {
        CostTarget::Abilities(f) => g.matches(src, f, ctx),
        CostTarget::Keyword(k) => crate::keyword_impls::ability_from_keyword(a) == Some(*k),
        CostTarget::KeywordAbilitiesOf(k, f) => {
            crate::keyword_impls::ability_from_keyword(a) == Some(*k) && g.matches(src, f, ctx)
        }
        // CR 606.4: the cost of a loyalty ability may be modified by other effects.
        CostTarget::LoyaltyAbilities(f) => act.is_loyalty && g.matches(src, f, ctx),
        CostTarget::ActivatedAbilities(scope) => scope_matches(
            g,
            scope,
            p,
            src,
            a,
            act,
            stack,
            is_reduction(&cm.change),
            ctx,
        ),
        CostTarget::Spells(_) | CostTarget::ThisSpell => false,
    }
}

/// An alternative cost `p` may pay rather than the activation cost of the ability `a` of
/// `src` (CR 118.9): (the effect's source, the cost).
pub fn alternative_costs(
    g: &Game,
    p: PlayerId,
    src: ObjectId,
    a: &Ability,
    act: &ActivatedAbility,
) -> Vec<(ObjectId, Cost)> {
    let mut out = Vec::new();
    for (s, ctl, cm) in &g.statics.cost_modifiers {
        let CostChange::AlternativeCost(c) = &cm.change else {
            continue;
        };
        let ctx = Ctx::new(Some(*s), *ctl);
        if modifier_applies(g, cm, p, src, a, act, None, &ctx) {
            out.push((*s, c.clone()));
        }
    }
    out
}

/// The activation cost of `act` with `alt` paid rather than its cost: for an ability a
/// keyword defines ("Cycling [cost]" is "[Cost], Discard this card: ...", CR 702.29a),
/// the parts the keyword adds to its cost are still paid.
pub fn with_alternative(
    g: &Game,
    src: ObjectId,
    a: &Ability,
    act: &ActivatedAbility,
    alt: &Cost,
) -> Cost {
    // (Cost parts are compared by their description.)
    let key = |c: &CostPart| format!("{c:?}");
    let mut cost = alt.clone();
    let kw_cost = crate::keyword_impls::ability_from_keyword(a).and_then(|k| {
        g.obj(src)
            .chars
            .keywords()
            .find(|kw| {
                kw.kind == k
                    && kw.cost.as_ref().is_some_and(|c| {
                        c.mana == act.cost.mana
                            && c.parts
                                .iter()
                                .all(|x| act.cost.parts.iter().any(|y| key(x) == key(y)))
                    })
            })
            .and_then(|kw| kw.cost.clone())
    });
    if let Some(kc) = kw_cost {
        let mut own: Vec<String> = kc.parts.iter().map(key).collect();
        for part in &act.cost.parts {
            match own.iter().position(|x| *x == key(part)) {
                Some(i) => {
                    own.remove(i);
                }
                None => cost.parts.push(part.clone()),
            }
        }
    }
    cost
}

/// The activation permissions that apply to `p` activating the ability `a` of `src`.
fn permissions<'g>(
    g: &'g Game,
    p: PlayerId,
    src: ObjectId,
    a: &'g Ability,
    act: &'g ActivatedAbility,
) -> impl Iterator<Item = &'g ActivationPermission> + 'g {
    g.statics.other.iter().filter_map(move |(s, ctl, e)| {
        let StaticEffect::ActivationPermission(perm) = e else {
            return None;
        };
        let ctx = Ctx::new(Some(*s), *ctl);
        (*ctl == p && scope_matches(g, &perm.scope, p, src, a, act, None, false, &ctx))
            .then_some(perm)
    })
}

/// Whether `p` may activate the ability any time they could cast an instant although it
/// would need sorcery timing (CR 602.5d, 606.3).
pub fn instant_timing_allowed(
    g: &Game,
    p: PlayerId,
    src: ObjectId,
    a: &Ability,
    act: &ActivatedAbility,
) -> bool {
    permissions(g, p, src, a, act).any(|x| x.instant_timing)
}

/// How many times each turn `p` may activate loyalty abilities of `src` (CR 606.3: once,
/// unless an effect says otherwise; several such effects don't add up).
pub fn loyalty_activations_per_turn(
    g: &Game,
    p: PlayerId,
    src: ObjectId,
    a: &Ability,
    act: &ActivatedAbility,
) -> u32 {
    permissions(g, p, src, a, act)
        .filter_map(|x| x.loyalty_per_turn)
        .fold(1, u32::max)
}

/// Whether `p` may activate {T}/{Q} abilities of the creature `src` as though it had
/// haste (CR 302.6, 609.4).
pub fn as_though_haste(g: &Game, p: PlayerId, src: ObjectId) -> bool {
    g.statics.other.iter().any(|(s, ctl, e)| {
        let StaticEffect::ActivationPermission(perm) = e else {
            return false;
        };
        *ctl == p
            && perm.as_though_haste
            && g.matches(src, &perm.scope.sources, &Ctx::new(Some(*s), *ctl))
    })
}

/// "Activate only as an instant" (CR 602.5e): the player has priority and isn't in the
/// middle of casting a spell, activating an ability or paying a cost.
pub fn as_instant_ok(g: &Game, p: PlayerId) -> bool {
    g.has_priority(p) && g.special.casting == 0 && g.mana_hint.is_none()
}
