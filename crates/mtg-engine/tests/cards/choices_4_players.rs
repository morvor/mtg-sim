//! Player phrases compared across players: "each player with fewer cards in hand than the
//! player with the most cards in hand ... the difference", "each player who controls
//! fewer lands than the player who controls the most lands ...", "the player with the
//! lowest life total" with ties broken by a choice.

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
fn tales_of_the_ancestors_each_player_draws_up_to_the_most_cards_in_hand() {
    cr!("608.2h");
    supported("Tales of the Ancestors");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Tales of the Ancestors");
    // After the spell leaves P0's hand: P0 has 1 card, P1 has 4, P2 has 2.
    t.hand(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.hand(P1, "Grizzly Bears");
    }
    for _ in 0..2 {
        t.hand(P2, "Grizzly Bears");
    }
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (4, 4, 4)
    );
}

#[test]
fn scholarship_sponsor_players_search_for_the_difference_in_lands() {
    cr!("608.2h");
    ruling!(
        "Scholarship Sponsor",
        "the first opponent will search their library for one basic land card"
    );
    supported("Scholarship Sponsor");
    let mut t = TestGame::new(3);
    // P0 controls four lands, P1 six, P2 seven.
    t.lands(P0, "Plains", 4);
    t.lands(P1, "Forest", 6);
    t.lands(P2, "Mountain", 7);
    let p0_lands: Vec<_> = (0..3).map(|_| t.library_top(P0, "Plains")).collect();
    let p1_land = t.library_top(P1, "Forest");
    t.library_top(P2, "Mountain");
    let sponsor = t.hand(P0, "Scholarship Sponsor");
    t.answer_choose(
        P0,
        &p0_lands
            .iter()
            .map(|o| Entity::Object(*o))
            .collect::<Vec<_>>(),
    );
    t.answer_choose(P1, &[Entity::Object(p1_land)]);
    t.cast(P0, sponsor).go();
    t.resolve_all();
    let lands = |t: &TestGame, p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|o| {
                let o = t.obj_now(**o);
                o.controller == p && o.chars.card_types.contains(types::CardType::Land)
            })
            .count()
    };
    assert_eq!((lands(&t, P0), lands(&t, P1), lands(&t, P2)), (7, 7, 7));
}

#[test]
fn loxodon_peacekeeper_goes_to_the_player_with_the_lowest_life_total() {
    cr!("613.1b");
    supported("Loxodon Peacekeeper");
    let mut t = TestGame::new(3);
    let ele = t.battlefield(P0, "Loxodon Peacekeeper");
    t.g.player_mut(P2).life = 12;
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(ele).controller, P2);
}

#[test]
fn loxodon_peacekeeper_tie_is_broken_by_its_controller() {
    cr!("608.2c", "613.1b");
    supported("Loxodon Peacekeeper");
    let mut t = TestGame::new(3);
    let ele = t.battlefield(P0, "Loxodon Peacekeeper");
    t.g.player_mut(P1).life = 12;
    t.g.player_mut(P2).life = 12;
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(ele).controller, P2);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, decision::Decision::ChooseEntities { candidates, .. } if candidates.len() == 2)));
}

#[test]
fn curse_of_surveillance_other_target_players_draw_per_curse() {
    cr!("115.1", "303.4");
    supported("Curse of Surveillance");
    let mut t = TestGame::new(3);
    let c1 = t.battlefield(P0, "Curse of Surveillance");
    let c2 = t.battlefield(P0, "Curse of Death's Hold");
    t.g.attach(c1, Entity::Player(P1));
    t.g.attach(c2, Entity::Player(P1));
    // P0 targets P0 and P2 (not P1, the enchanted player).
    t.answer_targets(P0, &[Entity::Player(P0), Entity::Player(P2)]);
    t.advance_to(P1, Step::Upkeep);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.resolve_all();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (hands.0 + 2, hands.1, hands.2 + 2)
    );
    // P1 wasn't a legal target.
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| !c.contains(&Entity::Player(P1))));
}

#[test]
fn zurzoth_you_and_the_players_devils_attack_draw_then_discard_at_random() {
    cr!("101.4", "608.2e");
    ruling!(
        "Zurzoth, Chaos Rider",
        "Devils you control that attack a planeswalker won't cause any player to draw or discard a card."
    );
    supported("Zurzoth, Chaos Rider");
    let mut t = TestGame::new(4);
    let z = t.battlefield(P0, "Zurzoth, Chaos Rider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let devil = t.battlefield(P0, "Forge Devil");
    let jace = t.battlefield(P3, "Jace Beleren");
    for p in [P0, P1, P2, P3] {
        t.hand(p, "Lightning Bolt");
    }
    let hands: Vec<usize> = [P0, P1, P2, P3].iter().map(|p| t.hand_size(*p)).collect();
    // Zurzoth attacks P1, the Bears (not a Devil) attack P2, a Devil attacks P3's
    // planeswalker.
    t.attack(
        &[
            (z, Entity::Player(P1)),
            (bears, Entity::Player(P2)),
            (devil, Entity::Object(jace)),
        ],
        &[],
    );
    let gy: Vec<usize> = [P0, P1, P2, P3]
        .iter()
        .map(|p| t.graveyard_size(*p))
        .collect();
    assert_eq!(gy, vec![1, 1, 0, 0]);
    let now: Vec<usize> = [P0, P1, P2, P3].iter().map(|p| t.hand_size(*p)).collect();
    assert_eq!(now, hands);
}

#[test]
fn nelly_borca_you_and_the_attacking_player_draw() {
    cr!("101.4");
    supported("Nelly Borca, Impulsive Accuser");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Nelly Borca, Impulsive Accuser");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    let hands = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
    t.attack(&[(bears, Entity::Player(P2))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), 18);
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)),
        (hands.0 + 1, hands.1 + 1, hands.2)
    );
}

fn concussive(target_jace: bool) -> (TestGame, ObjectId) {
    supported("Concussive Bolt");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 5);
    for _ in 0..3 {
        t.battlefield(P0, "Ornithopter");
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    let spell = t.hand(P0, "Concussive Bolt");
    let target = if target_jace {
        Entity::Object(jace)
    } else {
        Entity::Player(P1)
    };
    t.cast(P0, spell).target(target).go();
    t.resolve();
    (t, bears)
}

#[test]
fn concussive_bolt_that_players_creatures_cant_block() {
    cr!("611.2c");
    let (mut t, bears) = concussive(false);
    assert_eq!(t.life(P1), 16);
    assert!(!t.g.can_block_at_all(bears));
    // A creature they control later this turn can't block either.
    let later = t.battlefield(P1, "Hill Giant");
    assert!(!t.g.can_block_at_all(later));
    let other = t.battlefield(P2, "Hill Giant");
    assert!(t.g.can_block_at_all(other));
}

#[test]
fn concussive_bolt_that_planeswalkers_controllers_creatures_cant_block() {
    cr!("611.2c", "306.8");
    let (t, bears) = concussive(true);
    assert_eq!(t.life(P1), 20);
    // Four damage removed its three loyalty counters: it's gone, and its controller (as
    // it last existed) is still the player whose creatures can't block.
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    assert!(!t.g.can_block_at_all(bears));
}

#[test]
fn keen_duelist_each_loses_the_other_players_revealed_mana_value() {
    cr!("701.20a");
    supported("Keen Duelist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keen Duelist");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // On P1's turn (after their draw).
    t.advance_to(P1, Step::End);
    t.library_top(P0, "Hill Giant");
    t.library_top(P1, "Grizzly Bears");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 16));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn parker_luck_two_target_players_each_lose_the_others_mana_value() {
    cr!("701.20a", "115.3");
    supported("Parker Luck");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Parker Luck");
    t.library_top(P1, "Hill Giant");
    t.library_top(P2, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 19, 16));
    assert!(t.in_hand(P1, "Hill Giant"));
    assert!(t.in_hand(P2, "Lightning Bolt"));
}

#[test]
fn delirium_the_active_players_creature_deals_damage_to_them() {
    cr!("608.2c");
    ruling!(
        "Delirium",
        "Tapping the creature is part of the effect and not the cost, therefore you can cast this targeting a tapped creature."
    );
    supported("Delirium");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Delirium");
    t.advance_to(P1, Step::PrecombatMain);
    t.g.objects[giant.0 as usize].tapped = true;
    t.cast(P0, spell).target(Entity::Object(giant)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn keeper_of_the_dead_targets_an_opponent_with_two_fewer_creature_cards() {
    cr!("115.1", "601.2c");
    ruling!(
        "Keeper of the Dead",
        "It is only necessary that the condition be true as you activate the ability."
    );
    supported("Keeper of the Dead");
    let mut t = TestGame::new(3);
    let keeper = t.battlefield(P0, "Keeper of the Dead");
    t.lands(P0, "Swamp", 1);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.graveyard(P2, "Grizzly Bears");
    t.graveyard(P2, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Hill Giant");
    t.activate(P0, keeper, 0, &[Entity::Player(P1), Entity::Object(bears)])
        .unwrap();
    // The condition no longer holds as it resolves: it still resolves.
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Hill Giant");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // P2 (one fewer) wasn't a legal target.
    let players: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. }
                if candidates.iter().all(|c| matches!(c, Entity::Player(_))) =>
            {
                Some(candidates)
            }
            _ => None,
        })
        .collect();
    assert_eq!(players, vec![vec![Entity::Player(P1)]]);
}

#[test]
fn oath_of_ghouls_targets_an_opponent_with_fewer_creature_cards_in_their_graveyard() {
    cr!("115.1", "608.2b");
    ruling!(
        "Oath of Ghouls",
        "A player can’t be targeted by this ability unless it’s true"
    );
    supported("Oath of Ghouls");
    let mut t = TestGame::new(3);
    // (Set up after the other players' upkeeps.)
    t.advance_to(P2, Step::End);
    t.battlefield(P0, "Oath of Ghouls");
    let a = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P2, "Grizzly Bears");
    t.graveyard(P2, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // In P0's upkeep only P1 (one creature card against P0's two) could be targeted, not
    // P2 (two, not fewer).
    let players: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(players, vec![vec![Entity::Player(P1)]]);
}

#[test]
fn oath_of_druids_reveals_until_a_creature_if_the_first_player_does() {
    cr!("115.1", "701.20a");
    supported("Oath of Druids");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Druids");
    t.battlefield(P1, "Hill Giant");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::Upkeep);
    // The draw step hasn't happened: the revealed cards are the ones put on top.
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, "Forest"));
}
