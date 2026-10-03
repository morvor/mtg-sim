//! Rulings batch P226 — Treasure tokens (CR 111.10a) made and counted by triggered and
//! activated abilities: when conditions are checked (CR 603.4, 608.2h), who creates the
//! tokens, sacrificing Treasures, and reflexive triggers (CR 603.12).

use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s13_common::add;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn treasures(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    with_subtype(t, p, "Treasure")
}

fn make_treasures(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| create_token(t, p, "Treasure")).collect()
}

/// `p` sacrifices the permanent (as an effect would), then settles.
fn sacrifice(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let id = t.g.current(id);
    t.g.sacrifice(id, p);
    t.g.flush_events();
    t.settle();
}

#[test]
fn black_market_tycoon_triggers_without_treasures_and_counts_them_on_resolution() {
    cr!("603.2", "608.2h", "111.10a");
    ruling!(
        "Black Market Tycoon",
        "Black Market Tycoon's ability triggers at the beginning of your upkeep whether or not you actually have any Treasures, and it counts the number of Treasures you have as it resolves."
    );
    supported("Black Market Tycoon");
    // "At the beginning of your upkeep, this creature deals 2 damage to you for each
    // Treasure you control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Black Market Tycoon");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Treasure you control"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // One Treasure as it triggers, two as it resolves: 4 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Black Market Tycoon");
    make_treasures(&mut t, P0, 1);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    make_treasures(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn burdened_aerialist_flies_whenever_you_sacrifice_any_token() {
    cr!("603.2", "701.21a", "111.1");
    ruling!(
        "Burdened Aerialist",
        "Burdened Aerialist's ability will trigger whenever you sacrifice a token for any reason, not just when you sacrifice Treasure tokens."
    );
    supported("Burdened Aerialist");
    // "Whenever you sacrifice a token, this creature gains flying until end of turn."
    let mut t = TestGame::new(2);
    let aerialist = t.battlefield(P0, "Burdened Aerialist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    sacrifice(&mut t, P0, bears);
    t.resolve_all();
    assert!(!t
        .obj_now(aerialist)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    let food = create_token(&mut t, P0, "Food");
    sacrifice(&mut t, P0, food);
    t.resolve_all();
    assert!(t
        .obj_now(aerialist)
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn captain_lannery_storm_sees_treasures_sacrificed_for_any_reason_but_not_to_cast_her() {
    cr!("603.2", "601.2g", "605.3a", "111.10a");
    ruling!(
        "Captain Lannery Storm",
        "Captain Lannery Storm's last ability triggers whenever you sacrifice a Treasure for any reason, not just to activate a Treasure's mana ability."
    );
    ruling!(
        "Captain Lannery Storm",
        "You can activate the mana ability of a Treasure even if you have nothing to spend that mana on."
    );
    ruling!(
        "Captain Lannery Storm",
        "If you sacrifice Treasures to cast Captain Lannery Storm, its last ability won't trigger for those Treasures."
    );
    supported("Captain Lannery Storm");
    // "Whenever you sacrifice a Treasure, Captain Lannery Storm gets +1/+0 until end of
    // turn."
    let mut t = TestGame::new(2);
    let storm = t.battlefield(P0, "Captain Lannery Storm");
    // A Treasure's mana ability, with nothing to spend the mana on.
    let tr = make_treasures(&mut t, P0, 1);
    t.activate(P0, tr[0], 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    t.resolve_all();
    assert_eq!(t.pt(storm), (3, 2));
    // Ruthless Knave's "Sacrifice three Treasures: Draw a card."
    let knave = t.battlefield(P0, "Ruthless Knave");
    make_treasures(&mut t, P0, 3);
    activate_containing(&mut t, P0, knave, "Sacrifice three Treasures").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(storm), (6, 2));
    // Casting her with two Treasures and a Mountain: no triggers.
    let mut t = TestGame::new(2);
    make_treasures(&mut t, P0, 2);
    t.lands(P0, "Mountain", 1);
    let card = t.hand(P0, "Captain Lannery Storm");
    t.cast(P0, card).go();
    assert!(treasures(&t, P0).is_empty());
    t.resolve_all();
    let storm = t.named_on_battlefield("Captain Lannery Storm")[0];
    assert_eq!(t.pt(storm), (2, 2));
}

#[test]
fn ruthless_knaves_treasures_are_gone_once_sacrificed_for_its_cost() {
    cr!("602.2b", "118.3", "701.21a");
    ruling!(
        "Ruthless Knave",
        "The Treasures you sacrifice to activate Ruthless Knave's last ability can't also be sacrificed for mana."
    );
    supported("Ruthless Knave");
    let mut t = TestGame::new(2);
    let knave = t.battlefield(P0, "Ruthless Knave");
    // Two Treasures aren't enough.
    make_treasures(&mut t, P0, 2);
    assert!(activate_containing(&mut t, P0, knave, "Sacrifice three Treasures").is_err());
    make_treasures(&mut t, P0, 1);
    activate_containing(&mut t, P0, knave, "Sacrifice three Treasures").unwrap();
    assert!(treasures(&t, P0).is_empty());
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // "{2}{B}, Sacrifice a creature: Create two Treasure tokens."
    t.lands(P0, "Swamp", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, knave, "Sacrifice a creature").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(treasures(&t, P0).len(), 2);
}

/// P0's Battle Angels of Tyr deals combat damage to P1 unblocked, and its trigger
/// resolves.
fn angels_connect(t: &mut TestGame) {
    let angels = t.battlefield(P0, "Battle Angels of Tyr");
    t.attack(&[(angels, Entity::Player(P1))], &[]);
    t.resolve_all();
}

#[test]
fn battle_angels_of_tyr_checks_each_condition_on_its_own() {
    cr!("608.2c", "510.2");
    ruling!(
        "Battle Angels of Tyr",
        "Each condition is checked, even if you don't get the benefit of one or more of them. You can create a Treasure token even if you didn't draw a card, for example."
    );
    supported("Battle Angels of Tyr");
    // "... draw a card if that player has more cards in hand than each other player.
    // Then you create a Treasure token if that player controls more lands than each
    // other player. Then you gain 3 life if that player has more life than each other
    // player."
    // P1: fewer cards, more lands, less life after the damage: only the Treasure.
    let mut t = TestGame::new(2);
    t.hand(P0, "Forest");
    t.lands(P1, "Plains", 2);
    angels_connect(&mut t);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(treasures(&t, P0).len(), 1);
    assert_eq!(t.life(P0), 20);
    // P1: more cards, no lands, more life: the card and the life, no Treasure.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    t.g.players[0].life = 10;
    angels_connect(&mut t);
    assert_eq!(t.hand_size(P0), 1);
    assert!(treasures(&t, P0).is_empty());
    assert_eq!(t.life(P0), 13);
}

#[test]
fn tempting_contract_asks_each_opponent_in_turn_order_then_you_create_yours() {
    cr!("101.4", "608.2c", "111.10a");
    ruling!(
        "Tempting Contract",
        "Each opponent in turn order decides whether or not to create a Treasure token. They will each know the decisions of players before them. Those Treasure tokens are all created at the same time. Then you create your Treasure tokens all at the same time."
    );
    supported("Tempting Contract");
    // "At the beginning of your upkeep, each opponent may create a Treasure token. For
    // each opponent who does, you create a Treasure token."
    for (p1, p2) in [(true, false), (true, true), (false, false)] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Tempting Contract");
        t.advance_to(P0, Step::Upkeep);
        let from = t.asked().len();
        t.answer_yes(P1, p1);
        t.answer_yes(P2, p2);
        t.resolve_all();
        let asked: Vec<PlayerId> = asked_since(&t, from)
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .map(|(p, _)| p)
            .collect();
        assert_eq!(asked, vec![P1, P2]);
        assert_eq!(treasures(&t, P1).len(), usize::from(p1));
        assert_eq!(treasures(&t, P2).len(), usize::from(p2));
        assert_eq!(treasures(&t, P0).len(), usize::from(p1) + usize::from(p2));
    }
}

#[test]
fn revel_in_riches_sees_a_creature_dying_with_it_and_checks_ten_treasures_twice() {
    cr!("603.10a", "603.4", "104.2b");
    ruling!(
        "Revel in Riches",
        "If an opponent's creature dies at the same time that Revel in Riches is destroyed, you'll get a Treasure."
    );
    ruling!(
        "Revel in Riches",
        "If you don't control ten Treasures as the second ability of Revel in Riches resolves, you won't win the game."
    );
    ruling!(
        "Revel in Riches",
        "If you don't control ten Treasures as your upkeep begins, the second ability of Revel in Riches won't trigger. You can't take any actions during your turn before your upkeep begins."
    );
    supported("Revel in Riches");
    // "Whenever a creature an opponent controls dies, create a Treasure token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Revel in Riches");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Planar Cleansing");
    let c = t.hand(P0, "Planar Cleansing");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Revel in Riches"));
    assert_eq!(treasures(&t, P0).len(), 1);
    // "At the beginning of your upkeep, if you control ten or more Treasures, you win
    // the game." Nine: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Revel in Riches");
    make_treasures(&mut t, P0, 9);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Ten as it triggers, nine as it resolves: no win.
    let tenth = make_treasures(&mut t, P0, 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    sacrifice(&mut t, P0, tenth[0]);
    t.resolve_all();
    assert!(!t.has_lost(P1));
    // Ten both times: P0 wins.
    make_treasures(&mut t, P0, 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn contested_game_balls_counter_check_happens_only_as_its_ability_resolves() {
    cr!("608.2c", "122.1");
    ruling!(
        "Contested Game Ball",
        "If point counters are put on Contested Game Ball some way other than its last ability, you won't sacrifice it or create a Treasure token. The check for five or more point counters happens only as the activated ability resolves."
    );
    // (Its first ability doesn't compile; the activated one does.) "{2}, {T}: Draw a card
    // and put a point counter on this artifact. Then if it has five or more point
    // counters on it, sacrifice it and create a Treasure token."
    let mut t = TestGame::new(2);
    let ball = t.battlefield(P0, "Contested Game Ball");
    add(&mut t, ball, "point", 5);
    t.settle();
    assert!(t.on_battlefield(ball));
    assert!(treasures(&t, P0).is_empty());
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, ball, "point counter").unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(!t.on_battlefield(ball));
    assert_eq!(treasures(&t, P0).len(), 1);
    // Three counters: the fourth doesn't do it.
    let mut t = TestGame::new(2);
    let ball = t.battlefield(P0, "Contested Game Ball");
    add(&mut t, ball, "point", 3);
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, ball, "point counter").unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(ball));
    assert_eq!(t.counters(ball, "point"), 4);
    assert!(treasures(&t, P0).is_empty());
}

#[test]
fn edward_kenway_counts_each_tapped_permanent_once() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Edward Kenway",
        "If a tapped permanent you control has more than one of the subtypes listed in Edward’s first ability, count it only once when determining how many Treasure tokens to create."
    );
    supported("Edward Kenway");
    // "At the beginning of your end step, create a Treasure token for each tapped
    // Assassin, Pirate, and/or Vehicle you control." Edward is an Assassin Pirate.
    let mut t = TestGame::new(2);
    let edward = t.battlefield(P0, "Edward Kenway");
    let knave = t.battlefield(P0, "Ruthless Knave");
    t.battlefield(P0, "Captain Lannery Storm");
    t.g.objects[edward.0 as usize].tapped = true;
    t.g.objects[knave.0 as usize].tapped = true;
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(treasures(&t, P0).len(), 2);
}

#[test]
fn bonehoard_dracosaur_makes_one_dinosaur_and_one_treasure_at_most() {
    cr!("608.2c", "111.1");
    ruling!(
        "Bonehoard Dracosaur",
        "If you exile a land card and a nonland card, you'll create a Dinosaur token, then a Treasure token."
    );
    ruling!(
        "Bonehoard Dracosaur",
        "If you exile two land cards, you'll create only one Dinosaur token. Similarly, if you exile two nonland cards, you'll create only one Treasure token."
    );
    supported("Bonehoard Dracosaur");
    // "At the beginning of your upkeep, exile the top two cards of your library. You may
    // play them this turn. If you exiled a land card this way, create a 3/1 red Dinosaur
    // creature token. If you exiled a nonland card this way, create a Treasure token."
    for (top, dinos, trs) in [
        (["Forest", "Grizzly Bears"], 1, 1),
        (["Forest", "Island"], 1, 0),
        (["Grizzly Bears", "Lightning Bolt"], 0, 1),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Bonehoard Dracosaur");
        t.advance_to(P1, Step::Upkeep);
        stack_library(&mut t, P0, &top);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        let d = with_subtype(&t, P0, "Dinosaur")
            .into_iter()
            .filter(|o| t.obj(*o).is_token())
            .collect::<Vec<_>>();
        let tr = treasures(&t, P0);
        assert_eq!((d.len(), tr.len()), (dinos, trs), "{top:?}");
        if dinos == 1 && trs == 1 {
            // The Dinosaur first, then the Treasure.
            assert!(t.obj(d[0]).timestamp < t.obj(tr[0]).timestamp);
        }
    }
}

#[test]
fn heartless_pillage_makes_one_treasure_after_any_attack_even_with_no_discards() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Heartless Pillage",
        "If you've attacked with a creature this turn, you'll get a Treasure even if the target opponent discards one or zero cards."
    );
    ruling!(
        "Heartless Pillage",
        "You create only one Treasure token if you attacked this turn, no matter how many creatures you attacked with beyond the first."
    );
    supported("Heartless Pillage");
    // "Target opponent discards two cards. Raid — If you attacked this turn, create a
    // Treasure token." P1 has no cards in hand; P0 attacked with two creatures.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    give_mana_for(&mut t, P0, "Heartless Pillage");
    let card = t.hand(P0, "Heartless Pillage");
    t.cast(P0, card).target(P1).go();
    t.resolve_all();
    assert_eq!(treasures(&t, P0).len(), 1);
    // No attack: no Treasure; P1 discards.
    let mut t = TestGame::new(2);
    t.hand(P1, "Forest");
    give_mana_for(&mut t, P0, "Heartless Pillage");
    let card = t.hand(P0, "Heartless Pillage");
    t.cast(P0, card).target(P1).go();
    t.resolve_all();
    assert!(treasures(&t, P0).is_empty());
    assert!(t.in_graveyard(P1, "Forest"));
}

#[test]
fn pain_distributors_treasure_comes_after_the_spell_is_paid_for() {
    cr!("601.2h", "603.3", "111.10a");
    ruling!(
        "Pain Distributor",
        "Pain Distributor’s ability resolves before the spell that caused it to trigger, but not before you have to pay for the original spell. You won’t be able to use the Treasure token you create to pay for it."
    );
    supported("Pain Distributor");
    // "Whenever a player casts their first spell each turn, they create a Treasure
    // token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pain Distributor");
    // With no mana, the Treasure doesn't help pay.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert!(treasures(&t, P0).is_empty());
    t.lands(P0, "Mountain", 1);
    let spell = t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(treasures(&t, P0).len(), 1);
    assert_eq!(*t.g.stack.last().unwrap(), spell);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The second spell: no trigger.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn safana_needs_the_initiative_as_her_trigger_triggers_and_resolves() {
    cr!("603.4", "726.1", "309.7");
    ruling!(
        "Safana, Calimport Cutthroat",
        "Safana's second ability triggers only if you have the initiative as your end step begins. If the ability does trigger, it will check if you have the initiative again as it tries to resolve. If you don't still have the initiative at that time, the ability won't resolve and none of its effects will happen. Notably, if you don't have the initiative as your end step begins but you have completed a dungeon, the ability won't trigger and you won't create any Treasures."
    );
    supported("Safana, Calimport Cutthroat");
    // "At the beginning of your end step, if you have the initiative, create a Treasure
    // token. Create three of those tokens instead if you've completed a dungeon."
    // Initiative as it triggers, lost before it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Safana, Calimport Cutthroat");
    t.g.initiative = Some(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.initiative = Some(P1);
    t.resolve_all();
    assert!(treasures(&t, P0).is_empty());
    // A completed dungeon but no initiative: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Safana, Calimport Cutthroat");
    t.g.players[0].dungeons_completed = 1;
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Both: three Treasures. Just the initiative: one.
    for (dungeon, n) in [(1, 3), (0, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Safana, Calimport Cutthroat");
        t.g.players[0].dungeons_completed = dungeon;
        t.g.initiative = Some(P0);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(treasures(&t, P0).len(), n);
    }
}

#[test]
fn shiny_impetus_gives_its_controller_the_treasure() {
    cr!("603.2", "701.15a", "111.10a");
    ruling!(
        "Shiny Impetus",
        "Shiny Impetus causes you to create a Treasure token, not the attacking creature's controller."
    );
    supported("Shiny Impetus");
    // "Enchanted creature gets +2/+2 and is goaded." / "Whenever enchanted creature
    // attacks, you create a Treasure token."
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Shiny Impetus", bears);
    assert_eq!(t.pt(bears), (4, 4));
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    t.resolve_all();
    assert_eq!(treasures(&t, P0).len(), 1);
    assert!(treasures(&t, P1).is_empty());
}

#[test]
fn goldvein_hydra_uses_its_last_known_power() {
    cr!("603.10a", "608.2h", "111.10a");
    ruling!(
        "Goldvein Hydra",
        "Use Goldvein Hydra's power as it last existed on the battlefield to determine how many Treasure tokens to create."
    );
    supported("Goldvein Hydra");
    // "This creature enters with X +1/+1 counters on it." / "When this creature dies,
    // create a number of tapped Treasure tokens equal to its power."
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let card = t.hand(P0, "Goldvein Hydra");
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    let hydra = t.named_on_battlefield("Goldvein Hydra")[0];
    assert_eq!(t.pt(hydra), (3, 3));
    let o = t.obj_now(hydra);
    for kw in [
        mtg_engine::keywords::KeywordKind::Vigilance,
        mtg_engine::keywords::KeywordKind::Trample,
        mtg_engine::keywords::KeywordKind::Haste,
    ] {
        assert!(o.has_keyword(kw));
    }
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(hydra).go();
    t.resolve_all();
    assert_eq!(t.pt(hydra), (6, 6));
    destroy(&mut t, hydra);
    t.resolve_all();
    let tr = treasures(&t, P0);
    assert_eq!(tr.len(), 6);
    assert!(tr.iter().all(|o| t.obj(*o).tapped));
}

#[test]
fn generous_plunderers_second_treasure_comes_from_a_reflexive_trigger() {
    cr!("603.12", "603.3d", "700.13", "111.10a");
    ruling!(
        "Generous Plunderer",
        "You don’t choose a target for Generous Plunderer’s first triggered ability at the time it triggers. Rather, a second “reflexive” ability triggers when you create a Treasure token this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal. Notably, the triggered ability isn’t a crime, but the reflexive triggered ability potentially is."
    );
    supported("Generous Plunderer");
    // "At the beginning of your upkeep, you may create a Treasure token. When you do,
    // target opponent creates a tapped Treasure token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Generous Plunderer");
    t.advance_to(P1, Step::Upkeep);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // No target was chosen as it triggered: no crime yet.
    assert!(target_choices(&t, from).is_empty());
    assert_eq!(crimes(&t), 0);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(treasures(&t, P0).len(), 1);
    // The reflexive trigger is on the stack, with its target; P1 hasn't created one yet.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(target_choices(&t, from).len(), 1);
    assert!(treasures(&t, P1).is_empty());
    // Targeting an opponent with it is a crime (CR 700.13).
    assert_eq!(crimes(&t), 1);
    t.resolve_all();
    let p1 = treasures(&t, P1);
    assert_eq!(p1.len(), 1);
    assert!(t.obj(p1[0]).tapped);
    // Declining: no reflexive trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Generous Plunderer");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert!(treasures(&t, P0).is_empty() && treasures(&t, P1).is_empty());
}

/// Crimes P0 committed this turn (CR 700.13).
fn crimes(t: &TestGame) -> u32 {
    t.g.history.crimes.get(&P0).copied().unwrap_or(0)
}

/// The target choices asked since decision `from`.
fn target_choices(t: &TestGame, from: usize) -> Vec<Decision> {
    asked_since(t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(_, d)| d)
        .collect()
}

#[test]
fn widespread_thieving_can_use_its_own_treasure_to_pay_for_the_hideaway_card() {
    cr!("608.2c", "702.75a", "605.3a");
    ruling!(
        "Widespread Thieving",
        "You may use the Treasure token created by the first part of Widespread Thieving's last ability to pay part of the {W}{U}{B}{R}{G} cost in the second part of that ability."
    );
    supported("Widespread Thieving");
    // "Hideaway 5" / "Whenever you cast a multicolored spell, create a Treasure token.
    // Then you may pay {W}{U}{B}{R}{G}. If you do, you may play the exiled card without
    // paying its mana cost."
    let mut t = TestGame::new(2);
    let top = stack_library(
        &mut t,
        P0,
        &["Forest", "Island", "Serra Angel", "Swamp", "Mountain"],
    );
    t.answer_choose(P0, &[Entity::Object(top[2])]);
    t.enter(P0, "Widespread Thieving");
    t.resolve_all();
    assert_eq!(t.zone(top[2]), mtg_engine::object::Zone::Exile);
    // Lands for Lightning Helix ({R}{W}) and for {W}{U}{B}{R} — the {G} comes from the
    // Treasure.
    for land in [
        "Mountain", "Plains", "Plains", "Island", "Swamp", "Mountain",
    ] {
        t.lands(P0, land, 1);
    }
    let helix = t.hand(P0, "Lightning Helix");
    t.cast(P0, helix).target(P1).go();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(untapped_count(&t), 0);
    assert!(treasures(&t, P0).is_empty());
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1);
    assert_eq!(t.life(P1), 17);
}

fn untapped_count(t: &TestGame) -> usize {
    crate::r_s04_common::untapped_lands(t, P0)
}

#[test]
fn ragavan_makes_a_treasure_even_if_the_library_is_empty() {
    cr!("608.2c", "510.2", "111.10a");
    ruling!(
        "Ragavan, Nimble Pilferer",
        "You'll create a Treasure token even if that player has no cards left in their library to exile."
    );
    supported("Ragavan, Nimble Pilferer");
    // "Whenever Ragavan deals combat damage to a player, create a Treasure token and
    // exile the top card of that player's library. Until end of turn, you may cast that
    // card."
    let mut t = TestGame::new(2);
    let ragavan = t.battlefield(P0, "Ragavan, Nimble Pilferer");
    t.g.players[1].library.clear();
    t.attack(&[(ragavan, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(treasures(&t, P0).len(), 1);
    assert!(t.g.exile.is_empty());
    // With a library: the Treasure and an exiled card.
    let mut t = TestGame::new(2);
    let ragavan = t.battlefield(P0, "Ragavan, Nimble Pilferer");
    t.attack(&[(ragavan, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(treasures(&t, P0).len(), 1);
    assert_eq!(t.g.exile.len(), 1);
}
