//! The worked examples of CR 112 (spells), played out as tests.

use super::r105_util::cs;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_color_change_on_a_creature_spell_continues_on_the_permanent() {
    cr!("112.4");
    cr_example!("112.4", "changes a black creature spell to white");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Walking Corpse: a black creature spell.
    t.lands(P0, "Swamp", 2);
    let corpse = t.hand(P0, "Walking Corpse");
    let spell = t.cast(P0, corpse).go();
    assert_eq!(t.obj_now(spell).chars.colors, cs("B"));
    // Purelace: "Target spell or permanent becomes white."
    t.lands(P1, "Plains", 1);
    let lace = t.hand(P1, "Purelace");
    t.g.turn.priority = Some(P1);
    t.cast(P1, lace).target(spell).go();
    t.resolve();
    assert_eq!(t.obj_now(spell).chars.colors, cs("W"));
    t.resolve();
    // It's white when it enters the battlefield, and remains white.
    let perm = t.named_on_battlefield("Walking Corpse")[0];
    assert_eq!(t.obj_now(perm).chars.colors, cs("W"));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(perm).chars.colors, cs("W"));
}
