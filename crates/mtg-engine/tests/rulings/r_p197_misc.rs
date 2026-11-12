//! Rulings batch P197 — the timing, cost and rules rulings of cards whose effects have
//! no duration: abilities that don't resolve (CR 608.2b), summoning sickness of a land
//! that became a creature (302.6), cost reductions (601.2f, 602.2b), maximum hand size
//! (402.2), additional land plays (305.2), delayed triggers (603.7), simultaneous entry
//! (603.6a) and replacement of mana (614.1a).

use crate::r_p053_common::can_activate_text;
use crate::r_p197_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn lucius_stays_in_the_graveyard_if_its_target_leaves_first() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Lucius the Eternal",
        "If the target creature leaves the battlefield before Lucius the Eternal's ability resolves, Lucius will stay in your graveyard and will not be exiled or return to the battlefield."
    );
    supported("Lucius the Eternal");
    let mut t = TestGame::new(2);
    let lucius = t.battlefield(P0, "Lucius the Eternal");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    kill(&mut t, lucius);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lucius the Eternal"));
    assert!(t.named_on_battlefield("Lucius the Eternal").is_empty());
    // With the target still there: exiled, and back when it leaves.
    let mut t = TestGame::new(2);
    let lucius = t.battlefield(P0, "Lucius the Eternal");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    kill(&mut t, lucius);
    t.resolve_all();
    assert!(t.in_exile("Lucius the Eternal"));
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Lucius the Eternal").len(), 1);
}

#[test]
fn kefnets_last_word_with_an_illegal_target_doesnt_stop_your_lands_untapping() {
    cr!("608.2b", "502.3");
    ruling!(
        "Kefnet's Last Word",
        "If the target permanent is an illegal target by the time Kefnet’s Last Word resolves, the entire spell doesn’t resolve. Your lands will untap during your next untap step as normal."
    );
    for legal in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let lands = t.lands(P0, "Island", 4);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let c = t.hand(P0, "Kefnet's Last Word");
        t.cast(P0, c).target(bears).go();
        assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
        if !legal {
            kill(&mut t, bears);
        }
        t.resolve_all();
        pass_turn(&mut t);
        pass_turn(&mut t);
        let untapped = lands.iter().filter(|l| !t.obj_now(**l).tapped).count();
        assert_eq!(untapped, if legal { 0 } else { 4 }, "legal {legal}");
        if legal {
            assert_eq!(ctl(&t, bears), P0);
        }
    }
}

#[test]
fn cyclopean_giant_stays_in_the_graveyard_if_its_trigger_doesnt_resolve() {
    cr!("608.2b", "603.10a");
    ruling!(
        "Cyclopean Giant",
        "If the triggered ability doesn’t resolve (the targeted land becomes an illegal target, for example), Cyclopean Giant remains in the graveyard."
    );
    supported("Cyclopean Giant");
    for legal in [false, true] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Cyclopean Giant");
        let land = t.battlefield(P1, "Island");
        t.answer_targets(P0, &[obj(land)]);
        kill(&mut t, g);
        if !legal {
            bounce(&mut t, land);
        }
        t.resolve_all();
        assert_eq!(
            t.in_graveyard(P0, "Cyclopean Giant"),
            !legal,
            "legal {legal}"
        );
        if legal {
            assert!(t.in_exile("Cyclopean Giant"));
            assert!(t.obj_now(land).chars.has_subtype("Swamp"));
        }
    }
}

#[test]
fn great_hall_animated_the_turn_it_came_under_your_control_cant_tap_or_attack() {
    cr!("302.6", "305.1");
    ruling!(
        "Great Hall of the Biblioplex",
        "If this land becomes a creature but you haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it that turn."
    );
    supported("Great Hall of the Biblioplex");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let hall = t.battlefield_sick(P0, "Great Hall of the Biblioplex");
    act(&mut t, P0, hall, 2, &[]);
    assert!(t.obj_now(hall).is(mtg_engine::types::CardType::Creature));
    t.g.players[0].mana_pool.mana.clear();
    assert!(t.activate(P0, hall, 0, &[]).is_err());
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, hall));
    // Next turn it can.
    pass_turn(&mut t);
    pass_turn(&mut t);
    // Still a creature (no duration), and now it can tap for mana.
    assert!(t.obj_now(hall).is(mtg_engine::types::CardType::Creature));
    t.activate(P0, hall, 0, &[]).expect("tap for {C}");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
}

/// P0 with Tezzeret, Betrayer of Flesh, Rod of Ruin ("{3}, {T}: 1 damage to any target")
/// and a Sol Ring, and no other mana.
fn tezzeret_setup(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Rod of Ruin");
    t.set_step(P0, Step::PrecombatMain);
    walker(t, P0, "Tezzeret, Betrayer of Flesh", 3);
    let rod = t.battlefield(P0, "Rod of Ruin");
    let ring = t.battlefield(P0, "Sol Ring");
    (rod, ring)
}

#[test]
fn tezzerets_discount_goes_to_the_ability_begun_before_its_mana_abilities() {
    cr!("601.2f", "602.2b", "605.3a");
    ruling!(
        "Tezzeret, Betrayer of Flesh",
        "If you begin to activate an ability that costs mana before activating any mana abilities you need to pay for it, that ability will be the one to get Tezzeret's discount. The total cost for that ability becomes locked in, and then you can activate mana abilities to pay for it."
    );
    // The Rod's {3} becomes {1}, paid by tapping Sol Ring during the activation.
    let mut t = TestGame::new(2);
    let (rod, ring) = tezzeret_setup(&mut t);
    t.activate(P0, rod, 0, &[pl(P1)])
        .expect("{1} payable with Sol Ring");
    assert!(t.obj_now(ring).tapped);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
}

#[test]
fn tezzerets_discount_applies_only_to_the_first_artifact_ability_even_a_mana_ability() {
    cr!("601.2f", "602.2b", "605.1a");
    ruling!(
        "Tezzeret, Betrayer of Flesh",
        "Tezzeret, Betrayer of Flesh's first ability reduces the cost of the first activated ability of an artifact that you begin to activate each turn. The second ability (and any subsequent abilities) of an artifact you activate that turn won't cost {2} less to activate, even if the first ability you activated didn't cost any mana to begin with. Notably, this does not exclude mana abilities."
    );
    // Sol Ring tapped for mana first (the first artifact ability this turn): the Rod then
    // costs its full {3}.
    let mut t = TestGame::new(2);
    let (rod, ring) = tezzeret_setup(&mut t);
    t.activate(P0, ring, 0, &[]).expect("Sol Ring");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 2);
    assert!(
        t.activate(P0, rod, 0, &[pl(P1)]).is_err(),
        "{{3}} with {{C}}{{C}}"
    );
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, rod, 0, &[pl(P1)]).expect("{3} payable now");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Next turn the first artifact ability gets it again.
    pass_turn(&mut t);
    pass_turn(&mut t);
    t.g.players[0].mana_pool.mana.clear();
    mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, rod, 0, &[pl(P1)]).expect("{1} again");
}

#[test]
fn reducing_maximum_hand_size_while_you_have_none() {
    cr!("402.2", "613.11", "611.2a");
    ruling!(
        "Inspired Idea",
        "If you currently have no maximum hand size, then reducing your maximum hand size by three will have no effect. If the effect causing you to have no maximum hand size ends, Inspired Idea's effect will apply at that time."
    );
    ruling!(
        "Praetor's Counsel",
        "Later effects that modify your maximum hand size, but don't set it to a specific number, will have no effect."
    );
    supported("Inspired Idea");
    supported("Spellbook");
    // Spellbook ("You have no maximum hand size"), then Inspired Idea: none; Spellbook
    // leaves: 4.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let book = t.battlefield(P0, "Spellbook");
    cast_at(&mut t, P0, "Inspired Idea", &[]);
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
    exile(&mut t, book);
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, Some(4));
    // Praetor's Counsel, then Inspired Idea: still none, for the rest of the game.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    cast_at(&mut t, P0, "Praetor's Counsel", &[]);
    t.resolve_all();
    cast_at(&mut t, P0, "Inspired Idea", &[]);
    t.resolve_all();
    pass_turn(&mut t);
    t.g.recompute();
    assert_eq!(t.g.player(P0).max_hand_size, None);
}

#[test]
fn two_druid_classes_at_level_two_give_two_additional_land_plays() {
    cr!("305.2", "305.2a", "716.2a");
    ruling!(
        "Druid Class",
        "If you have more than one Druid Class at level 2 or higher, you can play that many additional lands per turn. For example, with two Druid Classes at level 2, you can play three lands during your turn."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..2 {
        let c = t.battlefield(P0, "Druid Class");
        act(&mut t, P0, c, 0, &[]);
        assert_eq!(t.obj_now(c).class_level, 2);
    }
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        t.play_land(P0, *l).expect("a land play left");
    }
    assert!(t.play_land(P0, lands[3]).is_err());
}

#[test]
fn sea_gate_restoration_with_an_empty_hand_draws_one() {
    cr!("121.1", "608.2h");
    ruling!(
        "Sea Gate Restoration // Sea Gate, Reborn",
        "If you have no cards in hand as Sea Gate Restoration resolves, you draw one card."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    lots(&mut t, P0);
    let c = t.hand(P0, "Sea Gate Restoration // Sea Gate, Reborn");
    assert_eq!(t.hand_size(P0), 1);
    t.cast(P0, c).go();
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn liliana_plus_one_with_an_empty_hand() {
    cr!("101.4", "701.9a");
    ruling!(
        "Liliana, Waker of the Dead",
        "If you have no cards in hand, you just don't discard anything. You don't lose 3 life. Other players still discard if able."
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let l = walker(&mut t, P0, "Liliana, Waker of the Dead", 3);
    // P1 has a card, P2 none; P0 none.
    t.hand(P1, "Island");
    act(&mut t, P0, l, 0, &[]);
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Island"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 17);
}

#[test]
fn chance_for_glory_skipped_extra_turn_never_triggers() {
    cr!("614.10", "603.7", "500.7");
    ruling!(
        "Chance for Glory",
        "If you somehow skip the extra turn Chance for Glory gives you or skip that turn’s end step, the delayed triggered ability never triggers."
    );
    supported("Chance for Glory");
    supported("Stranglehold");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stranglehold");
    t.set_step(P0, Step::PrecombatMain);
    cast_at(&mut t, P0, "Chance for Glory", &[]);
    t.resolve_all();
    // The extra turn is skipped: P1's turn comes next, then P0's normal turn ends safely.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P0));
    // Without Stranglehold, P0 loses at the extra turn's end step.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    cast_at(&mut t, P0, "Chance for Glory", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.has_lost(P0), "not this turn's end step");
    t.advance_to(P0, Step::Upkeep);
    assert!(t.g.turn.extra);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P0));
}

#[test]
fn grave_betrayals_of_two_players_the_first_to_resolve_returns_it() {
    cr!("603.7", "101.4", "405.5");
    ruling!(
        "Grave Betrayal",
        "In a multiplayer game, if more than one player controls a Grave Betrayal and a creature controlled by none of those players dies, each Grave Betrayal will create a delayed triggered ability at the beginning of the next end step. The first such ability to resolve will return the creature. Usually, this will be one controlled by the player furthest from the active player in turn order."
    );
    supported("Grave Betrayal");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Grave Betrayal");
    t.battlefield(P2, "Grave Betrayal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    kill(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(ctl(&t, back[0]), P2);
    assert_eq!(t.counters(back[0], "+1/+1"), 1);
    assert!(t.obj_now(back[0]).chars.has_subtype("Zombie"));
}

#[test]
fn kiora_chapter_two_already_tapped_permanents_dont_untap() {
    cr!("714.2b", "502.3", "701.26a");
    ruling!(
        "Kiora Bests the Sea God",
        "Nonland permanents target opponent controls that are already tapped when the second chapter ability resolves won’t be tapped again, but they won’t untap during their controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = blank_saga(&mut t, P0, "Kiora Bests the Sea God");
    t.g.objects[s.0 as usize].counters.insert("lore".into(), 1);
    let tapped = t.battlefield(P1, "Grizzly Bears");
    let untapped = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.g.tap(tapped);
    t.g.tap(land);
    t.answer_targets(P0, &[pl(P1)]);
    add_lore(&mut t, s, 1);
    t.resolve_all();
    assert!(t.obj_now(untapped).tapped);
    pass_turn(&mut t);
    assert!(t.obj_now(tapped).tapped && t.obj_now(untapped).tapped);
    assert!(!t.obj_now(land).tapped);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert!(!t.obj_now(tapped).tapped && !t.obj_now(untapped).tapped);
}

#[test]
fn cocoons_creature_can_still_be_untapped_by_effects() {
    cr!("502.3", "701.26b");
    ruling!(
        "Cocoon",
        "The creature can still be untapped using spells and abilities."
    );
    supported("Cocoon");
    supported("Jace, Ingenious Mind-Mage");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Cocoon", &[obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    // Jace's "+1: Untap all creatures you control."
    let jace = walker(&mut t, P0, "Jace, Ingenious Mind-Mage", 3);
    act(&mut t, P0, jace, 1, &[]);
    assert!(!t.obj_now(bears).tapped);
    // Tapped again, its controller's untap step doesn't untap it (pupa counters remain).
    t.g.tap(bears);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn life_matrix_only_the_creatures_controller_can_remove_the_counter() {
    cr!("602.2", "113.6");
    ruling!(
        "Life Matrix",
        "Once the counter is placed on the creature, the ability to remove it can only be activated by the player who controls the creature."
    );
    supported("Life Matrix");
    let mut t = TestGame::new(2);
    let matrix = t.battlefield(P0, "Life Matrix");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::Upkeep);
    act(&mut t, P0, matrix, 0, &[obj(bears)]);
    assert_eq!(t.counters(bears, "matrix"), 1);
    assert!(!can_activate_text(&mut t, P0, bears, "Regenerate"));
    assert!(can_activate_text(&mut t, P1, bears, "Regenerate"));
    // Control changes: now P0 can, P1 can't.
    give_control(&mut t, bears, P0);
    assert!(can_activate_text(&mut t, P0, bears, "Regenerate"));
    assert!(!can_activate_text(&mut t, P1, bears, "Regenerate"));
}

#[test]
fn quarum_trench_gnomes_replace_only_white_mana() {
    cr!("614.1a", "106.6");
    ruling!(
        "Quarum Trench Gnomes",
        "The ability only replaces white mana produced by the Plains. If a Plains somehow tries to produce mana of a different color (for instance, if you control a Joiner Adept), this effect will not cause it to produce colorless mana."
    );
    supported("Joiner Adept");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let gnomes = t.battlefield(P0, "Quarum Trench Gnomes");
    t.battlefield(P0, "Joiner Adept");
    let plains = t.battlefield(P0, "Plains");
    act(&mut t, P0, gnomes, 0, &[obj(plains)]);
    t.g.players[0].mana_pool.mana.clear();
    // Joiner Adept's ability, red: red.
    choose_words(&mut t, P0, &["R", "red", "Red", "{R}"]);
    crate::r_s06_common::activate_containing(&mut t, P0, plains, "any color").expect("tap");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 0);
    // Its own ability: colorless instead of white.
    t.g.untap(plains);
    t.g.players[0].mana_pool.mana.clear();
    crate::r_s06_common::activate_containing(&mut t, P0, plains, "{W}").expect("tap");
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::W), 0);
}

#[test]
fn hidden_spider_checks_the_spell_for_flying() {
    cr!("603.2", "601.2", "603.4");
    ruling!(
        "Hidden Spider",
        "The ability only triggers if the creature spell has flying, regardless of whether an effect would cause the creature to have flying once it entered."
    );
    supported("Hidden Spider");
    supported("Levitation");
    supported("Wind Drake");
    for (spell, triggers) in [("Grizzly Bears", false), ("Wind Drake", true)] {
        let mut t = TestGame::new(2);
        let spider = t.battlefield(P0, "Hidden Spider");
        // P1's Levitation: "Creatures you control have flying."
        t.battlefield(P1, "Levitation");
        t.set_step(P1, Step::PrecombatMain);
        cast_at(&mut t, P1, spell, &[]);
        t.settle();
        t.resolve_all();
        let c = t.named_on_battlefield(spell)[0];
        assert!(t
            .obj_now(c)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
        assert_eq!(
            t.obj_now(spider).is(mtg_engine::types::CardType::Creature),
            triggers,
            "{spell}"
        );
    }
}

#[test]
fn dread_slaver_does_nothing_if_undying_returned_the_card_first() {
    cr!("603.3b", "608.2b", "400.7");
    ruling!(
        "Dread Slaver",
        "The card will return to the battlefield under your control only if it's still in the graveyard when the ability resolves. If it's not (perhaps because an ability like undying has already returned it to the battlefield), nothing happens."
    );
    supported("Dread Slaver");
    supported("Undying Evil");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let slaver = t.battlefield(P0, "Dread Slaver");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P1, "Undying Evil", &[obj(bears)]);
    t.resolve_all();
    crate::r_s06_common::damage(&mut t, slaver, 3, bears);
    t.settle();
    // P0's (active) trigger goes on the stack first; P1's undying resolves first.
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(ctl(&t, back[0]), P1);
    assert_eq!(t.counters(back[0], "+1/+1"), 1);
    assert!(!t.obj_now(back[0]).chars.has_subtype("Zombie"));
    // Without undying: P0 gets it as a black Zombie.
    let mut t = TestGame::new(2);
    let slaver = t.battlefield(P0, "Dread Slaver");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::damage(&mut t, slaver, 3, bears);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(ctl(&t, back[0]), P0);
    assert!(t.obj_now(back[0]).chars.has_subtype("Zombie"));
}

#[test]
fn hall_of_the_bandit_lord_mana_on_an_additional_cost_gives_haste() {
    cr!("106.6", "601.2f", "702.33a");
    ruling!(
        "Hall of the Bandit Lord",
        "The creature has haste if the mana is spent to cover any cost of the spell, even an additional cost."
    );
    supported("Kavu Titan");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let hall = t.battlefield(P0, "Hall of the Bandit Lord");
    // Kavu Titan ({1}{G}, kicker {2}{G}) kicked: {3}{G}{G}. The mana cost is paid by
    // {G}{G}; the Hall's {C} and two more cover the kicker.
    t.activate(P0, hall, 0, &[]).expect("Hall");
    assert_eq!(t.life(P0), 17);
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    let c = t.hand(P0, "Kavu Titan");
    t.cast(P0, c).kicked(true).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    let kavu = t.named_on_battlefield("Kavu Titan")[0];
    assert_eq!(t.counters(kavu, "+1/+1"), 3);
    assert!(t
        .obj_now(kavu)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn ghouls_night_out_creatures_enter_together_and_see_each_other() {
    cr!("603.6a", "101.4");
    ruling!(
        "Ghouls' Night Out",
        "The creatures will enter the battlefield simultaneously. They will see each other enter for the purposes of triggered abilities."
    );
    supported("Ghouls' Night Out");
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let a = t.graveyard(P0, "Soul Warden");
    let b = t.graveyard(P1, "Soul Warden");
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    cast_at(&mut t, P0, "Ghouls' Night Out", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soul Warden").len(), 2);
    // Each Soul Warden saw the other enter: P0 gained 2 life.
    assert_eq!(t.life(P0), 22);
}
