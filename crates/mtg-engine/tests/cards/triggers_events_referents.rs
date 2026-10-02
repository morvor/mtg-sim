//! What trigger bodies refer to (CR 603.2, 608.2h): "enchanted Forest" (the enchanted
//! permanent, CR 303.4), "that creature's toughness to the creature's controller", "it"
//! after an intervening "if ~ ..." condition when the event has no object of its own,
//! the owner named by "an opponent owns", "[Title] [Name]" for a legendary card, and "the
//! monarch's end step" (CR 725.1).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

#[test]
fn enchanted_forest_is_put_into_a_graveyard() {
    cr!("303.4", "700.4", "603.6e");
    supported("Genju of the Cedars");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let genju = t.battlefield(P0, "Genju of the Cedars");
    t.g.attach(genju, Entity::Object(forest));
    t.lands(P0, "Forest", 2);
    // "{2}: Enchanted Forest becomes a 4/4 green Spirit creature until end of turn."
    t.activate(P0, genju, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(forest), (4, 4));
    // When it's put into a graveyard, return the Genju from the graveyard to the hand.
    t.answer_yes(P0, true);
    t.g.destroy(forest, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Genju of the Cedars"));
}

#[test]
fn damage_equal_to_that_creatures_toughness_to_the_creatures_controller() {
    cr!("603.10a", "303.4");
    supported("Creature Bond");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bond = t.battlefield(P0, "Creature Bond");
    t.g.attach(bond, Entity::Object(wurm));
    t.g.destroy(wurm, None);
    t.resolve_all();
    // Craw Wurm is a 6/4: 4 damage to its controller.
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn whenever_you_scry_if_this_is_tapped_untap_it() {
    cr!("603.4", "701.22a");
    supported("Legolas, Counter of Kills");
    ruling!(
        "Legolas, Counter of Kills",
        "lets you choose whether or not to untap Legolas"
    );
    let mut t = TestGame::new(2);
    let legolas = t.battlefield(P0, "Legolas, Counter of Kills");
    t.lands(P0, "Island", 3);
    t.g.tap(legolas);
    let scry = |t: &mut TestGame, untap: bool| {
        let opt = t.hand(P0, "Opt");
        t.answer_yes(P0, untap);
        t.cast(P0, opt).go();
        t.resolve_all();
    };
    // Declined: it triggers again the next time.
    scry(&mut t, false);
    assert!(t.obj_now(legolas).tapped);
    scry(&mut t, true);
    assert!(!t.obj_now(legolas).tapped);
    // Done this turn.
    t.g.tap(legolas);
    scry(&mut t, true);
    assert!(t.obj_now(legolas).tapped);
    // Untapped: the intervening "if" fails.
    let mut t = TestGame::new(2);
    let legolas = t.battlefield(P0, "Legolas, Counter of Kills");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0, "no trigger while it's untapped");
    assert!(!t.obj_now(legolas).tapped);
}

#[test]
fn a_permanent_an_opponent_owns_enters_under_your_control() {
    cr!("603.6a", "108.3");
    supported("Brainstealer Dragon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brainstealer Dragon");
    let ring = t.battlefield(P1, "Sol Ring");
    // Gain control of it by putting it onto the battlefield under your control.
    let exiled = t.g.exile_object(ring, None).unwrap();
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: exiled,
        to: mtg_engine::object::Zone::Battlefield,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P0),
            ..Default::default()
        },
        source: None,
    });
    t.resolve_all();
    // "They" — the owner — lose life equal to its mana value.
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_legendary_card_called_by_its_title_and_name() {
    cr!("201.5c", "603.6a");
    supported("General Kudro of Drannith");
    let mut t = TestGame::new(2);
    let target = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.enter(P0, "General Kudro of Drannith");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn at_the_beginning_of_the_monarchs_end_step() {
    cr!("725.1");
    // (Its first ability, about your commander, isn't compiled yet.)
    let archivist = card("Archivist of Gondor");
    let unsupported = archivist.unsupported_text();
    assert!(
        unsupported
            .iter()
            .all(|t| !t.contains("monarch's end step")),
        "{unsupported:?}"
    );
    // The same turns with and without Archivist of Gondor (the monarch also draws a card
    // at the beginning of their end step by the rules, CR 725.2).
    let run = |archivist: bool| {
        let mut t = TestGame::new(2);
        if archivist {
            t.battlefield(P0, "Archivist of Gondor");
        }
        t.g.monarch = Some(P1);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let p0 = t.hand_size(P0);
        t.advance_to(P1, Step::End);
        t.resolve_all();
        (p0, t.hand_size(P1))
    };
    let (p0_without, p1_without) = run(false);
    let (p0_with, p1_with) = run(true);
    assert_eq!(p0_with, p0_without, "not the monarch's end step");
    assert_eq!(p1_with, p1_without + 1, "that player draws a card");
}

#[test]
fn another_creature_you_control_named_this_card_enters() {
    cr!("201.2", "603.6a");
    supported("Gladewalker Ritualist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gladewalker Ritualist");
    let hand = t.hand_size(P0);
    // Another Shapeshifter that isn't named Gladewalker Ritualist: no.
    t.enter(P0, "Changeling Outcast");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // An opponent's: no.
    t.enter(P1, "Gladewalker Ritualist");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // The first one sees the second enter; the second doesn't see itself ("another").
    t.enter(P0, "Gladewalker Ritualist");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn enchanted_creature_or_another_modified_creature_you_control_dies() {
    cr!("700.9", "603.10a", "303.4");
    supported("One with the Kami");
    ruling!(
        "One with the Kami",
        "A creature with a counter on it is considered modified no matter what kind of counter"
    );
    ruling!(
        "One with the Kami",
        "An Aura controlled by an opponent does not cause a creature you control to be modified"
    );
    let spirits = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Spirit"))
            .count()
    };
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let kami = t.battlefield(P0, "One with the Kami");
    t.g.attach(kami, Entity::Object(bear));
    // Unmodified, or enchanted only by an opponent's Aura: no.
    let plain = t.battlefield(P0, "Craw Wurm");
    t.g.destroy(plain, None);
    t.resolve_all();
    let theirs = t.battlefield(P0, "Craw Wurm");
    let pacifism = t.battlefield(P1, "Pacifism");
    t.g.attach(pacifism, Entity::Object(theirs));
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!(spirits(&t), 0);
    // A -1/-1 counter an opponent put on it: a 5/3, five Spirits.
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.g.add_counters(Entity::Object(wurm), "-1/-1", 1, None);
    t.g.destroy(wurm, None);
    t.resolve_all();
    assert_eq!(spirits(&t), 5);
    // The enchanted creature (a 2/2): two more.
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(spirits(&t), 7);
}
