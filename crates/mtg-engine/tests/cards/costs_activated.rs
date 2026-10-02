//! Activation costs built from cost parts (CR 602.2b, 601.2h, 118.3) and changes to them
//! (CR 601.2f, 118.7): removing counters from another permanent, "remove all" / "one or
//! more" counters as the ability's X, exiling from the top of a library, several
//! sacrifices, a choice between two costs, "This ability costs {N} less to activate if
//! ...", "Activated abilities cost {2} more ...", mana abilities that cost life, and the
//! amount paid this way in the effect ("that much", "that many", "with power less than or
//! equal to the number of counters removed this way").

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
}

fn untapped(t: &TestGame, lands: &[ObjectId]) -> usize {
    lands.iter().filter(|l| !t.obj_now(**l).tapped).count()
}

#[test]
fn removes_a_counter_from_another_creature_you_control() {
    cr!("602.2b", "118.3");
    compiles("Spike Rogue");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 4);
    let rogue = t.battlefield(P0, "Spike Rogue");
    // A 0/0 that enters with two +1/+1 counters.
    counters(&mut t, rogue, "+1/+1", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = t.counters(rogue, "+1/+1");
    counters(&mut t, bears, "+1/+1", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, rogue, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(rogue, "+1/+1"), before + 1);
    // An opponent's creature with a counter doesn't help pay.
    compiles("Shapers of Nature");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    let shapers = t.battlefield(P0, "Shapers of Nature");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    counters(&mut t, theirs, "+1/+1", 1);
    assert!(t.activate(P0, shapers, 1, &[]).is_err());
    let mine = t.battlefield(P0, "Grizzly Bears");
    counters(&mut t, mine, "+1/+1", 1);
    let hand = t.hand_size(P0);
    t.activate(P0, shapers, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
}

#[test]
fn remove_all_counters_deals_that_much_damage() {
    cr!("602.2b", "107.3k");
    compiles("Relic Amulet");
    compiles("Molten Hydra");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 2);
    let amulet = t.battlefield(P0, "Relic Amulet");
    counters(&mut t, amulet, "charge", 3);
    let golem = t.battlefield(P1, "Craw Wurm"); // 6/4
    t.activate(P0, amulet, 0, &[Entity::Object(golem)]).unwrap();
    assert_eq!(t.counters(amulet, "charge"), 0);
    t.resolve();
    assert_eq!(t.obj_now(golem).damage, 3);

    // "It deals damage to any target equal to the number of +1/+1 counters removed this
    // way."
    let hydra = t.battlefield(P0, "Molten Hydra");
    counters(&mut t, hydra, "+1/+1", 2);
    t.activate(P0, hydra, 1, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.counters(hydra, "+1/+1"), 0);
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn one_or_more_counters_is_the_amount_chosen() {
    cr!("107.3a", "107.3k", "602.2b");
    compiles("Ooze Flux");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    let flux = t.battlefield(P0, "Ooze Flux");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Runeclaw Bear");
    counters(&mut t, a, "+1/+1", 2);
    counters(&mut t, b, "+1/+1", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, flux, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(a, "+1/+1") + t.counters(b, "+1/+1"), 0);
    let oozes: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token())
        .map(|o| o.id)
        .collect();
    assert_eq!(oozes.len(), 1);
    assert_eq!(t.pt(oozes[0]), (3, 3));
    // "One or more": not zero.
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    assert!(t.activate(P0, flux, 0, &[]).is_err());
}

#[test]
fn simic_manipulator_steals_a_creature_no_bigger_than_the_counters_removed() {
    cr!("602.2b", "107.3k");
    compiles("Simic Manipulator");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let manip = t.battlefield(P0, "Simic Manipulator");
    let have = t.counters(manip, "+1/+1");
    counters(&mut t, manip, "+1/+1", 3 - have);
    let bears = t.battlefield(P1, "Grizzly Bears"); // power 2
    let wurm = t.battlefield(P1, "Craw Wurm"); // power 6
    // Removing two counters: the Wurm can't be targeted.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, manip, 0, &[Entity::Object(bears)]).unwrap();
    let offered = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert!(offered.contains(&Entity::Object(bears)));
    assert!(!offered.contains(&Entity::Object(wurm)));
    t.resolve();
    assert_eq!(t.counters(manip, "+1/+1"), 1);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(wurm).controller, P1);
}

#[test]
fn exiles_the_top_card_of_your_library_as_a_cost() {
    cr!("602.2b", "118.3", "601.2h");
    compiles("Royal Herbalist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 4);
    let herbalist = t.battlefield(P0, "Royal Herbalist");
    let top = t.library_top(P0, "Shock");
    let lib = t.library_size(P0);
    t.activate(P0, herbalist, 0, &[]).unwrap();
    assert_eq!(t.zone(top), Zone::Exile);
    assert_eq!(t.library_size(P0), lib - 1);
    t.resolve();
    assert_eq!(t.life(P0), 21);
    // An empty library can't pay it.
    t.g.players[P0.idx()].library.clear();
    assert!(t.activate(P0, herbalist, 0, &[]).is_err());
}

#[test]
fn heralds_sacrifice_three_creatures_of_different_colors() {
    cr!("602.2b", "118.3", "701.21a");
    compiles("Demon's Herald");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 3);
    let herald = t.battlefield(P0, "Demon's Herald");
    let blue = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let black = t.battlefield(P0, "Typhoid Rats");
    // No red creature: the cost can't be paid.
    assert!(t.activate(P0, herald, 0, &[]).is_err());
    let red = t.battlefield(P0, "Raging Goblin");
    let prince = t.library_top(P0, "Prince of Thralls");
    // The Herald is a black creature too; the Rats are sacrificed instead.
    for c in [blue, black, red] {
        t.answer_choose(P0, &[Entity::Object(c)]);
    }
    t.activate(P0, herald, 0, &[]).unwrap();
    assert!(t.on_battlefield(herald));
    t.answer_choose(P0, &[Entity::Object(prince)]);
    for c in [blue, black, red] {
        assert_eq!(t.zone(c), Zone::Graveyard(P0));
    }
    t.resolve();
    assert_eq!(t.named_on_battlefield("Prince of Thralls").len(), 1);
}

#[test]
fn either_cost_sacrifice_an_artifact_or_discard_a_nonland_card() {
    cr!("602.2b", "118.3");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 8);
    let bullseye = t.battlefield(P0, "Bullseye, Death Dealer");
    // No artifact: only the discard version can be paid.
    let shock = t.hand(P0, "Shock");
    let sac = 0;
    let discard = 1;
    assert!(t.activate(P0, bullseye, sac, &[Entity::Player(P1)]).is_err());
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.activate(P0, bullseye, discard, &[Entity::Player(P1)])
        .unwrap();
    assert!(t.in_graveyard(P0, "Shock"));
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // With an artifact, sacrificing it pays the other version.
    t.g.obj_mut(bullseye).tapped = false;
    let bauble = t.battlefield(P0, "Mishra's Bauble");
    t.activate(P0, bullseye, sac, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.zone(bauble), Zone::Graveyard(P0));
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn ion_storm_removes_either_kind_of_counter() {
    cr!("602.2b", "118.3");
    compiles("Ion Storm");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    let storm = t.battlefield(P0, "Ion Storm");
    let rock = t.battlefield(P0, "Everflowing Chalice");
    counters(&mut t, rock, "charge", 1);
    // No +1/+1 counter: only the charge-counter version.
    assert!(t.activate(P0, storm, 0, &[Entity::Player(P1)]).is_err());
    t.answer_choose(P0, &[Entity::Object(rock)]);
    t.activate(P0, storm, 1, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.counters(rock, "charge"), 0);
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn costs_less_if_an_opponent_controls_four_nonbasic_lands() {
    cr!("601.2f", "602.2b", "113.6m");
    compiles("Razorlash Transmogrant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let swamps = t.lands(P0, "Swamp", 2);
    let razorlash = t.graveyard(P0, "Razorlash Transmogrant");
    // Three nonbasic lands: {4}{B}{B} is too much.
    for _ in 0..3 {
        t.battlefield(P1, "Ancient Tomb");
    }
    assert!(t.activate(P0, razorlash, 0, &[]).is_err());
    t.battlefield(P1, "Ancient Tomb");
    t.activate(P0, razorlash, 0, &[]).unwrap();
    assert_eq!(untapped(&t, &swamps), 0);
    t.resolve();
    let on = t.named_on_battlefield("Razorlash Transmogrant");
    assert_eq!(on.len(), 1);
    assert_eq!(t.counters(on[0], "+1/+1"), 1);

    // One opponent must control all four: two opponents with two each don't count.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let razorlash = t.graveyard(P0, "Razorlash Transmogrant");
    for _ in 0..2 {
        t.battlefield(P1, "Ancient Tomb");
        t.battlefield(P2, "Ancient Tomb");
    }
    assert!(t.activate(P0, razorlash, 0, &[]).is_err());
    t.battlefield(P2, "Ancient Tomb");
    t.battlefield(P2, "Ancient Tomb");
    t.activate(P0, razorlash, 0, &[]).unwrap();
}

#[test]
fn suppression_field_taxes_activated_abilities_but_not_mana_abilities() {
    cr!("601.2f", "602.2b", "605.1a");
    compiles("Suppression Field");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Suppression Field");
    let plains = t.lands(P0, "Plains", 3);
    let herbalist = t.battlefield(P0, "Royal Herbalist");
    t.library_top(P0, "Shock");
    // {2} + {2}: four mana needed, three available.
    assert!(t.activate(P0, herbalist, 0, &[]).is_err());
    assert_eq!(untapped(&t, &plains), 3);
    t.lands(P0, "Plains", 1);
    t.activate(P0, herbalist, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn thran_portal_mana_costs_a_life() {
    cr!("602.2b", "605.1a", "118.8", "118.3");
    ruling!("Thran Portal", "You still have to pay that mana ability's other costs");
    ruling!("Thran Portal", "the last ability of each of them applies only to itself");
    compiles("Thran Portal");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // As it enters, it becomes a Plains (the first basic land type offered).
    let portal = t.enter(P0, "Thran Portal");
    let other = t.enter(P0, "Thran Portal");
    t.activate(P0, portal, 0, &[]).unwrap();
    // One life (not two for two Portals), and the land is tapped.
    assert_eq!(t.life(P0), 19);
    assert!(t.obj_now(portal).tapped);
    assert!(!t.obj_now(other).tapped);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    // Tapped: the other cost can't be paid again.
    assert!(t.activate(P0, portal, 0, &[]).is_err());
    assert_eq!(t.life(P0), 19);
}

#[test]
fn automatic_payment_counts_the_life_a_mana_ability_costs() {
    cr!("601.2g", "118.3", "119.4");
    // A Plains pays {W} rather than the Portal, whose mana costs a life.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let portal = t.enter(P0, "Thran Portal");
    let plains = t.lands(P0, "Plains", 1);
    let lions = t.hand(P0, "Savannah Lions");
    t.cast(P0, lions).go();
    assert_eq!(t.life(P0), 20);
    assert!(!t.obj_now(portal).tapped);
    assert!(t.obj_now(plains[0]).tapped);
    // With no life to pay (at 0 life under Platinum Angel), the Portal can't help pay, and
    // the Plains still does.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Platinum Angel");
    t.g.players[P0.idx()].life = 0;
    let portal = t.enter(P0, "Thran Portal");
    t.lands(P0, "Plains", 1);
    let lions = t.hand(P0, "Savannah Lions");
    t.cast(P0, lions).go();
    assert!(!t.obj_now(portal).tapped);
    let lions2 = t.hand(P0, "Savannah Lions");
    assert!(t.cast(P0, lions2).try_go().is_err());
    assert_eq!(t.life(P0), 0);
}

#[test]
fn jetfire_has_target_player_add_the_colorless_mana() {
    cr!("106.4", "605.1a", "107.3k");
    compiles("Jetfire, Ingenious Scientist // Jetfire, Air Guardian");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let jetfire = t.battlefield(P0, "Jetfire, Ingenious Scientist // Jetfire, Air Guardian");
    let rock = t.battlefield(P0, "Mind Stone");
    counters(&mut t, rock, "+1/+1", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    // A targeted ability isn't a mana ability: it uses the stack.
    t.activate(P0, jetfire, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(rock, "+1/+1"), 0);
    t.resolve();
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::C), 2);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn danitha_reduces_aura_and_equipment_spells() {
    cr!("601.2f", "118.7a");
    compiles("Danitha Capashen, Paragon");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Danitha Capashen, Paragon");
    // Bonesplitter ({1}) and Pacifism ({1}{W}) each cost one less; Shock doesn't.
    let splitter = t.hand(P0, "Bonesplitter");
    t.cast(P0, splitter).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1);
    let plains = t.lands(P0, "Plains", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = t.hand(P0, "Pacifism");
    t.cast(P0, pacifism).target(bears).go();
    assert_eq!(untapped(&t, &plains), 0);
}

#[test]
fn gallia_returns_with_a_counter_on_her() {
    cr!("602.2b", "113.6m");
    compiles("Gallia, Tragic Host");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 5);
    let gallia = t.graveyard(P0, "Gallia, Tragic Host");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, gallia, 0, &[]).unwrap();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.resolve();
    let on = t.named_on_battlefield("Gallia, Tragic Host");
    assert_eq!(on.len(), 1);
    assert!(t.obj_now(on[0]).tapped);
    assert_eq!(t.counters(on[0], "+1/+1"), 1);
}

#[test]
fn king_darien_is_sacrificed_by_his_short_name() {
    cr!("602.2b");
    compiles("King Darien XLVIII");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let king = t.battlefield(P0, "King Darien XLVIII");
    t.activate(P0, king, 1, &[]).unwrap();
    assert_eq!(t.zone(king), Zone::Graveyard(P0));
}

#[test]
fn radiant_lotus_adds_three_mana_per_artifact_sacrificed() {
    cr!("106.4", "605.1a", "107.3k");
    compiles("Radiant Lotus");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let lotus = t.battlefield(P0, "Radiant Lotus");
    let a = t.battlefield(P0, "Mishra's Bauble");
    let b = t.battlefield(P0, "Mind Stone");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, lotus, 0, &[Entity::Player(P0)]).unwrap();
    // It targets, so it isn't a mana ability: it waits on the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    assert_eq!(t.zone(b), Zone::Graveyard(P0));
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.total(), 6);
}

#[test]
fn a_long_flavor_word_before_an_activated_ability() {
    cr!("207.2d", "602.2b");
    compiles("Ignis Scientia");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    let ignis = t.battlefield(P0, "Ignis Scientia");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let idx = t
        .g
        .obj(ignis)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.contains("Exile target card"))
        .unwrap();
    t.activate(P0, ignis, idx, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(
        t.g.permanents().filter(|o| o.is_token()).count(),
        1,
        "a Food token"
    );
}


#[test]
fn target_player_adds_mana_of_a_color_its_controller_chooses() {
    cr!("605.1a", "608.2d", "106.4");
    ruling!("The Warring Triad", "last ability isn't a mana ability");
    compiles("The Warring Triad");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Fewer than eight cards in the graveyard: not a creature, so {T} can be paid.
    let triad = t.battlefield(P0, "The Warring Triad");
    t.library_top(P0, "Shock");
    t.activate(P0, triad, 0, &[Entity::Player(P1)]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
    // The controller picks black (W, U, B, R, G); P1's answer is never asked for.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.answer(P1, DecisionKind::Option, Answer::Index(4));
    t.resolve();
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::B), 1);
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn x_counters_removed_is_bounded_by_the_counters_not_the_mana() {
    cr!("107.3a", "602.2b", "118.3");
    compiles("Retribution of the Ancients");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 1);
    let retribution = t.battlefield(P0, "Retribution of the Ancients");
    let bears = t.battlefield(P0, "Grizzly Bears");
    counters(&mut t, bears, "+1/+1", 3);
    let wurm = t.battlefield(P1, "Craw Wurm"); // 6/4
    // One mana, but three counters: X can be 3.
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, retribution, 0, &[Entity::Object(wurm)]).unwrap();
    let max = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseX { max, .. } => Some(max),
            _ => None,
        })
        .unwrap();
    assert!(max >= 3, "X offered up to {max}");
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    t.resolve();
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn corpseweft_token_is_twice_the_cards_exiled() {
    cr!("107.3a", "602.2b");
    compiles("Corpseweft");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 2);
    let weft = t.battlefield(P0, "Corpseweft");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Craw Wurm");
    t.graveyard(P0, "Shock");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, weft, 0, &[]).unwrap();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    t.resolve();
    let tokens: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token())
        .map(|o| o.id)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (4, 4));
    assert!(t.obj_now(tokens[0]).tapped);
}
