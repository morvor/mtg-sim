//! Rulings batch S24 — devotion (CR 700.5): only the colored mana symbols among the mana
//! costs of permanents you control count, as the effect is applied.

use crate::r_s01_common::supported;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::attach_new;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn mana_symbols_in_text_boxes_dont_count_toward_devotion() {
    cr!("700.5");
    ruling!(
        "Gray Merchant of Asphodel",
        "Mana symbols in the text boxes of permanents you control don't count toward your devotion to any color."
    );
    supported("Gray Merchant of Asphodel");
    supported("Orzhov Signet");
    // Orzhov Signet ({2}): "{1}, {T}: Add {W}{B}." — a {B} in its text box only.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Orzhov Signet");
    enter(&mut t, P0, "Gray Merchant of Asphodel");
    t.resolve_all();
    // Only the Merchant's own {B}{B}: devotion to black is two.
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn mana_symbols_in_text_boxes_dont_count_toward_devotion_curly() {
    cr!("700.5");
    ruling!(
        "Callaphe, Beloved of the Sea",
        "Mana symbols in the text boxes of permanents you control don’t count toward your devotion to any color."
    );
    supported("Callaphe, Beloved of the Sea");
    supported("Azorius Signet");
    // "Callaphe's power is equal to your devotion to blue."
    let mut t = TestGame::new(2);
    let callaphe = t.battlefield(P0, "Callaphe, Beloved of the Sea");
    assert_eq!(t.pt(callaphe).0, 2);
    // Azorius Signet: "{1}, {T}: Add {W}{U}." Its {U} is in its text box, not its cost.
    t.battlefield(P0, "Azorius Signet");
    t.battlefield(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(callaphe).0, 2);
    // A {U} in a mana cost does count.
    t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.g.recompute();
    assert_eq!(t.pt(callaphe).0, 3);
}

#[test]
fn numeric_mana_symbols_dont_count_toward_devotion() {
    cr!("700.5", "107.4b");
    ruling!(
        "Karametra's Acolyte",
        "Numeric mana symbols ({0}, {1}, and so on) in mana costs of permanents you control don't count toward your devotion to any color."
    );
    supported("Karametra's Acolyte");
    // Karametra's Acolyte ({3}{G}): "{T}: Add an amount of {G} equal to your devotion to
    // green." Craw Wurm is {4}{G}{G}, Sol Ring {1}, Ornithopter {0}.
    let mut t = TestGame::new(2);
    let acolyte = t.battlefield(P0, "Karametra's Acolyte");
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Ornithopter");
    assert!(tap_for_mana(&mut t, P0, acolyte, "equal to your devotion"));
    assert_eq!(pool(&t, P0, ManaType::G), 3);
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

#[test]
fn numeric_mana_symbols_dont_count_toward_devotion_curly() {
    cr!("700.5", "107.4b");
    ruling!(
        "Aspect of Hydra",
        "Numeric mana symbols ({0}, {1}, and so on) in mana costs of permanents you control don’t count toward your devotion to any color."
    );
    supported("Aspect of Hydra");
    // "Target creature gets +X/+X until end of turn, where X is your devotion to green."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Craw Wurm");
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Forest");
    let aspect = t.hand(P0, "Aspect of Hydra");
    t.cast(P0, aspect).target(bears).go();
    t.resolve_all();
    // Craw Wurm's {G}{G} and the Bears' {G}: X is 3, not 3 + 4 + 1 + 1.
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn colorless_and_generic_symbols_dont_count_toward_devotion() {
    cr!("700.5", "107.4b", "107.4c");
    ruling!(
        "Renata, Called to the Hunt",
        "Colorless and generic mana symbols ({C}, {0}, {1}, {2}, {X}, and so on) in mana costs of permanents you control don't count toward your devotion to any color."
    );
    supported("Renata, Called to the Hunt");
    supported("Thought-Knot Seer");
    supported("Hangarback Walker");
    // "Renata's power is equal to your devotion to green." Renata is {2}{G}{G}.
    let mut t = TestGame::new(2);
    let renata = t.battlefield(P0, "Renata, Called to the Hunt");
    assert_eq!(t.pt(renata).0, 2);
    // Thought-Knot Seer {3}{C}, Hangarback Walker {X}{X}.
    t.battlefield(P0, "Thought-Knot Seer");
    t.battlefield(P0, "Hangarback Walker");
    t.g.recompute();
    assert_eq!(t.pt(renata).0, 2);
    t.battlefield(P0, "Llanowar Elves");
    t.g.recompute();
    assert_eq!(t.pt(renata).0, 3);
}

#[test]
fn colorless_and_generic_symbols_dont_count_toward_devotion_curly() {
    cr!("700.5", "107.4b", "107.4c");
    ruling!(
        "Callaphe, Beloved of the Sea",
        "Colorless and generic mana symbols ({C}, {0}, {1}, {2}, {X}, and so on) in mana costs of permanents you control don’t count toward your devotion to any color."
    );
    supported("Callaphe, Beloved of the Sea");
    supported("Endless One");
    let mut t = TestGame::new(2);
    let callaphe = t.battlefield(P0, "Callaphe, Beloved of the Sea");
    // Thought-Knot Seer {3}{C}, Endless One {X}, Sol Ring {1}.
    t.battlefield(P0, "Thought-Knot Seer");
    t.battlefield(P0, "Endless One");
    t.battlefield(P0, "Sol Ring");
    t.g.recompute();
    assert_eq!(t.pt(callaphe).0, 2);
}

#[test]
fn your_aura_on_an_opponents_permanent_counts_toward_your_devotion() {
    cr!("700.5", "303.4e");
    ruling!(
        "Daxos, Blessed by the Sun",
        "If you put an Aura on an opponent's permanent, you still control the Aura, and mana symbols in its mana cost count towards your devotion."
    );
    supported("Daxos, Blessed by the Sun");
    // "Daxos's toughness is equal to your devotion to white." Daxos is {W}{W}.
    let mut t = TestGame::new(2);
    let daxos = t.battlefield(P0, "Daxos, Blessed by the Sun");
    assert_eq!(t.pt(daxos).1, 2);
    // P0's Pacifism ({1}{W}) on P1's creature counts for P0, not for P1.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pacifism = attach_new(&mut t, P0, "Pacifism", bears);
    t.g.recompute();
    assert_eq!(t.obj_now(pacifism).controller, P0);
    assert_eq!(t.pt(daxos).1, 3);
    // An opponent's own white permanent doesn't count.
    t.battlefield(P1, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.pt(daxos).1, 3);
}

#[test]
fn your_aura_on_an_opponents_permanent_counts_toward_your_devotion_curly() {
    cr!("700.5", "303.4e");
    ruling!(
        "Callaphe, Beloved of the Sea",
        "If you put an Aura on an opponent’s permanent, you still control the Aura, and mana symbols in its mana cost count towards your devotion."
    );
    supported("Claustrophobia");
    let mut t = TestGame::new(2);
    let callaphe = t.battlefield(P0, "Callaphe, Beloved of the Sea");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Claustrophobia is {1}{U}{U}.
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Island");
    let claustrophobia = t.hand(P0, "Claustrophobia");
    t.cast(P0, claustrophobia).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.pt(callaphe).0, 4);
}

#[test]
fn an_aura_on_an_opponents_permanent_and_a_siege_they_protect_count_toward_your_devotion() {
    cr!("700.5", "303.4e", "310.9", "310.12a");
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "Similarly, if you make an opponent the protector of a Siege you control, mana symbols in that battle's mana cost count toward your devotion."
    );
    supported("Clive, Ifrit's Dominant // Ifrit, Warden of Inferno");
    supported("Firebreathing");
    supported("Invasion of Kaladesh");
    // Clive ({4}{R}{R}): "When Clive enters, you may discard your hand, then draw cards
    // equal to your devotion to red."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P0's Firebreathing ({R}) on P1's creature.
    attach_new(&mut t, P0, "Firebreathing", bears);
    // P0's Invasion of Kaladesh ({U}{R}), a Siege whose protector is P1.
    let siege = t.enter(P0, "Invasion of Kaladesh");
    t.resolve_all();
    assert_eq!(t.obj_now(siege).controller, P0);
    assert_eq!(
        mtg_engine::battle::protector(&t.g, t.g.current(siege)),
        Some(P1)
    );
    t.hand(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    assert!(hand >= 1);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno");
    t.resolve_all();
    // {R}{R} + {R} + {R}: four cards.
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn devotion_is_counted_as_the_ability_resolves_including_its_source_if_still_there() {
    cr!("700.5", "608.2h");
    ruling!(
        "Reverent Hoplite",
        "If an activated ability or triggered ability has an effect that depends on your devotion to a color, you count the number of mana symbols of that color among the mana costs of permanents you control as the ability resolves. The permanent with that ability will be counted if it’s still on the battlefield at that time."
    );
    supported("Reverent Hoplite");
    // "When this creature enters, create a number of 1/1 white Human Soldier creature
    // tokens equal to your devotion to white." Hoplite is {4}{W}.
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Reverent Hoplite");
    // A white permanent arriving before the trigger resolves counts.
    t.battlefield(P0, "Savannah Lions");
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 2);
    // The Hoplite leaving before it resolves isn't counted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Savannah Lions");
    let hoplite = enter(&mut t, P0, "Reverent Hoplite");
    move_to(&mut t, hoplite, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}
