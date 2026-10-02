//! Rulings batch P161 — "you may put a [creature/land] card from your hand onto the
//! battlefield": it's optional (CR 603.5, 608.2d), it isn't casting or playing (CR 305.4,
//! 601.1), costs and X aren't paid (CR 107.3b), creatures put onto the battlefield
//! attacking were never declared as attackers (CR 508.4), and last known information
//! (CR 608.2h).

use crate::r_p160_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The candidates of the last "choose cards" decision asked.
fn last_choice(t: &TestGame) -> Option<(Vec<Entity>, u32)> {
    t.asked().iter().rev().find_map(|(_, d)| match d {
        Decision::ChooseEntities {
            candidates, min, ..
        } => Some((candidates.clone(), *min)),
        _ => None,
    })
}

fn attackers(t: &TestGame) -> Vec<(ObjectId, Option<Entity>)> {
    t.g.combat
        .as_ref()
        .map(|c| c.attackers.iter().map(|a| (a.id, a.target)).collect())
        .unwrap_or_default()
}

#[test]
fn putting_the_card_onto_the_battlefield_is_optional() {
    cr!("608.2d", "603.5");
    ruling!(
        "Elvish Piper",
        "Putting the card onto the battlefield is optional. When the ability resolves, you can choose not to."
    );
    supported("Elvish Piper");
    let mut t = TestGame::new(2);
    let piper = t.battlefield(P0, "Elvish Piper");
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    activate_resolve(&mut t, P0, piper, 0, &[]);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Hand(P0));
    // And it can choose to.
    let mut t = TestGame::new(2);
    let piper = t.battlefield(P0, "Elvish Piper");
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    activate_resolve(&mut t, P0, piper, 0, &[]);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn dragon_arch_multicolored_means_more_than_one_color() {
    cr!("105.4", "202.2c");
    ruling!(
        "Dragon Arch",
        "A multicolored card has more than one color in its mana cost."
    );
    supported("Dragon Arch");
    supported("Azorius Guildmage");
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Dragon Arch");
    let mage = t.hand(P0, "Azorius Guildmage");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Serra Angel");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(mage)]);
    activate_resolve(&mut t, P0, arch, 0, &[]);
    let (candidates, _) = last_choice(&t).expect("a choice of cards");
    assert_eq!(candidates, vec![Entity::Object(mage)]);
    assert_eq!(t.named_on_battlefield("Azorius Guildmage").len(), 1);
}

#[test]
fn champion_of_rhonas_creature_isnt_attacking() {
    cr!("508.1", "508.1g");
    ruling!(
        "Champion of Rhonas",
        "All attackers are chosen at once. You can’t attack with Champion of Rhonas, put a creature card onto the battlefield, and then attack with that creature."
    );
    supported("Champion of Rhonas");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Champion of Rhonas");
    let giant = t.hand(P0, "Hill Giant");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(champ, Entity::Player(P1))]),
    );
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert!(t.obj(champ).tapped);
    let attacking: Vec<ObjectId> = attackers(&t).into_iter().map(|(a, _)| a).collect();
    assert_eq!(attacking, vec![champ]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn quicksilver_amulet_pays_no_costs_and_x_is_zero() {
    cr!("107.3b", "601.2b", "118.8");
    ruling!(
        "Quicksilver Amulet",
        "Any 'X' in the creature's cost is treated as zero."
    );
    ruling!(
        "Quicksilver Amulet",
        "You don't pay any costs of that creature card, including additional costs."
    );
    supported("Quicksilver Amulet");
    supported("Endless One");
    supported("Demon of Catastrophes");
    // Endless One ("enters with X +1/+1 counters") enters with none and dies as a 0/0.
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Quicksilver Amulet");
    let one = t.hand(P0, "Endless One");
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(one)]);
    activate_resolve(&mut t, P0, amulet, 0, &[]);
    assert!(t.in_graveyard(P0, "Endless One"));
    // Demon of Catastrophes ("As an additional cost to cast this spell, sacrifice a
    // creature"): nothing is sacrificed.
    let mut t = TestGame::new(2);
    let amulet = t.battlefield(P0, "Quicksilver Amulet");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let demon = t.hand(P0, "Demon of Catastrophes");
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(demon)]);
    activate_resolve(&mut t, P0, amulet, 0, &[]);
    assert_eq!(t.named_on_battlefield("Demon of Catastrophes").len(), 1);
    assert!(t.on_battlefield(bears));
}

/// P0 attacks with Preeminent Captain; its trigger puts `soldier` onto the battlefield
/// attacking `target`.
fn captain_attack(t: &mut TestGame, captain: ObjectId, soldier: ObjectId, target: Entity) {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(captain, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(soldier)]);
    t.answer_choose(P0, &[target]);
    t.resolve();
}

#[test]
fn preeminent_captain_soldier_wasnt_declared_as_an_attacker() {
    cr!("508.4", "508.3a");
    ruling!(
        "Preeminent Captain",
        "Any abilities of the Soldier creature card that trigger “Whenever [this creature] attacks” won’t trigger because the creature was never declared as an attacking creature."
    );
    supported("Preeminent Captain");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Preeminent Captain");
    let second = t.hand(P0, "Preeminent Captain");
    captain_attack(&mut t, captain, second, Entity::Player(P1));
    // The second Captain is attacking, but its own attack trigger didn't trigger.
    let second = t
        .named_on_battlefield("Preeminent Captain")
        .into_iter()
        .find(|id| *id != captain)
        .expect("second Captain");
    assert!(attackers(&t).iter().any(|(a, _)| *a == second));
    assert!(t.obj_now(second).tapped);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn preeminent_captain_you_choose_what_the_soldier_attacks() {
    cr!("508.4", "508.4a");
    ruling!(
        "Preeminent Captain",
        "You choose what the Soldier you put onto the battlefield is attacking. It doesn’t have to attack the same player or planeswalker as Preeminent Captain."
    );
    supported("Preeminent Captain");
    supported("Elite Vanguard");
    supported("Oko, Thief of Crowns");
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Preeminent Captain");
    let oko = t.battlefield(P1, "Oko, Thief of Crowns");
    let soldier = t.hand(P0, "Elite Vanguard");
    captain_attack(&mut t, captain, soldier, Entity::Object(oko));
    let soldier = t.named_on_battlefield("Elite Vanguard")[0];
    let att = attackers(&t);
    assert!(att.contains(&(captain, Some(Entity::Player(P1)))));
    assert!(att.contains(&(soldier, Some(Entity::Object(oko)))), "{att:?}");
}

#[test]
fn aether_vial_uses_last_known_charge_counters() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Aether Vial",
        "If Aether Vial leaves the battlefield while its second ability is on the stack, use its last known number of charge counters to determine what you may put from your hand onto the battlefield."
    );
    supported("Aether Vial");
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Aether Vial");
    t.g.add_counters(Entity::Object(vial), counters::CHARGE, 2, None);
    t.g.flush_events();
    let bears = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    t.activate(P0, vial, 0, &[]).unwrap();
    t.g.destroy(vial, None);
    t.settle();
    assert!(!t.on_battlefield(vial));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    let (candidates, _) = last_choice(&t).expect("a choice");
    assert_eq!(candidates, vec![Entity::Object(bears)]);
}

#[test]
fn arboreal_grazer_putting_a_land_isnt_playing_one() {
    cr!("305.4", "305.2");
    ruling!(
        "Arboreal Grazer",
        "Arboreal Grazer’s effect doesn’t count as playing a land. It can put a land card onto the battlefield even if you’ve already played your land for the turn."
    );
    supported("Arboreal Grazer");
    // After a land play.
    let mut t = TestGame::new(2);
    let first = t.hand(P0, "Forest");
    t.play_land(P0, first).unwrap();
    let second = t.hand(P0, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(second)]);
    t.enter(P0, "Arboreal Grazer");
    t.resolve_all();
    assert!(t.on_battlefield(second));
    assert!(t.obj_now(second).tapped);
    // Before a land play: the land play is still available.
    let mut t = TestGame::new(2);
    let first = t.hand(P0, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(first)]);
    t.enter(P0, "Arboreal Grazer");
    t.resolve_all();
    assert!(t.on_battlefield(first));
    let second = t.hand(P0, "Forest");
    t.play_land(P0, second).unwrap();
    assert!(t.on_battlefield(second));
}
