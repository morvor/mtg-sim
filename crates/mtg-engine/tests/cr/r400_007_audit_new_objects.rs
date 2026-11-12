//! CR 400.7 audit: an object that moves from one zone to another becomes a new object with
//! no memory of its previous existence, and the exceptions (400.7a–m) that let effects
//! follow it. A spell that moves itself ("shuffle ~ into its owner's library") is still
//! subject to an exile replacement a granted flashback gives the spell (400.7g); an
//! ability of a permanent that left doesn't affect the new object it became; a permanent
//! flickered after being kicked isn't kicked any more; counters that remain on an object
//! as it moves (Skullbriar) don't bring along the effects that applied to the old object.

use mtg_engine::card::card;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn pt(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(id)
}

/// Whether `p`'s library was shuffled this turn.
fn shuffled(t: &TestGame, p: PlayerId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::Shuffled { player } if *player == p))
}

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

#[test]
fn a_spell_shuffling_itself_with_a_granted_flashback_is_exiled_instead() {
    cr!("400.7g", "702.34a");
    ruling!(
        "Serene Remembrance",
        "If you give Serene Remembrance the flashback ability and cast it from your graveyard, it will be exiled instead of being shuffled into your library. Any other targets will still be shuffled into their owners’ library, however."
    );
    supported("Serene Remembrance");
    supported("Snapcaster Mage");
    let mut t = TestGame::new(2);
    let remembrance = t.graveyard(P0, "Serene Remembrance");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    // Snapcaster Mage gives it flashback (its mana cost, {G}).
    t.lands(P0, "Island", 2);
    let mage = t.hand(P0, "Snapcaster Mage");
    t.answer_targets(P0, &[Entity::Object(remembrance)]);
    t.cast(P0, mage).go();
    t.resolve_all();
    // "Shuffle Serene Remembrance and up to three target cards from a single graveyard
    // into their owners' libraries."
    t.lands(P0, "Forest", 1);
    let lib1 = t.library_size(P1);
    let lib0 = t.library_size(P0);
    t.cast(P0, remembrance)
        .method(FLASHBACK)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve_all();
    assert!(t.in_exile("Serene Remembrance"));
    assert_eq!(t.library_size(P0), lib0, "it isn't shuffled into P0's library");
    assert_eq!(t.library_size(P1), lib1 + 2, "the targets still are");
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(shuffled(&t, P1));
}

#[test]
fn a_spell_shuffling_itself_whose_targets_all_became_illegal_goes_to_the_graveyard() {
    cr!("608.2b", "400.7");
    ruling!(
        "Serene Remembrance",
        "If you choose at least one target for Serene Remembrance, and all of its targets are illegal when Serene Remembrance tries to resolve, it won’t resolve and none of its effects will happen. You won’t shuffle any library and Serene Remembrance will be put into its owner’s graveyard."
    );
    supported("Serene Remembrance");
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Forest", 1);
    let remembrance = t.hand(P0, "Serene Remembrance");
    let lib0 = t.library_size(P0);
    t.cast(P0, remembrance)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    // In response, Tormod's Crypt exiles P1's graveyard: both targets are new objects in
    // exile.
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.activate(P1, crypt, 0, &[Entity::Player(P1)])
        .expect("Tormod's Crypt");
    t.resolve();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Serene Remembrance"));
    assert_eq!(t.library_size(P0), lib0);
    assert!(!shuffled(&t, P0) && !shuffled(&t, P1));
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
}

#[test]
fn an_ability_of_a_permanent_that_left_doesnt_affect_the_object_it_became() {
    cr!("400.7", "113.7a");
    supported("Nantuko Shade");
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let shade = t.battlefield(P0, "Nantuko Shade");
    t.lands(P0, "Swamp", 1);
    // "{B}: This creature gets +1/+1 until end of turn." In response, the Shade is
    // exiled and returned: the ability's source left, and the new Shade isn't it.
    t.activate(P0, shade, 0, &[]).expect("pump");
    t.lands(P0, "Plains", 1);
    let cs = t.hand(P0, "Cloudshift");
    t.cast(P0, cs).target(shade).go();
    t.resolve();
    let new_shade = t.g.current(shade);
    assert_ne!(new_shade, shade);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pt(&mut t, new_shade), (2, 1));
}

#[test]
fn a_kicked_permanent_flickered_returns_unkicked() {
    cr!("400.7", "400.7d");
    ruling!(
        "Gatekeeper of Malakir",
        "If you put a permanent with a kicker ability onto the battlefield without casting it, you can't kick it."
    );
    supported("Gatekeeper of Malakir");
    supported("Cloudshift");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Kicked: "When this creature enters, if it was kicked, target player sacrifices a
    // creature." (the spell's kicker is information about the spell that became it).
    t.lands(P0, "Swamp", 3);
    let gk = t.hand(P0, "Gatekeeper of Malakir");
    t.cast(P0, gk).kicked(true).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let gk = t.named_on_battlefield("Gatekeeper of Malakir")[0];
    // Flickered, it's a new object that wasn't cast, let alone kicked: nothing triggers.
    t.lands(P0, "Plains", 1);
    let cs = t.hand(P0, "Cloudshift");
    t.cast(P0, cs).target(gk).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(t.g.current(gk)));
}

#[test]
fn a_card_cast_from_exile_and_exiled_again_is_no_longer_exiled_with_its_source() {
    cr!("400.7", "607.2a");
    ruling!(
        "King Narfi's Betrayal",
        "If you cast a spell with the chapter II/III ability, it becomes a new object. If it returns to exile, it's no longer exiled with King Narfi's Betrayal, and you won't be able to cast it again this way."
    );
    supported("King Narfi's Betrayal");
    supported("Dissipate");
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        for _ in 0..4 {
            t.library_top(p, "Forest");
        }
    }
    // I — "Each player mills four cards. Then you may exile a creature or planeswalker card
    // from each graveyard.": P1's Hill Giant.
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.enter(P0, "King Narfi's Betrayal");
    t.resolve_all();
    let exiled = t.g.current(giant);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // II — "Until end of turn, you may cast spells from among cards exiled with this Saga".
    let saga = t.named_on_battlefield("King Narfi's Betrayal")[0];
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    let spell = t.cast(P0, exiled).go();
    // P1's Dissipate counters it and exiles it: a new object in exile.
    t.lands(P1, "Island", 3);
    let dissipate = t.hand(P1, "Dissipate");
    t.cast(P1, dissipate).target(spell).go();
    t.resolve_all();
    let again = t.g.current(giant);
    assert_eq!(t.zone(again), Zone::Exile);
    assert_ne!(again, exiled);
    // It isn't a card exiled with the Saga any more: it can't be cast this way.
    t.lands(P0, "Mountain", 4);
    assert!(t.cast(P0, again).try_go().is_err());
    assert_eq!(t.zone(again), Zone::Exile);
}

#[test]
fn counters_that_remain_on_a_moving_object_dont_bring_effects_along() {
    cr!("400.7", "122.2", "611.2b");
    ruling!(
        "Skullbriar, the Walking Grave",
        "such as Aven Mimeomancer's, stop applying to Skullbriar once it leaves the battlefield. Even though Skullbriar retains the counters, it becomes a new object"
    );
    supported("Skullbriar, the Walking Grave");
    supported("Aven Mimeomancer");
    let mut t = TestGame::new(2);
    let skullbriar = t.battlefield(P0, "Skullbriar, the Walking Grave");
    t.battlefield(P0, "Aven Mimeomancer");
    // "At the beginning of your upkeep, you may put a feather counter on target creature.
    // If you do, that creature has base power and toughness 3/1 and has flying for as long
    // as it has a feather counter on it."
    t.advance_to(P1, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Object(skullbriar)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(skullbriar, "feather"), 1);
    assert_eq!(pt(&mut t, skullbriar), (3, 1));
    assert!(t.obj_now(skullbriar).has_keyword(KeywordKind::Flying));
    // It dies and returns to the battlefield: "Counters remain on Skullbriar as it moves
    // to any zone other than a player's hand or library", but it's a new object.
    t.g.destroy(skullbriar, None);
    t.settle();
    let card = t.g.current(skullbriar);
    assert!(matches!(t.zone(card), Zone::Graveyard(_)));
    assert_eq!(t.g.obj(card).counter("feather"), 1);
    t.g.move_object(card, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0));
    t.settle();
    let back = t.g.current(skullbriar);
    assert!(t.on_battlefield(back));
    assert_eq!(t.g.obj(back).counter("feather"), 1);
    assert_eq!(pt(&mut t, back), (1, 1));
    assert!(!t.obj_now(back).has_keyword(KeywordKind::Flying));
}
