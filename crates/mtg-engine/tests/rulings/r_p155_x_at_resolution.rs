//! Rulings batch P155 — a value an effect needs from the game is determined as the spell
//! or ability resolves, only once (CR 608.2h): the greatest mana value or power among
//! permanents, the number of artifacts, the creatures named so-and-so; an "activate only
//! if" condition checked only on activation (CR 602.5d); an intervening-if clause
//! checked again on resolution (CR 603.4); and a power raised "until end of turn" still
//! raised during the end step.

use crate::r_p155_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn boon_of_boseiju_determines_x_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Boon of Boseiju",
        "The greatest mana value among permanents you control is determined as Boon of Boseiju resolves."
    );
    supported("Boon of Boseiju");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cast(&mut t, P0, "Boon of Boseiju", "1G", &[obj(bears)]);
    // Hill Giant (4) leaves in response: X is 2 (Grizzly Bears' own).
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn attack_bonuses_from_the_greatest_mana_value_are_calculated_once() {
    cr!("608.2h", "611.2a");
    ruling!(
        "Edgar, Master Machinist",
        "The value of X is calculated only once, as Edgar's last ability resolves."
    );
    ruling!(
        "Pyreswipe Hawk",
        "The value of X is calculated only once, as Pyreswipe Hawk's third ability resolves."
    );
    for (name, base) in [("Edgar, Master Machinist", 2), ("Pyreswipe Hawk", 4)] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let lotus = t.battlefield(P0, "Gilded Lotus");
        attack_with(&mut t, &[(c, pl(P1))]);
        t.resolve_all();
        assert_eq!(t.pt(c).0, base + 5, "{name}: {}", t.dump_log());
        // The Lotus leaves: the bonus stays +5.
        kill(&mut t, lotus);
        assert_eq!(t.pt(c).0, base + 5, "{name}");
        // Another artifact entering doesn't change it either.
        t.battlefield(P0, "Darksteel Forge");
        t.settle();
        assert_eq!(t.pt(c).0, base + 5, "{name}");
    }
}

#[test]
fn imskir_calculates_x_once_as_its_triggered_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Imskir Iron-Eater",
        "The value of X is calculated only once, as Imskir Iron-Eater's triggered ability resolves."
    );
    supported("Imskir Iron-Eater");
    let mut t = TestGame::new(2);
    let memnites: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Memnite")).collect();
    t.enter(P0, "Imskir Iron-Eater");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Four artifacts as it triggered (X would be 2); one Memnite leaves in response: X
    // is 1 as the ability resolves.
    kill(&mut t, memnites[0]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "{}", t.dump_log());
    assert_eq!(t.life(P0), 19);
}

#[test]
fn wicks_patrol_calculates_x_as_the_reflexive_ability_resolves() {
    cr!("608.2h", "603.12");
    ruling!(
        "Wick's Patrol",
        "The value of X is calculated only once, as the reflexive ability resolves."
    );
    supported("Wick's Patrol");
    for exile_wurm in [false, true] {
        let mut t = TestGame::new(2);
        let ae = t.battlefield(P1, "Air Elemental");
        stack_library(&mut t, P0, &["Craw Wurm", "Forest", "Forest"]);
        t.answer_targets(P0, &[obj(ae)]);
        t.enter(P0, "Wick's Patrol");
        t.resolve();
        // The reflexive ability is on the stack, targeting Air Elemental.
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        if exile_wurm {
            let wurm = t.g.find_in_zone(Zone::Graveyard(P0), "Craw Wurm")[0];
            move_to(&mut t, wurm, Zone::Exile);
        }
        t.resolve_all();
        // X is 6 (Craw Wurm) or 0 (two Forests) as it resolves.
        assert_eq!(t.on_battlefield(ae), exile_wurm, "exile: {exile_wurm}: {}", t.dump_log());
    }
}

#[test]
fn muzzio_determines_x_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Muzzio, Visionary Architect",
        "The value of X is determined as the ability resolves. If, at that time, you control no artifacts, you won't look at any cards."
    );
    supported("Muzzio, Visionary Architect");
    let mut t = TestGame::new(2);
    let muzzio = t.battlefield(P0, "Muzzio, Visionary Architect");
    let lotus = t.battlefield(P0, "Gilded Lotus");
    t.library_top(P0, "Juggernaut");
    let from = t.asked().len();
    activate(&mut t, P0, muzzio, "Look at the top", "3U", &[]);
    kill(&mut t, lotus);
    t.resolve_all();
    assert!(entity_candidates(&t, P0, from).is_empty(), "looked at nothing: {}", t.dump_log());
    assert_eq!(count_named(&t, "Juggernaut"), 0);
    assert_eq!(t.library_size(P0), 31);
}

#[test]
fn summon_bahamut_uses_the_total_mana_value_as_the_chapter_resolves() {
    cr!("608.2h", "714.2b");
    ruling!(
        "Summon: Bahamut",
        "Use the total mana value of other permanents you control as Summon: Bahamut's fourth chapter ability resolves to determine how much damage is dealt."
    );
    supported("Summon: Bahamut");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Summon: Bahamut");
    let giant = t.battlefield(P0, "Hill Giant");
    let lotus = t.battlefield(P0, "Gilded Lotus");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    put(&mut t, saga, counters::LORE, 3);
    t.resolve_all();
    put(&mut t, saga, counters::LORE, 1);
    assert_eq!(t.stack_len(), 1, "chapter IV: {}", t.dump_log());
    // Gilded Lotus (5) leaves in response: 4 damage (Hill Giant), not 9.
    kill(&mut t, lotus);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    let _ = giant;
}

#[test]
fn fungal_sprouting_counts_the_greatest_power_as_it_resolves() {
    cr!("608.2h", "115.1");
    ruling!(
        "Fungal Sprouting",
        "Fungal Sprouting doesn’t target any creature. The greatest power among creatures you control is determined when Fungal Sprouting resolves."
    );
    supported("Fungal Sprouting");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    let spell = cast(&mut t, P0, "Fungal Sprouting", "3G", &[]);
    assert!(stack_targets(&t, spell).is_empty());
    assert!(target_candidates(&t, P0, from).is_empty(), "no targets asked");
    // +3/+3 in response: six Saprolings.
    cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0), 6, "{}", t.dump_log());
}

#[test]
fn rubblebelt_rioters_x_is_its_own_power_when_alone_and_is_fixed_on_resolution() {
    cr!("608.2h", "611.2a");
    ruling!(
        "Rubblebelt Rioters",
        "If Rubblebelt Rioters is the only creature you control as its triggered ability resolves, X is equal to its own power. Unless another effect or a +1/+1 counter has raised its power above 0, this won’t have any effect."
    );
    ruling!(
        "Rubblebelt Rioters",
        "The value of X is determined only as the triggered ability of Rubblebelt Rioters resolves. Once that happens, the value of X won’t change later in the turn even if the greatest power among creatures you control changes."
    );
    supported("Rubblebelt Rioters");
    // Alone with power 0: no change.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rubblebelt Rioters");
    attack_with(&mut t, &[(r, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(r), (0, 4));
    // Alone with a +1/+1 counter: X is 1.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rubblebelt Rioters");
    put(&mut t, r, counters::PLUS1, 1);
    attack_with(&mut t, &[(r, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(r), (2, 5));
    // With a Hill Giant: +3/+0, unchanged when the Giant grows later.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rubblebelt Rioters");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(r, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(r), (3, 4));
    cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(giant)]);
    assert_eq!(t.pt(r), (3, 4));
    kill(&mut t, giant);
    assert_eq!(t.pt(r), (3, 4));
}

#[test]
fn tuya_bearclaw_x_is_the_greatest_other_power_as_it_resolves_and_then_fixed() {
    cr!("608.2h", "611.2a");
    ruling!(
        "Tuya Bearclaw",
        "The value of X is calculated as Tuya Bearclaw's ability resolves. If you control no other creatures at that time or if the greatest power among other creatures you control is 0 or less, X will be 0."
    );
    ruling!(
        "Tuya Bearclaw",
        "Once the ability resolves and the bonus is applied, that bonus won't change, even if the greatest power among other creatures you control does."
    );
    supported("Tuya Bearclaw");
    // No other creature: X is 0.
    let mut t = TestGame::new(2);
    let tuya = t.battlefield(P0, "Tuya Bearclaw");
    attack_with(&mut t, &[(tuya, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(tuya), (2, 2));
    // Only an Ornithopter (0/2): X is 0 (Tuya's own power isn't counted).
    let mut t = TestGame::new(2);
    let tuya = t.battlefield(P0, "Tuya Bearclaw");
    t.battlefield(P0, "Ornithopter");
    attack_with(&mut t, &[(tuya, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(tuya), (2, 2));
    // A Hill Giant that dies in response: X is 0; one that dies after: the bonus stays.
    let mut t = TestGame::new(2);
    let tuya = t.battlefield(P0, "Tuya Bearclaw");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(tuya, pl(P1))]);
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.pt(tuya), (2, 2));
    let mut t = TestGame::new(2);
    let tuya = t.battlefield(P0, "Tuya Bearclaw");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(tuya, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(tuya), (5, 5));
    cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(giant)]);
    assert_eq!(t.pt(tuya), (5, 5));
    kill(&mut t, giant);
    assert_eq!(t.pt(tuya), (5, 5));
}

#[test]
fn metallurgic_summonings_checks_its_artifacts_only_on_activation() {
    cr!("602.2", "602.2b");
    ruling!(
        "Metallurgic Summonings",
        "The number of artifacts you control is checked only as you activate Metallurgic Summonings's last ability. It's not checked again as it resolves."
    );
    supported("Metallurgic Summonings");
    let mut t = TestGame::new(2);
    let ms = t.battlefield(P0, "Metallurgic Summonings");
    let memnites: Vec<ObjectId> = (0..6).map(|_| t.battlefield(P0, "Memnite")).collect();
    t.graveyard(P0, "Shock");
    t.graveyard(P0, "Divination");
    activate(&mut t, P0, ms, "Return all instant", "3UU", &[]);
    assert_eq!(t.zone(ms), Zone::Exile);
    kill(&mut t, memnites[0]);
    assert_eq!(count_named(&t, "Memnite"), 5);
    t.resolve_all();
    assert!(t.in_hand(P0, "Shock") && t.in_hand(P0, "Divination"), "{}", t.dump_log());
    // With five artifacts, it can't be activated.
    let mut t = TestGame::new(2);
    let ms = t.battlefield(P0, "Metallurgic Summonings");
    for _ in 0..5 {
        t.battlefield(P0, "Memnite");
    }
    mana(&mut t, P0, "3UU");
    assert!(act(&mut t, P0, ms, "Return all instant", &[]).is_err());
}

#[test]
fn cleric_of_the_forward_order_counts_clerics_as_the_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Cleric of the Forward Order",
        "Count the number of creatures named Cleric of the Forward Order you control as the ability resolves to determine how much life you gain. If the Cleric of the Forward Order with the ability that triggered is still on the battlefield, it will count itself."
    );
    supported("Cleric of the Forward Order");
    // Two Clerics: 4 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cleric of the Forward Order");
    t.enter(P0, "Cleric of the Forward Order");
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // The entering Cleric dies in response: only the other counts: 2 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cleric of the Forward Order");
    let c = t.enter(P0, "Cleric of the Forward Order");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, c);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // A third Cleric entering in response is counted too: 6 life for the first trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cleric of the Forward Order");
    t.enter(P0, "Cleric of the Forward Order");
    t.settle();
    t.enter(P0, "Cleric of the Forward Order");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P0), 26);
    t.resolve_all();
    assert_eq!(t.life(P0), 32);
}

#[test]
fn faerie_miscreant_checks_for_another_miscreant_on_trigger_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Faerie Miscreant",
        "Faerie Miscreant's triggered ability checks to see if you control another creature named Faerie Miscreant at the time the new Faerie Miscreant enters the battlefield. If you don't, the ability won't trigger at all. The ability will check again as it tries to resolve. If, at that time, you don't control another creature named Faerie Miscreant, the ability will have no effect."
    );
    supported("Faerie Miscreant");
    // Alone: no trigger.
    let mut t = TestGame::new(2);
    t.enter(P0, "Faerie Miscreant");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), 0);
    // With another: triggers; the other leaves in response: no card.
    let mut t = TestGame::new(2);
    let other = t.battlefield(P0, "Faerie Miscreant");
    t.enter(P0, "Faerie Miscreant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, other);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // The other stays: a card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faerie Miscreant");
    t.enter(P0, "Faerie Miscreant");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn a_power_raised_until_end_of_turn_is_still_raised_during_the_end_step() {
    cr!("514.2", "611.2a");
    ruling!(
        "Wall of Reverence",
        "If an effect changes a creature’s power “until end of turn,” it will still have the modified power during your end step."
    );
    supported("Wall of Reverence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wall of Reverence");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 25, "{}", t.dump_log());
    // The effect ends in the cleanup step.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(bears), (2, 2));
}
