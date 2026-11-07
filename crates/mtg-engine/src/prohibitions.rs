//! Prohibitions on what can happen to objects ([`Restriction::CantBe`]): "can't become
//! untapped" (no untapping at all, CR 701.26b), "can't phase in" (CR 702.26), "can't be
//! equipped" (CR 301.5c), "can't be enchanted by other Auras" (CR 303.4), "can't become
//! suspected" (CR 701.60); "can't be turned face up" is `rule_statics::face_up`. Each
//! check is a hook where the engine would otherwise perform that action.

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::ObjectId;

/// The restrictions of this kind that apply to `id`: (source, controller).
fn applying(g: &Game, id: ObjectId, action: ObjectAction) -> Vec<Option<ObjectId>> {
    g.all_restrictions()
        .into_iter()
        .filter(|(s, c, r, locked)| match r {
            Restriction::CantBe { what, action: a } if *a == action => {
                g.restriction_applies(id, what, &Ctx::new(*s, *c), locked)
            }
            _ => false,
        })
        .map(|(s, ..)| s)
        .collect()
}

/// Whether `action` can't happen to `id`.
pub fn object_cant(g: &Game, id: ObjectId, action: ObjectAction) -> bool {
    if !g.is_live(id) {
        return false;
    }
    !applying(g, id, action).is_empty()
}

/// Whether the Aura `aura` can't enchant `target` because of "can't be enchanted by
/// other Auras" (an Aura with that ability can still enchant it).
pub fn aura_prohibited(g: &Game, aura: ObjectId, target: ObjectId) -> bool {
    if !g.is_live(target) {
        return false;
    }
    applying(g, target, ObjectAction::EnchantedByOtherAuras)
        .iter()
        .any(|s| *s != Some(aura))
}

/// Whether "can't be enchanted" (CR 303.4) stops an Aura from being attached to `target`
/// ([`ObjectAction::Enchanted`]; `newly`: also [`ObjectAction::NewlyEnchanted`], "This
/// effect doesn't remove Auras already attached", which stops only new attachments).
pub fn cant_be_enchanted(g: &Game, target: ObjectId, newly: bool) -> bool {
    object_cant(g, target, ObjectAction::Enchanted)
        || (newly && object_cant(g, target, ObjectAction::NewlyEnchanted))
}

/// Whether "other players can't gain control of [it]" stops `player` from gaining
/// control of `id` from `from`, its controller so far: a restriction controlled by `from`
/// applies to it (evaluated as `from` controls it).
pub fn control_gain_prohibited(
    g: &Game,
    id: ObjectId,
    from: crate::types::PlayerId,
    player: crate::types::PlayerId,
) -> bool {
    if player == from || !g.is_live(id) {
        return false;
    }
    g.all_restrictions()
        .into_iter()
        .any(|(s, c, r, locked)| match r {
            Restriction::CantBe {
                what,
                action: ObjectAction::ControlGainedByOthers,
            } => {
                c == from
                    && c != player
                    && g.restriction_applies(id, &what, &Ctx::new(s, c), &locked)
            }
            _ => false,
        })
}

/// Whether spells and abilities `by` controls may target `id` as though it didn't have
/// `keyword` ([`Restriction::TargetableAsThoughNo`]).
pub fn targetable_as_though_no(
    g: &Game,
    id: ObjectId,
    by: crate::types::PlayerId,
    keyword: crate::keywords::KeywordKind,
) -> bool {
    g.all_restrictions()
        .into_iter()
        .any(|(s, c, r, locked)| match r {
            Restriction::TargetableAsThoughNo {
                what,
                by: who,
                keyword: k,
            } if k == keyword => {
                let ctx = Ctx::new(s, c);
                g.player_filter_matches(&who, by, &ctx)
                    && g.restriction_applies(id, &what, &ctx, &locked)
            }
            _ => false,
        })
}
