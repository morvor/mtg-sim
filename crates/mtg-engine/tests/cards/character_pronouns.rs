//! A legendary character's pronoun or plural name as the subject of an instruction
//! (`oracle/patterns/character_pronouns.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn kaito_phases_out_at_end_step_only_the_turn_he_entered() {
    cr!("702.26a", "603.4");
    assert_supported("Kaito Shizuki");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let k = t.hand(P0, "Kaito Shizuki");
    t.cast(P0, k).go();
    t.resolve_all();
    let k = t.named_on_battlefield("Kaito Shizuki")[0];
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.g.obj(k).phased_out, "{}", t.dump_log());
    // His controller's next turn: he phases in, and stays (he didn't enter that turn).
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!t.g.obj(k).phased_out);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.g.obj(k).phased_out, "{}", t.dump_log());
}

#[test]
fn tokka_and_rahzar_deal_damage_for_a_spell_cast_for_less_than_its_mana_value() {
    cr!("603.4", "202.3");
    assert_supported("Tokka & Rahzar, Terrible Twos");
    for (reduced, life) in [(true, 17), (false, 20)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(P0, "Tokka & Rahzar, Terrible Twos");
        if reduced {
            // "Spells cost {1} less to cast."
            t.battlefield(P0, "Helm of Awakening");
        }
        t.lands(P1, "Forest", 2);
        let bears = t.hand(P1, "Grizzly Bears");
        t.cast(P1, bears).go();
        t.resolve_all();
        assert_eq!(t.life(P1), life, "reduced {reduced}: {}", t.dump_log());
    }
}
