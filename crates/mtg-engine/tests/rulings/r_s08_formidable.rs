//! Rulings batch S08 — formidable (an ability word, CR 207.2c): "if creatures you control
//! have total power 8 or greater". Triggered formidable abilities have an intervening "if"
//! clause (CR 603.4); activated ones check only as they're activated (CR 602.5b).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use crate::r_s08_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_creature_with_negative_power_lowers_the_total_power() {
    cr!("207.2c", "107.1b");
    ruling!(
        "Dragon-Scarred Bear",
        "If you control a creature with power less than 0, use its actual power when calculating the total power of creatures you control. For example, if you control three creatures with powers 4, 5, and -2, the total power of creatures you control is 7."
    );
    supported("Dragon-Scarred Bear");
    supported("Sensory Deprivation");
    // Dragon-Scarred Bear (3/2): "Formidable — {1}{G}: Regenerate this creature. Activate
    // only if creatures you control have total power 8 or greater."
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Dragon-Scarred Bear");
    t.battlefield(P0, "Craw Wurm");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 2);
    // 3 + 6 + 1 = 10.
    assert!(can_activate(&mut t, P0, bear));
    // Sensory Deprivation (-3/-0) makes the Elves -2/1: 3 + 6 - 2 = 7.
    attach_new(&mut t, P1, "Sensory Deprivation", elves);
    assert_eq!(t.pt(elves), (-2, 1));
    assert!(!can_activate(&mut t, P0, bear));
}

#[test]
fn a_formidable_trigger_checks_the_total_power_again_as_it_resolves() {
    cr!("603.4", "207.2c");
    ruling!(
        "Stampeding Elk Herd",
        "Other formidable abilities are triggered abilities with an “intervening ‘if’” clause. Such abilities check the total power of creatures you control twice: once at the appropriate time to see if the ability will trigger, and again as the ability tries to resolve. If, at that time, the total power of creatures you control is no longer 8 or greater, the ability will have no effect."
    );
    supported("Stampeding Elk Herd");
    // Stampeding Elk Herd (5/5): "Formidable — Whenever this creature attacks, if
    // creatures you control have total power 8 or greater, creatures you control gain
    // trample until end of turn."
    for (partner, kill) in [
        ("Hill Giant", false),
        ("Hill Giant", true),
        ("Grizzly Bears", false),
    ] {
        let mut t = TestGame::new(2);
        let elk = t.battlefield(P0, "Stampeding Elk Herd");
        let other = t.battlefield(P0, partner);
        t.set_step(P0, Step::BeginningOfCombat);
        attack_with(&mut t, &[(elk, Entity::Player(P1))]);
        let triggered = t.stack_len() == 1;
        // 5 + 3 = 8: it triggers; 5 + 2 = 7: it doesn't.
        assert_eq!(triggered, partner == "Hill Giant", "{partner}");
        if kill {
            destroy(&mut t, other);
        }
        t.resolve_all();
        let trample = t.obj_now(elk).chars.has_keyword(KeywordKind::Trample);
        assert_eq!(trample, triggered && !kill, "{partner} {kill}");
    }
}

#[test]
fn a_beginning_of_combat_formidable_trigger_checks_again_as_it_resolves() {
    cr!("603.4", "207.2c");
    ruling!(
        "Surrak, the Hunt Caller",
        "Other formidable abilities are triggered abilities with an \"intervening 'if'\" clause. Such abilities check the total power of creatures you control twice: once at the appropriate time to see if the ability will trigger, and again as the ability tries to resolve."
    );
    supported("Surrak, the Hunt Caller");
    // Surrak (5/4): "Formidable — At the beginning of combat on your turn, if creatures you
    // control have total power 8 or greater, target creature you control gains haste
    // until end of turn."
    for kill in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Surrak, the Hunt Caller");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield_sick(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if kill {
            // 5 + 2 = 7 as it resolves.
            destroy(&mut t, giant);
        }
        t.resolve_all();
        assert_eq!(
            t.obj_now(bears).chars.has_keyword(KeywordKind::Haste),
            !kill
        );
    }
}

#[test]
fn an_activated_formidable_ability_doesnt_care_about_power_after_activation() {
    cr!("602.5b", "207.2c", "701.19a");
    ruling!(
        "Dragon-Scarred Bear",
        "Some formidable abilities are activated abilities that require creatures you control to have total power 8 or greater. Once you activate these abilities, it doesn’t matter what happens to the total power of creatures you control."
    );
    // Dragon-Scarred Bear's regeneration, activated with 3 + 6 = 9; the Wurm is destroyed
    // before it resolves.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Dragon-Scarred Bear");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    t.activate(P0, bear, 0, &[]).unwrap();
    destroy(&mut t, wurm);
    t.resolve_all();
    destroy(&mut t, bear);
    assert!(t.on_battlefield(bear), "regenerated");
    assert!(is_tapped(&t, bear));
}

#[test]
fn an_activated_formidable_pump_resolves_even_if_power_drops() {
    cr!("602.5b", "207.2c");
    ruling!(
        "Atarka Beastbreaker",
        "Some formidable abilities are activated abilities that require creatures you control to have total power 8 or greater. Once you activate these abilities, it doesn't matter what happens to the total power of creatures you control."
    );
    supported("Atarka Beastbreaker");
    // Atarka Beastbreaker (2/2): "Formidable — {4}{G}: This creature gets +4/+4 until end of
    // turn. Activate only if creatures you control have total power 8 or greater."
    let mut t = TestGame::new(2);
    let breaker = t.battlefield(P0, "Atarka Beastbreaker");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Forest", 5);
    t.activate(P0, breaker, 0, &[]).unwrap();
    destroy(&mut t, wurm);
    assert!(!can_activate(&mut t, P0, breaker));
    t.resolve_all();
    assert_eq!(t.pt(breaker), (6, 6));
}
