//! Rulings batch P038 — numbers of creatures counted as the spell or ability resolves
//! (CR 608.2h), effects that affect only the creatures there at that time (CR 611.2c),
//! and intervening "if" clauses checked again on resolution (CR 603.4).

use crate::r_p038_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::create_token;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// "When ~ enters, you gain/draw ... for each creature you control"
// ---------------------------------------------------------------------------

#[test]
fn enters_triggers_count_on_resolution_including_the_source() {
    cr!("608.2h", "603.3");
    ruling!(
        "Angel of Renewal",
        "Count the number of creatures you control as the ability resolves to determine how much life you gain. Angel of Renewal will count toward this number if it’s still on the battlefield."
    );
    ruling!(
        "Dwarven Priest",
        "The number of creatures you control is counted only as Dwarven Priest’s ability resolves. If Dwarven Priest is still on the battlefield, it will count itself."
    );
    ruling!(
        "Aven Gagglemaster",
        "The amount of life you gain is determined only as Aven Gagglemaster's triggered ability resolves. If Aven Gagglemaster is still on the battlefield at that time, it will be counted by its own ability."
    );
    ruling!(
        "Goldnight Redeemer",
        "The amount of life you gain is based on the number of other creatures you control when the ability resolves."
    );
    ruling!(
        "Regal Force",
        "If Regal Force is still on the battlefield as its triggered ability resolves, its ability will count itself."
    );
    // (card, the creature arriving in response, whether the source dies in response,
    // expected life gained, expected cards drawn)
    let cases: &[(&str, &str, bool, i32, usize)] = &[
        // One Bears beforehand, one more in response: itself + 2.
        ("Angel of Renewal", "Grizzly Bears", false, 3, 0),
        ("Angel of Renewal", "Grizzly Bears", true, 2, 0),
        ("Dwarven Priest", "Grizzly Bears", false, 3, 0),
        ("Dwarven Priest", "Grizzly Bears", true, 2, 0),
        // Flying creatures, two life each: itself + Serra Angel.
        ("Aven Gagglemaster", "Serra Angel", false, 4, 0),
        ("Aven Gagglemaster", "Serra Angel", true, 2, 0),
        // Other creatures, two life each: the two Bears.
        ("Goldnight Redeemer", "Grizzly Bears", false, 4, 0),
        // Green creatures: itself + two Bears.
        ("Regal Force", "Grizzly Bears", false, 0, 3),
        ("Regal Force", "Grizzly Bears", true, 0, 2),
    ];
    for &(card, extra, dies, life, draws) in cases {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        let hand = t.hand_size(P0);
        let src = enter(&mut t, P0, card);
        assert_eq!(t.stack_len(), 1, "{card}");
        add(&mut t, P0, extra);
        if dies {
            kill(&mut t, src);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + life, "{card} dies={dies}");
        assert_eq!(t.hand_size(P0), hand + draws, "{card} dies={dies}");
    }
}

// ---------------------------------------------------------------------------
// Spells
// ---------------------------------------------------------------------------

#[test]
fn spells_count_creatures_as_they_resolve() {
    cr!("608.2h");
    ruling!(
        "Congregate",
        "Count the number of creatures as Congregate resolves to determine how much life is gained."
    );
    ruling!(
        "Alive // Well",
        "Count the number of creatures you control when Well resolves to determine how much life you gain."
    );
    ruling!(
        "Chain Reaction",
        "The value of X is determined as Chain Reaction resolves."
    );
    ruling!(
        "King Harald's Revenge",
        "Use the number of creatures you control as King Harald's Revenge resolves to determine how big the power and toughness boost is."
    );
    ruling!(
        "Battle Hymn",
        "Battle Hymn uses the stack and can be responded to. The amount of mana produced is based on the number of creatures you control when Battle Hymn resolves."
    );
    // Congregate: two creatures when cast (one each), three as it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    cast_only(&mut t, P0, "Congregate", &[pl(P0)]);
    add(&mut t, P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 26);

    // Well: one creature when cast, two as it resolves.
    supported("Alive // Well");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    mana_all(&mut t, P0, 1);
    let card = t.hand(P0, "Alive // Well");
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 24);

    // Chain Reaction: two creatures when cast, three as it resolves: 3 damage each.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Grizzly Bears");
    cast_only(&mut t, P0, "Chain Reaction", &[]);
    add(&mut t, P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 3);

    // King Harald's Revenge: one creature when cast, two as it resolves.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_only(&mut t, P0, "King Harald's Revenge", &[obj(bear)]);
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));

    // Battle Hymn: it's on the stack (not a mana ability); one creature when cast, two
    // as it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    cast_only(&mut t, P0, "Battle Hymn", &[]);
    assert_eq!(t.stack_len(), 1);
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 2);
}

// ---------------------------------------------------------------------------
// Pumps: only the creatures there as it resolves, X locked in
// ---------------------------------------------------------------------------

#[test]
fn mass_pumps_affect_only_creatures_there_as_they_resolve_with_x_locked_in() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Craterhoof Behemoth",
        "Craterhoof Behemoth’s triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t gain trample and get +X/+X."
    );
    ruling!(
        "Craterhoof Behemoth",
        "The value of X is calculated only once, as Craterhoof Behemoth’s last ability resolves."
    );
    ruling!(
        "Moonshaker Cavalry",
        "Moonshaker Cavalry's triggered ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't be affected."
    );
    ruling!(
        "Moonshaker Cavalry",
        "The value of X is determined only as Moonshaker Cavalry's triggered ability resolves, potentially including Moonshaker Cavalry itself. It won't change later in the turn if the number of creatures you control changes."
    );
    ruling!(
        "Camaraderie",
        "The second part of Camaraderie's effect affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+1."
    );
    ruling!(
        "Ajani, Wise Counselor",
        "Ajani’s second ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +2/+2."
    );
    // Craterhoof / Moonshaker: one Bears, another in response: X = 3 (with itself).
    for (card, kw) in [
        ("Craterhoof Behemoth", mtg_engine::keywords::KeywordKind::Trample),
        ("Moonshaker Cavalry", mtg_engine::keywords::KeywordKind::Flying),
    ] {
        let mut t = TestGame::new(2);
        let b1 = t.battlefield(P0, "Grizzly Bears");
        enter(&mut t, P0, card);
        let b2 = add(&mut t, P0, "Grizzly Bears");
        t.resolve_all();
        assert_eq!(t.pt(b1), (5, 5), "{card}");
        assert_eq!(t.pt(b2), (5, 5), "{card}");
        assert!(t.obj_now(b1).has_keyword(kw));
        // X doesn't change; a creature arriving later gets nothing.
        kill(&mut t, b2);
        assert_eq!(t.pt(b1), (5, 5), "{card}");
        let late = add(&mut t, P0, "Grizzly Bears");
        assert_eq!(t.pt(late), (2, 2), "{card}");
        assert!(!t.obj_now(late).has_keyword(kw));
    }

    // Camaraderie: the +1/+1 only for the creatures there.
    let mut t = TestGame::new(2);
    let b1 = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Camaraderie", &[]);
    assert_eq!(t.pt(b1), (3, 3));
    let late = add(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));

    // Ajani's −3.
    supported("Ajani, Wise Counselor");
    let mut t = TestGame::new(2);
    let aj = t.battlefield(P0, "Ajani, Wise Counselor");
    let b1 = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, aj, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(b1), (4, 4));
    let late = add(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn x_is_calculated_once_as_the_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Heidegger, Shinra Executive",
        "The value of X is calculated only once, as Heidegger's first ability resolves."
    );
    ruling!(
        "Sally Pride, Lioness Leader",
        "The value of X is calculated only once, as Sally Pride's first ability resolves."
    );
    ruling!(
        "Thoughtweft Imbuer",
        "The value of X is calculated only once, as Thoughtweft Imbuer's ability resolves."
    );
    ruling!(
        "Angelic Exaltation",
        "The value of X is determined only as Angelic Exaltation’s ability begins to resolve. It won’t change later in the turn if the number of creatures you control changes."
    );
    ruling!(
        "Dáin of the Ancient Halls",
        "Use the number of Dwarves you control as Dáin's last ability resolves to determine how much damage is dealt."
    );
    // Heidegger (a Soldier): another Soldier arrives in response: +2/+0, and it stays
    // +2/+0 when that Soldier dies.
    supported("Heidegger, Shinra Executive");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Heidegger, Shinra Executive");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bear)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    let soldier = add(&mut t, P0, "Elite Vanguard");
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 2));
    kill(&mut t, soldier);
    kill(&mut t, h);
    assert_eq!(t.pt(bear), (4, 2));

    // Sally Pride: one nontoken creature beforehand, one more in response: 3 Mutants;
    // the count isn't redone as the Mutants arrive.
    supported("Sally Pride, Lioness Leader");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Sally Pride, Lioness Leader");
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Mutant").len(), 3);

    // Thoughtweft Imbuer / Angelic Exaltation: X counted as the attack trigger resolves,
    // and not changed by later changes.
    supported("Thoughtweft Imbuer");
    for card in ["Thoughtweft Imbuer", "Angelic Exaltation"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let attacker = t.battlefield(P0, "Kinsbaile Skirmisher");
        attack_with(&mut t, &[(attacker, pl(P1))]);
        assert_eq!(t.stack_len(), 1, "{card}");
        let extra = add(&mut t, P0, "Kinsbaile Skirmisher");
        t.resolve_all();
        // Kithkin: the attacker and the new one; creatures: the attacker, the new one,
        // and for the Imbuer case the Imbuer itself (a Kithkin).
        let x = if card == "Thoughtweft Imbuer" { 3 } else { 2 };
        assert_eq!(t.pt(attacker), (2 + x, 2 + x), "{card}");
        kill(&mut t, extra);
        assert_eq!(t.pt(attacker), (2 + x, 2 + x), "{card}");
    }

    // Dáin (a Dwarf): another Dwarf arrives in response: 2 damage to each opponent.
    supported("Dáin of the Ancient Halls");
    let mut t = TestGame::new(2);
    let dain = t.battlefield(P0, "Dáin of the Ancient Halls");
    attack_with(&mut t, &[(dain, pl(P1))]);
    add(&mut t, P0, "Dwarven Priest");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn rolling_hamsphere_counts_the_hamsters_it_just_made() {
    cr!("608.2h", "608.2c");
    ruling!(
        "Rolling Hamsphere",
        "The value of X is determined only once, as Rolling Hamsphere's second ability resolves."
    );
    supported("Rolling Hamsphere");
    let mut t = TestGame::new(2);
    let ham = t.battlefield(P0, "Rolling Hamsphere");
    let wurm = t.battlefield(P1, "Craw Wurm");
    // Crew 3 with a Hill Giant.
    let crew = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(crew)]);
    t.activate(P0, ham, 0, &[]).unwrap();
    t.resolve_all();
    t.answer_targets(P0, &[obj(wurm)]);
    attack_with(&mut t, &[(ham, pl(P1))]);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Hamster").len(), 3);
    assert_eq!(t.obj_now(wurm).damage, 3, "the three new Hamsters");
    // X stays 3 when another Hamster arrives.
    create_token(&mut t, P0, "Hamster");
    assert_eq!(t.obj_now(wurm).damage, 3);
}

#[test]
fn activated_and_loyalty_abilities_count_on_resolution() {
    cr!("608.2h", "606.4");
    ruling!(
        "Friendly Neighborhood",
        "The number of creatures you control is determined as the activated ability resolves."
    );
    ruling!(
        "Huatli, Radiant Champion",
        "The number of creatures you control is counted only as Huatli's first or second ability resolves. Once her second ability resolves, the bonus won't change, even if the number of creatures you control changes later in the turn."
    );
    ruling!(
        "Huatli, Radiant Champion",
        "If you activate Huatli's first ability while you control no creatures, she'll get one loyalty counter from the ability's activation cost and no others as it resolves."
    );
    ruling!(
        "Ajani, Wise Counselor",
        "The value of X is determined only as Ajani’s third ability resolves. If your life total changes later, the creature won’t gain or lose +1/+1 counters."
    );
    // Friendly Neighborhood's land: one creature when activated, two as it resolves.
    supported("Friendly Neighborhood");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Plains");
    let fnb = t.battlefield(P0, "Friendly Neighborhood");
    assert!(t.g.attach(fnb, obj(land)));
    let bear = t.battlefield(P0, "Grizzly Bears");
    mana_all(&mut t, P0, 1);
    let n = t
        .obj_now(land)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    t.activate(P0, land, n - 1, &[obj(bear)]).unwrap();
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));

    // Huatli's +1: counted as it resolves; with no creatures only the cost's counter.
    supported("Huatli, Radiant Champion");
    for creatures_now in [0usize, 2] {
        let mut t = TestGame::new(2);
        let h = t.battlefield(P0, "Huatli, Radiant Champion");
        t.activate(P0, h, 0, &[]).unwrap();
        bears(&mut t, P0, creatures_now);
        t.resolve_all();
        assert_eq!(t.counters(h, "loyalty"), 4 + creatures_now as u32);
    }
    // Huatli's −1: X counted as it resolves, fixed afterwards.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Huatli, Radiant Champion");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, h, 1, &[obj(bear)]).unwrap();
    let other = add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));
    kill(&mut t, other);
    add(&mut t, P0, "Grizzly Bears");
    add(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(bear), (4, 4));

    // Ajani's −9: X is the life total as it resolves; counters stay put afterwards.
    let mut t = TestGame::new(2);
    let aj = t.battlefield(P0, "Ajani, Wise Counselor");
    t.g.add_counters(obj(aj), counters::LOYALTY, 5, None);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].life = 10;
    t.activate(P0, aj, 2, &[obj(bear)]).unwrap();
    t.g.gain_life(P0, 2);
    t.resolve_all();
    assert_eq!(t.counters(bear, counters::PLUS1), 12);
    t.g.gain_life(P0, 5);
    t.settle();
    assert_eq!(t.counters(bear, counters::PLUS1), 12);
}

#[test]
fn end_step_triggers_count_as_they_resolve() {
    cr!("608.2h", "603.2");
    ruling!(
        "Throne of the God-Pharaoh",
        "Throne of the God-Pharaoh’s ability triggers at the beginning of each of your end steps, even if you control no tapped creatures. The number of tapped creatures you control is checked as the ability resolves."
    );
    ruling!(
        "Lydia Frye",
        "The value of X is calculated only once, as Lydia Frye’s last ability resolves."
    );
    // Throne: no tapped creatures as the end step begins; one is tapped in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Throne of the God-Pharaoh");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    tap(&mut t, bear);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);

    // Lydia Frye (an Assassin): untapped as it triggers, tapped in response: surveil 1.
    let mut t = TestGame::new(2);
    let lydia = t.battlefield(P0, "Lydia Frye");
    t.advance_to(P0, Step::End);
    t.settle();
    tap(&mut t, lydia);
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    let surveils = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::Surveil { .. }))
        .count();
    assert_eq!(surveils, 1);
    assert_eq!(t.library_size(P0), lib);
}

// ---------------------------------------------------------------------------
// Intervening "if" clauses
// ---------------------------------------------------------------------------

#[test]
fn intervening_if_creature_counts_are_checked_again_on_resolution() {
    cr!("603.4");
    ruling!(
        "Dundoolin Weaver",
        "If you don't control three or more creatures immediately after Dundoolin Weaver enters, its ability won't trigger. If you don't control three or more creatures as the ability tries to resolve, it won't resolve and none of its effects will happen. It doesn't matter if any of the creatures you control when the ability resolves are the same creatures as the ones you controlled when it triggered."
    );
    ruling!(
        "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun",
        "The last ability of Growing Rites of Itlimoc doesn't trigger if you don't control four or more creatures as your end step begins. If you don't control four or more creatures as it resolves, it does nothing."
    );
    ruling!(
        "Epic Struggle",
        "To win, you must control 20 or more creatures both at the beginning of upkeep and when it resolves."
    );
    supported("Dundoolin Weaver");
    // Weaver: one other creature -> no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    enter(&mut t, P0, "Dundoolin Weaver");
    assert_eq!(t.stack_len(), 0);
    // Two others: triggers; one dies and another arrives: still three, it resolves.
    let mut t = TestGame::new(2);
    let b = bears(&mut t, P0, 2);
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    enter(&mut t, P0, "Dundoolin Weaver");
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, b[0]);
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    // ... one dies and none arrives: nothing.
    let mut t = TestGame::new(2);
    let b = bears(&mut t, P0, 2);
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    enter(&mut t, P0, "Dundoolin Weaver");
    kill(&mut t, b[0]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));

    // Growing Rites: three creatures -> no trigger; four, one dies -> no transform.
    supported("Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun");
    for (n, dies, transforms) in [(3, false, false), (4, true, false), (4, false, true)] {
        let mut t = TestGame::new(2);
        let rites = t.battlefield(P0, "Growing Rites of Itlimoc // Itlimoc, Cradle of the Sun");
        let b = bears(&mut t, P0, n);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), (n >= 4) as usize, "n={n}");
        if dies {
            kill(&mut t, b[0]);
        }
        t.resolve_all();
        assert_eq!(
            t.obj_now(rites).chars.name.as_str() == "Itlimoc, Cradle of the Sun",
            transforms,
            "n={n} dies={dies}"
        );
    }

    // Epic Struggle: twenty creatures at upkeep, one dies in response: no win.
    supported("Epic Struggle");
    for dies in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Epic Struggle");
        let b = bears(&mut t, P0, 20);
        t.set_step(P0, Step::Untap);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if dies {
            kill(&mut t, b[0]);
        }
        t.resolve_all();
        assert_eq!(t.has_lost(P1), !dies, "dies={dies}");
    }
}

// ---------------------------------------------------------------------------
// Not mana abilities
// ---------------------------------------------------------------------------

#[test]
fn mana_adding_triggers_and_loyalty_abilities_use_the_stack() {
    cr!("605.1a", "605.1b", "608.2h", "106.1b");
    ruling!(
        "Xenagos, the Reveler",
        "Xenagos's first ability isn't a mana ability. It uses the stack and can be responded to. Count the number of creatures you control as the ability resolves to determine how much mana to add. You choose how much of that mana is red and how much is green at that time."
    );
    ruling!(
        "Muerra, Trash Tactician",
        "Muerra’s first ability is not a mana ability. It uses the stack and can be responded to."
    );
    // Xenagos: one creature when activated, two as it resolves.
    supported("Xenagos, the Reveler");
    let mut t = TestGame::new(2);
    let x = t.battlefield(P0, "Xenagos, the Reveler");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, x, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    add(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.total(), 2);
    assert_eq!(pool.count(ManaType::R) + pool.count(ManaType::G), 2);

    // Muerra (a Raccoon): the first-main-phase trigger goes on the stack; a Raccoon
    // arriving in response counts.
    supported("Muerra, Trash Tactician");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muerra, Trash Tactician");
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    create_token(&mut t, P0, "Raccoon");
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 2);
}

#[test]
fn bravado_is_continuously_calculated() {
    cr!("611.3a", "613.4c");
    ruling!("Bravado", "The bonus is continuously calculated.");
    supported("Bravado");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Bravado");
    assert!(t.g.attach(aura, obj(bear)));
    t.g.recompute();
    assert_eq!(t.pt(bear), (2, 2), "the enchanted creature isn't another creature");
    let a = add(&mut t, P0, "Grizzly Bears");
    add(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(bear), (4, 4));
    kill(&mut t, a);
    assert_eq!(t.pt(bear), (3, 3));
}

#[test]
fn other_creatures_on_an_aura_or_equipment_means_other_than_the_attached_one() {
    cr!("613.4c", "301.5", "303.4");
    // (Regression: "other" was read as "other than the Aura or Equipment", so the
    // enchanted or equipped creature counted itself.) A Bears with a +1/+1 counter,
    // enchanted or equipped, and another Bears with a +1/+1 counter.
    let cases: &[(&str, (i32, i32), (i32, i32))] = &[
        ("Bravado", (4, 4), (3, 3)),
        ("Conclave's Blessing", (3, 5), (3, 3)),
        ("Gleam of Authority", (4, 4), (3, 3)),
        // "Other creatures you control get -1/-1" spares the enchanted creature.
        ("Vampirism", (4, 4), (2, 2)),
        // "Other enchantment" is other than the Aura itself.
        ("Ancestral Mask", (3, 3), (3, 3)),
    ];
    for &(card, enchanted, other) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        let second = t.battlefield(P0, "Grizzly Bears");
        t.g.add_counters(obj(bear), counters::PLUS1, 1, None);
        t.g.add_counters(obj(second), counters::PLUS1, 1, None);
        let aura = t.battlefield(P0, card);
        assert!(t.g.attach(aura, obj(bear)));
        t.g.recompute();
        assert_eq!(t.pt(bear), enchanted, "{card}");
        assert_eq!(t.pt(second), other, "{card}");
    }
    // Sigil of Valor: attacking alone with one other creature: +1/+1.
    let mut t = TestGame::new(2);
    let b = bears(&mut t, P0, 2);
    let sigil = t.battlefield(P0, "Sigil of Valor");
    assert!(t.g.attach(sigil, obj(b[0])));
    attack_with(&mut t, &[(b[0], pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(b[0]), (3, 3));
}
