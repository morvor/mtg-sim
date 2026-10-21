//! The worked examples of CR 111 (tokens), played out as tests. Most are cited by the
//! tests in `r111_tokens.rs`.

use super::r105_util::tokens_of;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn dwarven_reinforcements_tokens_are_named_dwarf_berserker_token() {
    cr!("111.4");
    cr_example!("111.4", "Dwarven Reinforcements");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    let dr = t.hand(P0, "Dwarven Reinforcements");
    t.cast(P0, dr).go();
    t.resolve();
    let tokens = tokens_of(&t, P0);
    assert_eq!(tokens.len(), 2);
    for tok in tokens {
        let c = &t.obj_now(tok).chars;
        assert_eq!(c.name.as_str(), "Dwarf Berserker Token");
        assert!(c.has_subtype("Dwarf") && c.has_subtype("Berserker"));
        assert_eq!(t.pt(tok), (2, 1));
    }
}
