//! CR 106.4, 605.3b: "Add {G} or {W}": the player chooses the type as the mana ability
//! resolves. An answer that names none of the offered types gets the first one.

use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_out_of_range_mana_type_answer_gets_the_first_type() {
    cr!("106.4", "605.3b");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Selesnya Guildgate");
    let idx = t
        .obj(gate)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.contains("Add {G} or {W}"))
        .expect("the mana ability");
    // Two types are offered: index 2 is one past the end.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, gate, idx, &[]).expect("activates");
    let pool = &t.g.players[0].mana_pool;
    assert_eq!(pool.total(), 1);
    assert_eq!(pool.count(ManaType::G), 1);
}
