//! "Players can't pay life to cast spells or to activate abilities that aren't mana
//! abilities." (Karn's Sylex), "Players can't pay life or sacrifice nonland permanents to
//! cast spells or activate abilities." (Yasharn, Implacable Earth), "Players can't pay
//! life or sacrifice creatures to cast spells or activate abilities." (Angel of
//! Jubilation): [`Restriction::CantPayToCastOrActivate`].
//!
//! These stop players from taking those actions, whatever the words on the cards (Angel of
//! Jubilation ruling): a cost of casting a spell or activating an ability that would have
//! a player pay life (including life paid for Phyrexian mana, CR 107.4f) or sacrifice
//! such a permanent can't be paid that way (CR 118.3), so a spell or ability whose cost
//! requires it can't be cast or activated (a cost to "sacrifice an artifact" can still be
//! paid with a noncreature artifact under Angel of Jubilation). Paying 0 life is always
//! possible (CR 119.4b). Costs paid as a spell or ability resolves aren't costs of
//! casting or activating, so they still may be paid (CR 118.3; "Other things may still
//! cause players to pay life or sacrifice creatures, such as a resolving spell or
//! ability").
//!
//! What a payment is for travels with the context it's paid in ([`Ctx::cost_of`]) and the
//! mana payment's [`crate::mana::SpendContext::cost_of`].

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::{ObjectId, PlayerId};
use serde::{Deserialize, Serialize};

/// What the costs being paid are for (CR 601.2g–h, 602.2b, 605.3a).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostOf {
    /// Casting a spell.
    Spell,
    /// Activating an ability that isn't a mana ability.
    Ability,
    /// Activating a mana ability.
    ManaAbility,
}

/// The active restrictions on paying costs that apply to `p` paying the costs `of`.
fn restrictions<'a>(
    g: &'a Game,
    p: PlayerId,
    of: Option<CostOf>,
) -> impl Iterator<Item = (Ctx, bool, Option<&'a Filter>)> + 'a {
    g.statics
        .restrictions
        .iter()
        .filter_map(move |(s, c, r)| match r {
            Restriction::CantPayToCastOrActivate {
                who,
                life,
                sacrifice,
                mana_abilities,
            } => {
                let of = of?;
                if of == CostOf::ManaAbility && !mana_abilities {
                    return None;
                }
                let ctx = Ctx::new(Some(*s), *c);
                g.player_filter_matches(who, p, &ctx)
                    .then(|| (ctx, *life, sacrifice.as_ref()))
            }
            _ => None,
        })
}

/// Whether `p` can't pay life to pay the costs `of` (`None`: a cost a resolving spell or
/// ability asks for, which no such restriction affects).
pub fn forbids_life(g: &Game, p: PlayerId, of: Option<CostOf>) -> bool {
    restrictions(g, p, of).any(|(_, life, _)| life)
}

/// Whether `p` can't sacrifice the permanent `obj` to pay the costs `of`.
pub fn forbids_sacrifice(g: &Game, p: PlayerId, obj: ObjectId, of: Option<CostOf>) -> bool {
    restrictions(g, p, of).any(|(ctx, _, f)| f.is_some_and(|f| g.matches(obj, f, &ctx)))
}

impl Game {
    /// Whether the permanent `obj` may be sacrificed to pay a cost with `ctx`: it can be
    /// sacrificed, the spell or ability asking for the cost as it resolves may cause it
    /// (see `sacrifice_causes`), and paying costs of casting or activating that way isn't
    /// forbidden.
    pub fn may_sacrifice_for_cost(&self, obj: ObjectId, ctx: &Ctx) -> bool {
        let cause = super::sacrifice_causes::cost_cause(ctx);
        !self.sacrifice_forbidden(obj, cause.as_ref())
            && !forbids_sacrifice(self, self.obj(obj).controller, obj, ctx.cost_of)
    }

    /// Whether `p` may pay `n` life for a cost paid with `ctx` (CR 119.4, 119.4b).
    pub fn may_pay_life_for_cost(&self, p: PlayerId, n: u32, ctx: &Ctx) -> bool {
        self.can_pay_life(p, n) && (n == 0 || !forbids_life(self, p, ctx.cost_of))
    }
}
