//! Token-creation replacements II (CR 614.1a, 111.10, 707.2): other tokens created
//! instead of the ones an event would create — one kind for another (Fisher's Talent),
//! one of each of several kinds (Academy Manufactor), a choice of two (Jinnie Fay), or
//! copies of a permanent the first time each turn (Moonlit Meditation, Esix, Mirrormind
//! Crown).

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn tokens_named(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name).len()
}

fn raise_the_alarm(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Plains", 2);
    let c = t.hand(p, "Raise the Alarm");
    t.cast(p, c).go();
    t.resolve_all();
}

#[test]
fn academy_manufactor_creates_that_many_of_each_instead() {
    cr!("614.1a", "111.10");
    ruling!(
        "Academy Manufactor",
        "If you control one Academy Manufactor and would create some number of Clue, Food, or Treasure tokens, you will instead create that many Clue tokens, that many Food tokens, and that many Treasure tokens."
    );
    ruling!(
        "Academy Manufactor",
        "If you control two Academy Manufactors and would create some number of Clue, Food, or Treasure tokens, you will instead create three times that many Clue tokens"
    );
    supported("Academy Manufactor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Academy Manufactor");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Strike It Rich");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Treasure Token"), 1);
    assert_eq!(tokens_named(&t, "Clue Token"), 1);
    assert_eq!(tokens_named(&t, "Food Token"), 1);
    // Other tokens aren't replaced.
    raise_the_alarm(&mut t, P0);
    assert_eq!(tokens_named(&t, "Soldier Token"), 2);
    assert_eq!(tokens_named(&t, "Clue Token"), 1);
    // Two of them: three of each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Academy Manufactor");
    t.battlefield(P0, "Academy Manufactor");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Strike It Rich");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Treasure Token"), 3);
    assert_eq!(tokens_named(&t, "Clue Token"), 3);
    assert_eq!(tokens_named(&t, "Food Token"), 3);
}

#[test]
fn jinnie_fay_may_replace_tokens_with_cats_or_dogs() {
    cr!("614.1a", "111.10");
    ruling!(
        "Jinnie Fay, Jetmir's Second",
        "The tokens' characteristics are entirely replaced"
    );
    supported("Jinnie Fay, Jetmir's Second");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jinnie Fay, Jetmir's Second");
    t.answer_yes(P0, true);
    // Cat (0) or Dog (1): Dogs.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    raise_the_alarm(&mut t, P0);
    let dogs = t.named_on_battlefield("Dog Token");
    assert_eq!(dogs.len(), 2);
    assert_eq!(t.pt(dogs[0]), (3, 1));
    assert!(t
        .obj_now(dogs[0])
        .has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
    assert_eq!(tokens_named(&t, "Soldier Token"), 0);
    // Declined: the Soldiers.
    t.answer_yes(P0, false);
    raise_the_alarm(&mut t, P0);
    assert_eq!(tokens_named(&t, "Soldier Token"), 2);
}

#[test]
fn moonlit_meditation_copies_enchanted_permanent_the_first_time_each_turn() {
    cr!("614.1a", "707.2");
    supported("Moonlit Meditation");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let m = t.hand(P0, "Moonlit Meditation");
    t.cast(P0, m).go();
    t.resolve_all();
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    let giants = t.named_on_battlefield("Hill Giant");
    assert_eq!(giants.len(), 3);
    assert_eq!(
        giants.iter().filter(|g| t.obj_now(**g).is_token()).count(),
        2
    );
    assert_eq!(tokens_named(&t, "Soldier Token"), 0);
    // Not the first time this turn: Soldiers.
    raise_the_alarm(&mut t, P0);
    assert_eq!(tokens_named(&t, "Soldier Token"), 2);
}

#[test]
fn esix_copies_a_chosen_creature_only_during_your_turn() {
    cr!("614.1a", "707.2");
    supported("Esix, Fractal Bloom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Esix, Fractal Bloom");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    raise_the_alarm(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
    assert_eq!(tokens_named(&t, "Soldier Token"), 0);
    // During an opponent's turn: no replacement.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Esix, Fractal Bloom");
    t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Plains", 2);
    let c = t.hand(P0, "Raise the Alarm");
    t.g.turn.priority = Some(P0);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Soldier Token"), 2);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn mirrormind_crown_works_only_while_attached() {
    cr!("614.1a", "707.2");
    supported("Mirrormind Crown");
    let mut t = TestGame::new(2);
    let crown = t.battlefield(P0, "Mirrormind Crown");
    let giant = t.battlefield(P0, "Hill Giant");
    // Unattached: Soldiers.
    raise_the_alarm(&mut t, P0);
    assert_eq!(tokens_named(&t, "Soldier Token"), 2);
    // Attached, the next turn: copies of the equipped creature.
    let mut t = TestGame::new(2);
    let crown2 = t.battlefield(P0, "Mirrormind Crown");
    let giant2 = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, crown2, 0, &[Entity::Object(giant2)])
        .unwrap();
    t.resolve_all();
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 3);
    let _ = (crown, giant);
}

#[test]
fn fishers_talent_level_2_makes_sharks_instead_of_fish() {
    cr!("614.1a", "716.2");
    ruling!(
        "Fisher's Talent",
        "The level 2 and level 3 class abilities of Fisher's Talent will replace any Fish or Shark tokens you create"
    );
    supported("Fisher's Talent");
    let mut t = TestGame::new(2);
    let talent = t.battlefield(P0, "Fisher's Talent");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.activate(P0, talent, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(talent).class_level, 2);
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let sharks = t.named_on_battlefield("Shark Token");
    assert_eq!(sharks.len(), 1);
    assert_eq!(t.pt(sharks[0]), (3, 3));
    assert_eq!(tokens_named(&t, "Fish Token"), 0);
}
