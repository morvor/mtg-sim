//! Rulings batch S18 — web-slinging (CR 702.188): "You may cast this spell for [cost] if
//! you also return a tapped creature you control to its owner's hand."

use crate::r_s01_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const WEB: CastMethod = CastMethod::Keyword(KeywordKind::WebSlinging);

#[test]
fn a_leaves_the_battlefield_trigger_of_the_returned_creature_resolves_first() {
    cr!("702.188a", "601.2i", "603.3", "405.2");
    ruling!(
        "Spider-Man, Web-Slinger",
        "If any permanent has an ability that triggers when the permanent you return leaves the battlefield, that ability will be put on the stack after you finish casting the spell, and it will resolve before that spell."
    );
    supported("Spider-Man, Web-Slinger");
    supported("Aven Riftwatcher");
    // Aven Riftwatcher: "When this creature enters or leaves the battlefield, you gain 2
    // life." It's returned to pay Spider-Man's web-slinging cost ({W}).
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let rift = t.battlefield(P0, "Aven Riftwatcher");
    t.g.objects[rift.0 as usize].tapped = true;
    let c = t.hand(P0, "Spider-Man, Web-Slinger");
    let spell = t.cast(P0, c).method(WEB).go();
    assert!(t.in_hand(P0, "Aven Riftwatcher"));
    // While the spell was being cast, the ability waited; now it goes on the stack above
    // the spell.
    t.settle();
    assert_eq!(t.g.stack.len(), 2);
    assert_eq!(t.g.stack[0], spell);
    assert_eq!(triggers_on_stack(&t, "gain 2 life"), 1);
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Spider-Man, Web-Slinger").len(), 1);
}
