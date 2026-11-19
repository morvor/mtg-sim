//! Rulings batch P078 — group slug: "each opponent" life loss and damage in Two-Headed
//! Giant (Infectious Horror, Ironclad Revolutionary, Storm Caller, Welder Automaton),
//! Manabarbs' triggers, and Ironclad Revolutionary's sacrifice.

use crate::r_p078_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn each_opponent_effects_hit_the_opposing_team_twice_in_two_headed_giant() {
    cr!("810.9", "102.3");
    ruling!(
        "Infectious Horror",
        "In a Two-Headed Giant game, Infectious Horror’s ability causes the opposing team to lose 4 life."
    );
    ruling!(
        "Ironclad Revolutionary",
        "In a Two-Headed Giant game, Ironclad Revolutionary’s ability causes the opposing team to lose a total of 4 life."
    );
    ruling!(
        "Storm Caller",
        "In a Two-Headed Giant game, Storm Caller's ability causes the opposing team to lose 4 life."
    );
    ruling!(
        "Welder Automaton",
        "In a Two-Headed Giant game, Welder Automaton's ability causes 2 damage total to be dealt to the opposing team."
    );
    for name in [
        "Infectious Horror",
        "Ironclad Revolutionary",
        "Storm Caller",
        "Welder Automaton",
    ] {
        supported(name);
    }
    let check = |t: &TestGame, before: (i32, i32), loss: i32, what: &str| {
        assert_eq!(t.life(P0), before.0, "{what}: your team");
        assert_eq!(t.life(P1), before.0, "{what}: your team");
        assert_eq!(t.life(P2), before.1 - loss, "{what}: the opposing team");
        assert_eq!(t.life(P3), before.1 - loss, "{what}: the opposing team");
    };
    // Infectious Horror attacks.
    let mut t = thg();
    let horror = t.battlefield(P0, "Infectious Horror");
    let before = (t.life(P0), t.life(P2));
    attack_with(&mut t, &[(horror, pl(P2))]);
    t.resolve_all();
    check(&t, before, 4, "Infectious Horror");
    // Ironclad Revolutionary enters and an artifact is sacrificed.
    let mut t = thg();
    let thopter = t.battlefield(P0, "Ornithopter");
    let before = (t.life(P0), t.life(P2));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.enter(P0, "Ironclad Revolutionary");
    t.resolve_all();
    check(&t, before, 4, "Ironclad Revolutionary");
    // Storm Caller enters.
    let mut t = thg();
    let before = (t.life(P0), t.life(P2));
    t.enter(P0, "Storm Caller");
    t.resolve_all();
    check(&t, before, 4, "Storm Caller");
    // Welder Automaton's ability: 1 damage to each of the two opponents.
    let mut t = thg();
    let welder = t.battlefield(P0, "Welder Automaton");
    t.lands(P0, "Mountain", 4);
    let before = (t.life(P0), t.life(P2));
    t.activate(P0, welder, 0, &[]).unwrap();
    t.resolve_all();
    check(&t, before, 2, "Welder Automaton");
}

#[test]
fn manabarbs_triggers_separately_for_each_land_and_uses_the_stack() {
    cr!("106.12a", "603.2", "605.1b", "603.3");
    ruling!(
        "Manabarbs",
        "The ability will trigger each time a land is tapped for mana. Each ability is separate."
    );
    ruling!(
        "Manabarbs",
        "This ability is not a mana ability. It goes on the stack and can be responded to."
    );
    ruling!(
        "Manabarbs",
        "This card has a triggered ability that causes a separate effect for 1 point of damage each time a land is tapped for mana."
    );
    supported("Manabarbs");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Manabarbs");
    let lands = t.lands(P0, "Mountain", 3);
    for l in &lands {
        assert!(tap_for_mana(&mut t, P0, *l, "{R}"));
    }
    // Not mana abilities: nothing has happened yet; three abilities wait for the stack.
    assert_eq!(t.life(P0), 20);
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn manabarbs_can_be_responded_to_with_the_mana_it_punishes() {
    cr!("605.3a", "117.3c", "405.5");
    ruling!(
        "Manabarbs",
        "On the other hand, a player can tap lands for mana, let the Manabarbs triggered abilities be put on the stack, and then respond to those abilities by casting an instant or activating an ability using that mana. In that case, the spell or ability will resolve first."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Manabarbs");
    let land = t.battlefield(P0, "Mountain");
    assert!(tap_for_mana(&mut t, P0, land, "{R}"));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The mana is still in P0's pool: it pays for Lightning Bolt in response.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(pl(P1)).go();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20, "the Bolt resolved first");
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn ironclad_revolutionary_sacrifices_once_chosen_as_it_resolves() {
    cr!("608.2b", "608.2c", "118.12");
    ruling!(
        "Ironclad Revolutionary",
        "You can sacrifice an artifact only once when Ironclad Revolutionary’s triggered ability resolves. You can’t sacrifice more to get more counters or more life loss."
    );
    ruling!(
        "Ironclad Revolutionary",
        "You choose whether to sacrifice an artifact or not as Ironclad Revolutionary’s ability resolves. No player may take actions between the time you choose whether to sacrifice an artifact and, if you do so, the time that counters are added and life is lost."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    let rev = t.enter(P0, "Ironclad Revolutionary");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // The choice isn't made until the ability resolves.
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !matches!(d, Decision::YesNo { .. })));
    // Trying to sacrifice both: only one goes.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    let from = t.asked().len();
    t.g.resolve_top();
    let during = t.asked()[from..].to_vec();
    t.settle();
    let left =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is(CardType::Artifact))
            .count();
    assert_eq!(left, 1);
    assert_eq!(t.counters(rev, "+1/+1"), 2);
    assert_eq!(t.life(P1), 18);
    // The choice was made during the resolution, with no priority in between.
    assert!(during
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert!(during
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
}
