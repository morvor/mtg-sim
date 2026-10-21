//! Rulings batch P126 — planeswalker-deck sidekicks and tutors, and creatures that grow
//! with others of their kind: damage stays marked when a static bonus ends (CR 120.6,
//! 704.5g), triggers on loyalty abilities (CR 603.2, 606), "you may sacrifice. If you
//! do" triggers with illegal targets (CR 608.2b), and deck-construction limits.

use crate::r_p076_common::mana;
use crate::r_p108_common::two_headed_giant;
use crate::r_s01_common::supported;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, damage};
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `creature` (controlled by P0) is dealt one damage less than its toughness while
/// `bonus` is around; then `remove` takes the bonus away and the damage is lethal.
fn becomes_lethal(t: &mut TestGame, creature: ObjectId, remove: impl FnOnce(&mut TestGame)) {
    t.g.recompute();
    let (_, toughness) = t.pt(creature);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(t, src, toughness - 1, creature);
    assert!(t.on_battlefield(creature), "nonlethal damage at first");
    remove(t);
    t.g.recompute();
    t.settle();
    assert!(!t.on_battlefield(creature), "the marked damage became lethal");
}

#[test]
fn damage_stays_marked_when_a_planeswalker_bonus_ends() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Celestial Messenger",
        "nonlethal damage dealt to Celestial Messenger may become lethal if your Yanling planeswalker leaves the battlefield during that turn"
    );
    ruling!(
        "Court Cleric",
        "nonlethal damage dealt to Court Cleric may become lethal if you cease to control an Ajani planeswalker during that turn"
    );
    ruling!(
        "Dovin's Automaton",
        "nonlethal damage dealt to Dovin’s Automaton may become lethal if Dovin leaves the battlefield during that turn"
    );
    ruling!(
        "Vivien's Crocodile",
        "nonlethal damage dealt to Vivien’s Crocodile may become lethal if your Vivien planeswalker leaves the battlefield during that turn"
    );
    ruling!(
        "Predatory Wurm",
        "nonlethal damage dealt to Predatory Wurm may become lethal if you controlled a Garruk planeswalker and he leaves the battlefield during that turn"
    );
    let cases = [
        ("Celestial Messenger", "Mu Yanling, Sky Dancer", false),
        // Court Cleric: P0 ceases to control the Ajani (it changes control).
        ("Court Cleric", "Ajani Goldmane", true),
        ("Dovin's Automaton", "Dovin Baan", false),
        ("Vivien's Crocodile", "Vivien Reid", false),
        ("Predatory Wurm", "Garruk Wildspeaker", false),
    ];
    for (name, walker, control_change) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let base = t.pt(c);
        let pw = t.battlefield(P0, walker);
        t.g.recompute();
        assert!(t.pt(c).1 > base.1, "{name} gets the bonus");
        becomes_lethal(&mut t, c, |t| {
            if control_change {
                crate::r_s06_common::give_control(t, pw, P1);
            } else {
                move_to(t, pw, Zone::Graveyard(P0));
            }
        });
    }
}

#[test]
fn damage_stays_marked_when_another_of_its_kind_leaves() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Relentless Rats",
        "nonlethal damage dealt to one Relentless Rats may become lethal if another Relentless Rats leaves the battlefield during that turn"
    );
    ruling!(
        "Seven Dwarves",
        "nonlethal damage dealt to one Seven Dwarves may become lethal if another Seven Dwarves you control leaves the battlefield during that turn"
    );
    for name in ["Relentless Rats", "Seven Dwarves"] {
        supported(name);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, name);
        let b = t.battlefield(P0, name);
        t.g.recompute();
        assert_eq!(t.pt(a), (3, 3), "{name}");
        becomes_lethal(&mut t, a, |t| {
            move_to(t, b, Zone::Graveyard(P0));
        });
    }
}

#[test]
fn heavenly_blademaster_bonus_ends_and_marked_damage_becomes_lethal() {
    cr!("120.6", "704.5g", "611.3a", "301.5c");
    ruling!(
        "Heavenly Blademaster",
        "nonlethal damage dealt to a creature you control may become lethal if Heavenly Blademaster leaves the battlefield during that turn or if an Aura or Equipment is removed from it"
    );
    supported("Heavenly Blademaster");
    for leave in [true, false] {
        let mut t = TestGame::new(2);
        let hb = t.battlefield(P0, "Heavenly Blademaster");
        let eq = attach_new(&mut t, P0, "Bonesplitter", hb);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.recompute();
        assert_eq!(t.pt(bears), (3, 3));
        becomes_lethal(&mut t, bears, |t| {
            if leave {
                move_to(t, hb, Zone::Graveyard(P0));
            } else {
                t.g.unattach(eq);
            }
        });
    }
}

/// P0 has Keral Keep Disciples and Chandra, Acolyte of Flame, and activates her 0 ability.
fn chandra_activates(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Keral Keep Disciples");
    let chandra = t.battlefield(P0, "Chandra, Acolyte of Flame");
    t.activate(P0, chandra, 0, &[]).unwrap();
    t.settle();
    chandra
}

#[test]
fn keral_keep_disciples_resolves_before_the_loyalty_ability() {
    cr!("603.3", "606.3", "405.5");
    ruling!(
        "Keral Keep Disciples",
        "The ability of Keral Keep Disciples resolves before the ability that caused it to trigger."
    );
    supported("Keral Keep Disciples");
    let mut t = TestGame::new(2);
    let chandra = chandra_activates(&mut t);
    assert_eq!(t.stack_len(), 2);
    let loyalty = t.counters(chandra, "loyalty");
    t.resolve();
    // The Disciples' trigger resolved; Chandra's ability is still waiting.
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(chandra, "loyalty"), loyalty);
    t.resolve();
    assert_eq!(t.counters(chandra, "loyalty"), loyalty + 1);
}

#[test]
fn keral_keep_disciples_in_two_headed_giant() {
    cr!("810.9", "810.4");
    ruling!(
        "Keral Keep Disciples",
        "In a Two-Headed Giant game, the ability of Keral Keep Disciples causes the opposing team to lose 2 life."
    );
    let mut t = two_headed_giant();
    let before = t.life(P2);
    chandra_activates(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P2), before - 2);
    assert_eq!(t.life(P3), before - 2);
    assert_eq!(t.life(P0), before);
}

#[test]
fn wasp_of_the_bitter_end_chooses_to_sacrifice_on_resolution() {
    cr!("603.3d", "608.2b", "608.2c");
    ruling!(
        "Wasp of the Bitter End",
        "you don’t choose whether to sacrifice Wasp of the Bitter End or not until that ability resolves. If the creature becomes an illegal target, the entire ability doesn’t resolve and you can’t sacrifice Wasp of the Bitter End."
    );
    supported("Wasp of the Bitter End");
    for legal in [true, false] {
        let mut t = TestGame::new(2);
        let wasp = t.battlefield(P0, "Wasp of the Bitter End");
        let victim = t.battlefield(P1, "Hill Giant");
        let bolas = t.hand(P0, "Nicol Bolas, Planeswalker");
        mana(&mut t, P0, ManaType::U, 1);
        mana(&mut t, P0, ManaType::B, 2);
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::C, 4);
        t.answer_targets(P0, &[obj(victim)]);
        t.cast(P0, bolas).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "the trigger is on the stack above Bolas");
        let from = t.asked().len();
        if !legal {
            move_to(&mut t, victim, Zone::Exile);
        }
        t.answer_yes(P0, true);
        t.resolve();
        let asked_yes = t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::YesNo { .. }));
        assert_eq!(asked_yes, legal);
        assert_eq!(t.on_battlefield(wasp), !legal);
        if legal {
            assert!(t.in_graveyard(P1, "Hill Giant"));
        }
    }
}

#[test]
fn planeswalker_deck_tutors_need_a_legal_target() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Tezzeret's Betrayal",
        "If the target creature becomes an illegal target for Tezzeret’s Betrayal, the spell won’t resolve and none of its effects will happen. You won’t search for Tezzeret, Master of Metal."
    );
    ruling!(
        "Liliana's Scorn",
        "You won't search for Liliana, Death Mage. If the target is legal but not destroyed (most likely because it has indestructible), you do search for Liliana."
    );
    let cases = [
        ("Tezzeret's Betrayal", "Tezzeret, Master of Metal", [ManaType::U, ManaType::B]),
        ("Liliana's Scorn", "Liliana, Death Mage", [ManaType::B, ManaType::B]),
    ];
    for (spell_name, walker, colors) in cases {
        supported(spell_name);
        // (target stays legal?, target indestructible?)
        for (legal, indestructible) in [(false, false), (true, true)] {
            let mut t = TestGame::new(2);
            let target = if indestructible {
                t.battlefield(P1, "Darksteel Myr")
            } else {
                t.battlefield(P1, "Hill Giant")
            };
            t.library_top(P0, walker);
            let s = t.hand(P0, spell_name);
            for c in colors {
                mana(&mut t, P0, c, 1);
            }
            mana(&mut t, P0, ManaType::C, 3);
            t.cast(P0, s).target(target).go();
            if !legal {
                move_to(&mut t, target, Zone::Exile);
            }
            t.answer(P0, DecisionKind::Any, Answer::Default);
            t.resolve_all();
            assert_eq!(t.in_hand(P0, walker), legal, "{spell_name}: searched?");
            if indestructible {
                assert!(t.on_battlefield(target));
            }
        }
    }
}

fn deck(cards: &[(&str, usize)]) -> Vec<Arc<CardDef>> {
    let mut d = Vec::new();
    for (n, k) in cards {
        d.extend(std::iter::repeat_n(card(n), *k));
    }
    d
}

#[test]
fn seven_dwarves_limit_applies_in_limited() {
    cr!("100.2b", "113.6n");
    ruling!(
        "Seven Dwarves",
        "If you're fortunate enough to open or draft eight or more Seven Dwarves during a Limited event, you can include only seven of them in your deck. The rest remain in your sideboard."
    );
    let pool = deck(&[("Seven Dwarves", 8), ("Grizzly Bears", 30)]);
    let eight = deck(&[("Seven Dwarves", 8), ("Grizzly Bears", 30), ("Mountain", 2)]);
    assert_eq!(
        check_limited(&eight, &pool),
        vec![DeckProblem::TooManyCopies {
            name: "Seven Dwarves".into(),
            have: 8,
            max: 7
        }]
    );
    let seven = deck(&[("Seven Dwarves", 7), ("Grizzly Bears", 30), ("Mountain", 3)]);
    assert!(check_limited(&seven, &pool).is_empty());
    let side = limited_sideboard(&pool, &seven);
    assert_eq!(side.iter().filter(|c| c.name == "Seven Dwarves").count(), 1);
}

#[test]
fn any_number_rules_dont_ignore_format_legality_or_the_card_pool() {
    cr!("100.2a", "100.2b", "100.6", "113.6n");
    ruling!(
        "Rat Colony",
        "Rat Colony's last ability lets you ignore only the \"four-of\" rule. It doesn't let you ignore format legality."
    );
    ruling!(
        "Relentless Rats",
        "The last ability of Relentless Rats lets you ignore the \"four-of\" rule. It doesn't let you ignore format legality."
    );
    ruling!(
        "Seven Dwarves",
        "The last ability of Seven Dwarves lets you ignore the \"four-of\" rule. It doesn't let you ignore format legality."
    );
    for (name, max) in [("Rat Colony", None), ("Relentless Rats", None), ("Seven Dwarves", Some(7))] {
        // The four-of rule is ignored in constructed.
        let d = deck(&[(name, 7), ("Swamp", 53)]);
        assert!(check_constructed(&d).is_empty(), "{name}");
        assert_eq!(max_copies(&card(name)), max, "{name}");
        // Limited: no more copies than the pool has (none from a personal collection).
        let pool = deck(&[(name, 2), ("Grizzly Bears", 38)]);
        let d = deck(&[(name, 3), ("Grizzly Bears", 37)]);
        assert_eq!(
            check_limited(&d, &pool),
            vec![DeckProblem::NotInPool {
                name: name.into(),
                have: 3,
                available: 2
            }],
            "{name}"
        );
        // Format legality still applies: none of them is legal in Standard.
        let d = deck(&[(name, 7), ("Swamp", 53)]);
        assert_eq!(
            check_format_legality(&d, &[], "standard"),
            vec![DeckProblem::NotLegalInFormat {
                name: name.into(),
                format: "standard".into()
            }],
            "{name}"
        );
    }
}
