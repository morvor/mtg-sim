//! Restrictions on what can cause a player to sacrifice permanents (CR 701.21):
//! "Spells and abilities your opponents control can't cause you to sacrifice permanents."
//! (Sigarda, Host of Herons; Tajuru Preserver) and "Triggered abilities you control can't
//! cause you to sacrifice or exile creature tokens you control." (The Master, Multiplied).
//!
//! What causes a sacrifice ([`Cause`]) is the spell or ability whose effect has the
//! player sacrifice the permanent, including a cost it asks the player to pay as it
//! resolves ("... unless they sacrifice a creature"). The costs of casting a spell or
//! activating an ability ([`Ctx::cost_of`]), special actions, and game rules (the legend
//! rule, lethal damage) aren't spells or abilities making the player sacrifice anything
//! (Sigarda and Tajuru Preserver rulings).
//!
//! As such a spell or ability resolves, a sacrifice it would force doesn't happen, and the
//! player can't choose to sacrifice a permanent it gives them the option to sacrifice
//! (the permanents aren't among those they may choose).

use crate::ability::*;
use crate::eval::Ctx;
use crate::event_causes::Cause;
use crate::game::Game;
use crate::object::StackKind;
use crate::types::{ObjectId, PlayerId};

/// What causes a sacrifice (or exile) performed with `ctx`: the resolving spell or ability
/// (or the source of the ability performing it), or nothing when `ctx` is paying the costs
/// of casting a spell or activating an ability.
pub fn cause_of(ctx: &Ctx) -> Option<Cause> {
    ctx.cost_of.is_none().then(|| Cause::of(ctx))
}

/// What causes a sacrifice made to pay a cost with `ctx`: the resolving spell or ability
/// asking for it; nothing for the costs of casting spells or activating abilities, special
/// actions and other costs no spell or ability on the stack asks for.
pub fn cost_cause(ctx: &Ctx) -> Option<Cause> {
    (ctx.cost_of.is_none() && ctx.stack_obj.is_some()).then(|| Cause::of(ctx))
}

/// Whether `cause` (a spell or ability, and its controller) is among those `by`
/// describes, relative to the player `you`.
fn cause_matches(g: &Game, by: SacrificeCauses, you: PlayerId, cause: &Cause) -> bool {
    let Some(ctl) = cause.by else {
        return false;
    };
    match by {
        SacrificeCauses::OpponentsSpellsAndAbilities => {
            cause.obj.is_some() && g.opponents(you).contains(&ctl)
        }
        SacrificeCauses::YourTriggeredAbilities => {
            ctl == you
                && cause.obj.is_some_and(|o| {
                    matches!(
                        g.obj(o).stack.as_deref().map(|s| &s.kind),
                        Some(StackKind::Triggered { .. })
                    )
                })
        }
    }
}

/// Whether an effect says `cause` can't make its player sacrifice (or, with `exile`,
/// exile) the permanent `obj`.
pub fn forbidden(g: &Game, obj: ObjectId, cause: &Cause, exile: bool) -> bool {
    g.statics.restrictions.iter().any(|(s, c, r)| match r {
        Restriction::CantCauseSacrifice {
            what,
            by,
            exile: also_exile,
        } => {
            (!exile || *also_exile)
                && g.obj(obj).controller == *c
                && cause_matches(g, *by, *c, cause)
                && g.matches(obj, what, &Ctx::new(Some(*s), *c))
        }
        _ => false,
    })
}

impl Game {
    /// Whether `obj` can't be sacrificed (CR 701.21), with `cause` the spell or ability
    /// that would cause it, if any: it can't be sacrificed at all, or not because of that.
    pub fn sacrifice_forbidden(&self, obj: ObjectId, cause: Option<&Cause>) -> bool {
        self.cant_be_sacrificed(obj) || cause.is_some_and(|c| forbidden(self, obj, c, false))
    }
}
