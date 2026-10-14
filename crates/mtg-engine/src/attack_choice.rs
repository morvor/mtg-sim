//! Effects that give another player the choice of attackers: "You choose which creatures
//! attack this turn" (Master Warcraft).
//!
//! The active player normally declares attackers (CR 508.1). With such an effect, its
//! controller chooses the complete group of creatures that attack, following the normal
//! rules for attacking (CR 508.1a–d), even on another player's turn; then, for each of
//! those creatures, the active player chooses what it attacks (CR 508.1b; Master Warcraft
//! rulings). The active player still pays the costs to attack (CR 508.1h) and may decline
//! to pay costs another player chose for them, in which case a new group is proposed. If
//! several such effects apply, the one that began last wins.

use crate::ability::Restriction;
use crate::combat::{attack_declaration_legal_with, total_attack_cost, AttackRequirement};
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::game::Game;
use crate::types::*;

/// `Restriction::Custom` name of a rule-modifying effect: its controller chooses which
/// creatures attack.
pub const CHOOSES_ATTACKERS: &str = "combat: controller chooses which creatures attack";

/// The player who chooses which creatures attack instead of the active player, if an
/// effect says so: the controller of the latest such effect whose controller is still in
/// the game.
pub fn attack_decider(g: &Game) -> Option<PlayerId> {
    g.all_restrictions()
        .into_iter()
        .rev()
        .find_map(|(_, controller, r, _)| match &r {
            Restriction::Custom(n) if n == CHOOSES_ATTACKERS => {
                g.player(controller).in_game().then_some(controller)
            }
            _ => None,
        })
}

/// A legal declaration of attackers made with another player (`chooser`) choosing the
/// attacking creatures and the active player `ap` choosing what each attacks; `fallback`
/// (a legal declaration needing no payment) if no acceptable one is proposed.
pub fn declaration_chosen_by(
    g: &mut Game,
    chooser: PlayerId,
    ap: PlayerId,
    options: &[(ObjectId, Vec<Entity>)],
    reqs: &[AttackRequirement],
    max: u32,
    fallback: &[(ObjectId, Entity)],
) -> Vec<(ObjectId, Entity)> {
    for _ in 0..3 {
        let proposed = match g.ask(
            chooser,
            Decision::DeclareAttackers {
                options: options.to_vec(),
            },
        ) {
            Answer::Attackers(v)
                if attack_declaration_legal_with(g, options, &v, reqs, max)
                    && g.can_pay_cost(ap, &total_attack_cost(g, &v), None, &Ctx::new(None, ap)) =>
            {
                v
            }
            // CR 508.1: an illegal declaration is undone; the engine declares a legal one.
            _ => return fallback.to_vec(),
        };
        let decl = choose_targets(g, ap, options, reqs, max, proposed);
        let cost = total_attack_cost(g, &decl);
        if cost.is_free() {
            return decl;
        }
        let prompt = format!(
            "Pay {} for the attacks another player chose?",
            crate::resolve::describe_cost(&cost)
        );
        if g.ask_yes_no(ap, None, &prompt, true) {
            return decl;
        }
    }
    fallback.to_vec()
}

/// The active player chooses what each creature of the group `proposed` attacks (with
/// `proposed`'s choices as the default): a declaration with exactly those creatures.
fn choose_targets(
    g: &mut Game,
    ap: PlayerId,
    options: &[(ObjectId, Vec<Entity>)],
    reqs: &[AttackRequirement],
    max: u32,
    proposed: Vec<(ObjectId, Entity)>,
) -> Vec<(ObjectId, Entity)> {
    let group: Vec<ObjectId> = proposed.iter().map(|(c, _)| *c).collect();
    let sub: Vec<(ObjectId, Vec<Entity>)> = options
        .iter()
        .filter(|(c, _)| group.contains(c))
        .cloned()
        .collect();
    if sub.iter().all(|(_, ts)| ts.len() <= 1) {
        return proposed;
    }
    match g.ask(ap, Decision::DeclareAttackers { options: sub }) {
        Answer::Attackers(v)
            if v.len() == group.len()
                && group.iter().all(|c| v.iter().any(|(a, _)| a == c))
                && attack_declaration_legal_with(g, options, &v, reqs, max)
                && g.can_pay_cost(ap, &total_attack_cost(g, &v), None, &Ctx::new(None, ap)) =>
        {
            v
        }
        _ => proposed,
    }
}
