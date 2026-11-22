//! CR 701.7: create; CR 701.8: destroy; CR 701.16: investigate; CR 701.26: tap and
//! untap.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn creating_tokens_puts_them_onto_the_battlefield() {
    cr!("701.7", "701.7a");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    // "Create two 1/1 white Soldier creature tokens."
    let s = t.hand(P0, "Raise the Alarm");
    t.cast(P0, s).go();
    t.resolve();
    let soldiers = t.named_on_battlefield("Soldier Token");
    assert_eq!(soldiers.len(), 2);
    for s in soldiers {
        let o = t.obj(s);
        assert_eq!(o.kind, ObjKind::Token);
        assert_eq!(o.controller, P0);
        assert_eq!((o.power(), o.toughness()), (1, 1));
        assert!(o.chars.colors.contains(Color::White));
        assert!(o.chars.has_subtype("Soldier"));
    }
}

#[test]
fn a_token_creation_replacement_sees_the_tokens_as_they_are_created() {
    cr!("701.7b");
    // Ojer Taq's replacement effect, and an effect that makes noncreature artifacts
    // creatures.
    let ojer = oracle_card(
        "Deep Foundation",
        "Legendary Creature — God",
        "{4}{W}{W}",
        Some((6, 6)),
        "If one or more creature tokens would be created under your control, three times that many of those tokens are created instead.",
    );
    let animate = oracle_card(
        "Animation Field",
        "Enchantment",
        "{2}",
        None,
        "Noncreature artifacts you control are 2/2 artifact creatures.",
    );
    let mut t = TestGame::new(2);
    t.custom(P0, ojer, Zone::Battlefield);
    t.custom(P0, animate, Zone::Battlefield);
    // A Treasure is created as a noncreature artifact token: it isn't tripled, even
    // though it's a creature once it's on the battlefield.
    let def = oracle_card("Find Gold", "Instant", "{0}", None, "Create a Treasure token.");
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    let treasures: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.obj(*o).chars.has_subtype("Treasure"))
        .collect();
    assert_eq!(treasures.len(), 1);
    assert!(t.obj(treasures[0]).is_creature());
    // Creature tokens are tripled.
    supported("Raise the Alarm");
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Raise the Alarm");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 6);
}

#[test]
fn token_doublers_multiply_the_tokens_created() {
    cr!("701.7a", "614.1a");
    supported("Parallel Lives");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Parallel Lives");
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Raise the Alarm");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 4);
    // Only tokens created under its controller's control.
    t.lands(P1, "Plains", 2);
    let s = t.hand(P1, "Raise the Alarm");
    t.cast(P1, s).go();
    t.resolve();
    let theirs = t
        .named_on_battlefield("Soldier Token")
        .into_iter()
        .filter(|o| t.obj(*o).controller == P1)
        .count();
    assert_eq!(theirs, 2);
}

#[test]
fn destroying_moves_a_permanent_to_its_owners_graveyard() {
    cr!("701.8", "701.8a", "701.8b");
    supported("Doom Blade");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    let blade = t.hand(P0, "Doom Blade");
    t.cast(P0, blade).target(giant).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    let destroyed = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Destroyed { .. }))
            .count()
    };
    assert_eq!(destroyed(&t), 1);
    // Lethal damage destroys it too (CR 704.5g).
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(bears, Entity::Object(bears), 2, false);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(destroyed(&t), 2);
    // Sacrificing isn't destroying: indestructible doesn't stop it, and it's no destroy
    // event.
    let bears = t.battlefield(P1, "Grizzly Bears");
    run(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![bears])),
            mods: vec![Modification::AddKeyword(
                mtg_engine::keywords::Keyword::new(KeywordKind::Indestructible),
            )],
            duration: Duration::EndOfTurn,
        },
    );
    t.g.sacrifice(bears, P1);
    t.g.flush_events();
    assert!(!t.g.is_live(bears));
    assert_eq!(destroyed(&t), 2);
}

#[test]
fn regeneration_replaces_destruction_but_not_sacrifice() {
    cr!("701.8b", "701.8c");
    supported("Drudge Skeletons");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Drudge Skeletons");
    t.lands(P0, "Swamp", 2);
    // "{B}: Regenerate this creature."
    t.activate(P0, troll, 0, &[]).unwrap();
    t.resolve();
    t.g.destroy(troll, None);
    t.g.recompute();
    assert!(t.on_battlefield(troll));
    assert!(t.obj(troll).tapped);
    // A regeneration shield doesn't stop a sacrifice.
    t.activate(P0, troll, 0, &[]).unwrap();
    t.resolve();
    t.g.sacrifice(troll, P0);
    assert!(t.in_graveyard(P0, "Drudge Skeletons"));
}

#[test]
fn investigate_creates_a_clue_token() {
    cr!("701.1", "701.16", "701.16a");
    // "Investigate. (Create a Clue token. It's an artifact with "{2}, Sacrifice this
    // token: Draw a card.")": the keyword means the game action, whatever the reminder
    // text says.
    let def = oracle_card(
        "Look Closer",
        "Instant",
        "{0}",
        None,
        "Investigate. (Create a Clue token. It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")",
    );
    let mut t = TestGame::new(2);
    let s = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    let clues = t.named_on_battlefield("Clue Token");
    assert_eq!(clues.len(), 1);
    let clue = clues[0];
    assert_eq!(t.obj(clue).kind, ObjKind::Token);
    assert!(t.obj(clue).chars.is(CardType::Artifact));
    assert!(t.obj(clue).chars.has_subtype("Clue"));
    // "{2}, Sacrifice this token: Draw a card."
    t.lands(P0, "Plains", 2);
    let hand = t.hand_size(P0);
    t.activate(P0, clue, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(!t.g.is_live(clue));
}

#[test]
fn only_untapped_permanents_can_be_tapped_and_only_tapped_ones_untapped() {
    cr!("701.26", "701.26a", "701.26b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let events = |t: &TestGame, tapped: bool| {
        t.g.turn_events
            .iter()
            .filter(|e| {
                if tapped {
                    matches!(e, Event::Tapped { .. })
                } else {
                    matches!(e, Event::Untapped { .. })
                }
            })
            .count()
    };
    assert!(t.g.tap(bears));
    t.g.flush_events();
    assert!(t.obj(bears).tapped);
    assert_eq!(events(&t, true), 1);
    // Tapping a tapped permanent doesn't happen: no event, nothing triggers.
    assert!(!t.g.tap(bears));
    t.g.flush_events();
    assert_eq!(events(&t, true), 1);
    assert!(t.g.untap(bears));
    t.g.flush_events();
    assert!(!t.obj(bears).tapped);
    assert_eq!(events(&t, false), 1);
    assert!(!t.g.untap(bears));
    t.g.flush_events();
    assert_eq!(events(&t, false), 1);
    // A "whenever this becomes tapped" trigger doesn't trigger for an already tapped
    // permanent.
    supported("Chrome Companion");
    let companion = t.battlefield(P0, "Chrome Companion");
    t.g.objects[companion.0 as usize].tapped = true;
    run(
        &mut t,
        P0,
        None,
        Effect::Tap {
            what: Sel::All(Filter::Objects(vec![companion])),
        },
    );
    t.settle();
    assert_eq!(t.stack_len(), 0);
}
