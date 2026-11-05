//! Rulings batch P022 — Vesuvan Doppelganger copying flip cards (CR 707.3, 710.2, 710.4)
//! and several myriad tokens for one player (CR 702.116a, 508.4, 614.1a).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const VESUVAN: &str = "Vesuvan Doppelganger";
const JUSHI: &str = "Jushi Apprentice // Tomoya the Revealer";
const LAVARUNNER: &str = "Akki Lavarunner // Tok-Tok, Volcano Born";

#[test]
fn vesuvan_doppelganger_copies_a_flip_card_in_its_own_flip_state() {
    cr!("707.3", "710.2", "707.2");
    ruling!("Vesuvan Doppelganger", "If a Doppelganger is flipped, and it copies a flip card in any state, it will copy both \"sides\" of that card but use the flipped side. If a Doppelganger is unflipped, and it copies a flip card in any state, it will copy both \"sides\" of that card but use the unflipped side. In the second case, if it ever meets the flip conditions of its new ability, it will flip and used the flipped side of what it is copying.");
    supported(VESUVAN);
    supported(JUSHI);
    supported(LAVARUNNER);
    let mut t = TestGame::new(2);
    // P1's flipped Jushi Apprentice (Tomoya the Revealer) and unflipped Akki Lavarunner.
    let jushi = t.battlefield(P1, JUSHI);
    assert!(mtg_engine::flip::flip(&mut t.g, jushi));
    t.g.recompute();
    assert_eq!(name_of(&t, jushi), "Tomoya the Revealer");
    let akki = t.battlefield(P1, LAVARUNNER);
    // The unflipped Doppelganger copies the flipped Jushi: it's a Jushi Apprentice.
    let v = enter_copying(&mut t, P0, VESUVAN, jushi);
    assert_eq!(name_of(&t, v), "Jushi Apprentice");
    assert!(!t.obj_now(v).flipped);
    // At P0's upkeep it becomes a copy of the Akki Lavarunner instead...
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(akki)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, v), "Akki Lavarunner");
    // ...and meets its flip condition ("Whenever this creature deals damage to an
    // opponent, flip it."): it flips into Tok-Tok, Volcano Born.
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(v, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.obj_now(v).flipped);
    assert_eq!(name_of(&t, v), "Tok-Tok, Volcano Born");
    // Flipped, it copies an unflipped Jushi Apprentice at the next upkeep: it uses the
    // flipped side.
    let jushi2 = t.battlefield(P1, JUSHI);
    assert_eq!(name_of(&t, jushi2), "Jushi Apprentice");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(jushi2)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, v), "Tomoya the Revealer");
    // Its color stays blue (it doesn't copy color).
    assert_eq!(t.obj_now(v).chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn a_token_copy_of_a_flipped_flip_card_is_unflipped() {
    cr!("707.3", "110.5b", "710.2");
    supported(JUSHI);
    let mut t = TestGame::new(2);
    let jushi = t.battlefield(P0, JUSHI);
    assert!(mtg_engine::flip::flip(&mut t.g, jushi));
    t.g.recompute();
    assert_eq!(name_of(&t, jushi), "Tomoya the Revealer");
    let tok = one(cast_spell(
        &mut t,
        P0,
        "Cackling Counterpart",
        "{1}{U}{U}",
        &[obj(jushi)],
        None,
        None,
    ));
    assert!(!t.obj_now(tok).flipped);
    assert_eq!(name_of(&t, tok), "Jushi Apprentice");
}

#[test]
fn myriad_tokens_for_one_player_each_choose_what_they_attack() {
    cr!("702.116a", "508.4", "614.1a");
    ruling!("Mass of Mysteries", "If an instance of myriad creates more than one token for any given player (due to an effect such as the one Doubling Season creates), you may choose separately for each token whether it's attacking the player or a planeswalker they control.");
    supported("Mass of Mysteries");
    supported("Jace Beleren");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Mass of Mysteries");
    t.battlefield(P0, "Doubling Season");
    let air = t.battlefield(P0, "Air Elemental");
    let jace = t.battlefield(P2, "Jace Beleren");
    t.g.add_counters(Entity::Object(jace), counters::LOYALTY, 3, None);
    // Mass of Mysteries' beginning-of-combat trigger gives the Air Elemental myriad; it
    // attacks P1; myriad makes two tokens for P2: one attacks P2, one Jace.
    t.answer_targets(P0, &[obj(air)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P0, &[obj(jace)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(air, Entity::Player(P1))]),
    );
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 2, "{}", t.dump_log());
    let mut targets: Vec<Entity> = t
        .g
        .combat
        .as_ref()
        .unwrap()
        .attackers
        .iter()
        .filter(|a| toks.contains(&a.id))
        .filter_map(|a| a.target)
        .collect();
    targets.sort_by_key(|e| format!("{e:?}"));
    let mut want = vec![Entity::Player(P2), obj(jace)];
    want.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(targets, want);
}
