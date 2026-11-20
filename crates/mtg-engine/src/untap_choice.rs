//! "You may choose not to untap ~ during your untap step." (CR 502.3: the active player
//! determines which permanents they control will untap) and the durations that go with
//! it, "for as long as ~ remains tapped" (CR 611.2b). Whether such a duration ended while
//! the ability creating the effect was on the stack: `duration_windows`.

use crate::ability::{Condition, Duration, Restriction};
use crate::eval::Ctx;
use crate::game::Game;
use crate::object::Zone;
use crate::types::{ObjectId, PlayerId};

/// `Restriction::Custom` of a static ability: its controller may choose not to untap the
/// source during their untap step.
pub const MAY_CHOOSE_NOT_TO_UNTAP: &str = "untap:may_choose_not_to_untap";

/// `Condition::Custom` for "for as long as ~ remains tapped": the effect's source is still
/// the same tapped permanent on the battlefield. A permanent that left the battlefield is a
/// new object (CR 400.7), and one that phased out can't be seen (CR 702.26f).
pub const SOURCE_REMAINS_TAPPED: &str = "untap:source_remains_tapped";
/// "for as long as you control ~ and ~ remains tapped".
pub const CONTROL_AND_SOURCE_REMAINS_TAPPED: &str = "untap:control_and_source_remains_tapped";

/// The duration "for as long as ~ remains tapped" (`control`: "for as long as you control
/// ~ and ~ remains tapped").
pub fn remains_tapped(control: bool) -> Duration {
    let name = if control {
        CONTROL_AND_SOURCE_REMAINS_TAPPED
    } else {
        SOURCE_REMAINS_TAPPED
    };
    Duration::WhileCondition(Condition::Custom(name.into()))
}

/// "For as long as ~ remains tapped" (and "for as long as you control ~ and ~ remains
/// tapped"): the source is still that tapped permanent (under the controller's control).
/// As the ability creating the effect resolves (`ctx` is its resolution), it must also have
/// stayed so since the ability became activated or triggered, even if it's tapped again
/// (CR 611.2b; Seasinger, Preacher, Rubinia Soulsinger and Entrancing Lyre rulings): see
/// `duration_windows`. Once an effect exists (its condition is checked without a resolving
/// ability), only an untap after that ends it, not one before another ability of the source
/// was activated.
pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    let control = match name {
        SOURCE_REMAINS_TAPPED => false,
        CONTROL_AND_SOURCE_REMAINS_TAPPED => true,
        _ => return None,
    };
    Some(ctx.source.is_some_and(|s| {
        let o = g.obj(s);
        g.is_live(s)
            && o.zone == Zone::Battlefield
            && !o.phased_out
            && o.tapped
            && (!control || o.controller == ctx.controller)
            && !crate::duration_windows::tapped_source_changed(g, ctx, control)
    }))
}

/// Whether a "remains tapped" effect from `src` is currently in force: the sensible
/// default is to keep such a permanent tapped.
fn has_remains_tapped_effect(g: &Game, src: ObjectId) -> bool {
    let tracks = |d: &Duration, s: Option<ObjectId>| {
        s == Some(src)
            && matches!(d, Duration::WhileCondition(Condition::Custom(n))
                if n.as_str() == SOURCE_REMAINS_TAPPED
                    || n.as_str() == CONTROL_AND_SOURCE_REMAINS_TAPPED)
    };
    g.effects.iter().any(|e| tracks(&e.duration, e.source))
        || g.rule_effects.iter().any(|e| tracks(&e.duration, e.source))
        || g.player_effects
            .iter()
            .any(|e| tracks(&e.duration, e.source))
}

impl Game {
    /// CR 502.3: for each permanent about to untap whose own static ability lets its
    /// controller choose not to untap it, the active player chooses.
    pub(crate) fn choose_untaps(&mut self, active: PlayerId, to_untap: &mut Vec<ObjectId>) {
        let optional: Vec<ObjectId> = to_untap
            .iter()
            .copied()
            .filter(|id| {
                self.statics.restrictions.iter().any(|(s, c, r)| {
                    s == id
                        && *c == active
                        && matches!(r, Restriction::Custom(n) if n.as_str() == MAY_CHOOSE_NOT_TO_UNTAP)
                })
            })
            .collect();
        for id in optional {
            let keep_default = has_remains_tapped_effect(self, id);
            let name = self.obj(id).chars.name.clone();
            let untap = self.ask_yes_no(
                active,
                Some(id),
                &format!("Untap {name} during your untap step?"),
                !keep_default,
            );
            if !untap {
                to_untap.retain(|o| *o != id);
            }
        }
    }

    /// "Untap [permanents you control] during each other player's untap step": those
    /// permanents untap along with the active player's, with no choice, and effects that
    /// apply during their controller's own untap step don't stop them (CR 502.3; Seedborn
    /// Muse and Murkfiend Liege rulings). Several such abilities untap a permanent once.
    pub(crate) fn untap_during_others_untap_steps(&mut self, to_untap: &mut Vec<ObjectId>) {
        let actives = self.active_players();
        let extra: Vec<ObjectId> = self
            .permanents()
            .filter(|o| o.tapped && !actives.contains(&o.controller))
            .map(|o| (o.id, o.controller))
            .filter(|(id, ctl)| {
                self.statics.restrictions.iter().any(|(s, c, r)| {
                    matches!(r, Restriction::UntapDuringOthersUntapSteps(f)
                        if c == ctl && self.matches(*id, f, &Ctx::new(Some(*s), *c)))
                })
            })
            .map(|(id, _)| id)
            .chain(crate::rule_statics::opponents_steps::extra_untaps(self))
            .collect();
        for id in extra {
            if !to_untap.contains(&id) {
                to_untap.push(id);
            }
        }
    }
}
