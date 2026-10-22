//! Choices of objects in several zones: "a card from their hand or a permanent they
//! control", "X permanents they control and/or cards from their hand", "a permanent you
//! control or a card from your hand or graveyard" (CR 101.4, 608.2e).

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn lichs_mastery_exiles_one_object_per_life_from_any_mix_of_zones() {
    cr!("119.3");
    ruling!(
        "Lich's Mastery",
        "You don’t have to exile all the cards from one place."
    );
    supported("Lich's Mastery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lich's Mastery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let h = t.hand(P0, "Lightning Bolt");
    let g = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Forest");
    t.answer_choose(
        P0,
        &[Entity::Object(bears), Entity::Object(h), Entity::Object(g)],
    );
    t.g.lose_life(P0, 3);
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn descent_into_madness_each_player_exiles_permanents_and_or_hand_cards() {
    cr!("101.4", "608.2e");
    ruling!(
        "Descent into Madness",
        "For each despair counter on Descent into Madness, you’ll exile a permanent you control or exile a card from your hand, not both."
    );
    supported("Descent into Madness");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Descent into Madness");
    t.g.add_counters(Entity::Object(d), "despair", 1, None);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let my_card = t.hand(P0, "Lightning Bolt");
    let theirs = t.battlefield(P1, "Hill Giant");
    let their_card = t.hand(P1, "Shock");
    t.answer_choose(P0, &[Entity::Object(mine), Entity::Object(my_card)]);
    t.answer_choose(P1, &[Entity::Object(theirs), Entity::Object(their_card)]);
    t.advance_to(P0, Step::Upkeep);
    // The draw step is later; act before it.
    t.resolve_all();
    assert_eq!(t.counters(d, "despair"), 2);
    for n in ["Grizzly Bears", "Lightning Bolt", "Hill Giant", "Shock"] {
        assert!(t.in_exile(n), "{n}");
    }
    // Each player chose two among their permanents and cards in hand.
    let maxes: Vec<(PlayerId, u32)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { max, .. } => Some((p, max)),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![(P0, 2), (P1, 2)]);
}

#[test]
fn nicol_bolas_each_opponent_exiles_a_hand_card_or_a_permanent() {
    cr!("101.4");
    ruling!(
        "Nicol Bolas, Dragon-God",
        "chooses a permanent they control or chooses a card in hand without revealing that card"
    );
    supported("Nicol Bolas, Dragon-God");
    let mut t = TestGame::new(3);
    let nb = t.battlefield(P0, "Nicol Bolas, Dragon-God");
    let c1 = t.hand(P1, "Lightning Bolt");
    t.battlefield(P1, "Forest");
    t.hand(P2, "Shock");
    let p2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(c1)]);
    t.answer_choose(P2, &[Entity::Object(p2)]);
    let hand = t.hand_size(P0);
    // The +1 is its first activated ability.
    t.activate(P0, nb, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P2, "Shock"));
}

#[test]
fn chaos_defiler_chooses_one_per_opponent_then_destroys_one_at_random() {
    cr!("608.2c", "603.10a");
    supported("Chaos Defiler");
    let mut t = TestGame::new(3);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    t.battlefield(P2, "Forest");
    let cd = t.hand(P0, "Chaos Defiler");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, cd).go();
    t.resolve_all();
    let left = [bears, giant]
        .iter()
        .filter(|o| t.on_battlefield(**o))
        .count();
    assert_eq!(left, 1);
    // It also triggers when it dies.
    let cd = t.named_on_battlefield("Chaos Defiler")[0];
    let other = [bears, giant]
        .into_iter()
        .find(|o| t.on_battlefield(*o))
        .unwrap();
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.g.destroy(cd, None);
    t.resolve_all();
    assert!(!t.on_battlefield(other));
}

#[test]
fn winnowing_each_player_keeps_creatures_sharing_a_type_with_the_chosen_one() {
    cr!("608.2c");
    supported("Winnowing");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bear2 = t.battlefield(P0, "Runeclaw Bear");
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Winnowing");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(angel)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(bear2));
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(angel));
    assert!(!t.on_battlefield(elves));
}

#[test]
fn kamiz_chooses_another_attacker_with_lesser_power() {
    cr!("508.1", "701.50a");
    supported("Kamiz, Obscura Oculus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kamiz, Obscura Oculus");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    let forest = t.hand(P0, "Forest");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.attack(
        &[
            (giant, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (angel, Entity::Player(P1)),
        ],
        &[],
    );
    assert!(t.in_graveyard(P0, "Forest"));
    // Bears (2) dealt double damage; the 3-power Giant and 4-power Angel couldn't be chosen.
    assert_eq!(t.life(P1), 20 - 3 - 4 - 2 * 2);
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert!(cands.contains(&vec![Entity::Object(bears)]));
}

#[test]
fn promise_of_loyalty_vowed_creatures_cant_attack_you_while_they_have_the_counter() {
    cr!("101.4", "611.2b");
    ruling!(
        "Promise of Loyalty",
        "If the vow counter is removed from the creature, it can attack as normal."
    );
    supported("Promise of Loyalty");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let angel = t.battlefield(P1, "Serra Angel");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Promise of Loyalty");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P1, &[Entity::Object(angel)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(elves));
    assert_eq!(t.counters(bears, "vow"), 1);
    assert_eq!(t.counters(angel, "vow"), 1);
    assert!(!t.g.can_attack_target(angel, Entity::Player(P0)));
    t.g.remove_counters(Entity::Object(angel), "vow", 1);
    t.g.recompute();
    assert!(t.g.can_attack_target(angel, Entity::Player(P0)));
}

#[test]
fn reins_of_power_you_and_that_opponent_swap_creatures() {
    cr!("608.2e", "611.2c");
    ruling!(
        "Reins of Power",
        "It is perfectly legal for either or both players to have zero creatures."
    );
    supported("Reins of Power");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P2, "Serra Angel");
    t.g.objects[bears.0 as usize].tapped = true;
    t.g.objects[giant.0 as usize].tapped = true;
    let spell = t.hand(P0, "Reins of Power");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    let g = t.obj_now(giant);
    assert_eq!(g.controller, P0);
    assert!(!g.tapped);
    assert!(g.has_keyword(keywords::KeywordKind::Haste));
    let b = t.obj_now(bears);
    assert_eq!(b.controller, P1);
    assert!(b.has_keyword(keywords::KeywordKind::Haste));
    assert_eq!(t.obj_now(other).controller, P2);
    // With no creatures on one side, the other side still changes control.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Reins of Power");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
}

#[test]
fn stardew_valley_another_player_of_your_choice_may_take_the_permanent() {
    cr!("608.2c");
    supported("Stardew Valley");
    let mut t = TestGame::new(3);
    let sv = t.battlefield(P0, "Stardew Valley");
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_yes(P2, true);
    let hand = t.hand_size(P0);
    // Its third activated ability (after the mana ability and the Food one).
    t.activate(P0, sv, 2, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.obj_now(bears).controller, P2);
}
