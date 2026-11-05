//! Rulings batch P060 — draw effects with a target: if the target is illegal as the spell
//! or ability tries to resolve, it doesn't resolve and no one draws (CR 608.2b); with no
//! legal target the triggered ability is removed from the stack (CR 603.3d); with one
//! target illegal among several, the others are still affected (CR 608.2b).

use crate::r_p060_common::*;
use crate::r_p007_common::loyalty;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// How a "you and target opponent each draw" ability is put on the stack in a fresh game:
/// `start` puts it there (with P1 as its target when asked).
struct Case {
    card: &'static str,
    /// Cards P0 and P1 draw when it resolves.
    draws: (usize, usize),
    start: fn(&mut TestGame),
}

fn farsight(t: &mut TestGame) {
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Farsight Adept");
}

fn love_song(t: &mut TestGame) {
    // Read ahead: start at chapter I.
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Love Song of Night and Day");
}

fn fall_of_the_first(t: &mut TestGame) {
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Fall of the First Civilization");
}

fn splinter(t: &mut TestGame) {
    let s = t.battlefield(P0, "Splinter, Aging Champion");
    t.answer_targets(P0, &[pl(P1)]);
    destroy(t, s);
}

fn flumph(t: &mut TestGame) {
    let f = t.battlefield(P0, "Flumph");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(f).go();
    t.answer_targets(P0, &[pl(P1)]);
    t.settle();
    t.g.resolve_top();
}

fn secret_rendezvous(t: &mut TestGame) {
    give_mana_for(t, P0, "Secret Rendezvous");
    let s = t.hand(P0, "Secret Rendezvous");
    t.cast(P0, s).target(pl(P1)).go();
}

const CASES: &[Case] = &[
    Case {
        card: "Farsight Adept",
        draws: (1, 1),
        start: farsight,
    },
    Case {
        card: "Love Song of Night and Day",
        draws: (2, 2),
        start: love_song,
    },
    Case {
        card: "Fall of the First Civilization",
        draws: (2, 2),
        start: fall_of_the_first,
    },
    Case {
        card: "Splinter, Aging Champion",
        draws: (1, 1),
        start: splinter,
    },
    Case {
        card: "Flumph",
        draws: (1, 1),
        start: flumph,
    },
    Case {
        card: "Secret Rendezvous",
        draws: (3, 3),
        start: secret_rendezvous,
    },
];

/// Runs `case`: P1 gets hexproof (Leyline of Sanctity) `before` the ability is put on the
/// stack, after it (in response), or never. Returns the cards P0 and P1 drew.
fn run(case: &Case, hexproof: Option<bool>) -> (usize, usize) {
    supported(case.card);
    let mut t = TestGame::new(2);
    if hexproof == Some(true) {
        player_hexproof(&mut t, P1);
    }
    let before = libraries(&t);
    (case.start)(&mut t);
    t.settle();
    if hexproof == Some(true) && case.card != "Secret Rendezvous" {
        // No legal target: the triggered ability is removed from the stack.
        assert_eq!(t.stack_len(), 0, "{}", case.card);
    } else {
        assert_eq!(t.stack_len(), 1, "{}", case.card);
    }
    if hexproof == Some(false) {
        player_hexproof(&mut t, P1);
    }
    t.resolve_all();
    let d = drawn_since(&t, &before);
    (d[0], d[1])
}

#[test]
fn you_and_target_opponent_each_draw_only_if_the_target_is_legal() {
    cr!("608.2b", "603.3d", "702.11b");
    ruling!(
        "Farsight Adept",
        "If Farsight Adept’s ability can’t legally target an opponent, or if the ability’s target becomes illegal while the ability is on the stack, you don’t draw a card."
    );
    ruling!(
        "Love Song of Night and Day",
        "If Love Song of Night and Day’s first chapter ability can’t legally target an opponent, or if the ability’s target becomes illegal while the ability is on the stack, you don’t draw any cards."
    );
    ruling!(
        "Fall of the First Civilization",
        "If the target of Fall of the First Civilization’s first chapter ability is an illegal target as the ability tries to resolve, it won’t resolve and none of its effects will happen. No player will draw cards."
    );
    ruling!(
        "Splinter, Aging Champion",
        "If the target player is an illegal target when Splinter's last ability tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    ruling!(
        "Flumph",
        "If the opponent is somehow an illegal target for Flumph's ability when it tries to resolve, neither player will draw a card."
    );
    ruling!(
        "Secret Rendezvous",
        "If the opponent is an illegal target as Secret Rendezvous tries to resolve, neither player will draw cards."
    );
    for case in CASES {
        // A legal target: both draw.
        assert_eq!(run(case, None), case.draws, "{}: legal target", case.card);
        // The target becomes illegal in response: no one draws.
        assert_eq!(run(case, Some(false)), (0, 0), "{}: illegal", case.card);
    }
    // Farsight Adept and Love Song: no opponent can be targeted at all, so the ability
    // isn't put on the stack and you don't draw either.
    for case in &CASES[..2] {
        assert_eq!(run(case, Some(true)), (0, 0), "{}: no target", case.card);
    }
}

/// P0's Nin, the Pain Artist with lands for X; activates it at `target` (not resolved).
fn activate_nin(t: &mut TestGame, x: i64, target: ObjectId) {
    supported("Nin, the Pain Artist");
    let nin = t.battlefield(P0, "Nin, the Pain Artist");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.activate(P0, nin, 0, &[obj(target)]).unwrap();
}

#[test]
fn nin_draws_only_if_the_creature_is_still_a_legal_target() {
    cr!("608.2b", "602.2b");
    ruling!(
        "Nin, the Pain Artist",
        "If the creature is an illegal target when Nin's ability tries to resolve, it won't resolve and none of its effects will happen. No player will draw cards."
    );
    // Positive control: 2 damage, and the controller draws two.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    activate_nin(&mut t, 2, giant);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(drawn_since(&t, &before), vec![0, 2]);
    // The creature is destroyed in response: no one draws.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    activate_nin(&mut t, 2, giant);
    destroy(&mut t, giant);
    let before = libraries(&t);
    t.resolve_all();
    assert!(t.stack.is_empty());
    assert_eq!(drawn_since(&t, &before), vec![0, 0]);
}

#[test]
fn nin_draws_x_even_if_the_damage_is_prevented_or_redirected() {
    cr!("608.2c", "615.1", "614.1a");
    ruling!(
        "Nin, the Pain Artist",
        "If the ability resolves, but the damage is prevented or redirected, the controller of the targeted creature will draw X cards, regardless of how much damage was dealt or what the damage was ultimately dealt to."
    );
    // Prevented: Mending Hands prevents the next 4 damage to the Giant.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Plains", 1);
    let mh = t.hand(P1, "Mending Hands");
    t.cast(P1, mh).target(giant).go();
    t.resolve_all();
    activate_nin(&mut t, 3, giant);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0, "prevented");
    assert_eq!(drawn_since(&t, &before), vec![0, 3]);
    // Redirected: P1's Palisade Giant takes the damage dealt to P1's other permanents.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let palisade = t.battlefield(P1, "Palisade Giant");
    activate_nin(&mut t, 3, giant);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0, "redirected away");
    assert_eq!(t.obj_now(palisade).damage, 3, "redirected to Palisade Giant");
    assert_eq!(drawn_since(&t, &before), vec![0, 3]);
}

#[test]
fn nin_draws_before_the_lethally_damaged_creature_is_destroyed() {
    cr!("704.5g", "704.3");
    ruling!(
        "Nin, the Pain Artist",
        "If the damage is lethal, the cards are drawn before the creature is destroyed (the next time state-based actions are checked)."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    activate_nin(&mut t, 3, giant);
    t.settle();
    let before = libraries(&t);
    t.g.resolve_top();
    // The ability has finished resolving: the cards are drawn and the Giant, with lethal
    // damage marked, is still on the battlefield until state-based actions are checked.
    assert_eq!(drawn_since(&t, &before), vec![0, 3]);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 3);
    t.settle();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn red_death_illegal_target_no_mana_and_no_draw() {
    cr!("608.2b", "605.5a", "701.15a");
    ruling!(
        "Red Death, Shipwrecker",
        "If the target creature is an illegal target as Red Death’s ability tries to resolve, it won’t resolve and none of its effects happen. You won’t add any mana, and the creature’s controller won’t draw a card."
    );
    ruling!(
        "Red Death, Shipwrecker",
        "Red Death’s ability isn’t a mana ability even though it causes you to add mana. It uses the stack and can be responded to."
    );
    supported("Red Death, Shipwrecker");
    for destroyed in [false, true] {
        let mut t = TestGame::new(2);
        let rd = t.battlefield(P0, "Red Death, Shipwrecker");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let before = libraries(&t);
        t.activate(P0, rd, 0, &[obj(bears)]).unwrap();
        // It uses the stack: no mana yet, and P1 gets priority to respond.
        assert_eq!(t.stack_len(), 1);
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
        if destroyed {
            destroy(&mut t, bears);
        }
        t.g.resolve_top();
        let red = t.g.player(P0).mana_pool.total();
        if destroyed {
            assert_eq!(red, 0, "no mana");
            assert_eq!(drawn_since(&t, &before), vec![0, 0]);
        } else {
            assert_eq!(red, 1, "{{R}} added");
            assert_eq!(drawn_since(&t, &before), vec![0, 1]);
            assert!(t.obj_now(bears).goaded_by.contains(&P0));
        }
    }
}

#[test]
fn ob_nixilis_draws_if_the_target_is_legal_even_if_not_destroyed() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Ob Nixilis, the Hate-Twisted",
        "If the target creature is an illegal target by the time Ob Nixilis's last ability tries to resolve, the ability doesn't resolve. No player draws two cards. If the target is legal but not destroyed (most likely because it has indestructible), its controller does draw two cards."
    );
    supported("Ob Nixilis, the Hate-Twisted");
    // Bears: destroyed, P1 draws two. Darksteel Myr: not destroyed, P1 still draws two.
    // Bears destroyed in response: no one draws.
    for (name, respond, drawn, dies) in [
        ("Grizzly Bears", false, 2, true),
        ("Darksteel Myr", false, 2, false),
        ("Grizzly Bears", true, 0, true),
    ] {
        let mut t = TestGame::new(2);
        let ob = pw(&mut t, P0, "Ob Nixilis, the Hate-Twisted", 5);
        let c = t.battlefield(P1, name);
        loyalty(&mut t, P0, ob, 0, &[obj(c)]);
        if respond {
            destroy(&mut t, c);
        }
        let before = libraries(&t);
        t.resolve_all();
        assert_eq!(drawn_since(&t, &before), vec![0, drawn], "{name}");
        assert_eq!(t.in_graveyard(P1, name), dies, "{name}");
        // Ob Nixilis's first ability: 1 damage for each card P1 drew.
        assert_eq!(t.life(P1), 20 - drawn as i32, "{name}");
    }
}

#[test]
fn ob_nixilis_gone_before_his_ability_resolves_doesnt_trigger_on_the_draws() {
    cr!("603.2", "704.5i", "608.2h");
    ruling!(
        "Ob Nixilis, the Hate-Twisted",
        "If Ob Nixilis leaves the battlefield before his last ability resolves, most likely because he only had 2 loyalty when you activated the ability, his first ability won't exist to trigger when the creature's controller draws two cards."
    );
    let mut t = TestGame::new(2);
    let ob = pw(&mut t, P0, "Ob Nixilis, the Hate-Twisted", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    loyalty(&mut t, P0, ob, 0, &[obj(bears)]);
    t.settle();
    // 0 loyalty: he's put into the graveyard before the ability resolves.
    assert!(t.in_graveyard(P0, "Ob Nixilis, the Hate-Twisted"));
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 2]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20, "no Ob Nixilis to deal damage");
}

#[test]
fn glen_elendra_guardian_no_draw_if_the_spell_is_gone() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Glen Elendra Guardian",
        "If the target noncreature spell is an illegal target as Glen Elendra Guardian's last ability tries to resolve (probably because that spell already left the stack due to another effect), it won't resolve and none of its effects will happen. No player will draw a card."
    );
    supported("Glen Elendra Guardian");
    for other_counter in [false, true] {
        let mut t = TestGame::new(2);
        let glen = t.enter(P0, "Glen Elendra Guardian");
        t.lands(P0, "Island", 4);
        t.lands(P1, "Mountain", 1);
        let shock = t.hand(P1, "Shock");
        t.cast(P1, shock).target(pl(P0)).go();
        t.activate(P0, glen, 0, &[obj(shock)]).unwrap();
        let before = libraries(&t);
        if other_counter {
            // Counterspell counters the Shock first.
            let cs = t.hand(P0, "Counterspell");
            t.cast(P0, cs).target(shock).go();
            t.resolve();
            assert!(t.in_graveyard(P1, "Shock"));
        }
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Shock"));
        assert_eq!(t.life(P0), 20);
        let drawn = if other_counter { 0 } else { 1 };
        assert_eq!(drawn_since(&t, &before), vec![0, drawn]);
    }
}

#[test]
fn thief_of_existence_gains_nothing_if_its_target_is_gone() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Thief of Existence",
        "If the target of Thief of Existence's triggered ability is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. Thief of Existence won't gain another ability."
    );
    supported("Thief of Existence");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P1, "Sol Ring");
        give_mana_for(&mut t, P0, "Thief of Existence");
        let thief = t.hand(P0, "Thief of Existence");
        t.answer_targets(P0, &[obj(ring)]);
        t.cast(P0, thief).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "the cast trigger is on the stack");
        if gone {
            destroy(&mut t, ring);
        }
        t.resolve_all();
        assert!(t.on_battlefield(thief));
        assert_eq!(t.in_exile("Sol Ring"), !gone);
        // The Thief leaves: only if it gained the ability does P1 draw.
        let before = libraries(&t);
        t.answer_targets(P0, &[pl(P1)]);
        destroy(&mut t, thief);
        t.resolve_all();
        let drawn = if gone { 0 } else { 1 };
        assert_eq!(drawn_since(&t, &before), vec![0, drawn], "gone: {gone}");
    }
}

#[test]
fn jace_memory_adept_plus_one_targets_and_draws() {
    cr!("608.2b", "701.17a", "121.1");
    ruling!(
        "Jace, Memory Adept",
        "If you activate Jace’s first ability, and the player is an illegal target when the ability tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    ruling!(
        "Jace, Memory Adept",
        "If you target yourself with Jace’s first ability, you’ll draw a card first, then put the top card of your library into your graveyard."
    );
    supported("Jace, Memory Adept");
    // Targeting P1, then P1 gets hexproof: nothing happens.
    let mut t = TestGame::new(2);
    let jace = pw(&mut t, P0, "Jace, Memory Adept", 4);
    loyalty(&mut t, P0, jace, 0, &[pl(P1)]);
    player_hexproof(&mut t, P1);
    let before = libraries(&t);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 0]);
    assert_eq!((t.hand_size(P0), t.graveyard_size(P1)), (0, 0));
    // Targeting P1 legally: P0 draws, P1 mills one.
    let mut t = TestGame::new(2);
    let jace = pw(&mut t, P0, "Jace, Memory Adept", 4);
    loyalty(&mut t, P0, jace, 0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.graveyard_size(P1)), (1, 1));
    // Targeting yourself: you draw the top card first, then mill the next one.
    let mut t = TestGame::new(2);
    let jace = pw(&mut t, P0, "Jace, Memory Adept", 4);
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    loyalty(&mut t, P0, jace, 0, &[pl(P0)]);
    t.resolve_all();
    assert_eq!(t.zone(cards[0]), Zone::Hand(P0), "drawn first");
    assert_eq!(t.zone(cards[1]), Zone::Graveyard(P0), "then milled");
}

#[test]
fn chandra_ablaze_discards_only_when_the_ability_resolves() {
    cr!("608.2b", "602.2b", "115.1");
    ruling!(
        "Chandra Ablaze",
        "If the creature targeted by the first ability is an illegal target by the time it resolves, the entire ability doesn't resolve. You won't discard a card."
    );
    ruling!(
        "Chandra Ablaze",
        "If you activate Chandra Ablaze's first ability, you don't discard a card until the ability resolves. You may activate the ability even if your hand is empty. You choose a target as you activate the ability even if you have no red cards in hand at that time."
    );
    supported("Chandra Ablaze");
    // A red card in hand: activating doesn't discard; resolving discards it and deals 4.
    let mut t = TestGame::new(2);
    let ch = pw(&mut t, P0, "Chandra Ablaze", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = t.hand(P0, "Lightning Bolt");
    loyalty(&mut t, P0, ch, 0, &[obj(giant)]);
    assert_eq!(t.zone(bolt), Zone::Hand(P0), "not discarded on activation");
    t.answer_choose(P0, &[obj(bolt)]);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert!(t.in_graveyard(P1, "Hill Giant"), "4 damage");
    // The target is destroyed in response: the ability doesn't resolve; no discard.
    let mut t = TestGame::new(2);
    let ch = pw(&mut t, P0, "Chandra Ablaze", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = t.hand(P0, "Lightning Bolt");
    loyalty(&mut t, P0, ch, 0, &[obj(giant)]);
    destroy(&mut t, giant);
    t.answer_choose(P0, &[obj(bolt)]);
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Hand(P0), "no discard");
    // An empty hand: it can be activated, with a target chosen; it does nothing.
    let mut t = TestGame::new(2);
    let ch = pw(&mut t, P0, "Chandra Ablaze", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    loyalty(&mut t, P0, ch, 0, &[obj(giant)]);
    assert!(
        crate::r_s02_common::target_candidates(&t, P0, from)
            .iter()
            .any(|c| c.contains(&obj(giant))),
        "a target is chosen as it's activated"
    );
    assert_eq!(t.counters(ch, "loyalty"), 6);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
    // Only a nonred card in hand: still a target chosen; the card is discarded, no damage.
    let mut t = TestGame::new(2);
    let ch = pw(&mut t, P0, "Chandra Ablaze", 5);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.hand(P0, "Grizzly Bears");
    loyalty(&mut t, P0, ch, 0, &[obj(giant)]);
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn explosion_affects_the_other_target_if_one_becomes_illegal() {
    cr!("608.2b", "709.3b");
    ruling!(
        "Expansion // Explosion",
        "If one of Explosion's two targets becomes illegal, the other is affected as appropriate."
    );
    supported("Expansion // Explosion");
    // The creature is destroyed in response: P1 still draws X. Then the player gets
    // hexproof in response: the creature is still dealt X damage.
    for creature_gone in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Island", 2);
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Wastes", 2);
        let card = t.hand(P0, "Expansion // Explosion");
        t.cast(P0, card)
            .method(mtg_engine::object::CastMethod::Half(1))
            .x(2)
            .targets(&[obj(giant)])
            .targets(&[pl(P1)])
            .go();
        if creature_gone {
            destroy(&mut t, giant);
        } else {
            player_hexproof(&mut t, P1);
        }
        let before = libraries(&t);
        t.resolve_all();
        if creature_gone {
            assert_eq!(drawn_since(&t, &before), vec![0, 2]);
        } else {
            assert_eq!(t.obj_now(giant).damage, 2);
            assert_eq!(drawn_since(&t, &before), vec![0, 0]);
        }
    }
}

#[test]
fn kozilek_command_does_as_much_as_it_can() {
    cr!("608.2b", "700.2");
    ruling!(
        "Kozilek's Command",
        "If all of Kozilek's Command's targets are illegal as it tries to resolve, it will do nothing. If at least one target is still legal, it will resolve and do as much as it can."
    );
    supported("Kozilek's Command");
    // Modes 2 (target player scries X, then draws) and 3 (exile target creature with
    // mana value X or less), X = 2.
    for (creature_gone, player_hexproofed) in [(false, false), (true, false), (true, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Wastes", 4);
        let card = t.hand(P0, "Kozilek's Command");
        t.cast(P0, card)
            .x(2)
            .modes(&[1, 2])
            .targets(&[pl(P1)])
            .targets(&[obj(bears)])
            .go();
        if creature_gone {
            destroy(&mut t, bears);
        }
        if player_hexproofed {
            player_hexproof(&mut t, P1);
        }
        let before = libraries(&t);
        t.resolve_all();
        let drawn = if player_hexproofed { 0 } else { 1 };
        assert_eq!(drawn_since(&t, &before), vec![0, drawn]);
        assert_eq!(t.in_exile("Grizzly Bears"), !creature_gone);
        assert!(t.in_graveyard(P0, "Kozilek's Command"));
    }
}
