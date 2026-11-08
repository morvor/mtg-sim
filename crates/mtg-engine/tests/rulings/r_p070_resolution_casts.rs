//! Rulings batch P070 — cards cast while an effect resolves (CR 608.2g): the card is cast
//! then or not at all (it can't be cast later), and type-based timing permissions don't
//! apply (CR 307.1, 302.1, 601.2), but other casting restrictions do (CR 101.2).
//!
//! Each scenario brings the game to the point where the ability (or spell) that lets P0
//! cast the card is on top of the stack, with P0's answer queued; `t.resolve()` then
//! resolves only that object: when the card is on the stack right after, it was cast
//! during the resolution.

use crate::r_p070_common::*;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Resolves the top of the stack (the ability or spell letting P0 cast `card`) and checks
/// the outcome: cast during that resolution when `yes`, otherwise left in exile with no
/// way to cast it later.
fn check(mut t: TestGame, card: ObjectId, yes: bool) {
    t.resolve();
    if yes {
        assert!(
            is_on_stack(&t, card),
            "cast during the resolution: {}",
            t.dump_log()
        );
        assert_eq!(t.obj_now(t.g.current(card)).controller, P0);
        t.resolve_all();
        assert!(!exiled(&t, card), "{}", t.dump_log());
    } else {
        not_castable_later(&mut t, card);
    }
}

// ---------------------------------------------------------------------------
// Triggered abilities
// ---------------------------------------------------------------------------

/// Rona, Tolarian Obliterator is dealt damage by P1's Lightning Bolt; P1's only other
/// card in hand is Grizzly Bears. Returns the game with Rona's trigger on top of the stack.
fn rona(yes: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let rona = crate::r_p079_common::back_face(
        &mut t,
        P0,
        "Rona, Herald of Invasion // Rona, Tolarian Obliterator",
    );
    let bears = t.hand(P1, "Grizzly Bears");
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, R, 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(rona).go();
    t.resolve();
    // Rona's trigger is on the stack.
    assert_eq!(
        stack_names(&t),
        ["Rona, Tolarian Obliterator ability"],
        "{}",
        t.dump_log()
    );
    t.answer_yes(P0, yes);
    (t, bears)
}

#[test]
fn rona_casts_the_exiled_card_during_its_triggered_ability() {
    cr!("608.2g", "302.1");
    ruling!(
        "Rona, Herald of Invasion // Rona, Tolarian Obliterator",
        "The spell you cast from exile without paying its mana cost is cast during the resolution of the triggered ability. Timing restrictions of that spell based on card type are ignored."
    );
    supported("Rona, Herald of Invasion // Rona, Tolarian Obliterator");
    for yes in [true, false] {
        let (t, bears) = rona(yes);
        check(t, bears, yes);
    }
}

/// Crabomination enters; P1's library has Grizzly Bears on top. Returns the game with the
/// enters trigger on the stack.
fn crabomination(yes: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.library_top(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Crabomination");
    t.settle();
    t.answer_choose_moved(P0, if yes { vec![o(bears)] } else { vec![] }.as_slice());
    t.answer_yes(P0, yes);
    (t, bears)
}

#[test]
fn crabomination_casts_the_spell_during_its_triggered_ability() {
    cr!("608.2g", "302.1");
    ruling!(
        "Crabomination",
        "The spell you cast without paying its mana cost is cast during the resolution of Crabomination's triggered ability. Timing restrictions of that spell based on card type are ignored."
    );
    ruling!(
        "Crabomination",
        "You choose whether or not to cast one of the exiled cards as Crabomination's triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Crabomination");
    for yes in [true, false] {
        let (t, bears) = crabomination(yes);
        check(t, bears, yes);
    }
}

#[test]
fn krang_and_shredder_casts_during_its_end_step_trigger_only() {
    cr!("608.2g", "302.1", "603.4");
    ruling!(
        "Krang & Shredder",
        "You cast the card while Krang & Shredder's last ability is resolving and still on the stack. You can't wait to play it later in the turn. Timing restrictions based on its types are ignored."
    );
    supported("Krang & Shredder");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.library_top(P1, "Grizzly Bears");
        t.library_top(P1, "Forest");
        t.enter(P0, "Krang & Shredder");
        t.resolve_all();
        assert!(exiled(&t, bears), "{}", t.dump_log());
        // A permanent leaves the battlefield under P0's control.
        let orni = t.battlefield(P0, "Ornithopter");
        move_to(&mut t, orni, Zone::Graveyard(P0));
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        t.answer_choose_moved(P0, if yes { vec![o(bears)] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        check(t, bears, yes);
    }
}

/// `attacker` (already on P0's battlefield) attacks P1 unblocked; returns with the combat
/// damage step's triggers on the stack.
fn connect(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

#[test]
fn grima_casts_the_card_while_its_ability_resolves_only() {
    cr!("608.2g", "307.1");
    ruling!(
        "Gríma, Saruman's Footman",
        "You cast the card while the ability is resolving and still on the stack. You can't wait to cast it later in the turn."
    );
    supported("Gríma, Saruman's Footman");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Gríma, Saruman's Footman");
        let div = t.library_top(P1, "Divination");
        connect(&mut t, g);
        t.answer_yes(P0, yes);
        if yes {
            check(t, div, true);
        } else {
            // Not cast: it's put on the bottom of P1's library.
            t.resolve();
            assert_eq!(t.zone(div), Zone::Library(P1), "{}", t.dump_log());
        }
    }
}

#[test]
fn vaan_casts_the_exiled_card_while_its_first_ability_resolves() {
    cr!("608.2g", "302.1");
    ruling!(
        "Vaan, Street Thief",
        "You cast the exiled card while Vaan's first ability is still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Vaan, Street Thief");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let v = t.battlefield(P0, "Vaan, Street Thief");
        let bears = t.library_top(P1, "Grizzly Bears");
        connect(&mut t, v);
        // Vaan's "You may cast it" isn't free: P0 pays {1}{G}.
        mana(&mut t, P0, G, 2);
        t.answer_yes(P0, yes);
        let treasures = crate::r_s01_common::tokens(&t, P0).len();
        if !yes {
            t.resolve();
            assert_eq!(
                crate::r_s01_common::tokens(&t, P0).len(),
                treasures + 1,
                "{}",
                t.dump_log()
            );
            not_castable_later(&mut t, bears);
            continue;
        }
        check(t, bears, yes);
    }
}

#[test]
fn the_infamous_cruelclaw_casts_while_its_ability_resolves() {
    cr!("608.2g", "302.1", "118.9");
    ruling!(
        "The Infamous Cruelclaw",
        "You cast the exiled card while the ability is resolving and still on the stack. You can’t wait to cast it later in the turn. Timing restrictions based on the card’s type are ignored."
    );
    supported("The Infamous Cruelclaw");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "The Infamous Cruelclaw");
        let bears = t.library_top(P0, "Grizzly Bears");
        let island = t.hand(P0, "Island");
        connect(&mut t, c);
        t.answer_yes(P0, yes);
        t.answer_choose(P0, &[o(island)]);
        let hand = t.hand_size(P0);
        if yes {
            t.resolve();
            assert!(is_on_stack(&t, bears), "{}", t.dump_log());
            assert_eq!(t.hand_size(P0), hand - 1, "a card discarded");
            t.resolve_all();
            assert!(t.on_battlefield(t.g.current(bears)));
        } else {
            check(t, bears, false);
        }
    }
}

#[test]
fn duelists_flame_trigger_casts_while_it_resolves() {
    cr!("608.2g", "302.1", "702.19c");
    ruling!(
        "Duelist's Flame",
        "You cast the exiled card while the triggered ability ability is resolving and still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Duelist's Flame");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let elves = t.battlefield(P1, "Llanowar Elves");
        let bears = t.library_top(P0, "Grizzly Bears");
        attack_with(&mut t, &[(giant, Entity::Player(P1))]);
        t.answer(
            P1,
            DecisionKind::Blockers,
            mtg_engine::decision::Answer::Blockers(vec![(elves, giant)]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        let flame = t.hand(P0, "Duelist's Flame");
        mana(&mut t, P0, R, 3);
        t.cast(P0, flame).x(1).target(giant).go();
        t.resolve_all();
        assert_eq!(t.pt(giant), (4, 3));
        // 1 damage to the Elves, 3 tramples over to P1.
        t.advance_to(P0, Step::CombatDamage);
        assert_eq!(t.life(P1), 17, "{}", t.dump_log());
        t.settle();
        t.answer_choose_moved(P0, if yes { vec![o(bears)] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        if yes {
            check(t, bears, true);
        } else {
            // Not exiled at all: there's nothing to cast later.
            t.resolve();
            assert_eq!(t.zone(bears), Zone::Library(P0), "{}", t.dump_log());
        }
    }
    // Exiled but not cast: it stays exiled and can't be cast later.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.library_top(P0, "Grizzly Bears");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(elves, giant)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let flame = t.hand(P0, "Duelist's Flame");
    mana(&mut t, P0, R, 3);
    t.cast(P0, flame).x(1).target(giant).go();
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    t.answer_choose_moved(P0, &[o(bears)]);
    t.answer_yes(P0, false);
    check(t, bears, false);
}

#[test]
fn ryan_sinclair_casts_the_spell_during_its_attack_trigger_only() {
    cr!("608.2g", "302.1");
    ruling!(
        "Ryan Sinclair",
        "You cast the spell as part of the resolution of Ryan's triggered ability. You can't wait and cast the spell at a later time."
    );
    supported("Ryan Sinclair");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Ryan Sinclair");
        let bears = t.library_top(P0, "Grizzly Bears");
        attack_with(&mut t, &[(r, Entity::Player(P1))]);
        t.answer_yes(P0, yes);
        if yes {
            check(t, bears, true);
        } else {
            // Not cast: it goes to the bottom of the library.
            t.resolve();
            assert_eq!(t.zone(bears), Zone::Library(P0), "{}", t.dump_log());
            t.set_step(P0, Step::PostcombatMain);
            mana(&mut t, P0, G, 2);
            assert!(!castable(&mut t, P0, bears));
        }
    }
}

#[test]
fn river_songs_diary_casts_the_card_during_its_upkeep_trigger_only() {
    cr!("608.2g", "302.1", "607.2a");
    ruling!(
        "River Song's Diary",
        "You cast the spell as part of the resolution of the last ability. You can't wait and cast the spell at a later time."
    );
    supported("River Song's Diary");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "River Song's Diary");
        let mut divs = vec![];
        for _ in 0..4 {
            let d = t.hand(P0, "Divination");
            mana(&mut t, P0, U, 3);
            t.cast(P0, d).go();
            t.resolve_all();
            divs.push(d);
        }
        assert!(divs.iter().all(|d| exiled(&t, *d)), "{}", t.dump_log());
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        t.answer_yes(P0, yes);
        let hand = t.hand_size(P0);
        t.resolve();
        let cast: Vec<_> = divs.iter().filter(|d| is_on_stack(&t, **d)).collect();
        assert_eq!(
            cast.len(),
            usize::from(yes),
            "a sorcery in the upkeep: {}",
            t.dump_log()
        );
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + if yes { 2 } else { 0 });
        let left: Vec<_> = divs.iter().copied().filter(|d| exiled(&t, *d)).collect();
        assert_eq!(left.len(), if yes { 3 } else { 4 });
        not_castable_later(&mut t, left[0]);
    }
}

#[test]
fn herald_of_amity_casts_the_aura_during_its_enters_trigger_only() {
    cr!("608.2g", "303.4a", "302.1");
    ruling!(
        "Herald of Amity",
        "You cast the spell while Herald of Amity's second ability is resolving and still on the stack. You can't wait to cast it later in the turn. Timing restrictions based on the card's types are ignored."
    );
    supported("Herald of Amity");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let hs = t.library_top(P0, "Holy Strength");
        let h = t.enter(P0, "Herald of Amity");
        t.settle();
        t.answer_choose_moved(P0, if yes { vec![o(hs)] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        t.answer_targets(P0, &[o(h)]);
        t.resolve();
        if yes {
            assert!(is_on_stack(&t, hs), "{}", t.dump_log());
            t.resolve_all();
            assert_eq!(t.pt(h), (3, 4));
        } else {
            // Not cast: it went to the bottom of the library with the rest.
            assert_eq!(t.zone(hs), Zone::Library(P0), "{}", t.dump_log());
        }
    }
}

#[test]
fn kotis_casts_spells_during_its_combat_damage_trigger_only() {
    cr!("608.2g", "302.1");
    ruling!(
        "Kotis, the Fangkeeper",
        "You cast the spells from among the exiled cards while Kotis’s last ability is resolving and still on the stack. You can’t wait to cast them later in the turn. Timing restrictions based on the card’s type are ignored."
    );
    supported("Kotis, the Fangkeeper");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Kotis, the Fangkeeper");
        let lib = stack_library(&mut t, P1, &["Grizzly Bears", "Forest"]);
        connect(&mut t, k);
        t.answer_choose_moved(P0, if yes { vec![o(lib[0])] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        check(t, lib[0], yes);
    }
}

#[test]
fn amped_raptor_casts_the_card_during_its_enters_trigger_only() {
    cr!("608.2g", "302.1", "118.9");
    ruling!(
        "Amped Raptor",
        "You choose whether or not to cast the exiled nonland card as Amped Raptor's triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Amped Raptor");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.library_top(P0, "Grizzly Bears");
        let raptor = t.hand(P0, "Amped Raptor");
        mana(&mut t, P0, R, 2);
        t.cast(P0, raptor).go();
        t.resolve();
        t.settle();
        t.answer_yes(P0, yes);
        if !yes {
            t.resolve();
            // Energy is still there, but the card can't be cast later.
            assert_eq!(t.g.player(P0).counter("energy"), 2);
            not_castable_later(&mut t, bears);
            continue;
        }
        check(t, bears, true);
    }
}

#[test]
fn smirking_spelljacker_casts_the_card_during_its_attack_trigger_only() {
    cr!("608.2g", "302.1");
    ruling!(
        "Smirking Spelljacker",
        "You choose whether to cast the exiled card as Smirking Spelljacker’s last ability resolves. If you do, you do so as part of the resolution of that ability. You can’t wait to cast it later in the turn. Timing restrictions based on the card’s types are ignored."
    );
    supported("Smirking Spelljacker");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let bears = t.hand(P1, "Grizzly Bears");
        mana(&mut t, P1, G, 2);
        let spell = t.cast(P1, bears).go();
        t.answer_targets(P0, &[o(spell)]);
        let sj = t.enter(P0, "Smirking Spelljacker");
        t.settle();
        t.resolve();
        assert!(exiled(&t, bears), "{}", t.dump_log());
        t.set_step(P0, Step::PrecombatMain);
        t.g.obj_mut(sj).summoning_sick = false;
        t.answer_yes(P0, yes);
        attack_with(&mut t, &[(sj, Entity::Player(P1))]);
        check(t, bears, yes);
    }
}

#[test]
fn the_key_to_the_vault_casts_the_card_during_its_trigger_only() {
    cr!("608.2g", "302.1");
    ruling!(
        "The Key to the Vault",
        "You choose whether to cast the exiled card as The Key to the Vault’s triggered ability resolves. If you do, you do so as part of the resolution of that ability. You can’t wait to cast it later in the turn. Timing restrictions based on the card’s types are ignored."
    );
    supported("The Key to the Vault");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let key = t.battlefield(P0, "The Key to the Vault");
        assert!(t.g.attach(key, o(bears)));
        let div = t.library_top(P0, "Divination");
        connect(&mut t, bears);
        t.answer_choose_moved(P0, &[o(div)]);
        t.answer_yes(P0, yes);
        check(t, div, yes);
    }
}

#[test]
fn kylox_casts_spells_during_its_attack_trigger_only() {
    cr!("608.2g", "307.1");
    ruling!(
        "Kylox, Visionary Inventor",
        "You choose which spells to cast (if any) as Kylox, Visionary Inventor's last ability resolves. If you choose to cast any spells, you do so as part of the resolution of that ability. You can't wait to cast them later in the turn. Timing restrictions based on the cards' types are ignored."
    );
    supported("Kylox, Visionary Inventor");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Kylox, Visionary Inventor");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let lib = stack_library(&mut t, P0, &["Divination", "Forest"]);
        t.answer_choose(P0, &[o(bears)]);
        t.answer_choose_moved(P0, if yes { vec![o(lib[0])] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        attack_with(&mut t, &[(k, Entity::Player(P1))]);
        check(t, lib[0], yes);
    }
}

#[test]
fn living_lore_casts_a_sorcery_on_an_opponents_turn() {
    cr!("608.2g", "307.1", "510.2");
    ruling!(
        "Living Lore",
        "You may cast a sorcery this way even if it's not your turn."
    );
    supported("Living Lore");
    let mut t = TestGame::new(2);
    let div = t.graveyard(P0, "Divination");
    t.answer_choose_moved(P0, &[o(div)]);
    let l = t.enter(P0, "Living Lore");
    assert_eq!(t.pt(l), (3, 3));
    // P1's turn: Living Lore blocks Grizzly Bears and deals combat damage.
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(l, bears)]),
    );
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve();
    assert!(is_on_stack(&t, div), "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.g.turn.active, P1);
}

#[test]
fn tasha_casts_during_its_minus_three_only() {
    cr!("608.2g", "606.3");
    ruling!(
        "Tasha, the Witch Queen",
        "You may cast a spell from among the exiled cards only as Tasha's last loyalty ability resolves. You may not wait and cast a spell from among them at a later time."
    );
    supported("Tasha, the Witch Queen");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let tasha = t.battlefield(P0, "Tasha, the Witch Queen");
        let div = t.graveyard(P1, "Divination");
        // +1: draw, and exile Divination from P1's graveyard with a page counter.
        t.answer_targets(P0, &[o(div)]);
        t.activate(P0, tasha, 0, &[]).unwrap();
        t.resolve_all();
        assert!(exiled(&t, div), "{}", t.dump_log());
        assert_eq!(t.counters(t.g.current(div), "page"), 1);
        // Next turn: -3.
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_choose_moved(P0, if yes { vec![o(div)] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        t.activate(P0, tasha, 1, &[]).unwrap();
        check(t, div, yes);
    }
}

// ---------------------------------------------------------------------------
// Spells and activated abilities
// ---------------------------------------------------------------------------

#[test]
fn improvisation_capstone_casts_during_its_resolution_only() {
    cr!("608.2g", "302.1");
    ruling!(
        "Improvisation Capstone",
        "You cast the spells while Improvisation Capstone is resolving and still on the stack. You can't wait to cast them later in the turn. Timing restrictions based on the cards' types are ignored."
    );
    supported("Improvisation Capstone");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let lib = stack_library(&mut t, P0, &["Grizzly Bears", "Grizzly Bears"]);
        let ic = t.hand(P0, "Improvisation Capstone");
        mana(&mut t, P0, R, 7);
        t.answer_choose_moved(P0, if yes { vec![o(lib[0])] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        t.cast(P0, ic).go();
        check(t, lib[0], yes);
    }
}

#[test]
fn cosmium_catalyst_casts_the_card_as_its_ability_resolves_only() {
    cr!("608.2g", "307.1", "702.167a");
    ruling!(
        "Ore-Rich Stalactite // Cosmium Catalyst",
        "You choose whether or not to cast the exiled card as Cosmium Catalyst's activated ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to cast it later in the turn. Timing restrictions based on the card's type are ignored."
    );
    supported("Ore-Rich Stalactite // Cosmium Catalyst");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let ore = t.battlefield(P0, "Ore-Rich Stalactite // Cosmium Catalyst");
        let spikes = crate::r_s07_common::graveyard_n(&mut t, P0, "Lava Spike", 4);
        t.lands(P0, "Mountain", 5);
        t.answer_choose(P0, &spikes.iter().map(|b| o(*b)).collect::<Vec<_>>());
        t.activate(P0, ore, 1, &[]).expect("craft");
        t.resolve_all();
        let cat = t.g.current(ore);
        assert!(t.on_battlefield(cat), "{}", t.dump_log());
        t.g.obj_mut(cat).summoning_sick = false;
        // On P1's turn: a sorcery (Lava Spike) cast as the ability resolves.
        t.set_step(P1, Step::PrecombatMain);
        mana(&mut t, P0, R, 2);
        t.answer_yes(P0, yes);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.activate(P0, cat, 0, &[]).expect("catalyst");
        t.resolve();
        let cast: Vec<_> = spikes.iter().filter(|s| is_on_stack(&t, **s)).collect();
        assert_eq!(cast.len(), usize::from(yes), "{}", t.dump_log());
        t.resolve_all();
        assert_eq!(t.life(P1), if yes { 17 } else { 20 });
        let left: Vec<_> = spikes.iter().copied().filter(|s| exiled(&t, *s)).collect();
        assert_eq!(left.len(), if yes { 3 } else { 4 });
        not_castable_later(&mut t, left[0]);
    }
}

#[test]
fn portent_of_calamity_casts_during_its_resolution_only() {
    cr!("608.2g", "302.1");
    ruling!(
        "Portent of Calamity",
        "You choose which spell to cast (if any) as Portent of Calamity resolves. If you choose to cast a spell this way, you do so as part of the resolution of Portent of Calamity. You can’t wait to cast it later in the turn. Timing restrictions based on the card’s types are ignored."
    );
    supported("Portent of Calamity");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let forest = t.library_top(P0, "Forest");
        let bears = t.library_top(P0, "Grizzly Bears");
        let bolt = t.library_top(P0, "Lightning Bolt");
        let relic = t.library_top(P0, "Darksteel Relic");
        let spell = t.hand(P0, "Portent of Calamity");
        mana(&mut t, P0, U, 5);
        t.answer_choose(P0, &[o(forest), o(bears), o(bolt), o(relic)]);
        t.answer_choose_moved(P0, if yes { vec![o(bears)] } else { vec![] }.as_slice());
        t.answer_yes(P0, yes);
        t.cast(P0, spell).x(4).go();
        t.resolve();
        if yes {
            assert!(is_on_stack(&t, bears), "{}", t.dump_log());
            t.resolve_all();
            assert!(t.on_battlefield(t.g.current(bears)));
        } else {
            // Not cast: it's put into P0's hand with the rest, not left castable.
            assert_eq!(t.zone(bears), Zone::Hand(P0), "{}", t.dump_log());
        }
    }
}

#[test]
fn collectors_cage_plays_the_card_as_its_ability_resolves_but_no_land_on_another_turn() {
    cr!("608.2g", "305.2", "305.3", "302.1");
    ruling!(
        "Collector's Cage",
        "You choose whether to play the exiled card as Collector's Cage's last ability resolves. If you do, you do so as part of the resolution of that ability. You can't wait to play it later in the turn. Timing restrictions based on the card's types are ignored. However, you can never play lands on other players' turns."
    );
    supported("Collector's Cage");
    // `hidden` is hidden away; the ability is activated in `turn`'s main phase.
    let cage = |hidden: &str, turn: PlayerId, yes: bool| -> (TestGame, ObjectId) {
        let mut t = TestGame::new(2);
        let card = t.library_top(P0, hidden);
        t.answer_choose_moved(P0, &[o(card)]);
        let cage = t.enter(P0, "Collector's Cage");
        t.resolve_all();
        assert!(exiled(&t, card), "{}", t.dump_log());
        let giant = t.battlefield(P0, "Hill Giant");
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Llanowar Elves");
        t.g.obj_mut(t.g.current(cage)).summoning_sick = false;
        t.set_step(turn, Step::PrecombatMain);
        mana(&mut t, P0, C, 1);
        t.answer_yes(P0, yes);
        t.activate(P0, cage, 0, &[o(giant)]).unwrap();
        (t, card)
    };
    // A creature card on P1's turn: cast as the ability resolves.
    let (t, bears) = cage("Grizzly Bears", P1, true);
    check(t, bears, true);
    let (t, bears) = cage("Grizzly Bears", P0, false);
    check(t, bears, false);
    // A land card on P0's own turn: played.
    let (mut t, forest) = cage("Forest", P0, true);
    t.resolve();
    assert!(t.on_battlefield(t.g.current(forest)), "{}", t.dump_log());
    // On P1's turn: it can't be played.
    let (mut t, forest) = cage("Forest", P1, true);
    t.resolve();
    assert!(exiled(&t, forest), "{}", t.dump_log());
}

#[test]
fn jeleva_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Jeleva, Nephalia's Scourge",
        "When casting an instant or sorcery card this way, ignore all timing restrictions based on the card's type. Other timing restrictions, such as \"Cast [this spell] only during combat,\" must be followed."
    );
    supported("Jeleva, Nephalia's Scourge");
    supported("Delirium");
    // Divination (a sorcery) and Delirium ("Cast this spell only during an opponent's
    // turn") are exiled with Jeleva; it attacks in P0's turn.
    for (name, cast) in [("Divination", true), ("Delirium", false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Grizzly Bears");
        let card = t.library_top(P0, name);
        let j = t.hand(P0, "Jeleva, Nephalia's Scourge");
        pool(&mut t, P0, &[U, B, R, C]);
        t.cast(P0, j).go();
        t.resolve_all();
        let j = t.g.current(j);
        t.g.obj_mut(j).summoning_sick = false;
        assert!(exiled(&t, card));
        let from = t.asked().len();
        t.answer_choose_moved(P0, &[o(card)]);
        t.answer_yes(P0, true);
        attack_with(&mut t, &[(j, Entity::Player(P1))]);
        t.resolve();
        assert_eq!(is_on_stack(&t, card), cast, "{name}: {}", t.dump_log());
        if !cast {
            assert!(!was_offered(&t, from, card), "{name}");
            assert!(exiled(&t, card));
        }
    }
}

#[test]
fn jace_ignores_type_timing_but_not_other_restrictions() {
    cr!("608.2g", "307.1", "601.3");
    ruling!(
        "Jace, Architect of Thought",
        "When casting a card this way, ignore timing restrictions based on the card's type. Other timing restrictions, such as “Cast [this card] only during combat,” must be followed."
    );
    supported("Jace, Architect of Thought");
    // P0 finds Panic ("Cast this spell only during combat before blockers are declared"),
    // P1 finds Grizzly Bears; −8 is activated in P0's main phase.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Architect of Thought");
    t.g.obj_mut(jace).counters.insert("loyalty".into(), 8);
    let panic = t.library_top(P0, "Panic");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.answer_choose(P0, &[o(panic)]);
    t.answer_choose(P0, &[o(bears)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.activate(P0, jace, 2, &[]).unwrap();
    t.resolve();
    assert!(is_on_stack(&t, bears), "{}", t.dump_log());
    assert!(
        exiled(&t, panic),
        "Panic can't be cast now: {}",
        t.dump_log()
    );
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
}
