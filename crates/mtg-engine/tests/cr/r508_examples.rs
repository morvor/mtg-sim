//! The worked examples of CR 508 (the declare attackers step), played out as tests. The
//! 508.1c and 508.1d examples are cited by tests in `r508_declare_attackers.rs`.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const TEXT: &str = "Whenever a green creature attacks, destroy that creature at end of combat.";

#[test]
fn a_blue_attacker_later_turned_green_doesnt_trigger_whenever_a_green_creature_attacks() {
    cr!("508.2a", "603.2");
    cr_example!("508.2a", "Whenever a green creature attacks");
    let mut t = TestGame::new(2);
    bf(&mut t, P1, custom_card("Green Watch", "Enchantment", None, TEXT));
    let blue = t.battlefield(P0, "Coral Merfolk");
    declare(&mut t, &[(blue, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(t.stack_len(), 0);
    // Turned green after it attacked.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetColors(ColorSet::single(Color::Green))],
            duration: Duration::EndOfTurn,
        },
        &[blue],
    );
    assert!(t.obj_now(blue).chars.colors.contains(Color::Green));
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    t.resolve_all();
    go_to(&mut t, Step::PostcombatMain);
    assert!(t.on_battlefield(blue), "{}", t.dump_log());

    // A green attacker does trigger it, and is destroyed at end of combat.
    let mut t = TestGame::new(2);
    bf(&mut t, P1, custom_card("Green Watch", "Enchantment", None, TEXT));
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    go_to(&mut t, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "{}", t.dump_log());
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
