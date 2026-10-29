//! Rulings batch P125 — choices made as a spell or ability resolves (CR 608.2d): a color
//! or creature type chosen then, with no chance for players to respond; what can be
//! chosen as a creature type (CR 205.3m).

use crate::r_p125_common::*;
use crate::r_s01_common::watch;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

/// Records, whenever P0 is asked to choose an option, how many objects are on the stack.
fn stack_when_choosing(t: &mut TestGame) -> crate::r_s01_common::Seen<usize> {
    watch(
        t,
        P0,
        |d| matches!(d, Decision::ChooseOption { .. }),
        |g| g.stack.len(),
    )
}

#[test]
fn bathe_in_light_chosen_color_is_unrelated_to_shared_colors() {
    cr!("702.16a", "608.2d");
    ruling!(
        "Bathe in Light",
        "The color you choose as the spell resolves has nothing to do with the color or colors shared by the affected creatures."
    );
    supported("Bathe in Light");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let shock = t.hand(P1, "Shock");
    choose_color(&mut t, P0, Color::Red);
    cast_new(&mut t, P0, "Bathe in Light", &[obj(bears)]);
    t.resolve_all();
    // The green creatures gain protection from red; the red Ogre shares no color with the
    // Bears.
    assert!(t.g.protected_from(bears, shock));
    assert!(t.g.protected_from(elves, shock));
    assert!(!t.g.protected_from(ogre, shock));
}

#[test]
fn brave_the_elements_color_is_chosen_as_it_resolves() {
    cr!("608.2d", "702.16a");
    ruling!(
        "Brave the Elements",
        "You choose a color as Brave the Elements resolves. Once the color is chosen, it’s too late for players to respond."
    );
    supported("Brave the Elements");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shock = t.hand(P1, "Shock");
    let seen = stack_when_choosing(&mut t);
    choose_color(&mut t, P0, Color::Red);
    cast_new(&mut t, P0, "Brave the Elements", &[]);
    assert!(seen.lock().unwrap().is_empty(), "nothing chosen on casting");
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![1], "chosen while it was resolving");
    assert!(t.g.protected_from(lions, shock));
    assert!(!t.g.protected_from(bears, shock), "not white");
}

#[test]
fn selfless_safewright_type_is_chosen_as_its_trigger_resolves() {
    cr!("608.2d", "603.3");
    ruling!(
        "Selfless Safewright",
        "You choose the creature type as Selfless Safewright's last ability resolves."
    );
    supported("Selfless Safewright");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let seen = stack_when_choosing(&mut t);
    choose_creature_type(&mut t, P0, "Elf");
    let wright = t.enter(P0, "Selfless Safewright");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(seen.lock().unwrap().is_empty(), "nothing chosen as it triggers");
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![1]);
    assert!(indestructible(&t, elves) && hexproof(&t, elves));
    assert!(!indestructible(&t, bears));
    assert!(!indestructible(&t, wright), "other permanents only");
}

#[test]
fn riders_of_gavony_chooses_an_existing_creature_type() {
    cr!("205.3m", "702.16a");
    ruling!(
        "Riders of Gavony",
        "You must choose an existing _Magic_ creature type, such as Zombie or Warrior. Card types such as artifact can't be chosen."
    );
    supported("Riders of Gavony");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Zombie");
    let riders = t.enter(P0, "Riders of Gavony");
    t.resolve_all();
    let asked = options_asked(&t, P0, from);
    assert_eq!(asked.len(), 1);
    let opts = &asked[0].1;
    assert!(opts.iter().any(|o| o == "Zombie") && opts.iter().any(|o| o == "Warrior"));
    assert!(!opts.iter().any(|o| o.eq_ignore_ascii_case("artifact")));
    assert!(!opts.iter().any(|o| o.eq_ignore_ascii_case("creature")));
    // Humans P0 controls have protection from Zombies.
    let zombie = t.battlefield(P1, "Walking Corpse");
    t.g.recompute();
    assert!(t.g.protected_from(riders, zombie));
}
