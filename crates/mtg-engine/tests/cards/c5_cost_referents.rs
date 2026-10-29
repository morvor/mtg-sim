//! cards5-referents-statics, family `cost-referents`: what a cost did, read by the effect
//! (CR 400.7j, 608.2h).

use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn synthesis_pod_reveals_a_card_with_one_more_than_the_exiled_spells_mana_value() {
    cr!("400.7j", "608.2h");
    ruling!(
        "Synthesis Pod",
        "you must exile a spell you control on the stack"
    );
    ruling!(
        "Synthesis Pod",
        "The spell you exiled as part of the cost to activate Synthesis Pod's ability will remain in exile"
    );
    supported("Synthesis Pod");
    let mut t = TestGame::new(2);
    let pod = t.battlefield(P0, "Synthesis Pod");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    // P1's library, top first: Island, Hill Giant (4), Centaur Courser (3), Island.
    t.library_top(P1, "Island");
    t.library_top(P1, "Centaur Courser");
    t.library_top(P1, "Hill Giant");
    t.library_top(P1, "Island");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    let lib = t.library_size(P1);
    t.answer_yes(P0, true);
    t.activate(P0, pod, 0, &[Entity::Player(P1)])
        .expect("activates exiling the Bears spell");
    assert!(t.in_exile("Grizzly Bears"));
    t.resolve_all();
    // Centaur Courser (1 + 2) was exiled and cast; the others went back.
    assert_eq!(t.named_on_battlefield("Centaur Courser").len(), 1);
    let courser = t.named_on_battlefield("Centaur Courser")[0];
    assert_eq!(t.obj_now(courser).controller, P0);
    assert_eq!(t.library_size(P1), lib - 1);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}
