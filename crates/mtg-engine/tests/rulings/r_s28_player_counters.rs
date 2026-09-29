//! Rulings batch S28 — counters players have (CR 122.1): rad counters and their inherent
//! triggered ability (CR 728.1), experience counters, and poison counters ("poisoned",
//! CR 122.1f). They're not on any permanent, and proliferate interacts with them.

use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::can_attack;
use crate::r_s11_common::empty_library;
use crate::r_s28_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn rad(t: &TestGame, p: PlayerId) -> u32 {
    player_counters(t, p, "rad")
}

/// P0 proliferates with Steady Progress, choosing `chosen`.
fn proliferate(t: &mut TestGame, chosen: &[Entity]) {
    t.answer_choose(P0, chosen);
    cast_card(t, P0, "Steady Progress");
    t.resolve_all();
}

#[test]
fn proliferate_gives_a_player_another_rad_counter() {
    cr!("122.1", "701.34a", "728.1");
    ruling!(
        "Megaton's Fate",
        "Any effects (such as proliferate) that interact with counters a player gets, has, or loses can interact with rad counters."
    );
    supported("Megaton's Fate");
    // "• Detonate — Megaton's Fate deals 8 damage to each creature. Each player gets four
    // rad counters."
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_card(&mut t, P0, "Megaton's Fate");
    t.resolve_all();
    assert_eq!(rad(&t, P0), 4);
    assert_eq!(rad(&t, P1), 4);
    proliferate(&mut t, &[Entity::Player(P1)]);
    assert_eq!(rad(&t, P0), 4);
    assert_eq!(rad(&t, P1), 5);
}

#[test]
fn rad_counters_stay_with_the_player_not_a_permanent() {
    cr!("122.1", "728.1");
    ruling!(
        "Acquired Mutation",
        "Rad counters are a kind of counter that a player may have. They’re not associated with any specific permanents."
    );
    supported("Acquired Mutation");
    // "Enchanted creature gets +2/+2 and is goaded. Whenever enchanted creature attacks,
    // defending player gets two rad counters."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_card(&mut t, P0, "Acquired Mutation");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(rad(&t, P0), 2);
    // The creature and the Aura leave: the counters stay with the player.
    crate::r_s02_common::destroy(&mut t, bears);
    assert!(t.in_graveyard(P0, "Acquired Mutation"));
    assert_eq!(rad(&t, P0), 2);
    assert!(t.g.permanents().all(|o| o.counter("rad") == 0));
}

#[test]
fn the_rad_trigger_mills_as_many_cards_as_it_can() {
    cr!("728.1", "701.17b");
    ruling!(
        "Strong, the Brutish Thespian",
        "If a player has fewer cards remaining in their library than the number of rad counters they have when the triggered ability resolves, they’ll mill as many cards as they can."
    );
    supported("Strong, the Brutish Thespian");
    // "Enrage — Whenever Strong is dealt damage, you get three rad counters and put three
    // +1/+1 counters on Strong. You gain life rather than lose life from radiation."
    let mut t = TestGame::new(2);
    let strong = t.battlefield(P0, "Strong, the Brutish Thespian");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, giant, 2, strong);
    t.resolve_all();
    assert_eq!(rad(&t, P0), 3);
    // Two cards left after P0's next draw: both are milled, one of them nonland.
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Grizzly Bears", "Forest", "Hill Giant"]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    t.resolve_all();
    assert_eq!(t.library_size(P0), 0);
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(rad(&t, P0), 2);
    assert_eq!(t.life(P0), 21);
    assert!(!t.has_lost(P0));
}

#[test]
fn every_experience_counter_counts_however_it_was_gotten() {
    cr!("122.1", "701.34a");
    ruling!(
        "Kalemne, Disciple of Iroas",
        "All experience counters are identical, no matter how you got them. For example, the last ability will count experience counters that you got from the first ability, from another ability, from proliferating, and so on."
    );
    supported("Kalemne, Disciple of Iroas");
    // "Whenever you cast a creature spell with mana value 5 or greater, you get an
    // experience counter. Kalemne gets +1/+1 for each experience counter you have."
    let mut t = TestGame::new(2);
    let kalemne = t.battlefield(P0, "Kalemne, Disciple of Iroas");
    assert_eq!(t.pt(kalemne), (3, 3));
    // One from another source.
    t.g.add_counters(Entity::Player(P0), "experience", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(kalemne), (4, 4));
    // One from its own ability.
    cast_card(&mut t, P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(player_counters(&t, P0, "experience"), 2);
    assert_eq!(t.pt(kalemne), (5, 5));
    // One from proliferating.
    proliferate(&mut t, &[Entity::Player(P0)]);
    assert_eq!(player_counters(&t, P0, "experience"), 3);
    assert_eq!(t.pt(kalemne), (6, 6));
}

#[test]
fn experience_and_poison_counters_are_different_kinds_of_player_counters() {
    cr!("122.1", "122.1f");
    ruling!(
        "Kalemne, Disciple of Iroas",
        "Experience counters are the second kind of counters a player can have, joining poison."
    );
    let mut t = TestGame::new(2);
    let kalemne = t.battlefield(P0, "Kalemne, Disciple of Iroas");
    t.g.add_counters(Entity::Player(P0), "poison", 2, None);
    t.g.recompute();
    // Poison counters aren't experience counters.
    assert_eq!(t.pt(kalemne), (3, 3));
    cast_card(&mut t, P0, "Craw Wurm");
    t.resolve_all();
    assert_eq!(player_counters(&t, P0, "experience"), 1);
    assert_eq!(player_counters(&t, P0, "poison"), 2);
    assert_eq!(t.pt(kalemne), (4, 4));
}

#[test]
fn a_player_with_a_poison_counter_is_poisoned() {
    cr!("122.1f", "508.1c");
    ruling!(
        "Chained Throatseeker",
        "A player is poisoned if that player has one or more poison counters."
    );
    supported("Chained Throatseeker");
    supported("Viridian Betrayers");
    // Chained Throatseeker: "This creature can't attack unless defending player is
    // poisoned." Viridian Betrayers: "This creature has infect as long as an opponent is
    // poisoned."
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Chained Throatseeker");
    let betrayers = t.battlefield(P0, "Viridian Betrayers");
    t.set_step(P0, Step::BeginningOfCombat);
    t.g.recompute();
    assert!(!can_attack(&mut t, seeker));
    assert!(!t.obj_now(betrayers).has_keyword(KeywordKind::Infect));
    t.g.add_counters(Entity::Player(P1), "poison", 1, None);
    t.g.recompute();
    assert!(can_attack(&mut t, seeker));
    assert!(t.obj_now(betrayers).has_keyword(KeywordKind::Infect));
}
