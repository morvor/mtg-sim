//! Rulings batch S18 — waterbend (CR 701.67): "Waterbend [cost]" means "Pay [cost]. For
//! each generic mana in that cost, you may tap an untapped artifact or creature you
//! control rather than pay that mana."

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s03_common::choice_candidates;
use crate::r_s06_common::activate_containing;
use mtg_engine::testing::*;
use mtg_engine::*;

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

#[test]
fn waterbend_can_tap_creatures_that_just_came_under_your_control() {
    cr!("701.67a", "302.6");
    ruling!(
        "Geyser Leaper",
        "You can tap any untapped creature or artifact you control to pay a waterbend cost, even one you haven't controlled continuously since the beginning of your most recent turn."
    );
    supported("Geyser Leaper");
    // Geyser Leaper ("Waterbend {4}: Draw a card, then discard a card."), a Grizzly Bears
    // and an Ornithopter all entered this turn; with one Island, they pay the {4}.
    let mut t = TestGame::new(2);
    let leaper = t.battlefield_sick(P0, "Geyser Leaper");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let rock = t.battlefield_sick(P0, "Ornithopter");
    let island = t.lands(P0, "Island", 1)[0];
    t.answer_choose(P0, &objs(&[leaper, bears, rock]));
    activate_containing(&mut t, P0, leaper, "Draw a card").expect("waterbend");
    for id in [leaper, bears, rock, island] {
        assert!(tapped(&t, id));
    }
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 1);
}

/// Whether P0 can cast Water Whip ({U}{U}, "As an additional cost to cast this spell,
/// waterbend {5}.") with `with` and `bears` Grizzly Bears on the battlefield and `islands`
/// Islands; if so, how many creatures and Islands were tapped.
fn water_whip(with: &str, bears: usize, islands: usize) -> Option<(usize, usize)> {
    let mut t = TestGame::new(2);
    let mut creatures = vec![t.battlefield(P0, with)];
    for _ in 0..bears {
        creatures.push(t.battlefield(P0, "Grizzly Bears"));
    }
    let lands = t.lands(P0, "Island", islands);
    let whip = t.hand(P0, "Water Whip");
    t.answer_targets(P0, &[]);
    t.cast(P0, whip).try_go().ok()?;
    Some((
        creatures.iter().filter(|c| tapped(&t, **c)).count(),
        lands.iter().filter(|c| tapped(&t, **c)).count(),
    ))
}

#[test]
fn cost_changes_dont_change_how_much_waterbend_can_tap_for() {
    cr!("701.67a", "701.67b", "601.2f");
    ruling!(
        "Water Whip",
        "Even if the cost of the spell or ability that waterbend is part of increases or decreases, you still may only tap artifacts or creatures to pay for costs up to the amount of the generic mana in the waterbend cost."
    );
    supported("Water Whip");
    supported("Thalia, Guardian of Thraben");
    supported("Goblin Electromancer");
    // Thalia ("Noncreature spells cost {1} more to cast."): {1}{U}{U} plus waterbend {5}.
    // Only five creatures can be tapped; the {1} must be paid with mana.
    assert_eq!(water_whip("Thalia, Guardian of Thraben", 4, 3), Some((5, 3)));
    assert_eq!(water_whip("Thalia, Guardian of Thraben", 5, 2), None);
    // Goblin Electromancer ("Instant and sorcery spells you cast cost {1} less to cast."):
    // {U}{U} plus {4} of the waterbend's generic mana, which four creatures can pay.
    assert_eq!(water_whip("Goblin Electromancer", 3, 2), Some((4, 2)));
    assert_eq!(water_whip("Goblin Electromancer", 2, 2), None);
}

#[test]
fn a_permanent_used_for_mana_cant_also_be_tapped_for_waterbend() {
    cr!("701.67a", "605.3a", "118.3");
    ruling!(
        "Geyser Leaper",
        "If an artifact or creature you control has a mana ability with {T} in the cost, activating that ability while casting a spell or activating an ability with waterbend will result in the artifact or creature being tapped before you pay the spell's costs. You won't be able to tap it again for waterbend."
    );
    supported("Llanowar Elves");
    // Geyser Leaper (waterbend {4}), Llanowar Elves and a Treasure: each pays for {1}
    // either way, but only once. Three of them can't pay {4}.
    let mut t = TestGame::new(2);
    let leaper = t.battlefield(P0, "Geyser Leaper");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let treasure = create_token(&mut t, P0, "Treasure");
    assert!(activate_containing(&mut t, P0, leaper, "Draw a card").is_err());
    assert!(!tapped(&t, leaper) && !tapped(&t, elves));
    assert!(t.on_battlefield(treasure));
    assert_eq!(t.stack_len(), 0);
    // With an Island too, all four pay.
    let island = t.lands(P0, "Island", 1)[0];
    activate_containing(&mut t, P0, leaper, "Draw a card").expect("waterbend");
    assert!(tapped(&t, leaper) && tapped(&t, elves) && tapped(&t, island));
    assert!(!t.on_battlefield(treasure));
    assert_eq!(t.stack_len(), 1);

    // Tapped for mana first (and the Treasure sacrificed for mana), the Elves and the
    // Treasure aren't among the permanents that can be tapped for waterbend.
    let mut t = TestGame::new(2);
    let leaper = t.battlefield(P0, "Geyser Leaper");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let treasure = create_token(&mut t, P0, "Treasure");
    t.lands(P0, "Island", 2);
    t.activate(P0, elves, 0, &[]).expect("mana");
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.activate(P0, treasure, 0, &[]).expect("mana");
    assert!(tapped(&t, elves));
    assert!(!t.on_battlefield(treasure));
    let from = t.asked().len();
    activate_containing(&mut t, P0, leaper, "Draw a card").expect("waterbend");
    let offered = choice_candidates(&t, from, "Waterbend");
    assert_eq!(offered, vec![vec![Entity::Object(leaper)]]);
}
