//! Rulings batch P116 — "double its power [and toughness]" in triggered abilities whose
//! "it" is the creature that triggered them (CR 701.10a, 701.10b): Maular, the Next
//! Evolution and Wolverine, Claws Out (their other abilities aren't supported). (Grunn, the
//! Lonely King is in `r_p116_power.rs`.)

use crate::r_p116_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn attack(t: &mut TestGame, attackers: &[ObjectId]) {
    let atk = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(atk));
    t.advance_to(P0, Step::DeclareAttackers);
    t.settle();
}

#[test]
fn maular_doubles_the_attacking_creature_not_itself() {
    cr!("701.10a", "701.10b");
    // "Whenever a creature you control with mana value 7 or greater attacks, double its
    // power and toughness until end of turn."
    let big = custom_card("Big Seven", "Creature — Beast", "{6}{G}", Some((3, 4)), "");
    let mut t = TestGame::new(2);
    let maular = t.battlefield(P0, "Maular, the Next Evolution");
    let maular_pt = t.pt(maular);
    let beast = t.custom(P0, big, Zone::Battlefield);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack(&mut t, &[beast, bears]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(beast), (6, 8));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(maular), maular_pt);
}

#[test]
fn wolverine_doubles_the_attacking_mutants_power_only() {
    cr!("701.10a", "701.10b");
    // "Whenever a Mutant you control attacks, double its power until end of turn."
    let mutant = custom_card("Some Mutant", "Creature — Mutant", "{2}", Some((3, 3)), "");
    let mut t = TestGame::new(2);
    let wolverine = t.battlefield(P0, "Wolverine, Claws Out");
    let m = t.custom(P0, mutant, Zone::Battlefield);
    attack(&mut t, &[m]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(m), (6, 3));
    assert_eq!(t.pt(wolverine), (2, 5));
}
