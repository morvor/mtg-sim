//! CR 607.5: abilities an object acquires together keep their links with each other and
//! "can't be linked to any other ability, regardless of what other abilities the object
//! may currently have or may have had in the past" — even after a game has handed out
//! every fresh link once.

use mtg_engine::ability::AbilityKind;
use mtg_engine::ability_grants::FRESH_LINK_BASE;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the activated ability of `source` whose text contains `text`.
fn activate_text(t: &mut TestGame, p: PlayerId, source: ObjectId, text: &str, targets: &[Entity]) {
    t.g.recompute();
    let i = t
        .obj(source)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains(text))
        .unwrap_or_else(|| panic!("no activated ability {text:?}"));
    t.activate(p, source, i, targets)
        .expect("activation failed");
    t.resolve();
}

/// The links of `id`'s abilities whose text contains `text`.
fn links(t: &mut TestGame, id: ObjectId, text: &str) -> Vec<u16> {
    t.g.recompute();
    t.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(text))
        .map(|a| a.link)
        .collect()
}

#[test]
fn links_an_effect_gives_stay_distinct_after_every_fresh_link_was_handed_out() {
    cr!("607.5");
    let mut t = TestGame::new(2);
    // "{U}: This creature gains all activated abilities of target creature until end of
    // turn. ..."
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    // "{R}, Exile the top ten cards of your library: This creature deals 2 damage to any
    // target."
    let slogger = t.battlefield(P1, "Arc-Slogger");
    // "{B}{G}: Exile target creature blocking or blocked by this creature." / "{2}{B}: Put
    // a creature card exiled with this creature onto the battlefield under your control."
    let sisters = t.battlefield(P1, "Sisters of Stone Death");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 8);
    let library: Vec<ObjectId> = (0..10).map(|_| t.library_top(P0, "Hill Giant")).collect();
    // The Elemental gains Arc-Slogger's ability: the game's first fresh link.
    activate_text(
        &mut t,
        P0,
        qe,
        "gains all activated",
        &[Entity::Object(slogger)],
    );
    assert_eq!(
        links(&mut t, qe, "Exile the top ten"),
        vec![FRESH_LINK_BASE]
    );
    activate_text(&mut t, P0, qe, "Exile the top ten", &[Entity::Player(P1)]);
    assert!(library.iter().all(|c| t.zone(*c) == Zone::Exile));
    // Every fresh link has been handed out: the next one would be the first again.
    t.g.fresh_links.issued = 0x4000;
    activate_text(
        &mut t,
        P0,
        qe,
        "gains all activated",
        &[Entity::Object(sisters)],
    );
    // Sisters of Stone Death's two abilities are linked to each other on the Elemental,
    // and not to the ability it gained from Arc-Slogger, which it still has.
    let exile = links(&mut t, qe, "Exile target creature blocking");
    let put = links(&mut t, qe, "Put a creature card exiled with");
    assert_eq!(exile.len(), 1);
    assert_eq!(exile, put);
    assert_ne!(exile[0], FRESH_LINK_BASE);
    assert_eq!(
        links(&mut t, qe, "Exile the top ten"),
        vec![FRESH_LINK_BASE]
    );
    // So the return ability can return only the card its exile ability exiled.
    crate::r506_common::declare(&mut t, &[(qe, Entity::Player(P1))]);
    crate::r506_common::block(&mut t, P1, &[(blocker, qe)]);
    crate::r506_common::go_to(&mut t, Step::DeclareBlockers);
    activate_text(
        &mut t,
        P0,
        qe,
        "Exile target creature blocking",
        &[Entity::Object(blocker)],
    );
    let bears_card = t.g.current(blocker);
    assert_eq!(t.zone(bears_card), Zone::Exile);
    t.answer_choose(P0, &[Entity::Object(library[0])]);
    activate_text(&mut t, P0, qe, "Put a creature card exiled with", &[]);
    assert!(t.on_battlefield(bears_card));
    assert_eq!(t.obj_now(bears_card).controller, P0);
    assert!(library.iter().all(|c| t.zone(*c) == Zone::Exile));
}
