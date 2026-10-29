//! Rulings batch P053 — "eggs": artifacts (and a few creatures) that sacrifice themselves
//! or are activated for value. Choices made in turn order (CR 101.4), checks made as an
//! ability resolves (CR 608.2), costs (CR 118, 602.2), targets chosen before costs are
//! paid (CR 601.2c, 602.2b), reflexive triggers (CR 603.12), effects that create
//! triggered abilities (CR 603.7) and controlling another player (CR 723).

use crate::r_p053_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    crate::r_s06_common::has_kw(t, id, kw)
}

fn doom_counters_anywhere(g: &mtg_engine::game::Game) -> u32 {
    g.permanents().map(|o| o.counter("doom")).sum()
}

#[test]
fn eye_of_doom_choices_in_turn_order_then_counters_at_once() {
    cr!("101.4", "608.2e");
    ruling!(
        "Eye of Doom",
        "The active player chooses a permanent first, followed by each other player in turn order. Each player will know what previous players have chosen when making their own choice. Then the doom counters are put on all the chosen permanents simultaneously."
    );
    supported("Eye of Doom");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Gray Ogre");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(ogre)]);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        doom_counters_anywhere,
    );
    let from = t.asked().len();
    t.enter(P0, "Eye of Doom");
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    // No counter was put on P0's choice before P1 chose.
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    assert_eq!(t.counters(bears, "doom"), 1);
    assert_eq!(t.counters(ogre, "doom"), 1);
}

#[test]
fn orzhov_advokist_creatures_chosen_in_turn_order_then_counters_at_once() {
    // The same rule when the choice is part of the counter instruction ("each player may
    // put two +1/+1 counters on a creature they control").
    cr!("101.4", "101.4b", "608.2e");
    ruling!(
        "Orzhov Advokist",
        "Then simultaneously each player who chose a creature puts two +1/+1 counters on the creature they chose."
    );
    supported("Orzhov Advokist");
    let mut t = TestGame::new(2);
    let advokist = t.battlefield(P0, "Orzhov Advokist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let other = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P1, &[obj(ogre)]);
    fn plus1_anywhere(g: &mtg_engine::game::Game) -> u32 {
        g.permanents().map(|o| o.counter(counters::PLUS1)).sum()
    }
    let seen = crate::r_s01_common::watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        plus1_anywhere,
    );
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    // P0's counters weren't on the Bears yet when P1 chose.
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(ogre, counters::PLUS1), 2);
    assert_eq!(t.counters(advokist, counters::PLUS1), 0);
    assert_eq!(t.counters(other, counters::PLUS1), 0);
}

#[test]
fn checks_made_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Bottle of Suleiman",
        "The coin is flipped on resolution and not on declaration of the ability."
    );
    ruling!(
        "Scroll of Griselbrand",
        "Whether you control a Demon is checked when the ability resolves."
    );
    ruling!(
        "Scroll of Avacyn",
        "Whether you control an Angel is checked when the ability resolves."
    );
    supported("Bottle of Suleiman");
    supported("Scroll of Griselbrand");
    supported("Scroll of Avacyn");
    // Bottle: no flip until it resolves (won: a Djinn).
    let mut t = TestGame::new(2);
    let bottle = t.battlefield(P0, "Bottle of Suleiman");
    t.lands(P0, "Wastes", 1);
    crate::r_p017_common::load_coins(&mut t, &[true]);
    t.activate(P0, bottle, 0, &[]).unwrap();
    assert!(crate::r_p017_common::flip_results(&t).is_empty());
    t.resolve_all();
    assert_eq!(crate::r_p017_common::flip_results(&t).len(), 1);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    // Griselbrand: a Demon arrives in response.
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "Scroll of Griselbrand");
    t.hand(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, scroll, 0, &[Entity::Player(P1)]).unwrap();
    t.battlefield(P0, "Lord of the Pit");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Avacyn: the Angel leaves in response.
    let mut t = TestGame::new(2);
    let scroll = t.battlefield(P0, "Scroll of Avacyn");
    let angel = t.battlefield(P0, "Serra Angel");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, scroll, 0, &[]).unwrap();
    destroy(&mut t, angel);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn hemlock_vial_affects_only_whats_equipped_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Hemlock Vial",
        "The set of creatures that gain deathtouch is determined as Hemlock Vial’s last ability resolves. Creatures you control that become equipped later in the turn won’t gain deathtouch, even if the Equipment that becomes attached to them has deathtouch."
    );
    supported("Hemlock Vial");
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Hemlock Vial");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    let sword = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    t.lands(P0, "Swamp", 1);
    t.activate(P0, vial, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Deathtouch));
    assert!(has_kw(&t, sword, KeywordKind::Deathtouch));
    // The Equipment (with deathtouch) moves to the Ogre; another Equipment too.
    assert!(t.g.attach(sword, obj(ogre)));
    t.g.recompute();
    assert!(!has_kw(&t, ogre, KeywordKind::Deathtouch));
    assert!(has_kw(&t, bears, KeywordKind::Deathtouch));
}

#[test]
fn costs_that_can_always_be_paid() {
    cr!("118.3", "701.21a", "602.2b");
    ruling!(
        "Agent's Toolkit",
        "The shield counter on Agent's Toolkit won't prevent its controller from sacrificing it."
    );
    ruling!(
        "Bomat Courier",
        "You can pay the cost of “discard your hand” even if your hand contains zero cards."
    );
    supported("Agent's Toolkit");
    supported("Bomat Courier");
    let mut t = TestGame::new(2);
    let kit = t.enter(P0, "Agent's Toolkit");
    t.resolve_all();
    assert_eq!(t.counters(kit, counters::SHIELD), 1);
    t.lands(P0, "Wastes", 2);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, kit, "Draw a card").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Agent's Toolkit"));
    assert_eq!(t.hand_size(P0), hand + 1);
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P0, "Bomat Courier");
    t.lands(P0, "Mountain", 1);
    assert_eq!(t.hand_size(P0), 0);
    assert!(can_activate(&mut t, P0, courier));
    activate_containing(&mut t, P0, courier, "Discard your hand").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bomat Courier"));
}

#[test]
fn esoteric_duplicator_copies_the_printed_artifact() {
    cr!("707.2", "603.7");
    ruling!(
        "Esoteric Duplicator",
        "The token copies exactly what was printed on the original artifact and nothing else"
    );
    supported("Esoteric Duplicator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Esoteric Duplicator");
    let block = t.battlefield(P0, "Nutrient Block");
    // A non-copy effect makes the Nutrient Block a 5/5 creature; it has a counter.
    crate::r_s26_common::modify_until_eot(
        &mut t,
        block,
        vec![
            mtg_engine::ability::Modification::AddTypes(vec![CardType::Creature]),
            mtg_engine::ability::Modification::SetPT(
                Some(mtg_engine::ability::Value::c(5)),
                Some(mtg_engine::ability::Value::c(5)),
            ),
        ],
    );
    put_counters(&mut t, block, counters::PLUS1, 1);
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, block, "gain 3 life").unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let toks = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.chars.name, "Nutrient Block");
    assert!(!tok.is(CardType::Creature));
    assert!(!tok.tapped);
    assert_eq!(t.counters(toks[0], counters::PLUS1), 0);
}

#[test]
fn delifs_cone_and_cube_create_triggers_that_watch_the_target() {
    cr!("603.7a");
    ruling!(
        "Delif's Cone",
        "This card has an activated ability that creates a triggered ability which watches the target creature."
    );
    ruling!(
        "Delif's Cube",
        "This card has an activated ability that creates a triggered ability which watches the target creature."
    );
    supported("Delif's Cone");
    supported("Delif's Cube");
    let mut t = TestGame::new(2);
    let cone = t.battlefield(P0, "Delif's Cone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, cone, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 20);
    // Declining the life: it deals its damage as usual.
    let mut t = TestGame::new(2);
    let cone = t.battlefield(P0, "Delif's Cone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, cone, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18);
    let mut t = TestGame::new(2);
    let cube = t.battlefield(P0, "Delif's Cube");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, cube, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(cube, "cube"), 1);
}

#[test]
fn tainted_sigil_counts_all_life_lost_this_turn() {
    cr!("608.2h");
    ruling!(
        "Tainted Sigil",
        "When Tainted Sigil’s ability resolves, it checks how much life each player lost over the course of the turn, then it totals that number and you gain that much life in one shot."
    );
    supported("Tainted Sigil");
    let mut t = TestGame::new(2);
    crate::r_p009_common::lose(&mut t, P0, 2);
    crate::r_p009_common::lose(&mut t, P1, 3);
    // It wasn't on the battlefield when the life was lost.
    let sigil = t.battlefield(P0, "Tainted Sigil");
    let from = t.turn_events.len();
    t.activate(P0, sigil, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18 + 5);
    let gains = t.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, mtg_engine::events::Event::LifeGained { .. }))
        .count();
    assert_eq!(gains, 1);
}

#[test]
fn bag_of_holding_and_conch_horn_finish_before_draw_triggers() {
    cr!("608.2", "603.3");
    ruling!(
        "Bag of Holding",
        "You both draw and discard while Bag of Holding's second ability is resolving. No player may take any action—nor can anything else happen—until you've both drawn and discarded."
    );
    ruling!(
        "Conch Horn",
        "You draw two cards, then put one back as part of the same effect. You can’t do anything in between. Anything that triggers off of card drawing will wait until you finish resolving this ability before going on the stack."
    );
    supported("Bag of Holding");
    supported("Conch Horn");
    let mut t = TestGame::new(2);
    let bag = t.battlefield(P0, "Bag of Holding");
    t.lands(P0, "Wastes", 2);
    let drawn = t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(drawn)]);
    t.activate(P0, bag, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    // Its discard trigger (exile that card) went on the stack only afterwards.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Conch Horn with Underworld Dreams: both triggers wait until it has resolved.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Underworld Dreams");
    let horn = t.battlefield(P0, "Conch Horn");
    t.lands(P0, "Wastes", 1);
    let ids = crate::r_s01_common::stack_library(&mut t, P0, &["Grizzly Bears", "Gray Ogre"]);
    t.answer_choose(P0, &[obj(ids[0])]);
    t.activate(P0, horn, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(crate::r_s33_common::library_top_first(&t, P0)[0], t.g.current(ids[0]));
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn activating_sacrifice_abilities_in_response_to_themselves() {
    cr!("602.2", "608.2", "701.21a", "406.3");
    ruling!(
        "Goblin Cannon",
        "You can activate Goblin Cannon's ability multiple times in response to the first activation. Goblin Cannon will be sacrificed the first time the ability resolves, but each activation still deals damage when it resolves."
    );
    ruling!(
        "Knowledge Vault",
        "You can activate the second ability any time you have priority, including in response to itself. However, you will only be able to sacrifice the Vault as the first such ability resolves."
    );
    ruling!(
        "Knowledge Vault",
        "You don't get to look at the cards exiled with Knowledge Vault."
    );
    supported("Goblin Cannon");
    supported("Knowledge Vault");
    let mut t = TestGame::new(2);
    let cannon = t.battlefield(P0, "Goblin Cannon");
    t.lands(P0, "Wastes", 6);
    for _ in 0..3 {
        t.activate(P0, cannon, 0, &[Entity::Player(P1)]).unwrap();
    }
    assert_eq!(t.stack_len(), 3);
    t.resolve();
    assert!(t.in_graveyard(P0, "Goblin Cannon"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Knowledge Vault: exile a card face down, then {0} twice.
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Knowledge Vault");
    t.lands(P0, "Wastes", 2);
    let card = t.library_top(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, vault, "Exile the top card").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.obj_now(card).face_down);
    // Not even its controller may look at it (CR 406.3).
    assert!(!mtg_engine::player_control::can_see(&t.g, P0, t.g.current(card)));
    t.hand(P0, "Gray Ogre");
    activate_containing(&mut t, P0, vault, "Sacrifice ~.").unwrap();
    activate_containing(&mut t, P0, vault, "Sacrifice ~.").unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(t.in_graveyard(P0, "Knowledge Vault"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Gray Ogre"));
    // The second one can't sacrifice it: nothing happens (the Bears stay in hand).
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn idol_of_oblivion_counts_any_token_created_this_turn() {
    ruling!(
        "Idol of Oblivion",
        "You can activate Idol of Oblivion's first ability even if the token you've created was created before Idol of Oblivion entered the battlefield or if the token has left the battlefield."
    );
    supported("Idol of Oblivion");
    let mut t = TestGame::new(2);
    let treasure = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    let idol = t.battlefield(P0, "Idol of Oblivion");
    destroy(&mut t, treasure);
    assert!(can_activate_text(&mut t, P0, idol, "Draw a card"));
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Idol of Oblivion");
    assert!(!can_activate_text(&mut t, P0, idol, "Draw a card"));
}

#[test]
fn shrine_of_limitless_power_cant_take_a_spell_already_cast() {
    cr!("601.2a", "701.9a");
    ruling!(
        "Shrine of Limitless Power",
        "You can’t force an opponent to discard a card they cast by activating the last ability in response. The spell is on the stack by then."
    );
    supported("Shrine of Limitless Power");
    let mut t = TestGame::new(2);
    let shrine = t.battlefield(P0, "Shrine of Limitless Power");
    put_counters(&mut t, shrine, counters::CHARGE, 2);
    t.lands(P0, "Wastes", 4);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.activate(P0, shrine, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn food_sacrificed_for_one_cost_cant_pay_another() {
    cr!("118.3", "602.2b");
    ruling!(
        "Dubious Delicacy",
        "You can’t sacrifice a Food to pay multiple costs."
    );
    ruling!("Nutrient Block", "You can’t sacrifice a Food to pay multiple costs.");
    supported("Ragost, Deft Gastronaut");
    for food in ["Dubious Delicacy", "Nutrient Block"] {
        supported(food);
        let mut t = TestGame::new(2);
        let ragost = t.battlefield(P0, "Ragost, Deft Gastronaut");
        let f = t.battlefield(P0, food);
        t.lands(P0, "Wastes", 3);
        assert!(can_activate_text(&mut t, P0, ragost, "Sacrifice a Food"), "{food}");
        activate_containing(&mut t, P0, f, "gain 3 life").unwrap();
        assert!(!t.on_battlefield(f), "{food}");
        assert!(!can_activate_text(&mut t, P0, ragost, "Sacrifice a Food"), "{food}");
        t.resolve_all();
        assert_eq!(t.life(P0), 23, "{food}");
    }
}

#[test]
fn any_number_of_target_players_or_none() {
    cr!("115.1c", "601.2c");
    ruling!(
        "Stonespeaker Crystal",
        "You choose any number of target players as you put Stonespeaker Crystal's ability on the stack and exile their graveyards as it resolves. You may not target a player and then choose not to exile their graveyard."
    );
    ruling!(
        "Stonespeaker Crystal",
        "You may choose to target no players while activating the last ability if you just want to draw a card."
    );
    ruling!(
        "Nihil Spellbomb",
        "You may activate Nihil Spellbomb's first ability targeting any player, even one whose graveyard has no cards in it."
    );
    ruling!(
        "Boom Box",
        "You may choose zero, one, two, or three targets for Boom Box’s ability."
    );
    supported("Stonespeaker Crystal");
    supported("Nihil Spellbomb");
    supported("Boom Box");
    for targets in [vec![], vec![Entity::Player(P0), Entity::Player(P1)]] {
        let mut t = TestGame::new(2);
        let crystal = t.battlefield(P0, "Stonespeaker Crystal");
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P1, "Gray Ogre");
        t.lands(P0, "Wastes", 2);
        let from = t.asked().len();
        t.answer_targets(P0, &targets);
        activate_containing(&mut t, P0, crystal, "Exile any number").unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 1);
        let n = targets.len();
        // (The sacrificed Crystal is in P0's graveyard as it resolves.)
        assert_eq!(t.graveyard_size(P0), if n == 0 { 2 } else { 0 });
        assert_eq!(t.graveyard_size(P1), if n == 0 { 1 } else { 0 });
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    }
    // Nihil Spellbomb: P1's graveyard is empty.
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Nihil Spellbomb");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, bomb, 0, &[]).unwrap();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&Entity::Player(P1)));
    assert_eq!(crate::r_s25_common::targets_of(&t, top(&t)), vec![Entity::Player(P1)]);
    // Boom Box with no targets at all.
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Boom Box");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 6);
    for _ in 0..3 {
        t.answer_targets(P0, &[]);
    }
    t.activate(P0, bb, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Boom Box"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn codex_shredder_cant_return_itself() {
    cr!("602.2b", "601.2c");
    ruling!(
        "Codex Shredder",
        "You choose the target of Codex Shredder's last ability before paying its costs, so you can't return Codex Shredder to your hand this way."
    );
    supported("Codex Shredder");
    let mut t = TestGame::new(2);
    let shredder = t.battlefield(P0, "Codex Shredder");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 5);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, shredder, "Return target card").unwrap();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered, vec![vec![obj(bears)]]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Codex Shredder"));
}

#[test]
fn investigators_journal_uses_the_greatest_number_of_creatures() {
    cr!("614.1c");
    ruling!(
        "Investigator's Journal",
        "You don't choose an opponent to determine how many suspect counters are placed on Investigator's Journal as it enters the battlefield. You always use the greatest number possible."
    );
    supported("Investigator's Journal");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        t.battlefield(P1, "Grizzly Bears");
    }
    for _ in 0..3 {
        t.battlefield(P2, "Grizzly Bears");
    }
    let from = t.asked().len();
    let j = t.enter(P0, "Investigator's Journal");
    t.resolve_all();
    assert_eq!(t.counters(j, "suspect"), 3);
    assert!(asked_debug(&t, from).is_empty());
}

#[test]
fn smoke_bomb_reflexive_target_chosen_after_the_sacrifice() {
    cr!("603.12", "702.18a");
    ruling!(
        "Smoke Bomb",
        "You don’t choose a target for Smoke Bomb’s last ability at the time it triggers. Rather, a second “reflexive” ability triggers when you sacrifice Smoke Bomb this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Smoke Bomb");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Smoke Bomb");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    // While the Bomb is there, creatures have shroud and no target was chosen.
    assert!(crate::r_s02_common::target_candidates(&t, P0, from).is_empty());
    assert!(has_kw(&t, bears, KeywordKind::Shroud));
    let r = crate::r_p130_common::resolve_into_reflexive(&mut t, &[obj(bears)]);
    assert!(t.in_graveyard(P0, "Smoke Bomb"));
    assert_eq!(top(&t), r);
    t.resolve_all();
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(ogre, bears)]));
}

#[test]
fn searching_for_lands() {
    cr!("701.23a");
    ruling!(
        "Gem of Becoming",
        "You don’t have to find all three cards, even if they’re present in your library."
    );
    ruling!("Expedition Map", "You may find any land card, not just a basic land card.");
    supported("Gem of Becoming");
    supported("Expedition Map");
    let mut t = TestGame::new(2);
    let gem = t.battlefield(P0, "Gem of Becoming");
    let island = t.library_top(P0, "Island");
    let swamp = t.library_top(P0, "Swamp");
    t.library_top(P0, "Mountain");
    t.lands(P0, "Wastes", 3);
    t.answer_choose_moved(P0, &[obj(island)]);
    t.answer_choose_moved(P0, &[obj(swamp)]);
    t.answer_choose_moved(P0, &[]);
    t.activate(P0, gem, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
    assert!(t.in_hand(P0, "Swamp"));
    assert!(!t.in_hand(P0, "Mountain"));
    let mut t = TestGame::new(2);
    let map = t.battlefield(P0, "Expedition Map");
    let vault = t.library_top(P0, "Mutavault");
    t.lands(P0, "Wastes", 2);
    t.answer_choose_moved(P0, &[obj(vault)]);
    t.activate(P0, map, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Mutavault"));
}

#[test]
fn mishras_research_desk_normal_costs_and_timing() {
    cr!("302.1");
    ruling!(
        "Mishra's Research Desk",
        "You must pay all costs and follow all normal timing rules for spells cast using the first ability of Mishra's Research Desk."
    );
    supported("Mishra's Research Desk");
    let mut t = TestGame::new(2);
    let desk = t.battlefield(P0, "Mishra's Research Desk");
    let ids = crate::r_s01_common::stack_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(ids[0])]);
    t.activate(P0, desk, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(ids[0]);
    assert_eq!(t.zone(bears), Zone::Exile);
    // No mana: it can't be cast.
    assert!(!crate::r_s21_common::castable(&mut t, P0, bears));
    t.lands(P0, "Forest", 2);
    assert!(crate::r_s21_common::castable(&mut t, P0, bears));
    // Not at instant speed (with a spell on the stack).
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P1)).go();
    assert!(!crate::r_s21_common::castable(&mut t, P0, bears));
}

/// A 3-player game in which P0 controls P1 during P1's turn (via `take_control`) and P2
/// casts Master Warcraft in P1's beginning of combat step: P2, not P0, chooses P1's
/// attackers.
fn controlled_turn_with_master_warcraft(take_control: fn(&mut TestGame)) {
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P2, "Mountain", 4);
    let mw = t.hand(P2, "Master Warcraft");
    take_control(&mut t);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(mtg_engine::player_control::decider(&t.g, P1), P0);
    t.cast(P2, mw).go();
    t.resolve_all();
    // P0 (deciding for P1) would declare no attackers; P2 makes the Bears attack P0.
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.answer(
        P2,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    let turn = t.g.turn.number;
    assert!(t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers
            && g.turn.stage == mtg_engine::turn::Stage::Priority)
            || g.turn.number != turn
    }));
    t.settle();
    assert!(attacking(&t, bears));
}

#[test]
fn controlling_a_player_another_players_decision_effect_takes_precedence() {
    cr!("723.5", "508.1");
    ruling!(
        "Mindslaver",
        "If an effect causes another player to make decisions that the affected player would normally make (such as Master Warcraft does), that effect takes precedence."
    );
    ruling!(
        "Secret of Bloodbending",
        "If an effect causes another player to make decisions that the affected player would normally make (such as Master Warcraft does), that effect takes precedence."
    );
    ruling!(
        "Emrakul, the Promised End",
        "If an effect causes another player to make decisions that the affected player would normally make (such as Master Warcraft does), that effect takes precedence."
    );
    ruling!(
        "The Dominion Bracelet",
        "If an effect causes another player to make decisions that the affected player would normally make (such as Master Warcraft does), that effect takes precedence."
    );
    supported("Mindslaver");
    supported("Master Warcraft");
    controlled_turn_with_master_warcraft(|t| {
        let ms = t.battlefield(P0, "Mindslaver");
        t.lands(P0, "Wastes", 4);
        t.activate(P0, ms, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
    });
    supported("Emrakul, the Promised End");
    controlled_turn_with_master_warcraft(|t| {
        let e = t.hand(P0, "Emrakul, the Promised End");
        t.lands(P0, "Wastes", 13);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P0, e).go();
        t.resolve_all();
    });
}

#[test]
fn mindslaver_on_yourself_does_nothing() {
    cr!("723.1");
    ruling!(
        "Mindslaver",
        "You could gain control of yourself using Mindslaver, but gaining control of yourself doesn't really do anything."
    );
    let mut t = TestGame::new(2);
    let ms = t.battlefield(P0, "Mindslaver");
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    t.activate(P0, ms, 0, &[Entity::Player(P0)]).unwrap();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(offered[0].contains(&Entity::Player(P0)));
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(mtg_engine::player_control::decider(&t.g, P0), P0);
    assert_eq!(mtg_engine::player_control::decider(&t.g, P1), P1);
}

#[test]
fn memory_jar_exiles_face_down_until_they_return() {
    cr!("406.3", "603.7");
    ruling!(
        "Memory Jar",
        "You can’t look at the cards you exiled until they return to your hand."
    );
    supported("Memory Jar");
    let mut t = TestGame::new(2);
    let jar = t.battlefield(P0, "Memory Jar");
    let kept = t.hand(P0, "Grizzly Bears");
    t.activate(P0, jar, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(kept), Zone::Exile);
    assert!(t.obj_now(kept).face_down);
    assert!(!mtg_engine::player_control::can_see(&t.g, P0, t.g.current(kept)));
    assert_eq!(t.hand_size(P0), 7);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
