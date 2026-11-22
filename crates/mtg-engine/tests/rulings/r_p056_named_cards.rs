//! Rulings batch P056 — "Activated abilities of sources with the chosen name can't be
//! activated" (Pithing Needle, Sorcerous Spyglass, Voidstone Gargoyle): only activated
//! abilities (CR 602.1, 602.5), of objects in every zone, copies of spells included, and
//! only while the permanent is on the battlefield (CR 611.3a).

use crate::r_p056_common::*;
use crate::r_s01_common::supported;
use crate::r_s04_common::can_cycle;
use crate::r_s06_common::activate_containing;
use crate::r_s26_common::spell_copies;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p`'s `chooser` enters naming `name`.
fn enter_naming(t: &mut TestGame, p: PlayerId, chooser: &str, name: &str) -> ObjectId {
    t.answer(p, DecisionKind::Name, Answer::Text(name.to_string()));
    let id = t.enter(p, chooser);
    t.resolve_all();
    assert_eq!(
        t.obj_now(id).choices.card_name.as_deref(),
        Some(name),
        "{chooser}"
    );
    id
}

#[test]
fn needle_stops_activated_abilities_but_not_triggered_or_static_ones() {
    cr!("602.1", "602.5", "603.1");
    ruling!(
        "Pithing Needle",
        "Activated abilities include a colon and are written in the form \"[cost]: [effect].\" Triggered abilities and static abilities of the named card work normally."
    );
    ruling!(
        "Sorcerous Spyglass",
        "Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder text. Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected"
    );
    supported("Pithing Needle");
    supported("Sorcerous Spyglass");
    supported("Tyrant of Kher Ridges");
    // Tyrant of Kher Ridges: "Flying / When this creature enters, it deals 4 damage to any
    // target. / {R}: This creature gets +1/+0 until end of turn."
    for chooser in ["Pithing Needle", "Sorcerous Spyglass"] {
        let mut t = TestGame::new(2);
        enter_naming(&mut t, P0, chooser, "Tyrant of Kher Ridges");
        t.lands(P0, "Mountain", 1);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let tyrant = t.enter(P0, "Tyrant of Kher Ridges");
        t.resolve_all();
        assert_eq!(t.life(P1), 16, "{chooser}: the triggered ability works");
        assert!(t
            .obj_now(tyrant)
            .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
        assert!(
            activate_containing(&mut t, P0, tyrant, "+1/+0").is_err(),
            "{chooser}"
        );
    }
    // Equip is an activated ability.
    let mut t = TestGame::new(2);
    enter_naming(&mut t, P0, "Sorcerous Spyglass", "Bonesplitter");
    let blade = t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 1);
    assert!(activate_containing(&mut t, P0, blade, "Equip").is_err());
}

#[test]
fn needle_leaving_lets_the_abilities_be_activated_again() {
    cr!("611.3a");
    ruling!(
        "Pithing Needle",
        "Once Pithing Needle has left the battlefield, activated abilities of sources with the chosen name can be activated again."
    );
    let mut t = TestGame::new(2);
    let needle = enter_naming(&mut t, P0, "Pithing Needle", "Tyrant of Kher Ridges");
    let tyrant = t.battlefield(P0, "Tyrant of Kher Ridges");
    t.lands(P0, "Mountain", 1);
    assert!(activate_containing(&mut t, P0, tyrant, "+1/+0").is_err());
    let (p, th) = t.pt(tyrant);
    t.g.destroy(needle, None);
    t.g.flush_events();
    t.settle();
    activate_containing(&mut t, P0, tyrant, "+1/+0").expect("activates");
    t.resolve_all();
    assert_eq!(t.pt(tyrant), (p + 1, th));
}

#[test]
fn needle_and_spyglass_affect_cards_in_every_zone() {
    cr!("602.5", "702.29a");
    ruling!(
        "Pithing Needle",
        "Pithing Needle affects cards regardless of what zone they're in. This includes cards in hand, cards in the graveyard, and exiled cards. For example, a player can't cycle Eternal Dragon or return an Eternal Dragon from their graveyard to hand"
    );
    ruling!(
        "Sorcerous Spyglass",
        "Sorcerous Spyglass affects cards regardless of what zone they're in. This includes cards in hand, cards in the graveyard, and exiled cards."
    );
    supported("Eternal Dragon");
    for chooser in ["Pithing Needle", "Sorcerous Spyglass"] {
        let mut t = TestGame::new(2);
        enter_naming(&mut t, P1, chooser, "Eternal Dragon");
        t.lands(P0, "Plains", 5);
        let in_hand = t.hand(P0, "Eternal Dragon");
        assert!(!can_cycle(&mut t, P0, in_hand), "{chooser}");
        let in_yard = t.graveyard(P0, "Eternal Dragon");
        t.set_step(P0, Step::Upkeep);
        assert!(
            activate_containing(&mut t, P0, in_yard, "Return").is_err(),
            "{chooser}"
        );
        // Without it, both work.
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 7);
        let in_hand = t.hand(P0, "Eternal Dragon");
        assert!(can_cycle(&mut t, P0, in_hand));
        let in_yard = t.graveyard(P0, "Eternal Dragon");
        t.set_step(P0, Step::Upkeep);
        activate_containing(&mut t, P0, in_yard, "Return").expect("activates");
        t.resolve_all();
        assert!(t.in_hand(P0, "Eternal Dragon"));
    }
}

#[test]
fn voidstone_gargoyle_stops_the_abilities_of_copies_of_a_named_spell() {
    cr!("602.5", "707.10");
    ruling!(
        "Voidstone Gargoyle",
        "Activated abilities of copies of a spell with that name (such as Lightning Storm) cannot be activated."
    );
    supported("Voidstone Gargoyle");
    supported("Lightning Storm");
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let storm = t.hand(P0, "Lightning Storm");
    t.cast(P0, storm).target(Entity::Player(P1)).go();
    t.lands(P0, "Island", 2);
    let twin = t.hand(P0, "Twincast");
    let spell = t.g.current(storm);
    t.cast(P0, twin).target(spell).go();
    t.answer_yes(P0, false);
    t.resolve();
    let copy = spell_copies(&t)[0];
    // A copy exists; now the Gargoyle names Lightning Storm.
    let land = t.hand(P1, "Mountain");
    t.answer_choose(P1, &[obj(land)]);
    assert!(activate_containing(&mut t, P1, copy, "Discard a land").is_ok());
    t.answer_yes(P1, false);
    t.resolve();
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Storm".into()),
    );
    let gargoyle = t.enter(P0, "Voidstone Gargoyle");
    assert_eq!(
        t.obj_now(gargoyle).choices.card_name.as_deref(),
        Some("Lightning Storm")
    );
    assert!(t.g.is_live(copy));
    let land = t.hand(P1, "Mountain");
    t.answer_choose(P1, &[obj(land)]);
    assert!(activate_containing(&mut t, P1, copy, "Discard a land").is_err());
    assert!(t.in_hand(P1, "Mountain"));
}
