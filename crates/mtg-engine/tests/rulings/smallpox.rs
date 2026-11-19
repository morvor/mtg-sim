//! Smallpox: "Each player loses 1 life, discards a card, sacrifices a creature of their
//! choice, then sacrifices a land of their choice." Each step is done by every player at
//! the same time before the next (CR 608.2e, 101.4); what triggers waits until Smallpox
//! has resolved (CR 603.3).

use crate::r_s01_common::{supported, triggers_on_stack};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn smallpox_each_step_is_done_by_every_player_before_the_next() {
    cr!("608.2e", "101.4", "603.3", "603.10a");
    ruling!(
        "Smallpox",
        "As Smallpox resolves, first each player loses 1 life at the same time. Next, each player chooses a card in hand without revealing it, then all of the cards are discarded at the same time."
    );
    ruling!(
        "Smallpox",
        "Abilities of the creatures won't modify or trigger on the lands being sacrificed, but may modify or trigger on the cards being discarded."
    );
    ruling!(
        "Smallpox",
        "Any abilities that trigger while Smallpox is resolving will wait to be put onto the stack until Smallpox is done resolving."
    );
    supported("Smallpox");
    supported("Archfiend of Ifnir");
    supported("Titania, Protector of Argoth");
    // P0's only creature is Archfiend of Ifnir ("Whenever you cycle or discard another
    // card, put a -1/-1 counter on each creature your opponents control."): it sees P0's
    // discard before it's sacrificed. P1's only creature is Titania ("Whenever a land you
    // control is put into a graveyard from the battlefield, create a 5/3 green Elemental
    // creature token."): it's gone when the lands are sacrificed.
    let mut t = TestGame::new(2);
    let archfiend = t.battlefield(P0, "Archfiend of Ifnir");
    let titania = t.battlefield(P1, "Titania, Protector of Argoth");
    let swamps = t.lands(P0, "Swamp", 2);
    let forest = t.battlefield(P1, "Forest");
    t.hand(P0, "Island");
    t.hand(P1, "Island");
    let spell = t.hand(P0, "Smallpox");
    t.cast(P0, spell).go();
    let from = t.g.turn_events.len();
    t.resolve();
    // Every life loss, then every discard, then the creatures, then the lands.
    let mut steps: Vec<&str> = Vec::new();
    for e in &t.g.turn_events[from..] {
        let step = match e {
            Event::LifeLost { .. } => "life",
            Event::Discarded { .. } => "discard",
            Event::Sacrificed { obj, .. } if [archfiend, titania].contains(obj) => "creature",
            Event::Sacrificed { .. } => "land",
            _ => continue,
        };
        steps.push(step);
    }
    assert_eq!(
        steps,
        ["life", "life", "discard", "discard", "creature", "creature", "land", "land"]
    );
    assert!(!t.on_battlefield(archfiend) && !t.on_battlefield(titania));
    assert!(!t.on_battlefield(forest));
    assert!(swamps.iter().filter(|s| t.on_battlefield(**s)).count() == 1);
    // Smallpox has resolved; only then is the Archfiend's trigger put onto the stack.
    // Titania's didn't trigger.
    assert!(t.in_graveyard(P0, "Smallpox"));
    assert_eq!(t.stack_len(), 1);
    assert_eq!(triggers_on_stack(&t, "-1/-1 counter"), 1);
    assert!(
        t.g.permanents().all(|o| o.chars.name != "Elemental"),
        "no Elemental token"
    );
    assert_eq!((t.life(P0), t.life(P1)), (19, 19));
}
