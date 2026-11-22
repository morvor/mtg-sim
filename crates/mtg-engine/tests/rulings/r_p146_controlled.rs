//! Rulings batch P146 — "If you controlled that permanent, ..." after an effect that
//! destroys, returns or exiles it (Kellan, Inquisitive Prodigy and every card the phrase
//! made compile): the condition looks at who controlled it as it last existed on the
//! battlefield, or who controls it now if it's still there (CR 608.2c, 608.2h).

use crate::r_p146_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

const KELLAN: &str = "Kellan, Inquisitive Prodigy // Tail the Suspect";

/// Kellan attacks with the given target for its trigger; the trigger resolves.
fn kellan_attacks(t: &mut TestGame, target: Option<ObjectId>) -> ObjectId {
    let k = t.battlefield(P0, KELLAN);
    t.answer_targets(P0, &target.map(obj).into_iter().collect::<Vec<_>>());
    attack_with(t, &[(k, Entity::Player(P1))]);
    t.resolve_all();
    k
}

#[test]
fn kellan_draws_for_your_artifact_even_if_it_isnt_destroyed() {
    cr!("608.2c", "608.2h", "702.12b");
    ruling!(
        "Kellan, Inquisitive Prodigy // Tail the Suspect",
        "If the artifact you target with Kellan, Inquisitive Prodigy's triggered ability isn't destroyed (perhaps because it's indestructible), you will still draw a card as long as you control the artifact."
    );
    supported(KELLAN);
    // Flying, vigilance; "Whenever Kellan attacks, destroy up to one target artifact. If
    // you controlled that permanent, draw a card."
    // P0's indestructible Darksteel Relic: not destroyed, P0 draws.
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Darksteel Relic");
    let hand = t.hand_size(P0);
    let k = kellan_attacks(&mut t, Some(relic));
    assert!(t.on_battlefield(relic));
    assert_eq!(t.hand_size(P0), hand + 1);
    let o = t.obj(k);
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Vigilance));
    assert!(!o.tapped);
    // P0's Ornithopter: destroyed, P0 draws.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Ornithopter");
    let hand = t.hand_size(P0);
    kellan_attacks(&mut t, Some(o));
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.hand_size(P0), hand + 1);
    // The opponent's: destroyed, no card.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P1, "Ornithopter");
    let hand = t.hand_size(P0);
    kellan_attacks(&mut t, Some(o));
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(t.hand_size(P0), hand);
    // No target: nothing.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    kellan_attacks(&mut t, None);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn geistwave_and_boomerang_basics_draw_for_your_own_permanent() {
    cr!("608.2c", "608.2h");
    // "Return target nonland permanent to its owner's hand. If you controlled that
    // permanent, draw a card."
    for name in ["Geistwave", "Boomerang Basics"] {
        supported(name);
        // P0's own permanent: returned, and a card.
        let mut t = TestGame::new(2);
        let o = t.battlefield(P0, "Ornithopter");
        cast_new(&mut t, P0, name, &[obj(o)]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.zone(o), Zone::Hand(P0), "{name}");
        assert_eq!(t.hand_size(P0), hand + 2, "{name}");
        // A permanent P0 controls but P1 owns: it goes to P1's hand; P0 draws.
        let mut t = TestGame::new(2);
        let o = t.battlefield(P1, "Ornithopter");
        t.g.objects[o.0 as usize].controller = P0;
        t.g.objects[o.0 as usize].base_controller = P0;
        t.g.recompute();
        cast_new(&mut t, P0, name, &[obj(o)]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.zone(o), Zone::Hand(P1), "{name}");
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
        // The opponent's: no card.
        let mut t = TestGame::new(2);
        let o = t.battlefield(P1, "Ornithopter");
        cast_new(&mut t, P0, name, &[obj(o)]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.zone(o), Zone::Hand(P1), "{name}");
        assert_eq!(t.hand_size(P0), hand, "{name}");
    }
}

#[test]
fn gleeful_demolition_goblins_only_for_your_artifact() {
    cr!("608.2c", "608.2h");
    supported("Gleeful Demolition");
    // "Destroy target artifact. If you controlled that artifact, create three 1/1 red
    // Phyrexian Goblin creature tokens."
    let goblins = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Goblin"))
            .count()
    };
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Ornithopter");
    cast_resolve(&mut t, P0, "Gleeful Demolition", &[obj(o)]);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(goblins(&t), 3);
    let mut t = TestGame::new(2);
    let o = t.battlefield(P1, "Ornithopter");
    cast_resolve(&mut t, P0, "Gleeful Demolition", &[obj(o)]);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(goblins(&t), 0);
}

#[test]
fn hotshot_investigators_investigates_for_your_creature() {
    cr!("608.2c", "608.2h", "701.16a");
    supported("Hotshot Investigators");
    // "When this creature enters, return up to one other target creature to its owner's
    // hand. If you controlled it, investigate."
    let clues = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Clue"))
            .count()
    };
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    t.enter(P0, "Hotshot Investigators");
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert_eq!(clues(&t), 1);
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    t.enter(P0, "Hotshot Investigators");
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Hand(P1));
    assert_eq!(clues(&t), 0);
}

#[test]
fn azog_amasses_for_the_controller_and_draws_for_your_creature() {
    cr!("608.2c", "608.2h", "701.47a");
    supported("Azog, Moria's Ruin");
    // "When Azog enters, destroy up to one other target creature. Its controller amasses
    // Goblins X, where X is that creature's power. If you controlled that creature, draw a
    // card."
    let army = |t: &TestGame, p: PlayerId| {
        t.g.permanents()
            .find(|o| o.controller == p && o.chars.has_subtype("Army"))
            .map(|o| o.id)
    };
    // P1's Hill Giant (power 3): P1 amasses 3, P0 draws nothing.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(g)]);
    let hand = t.hand_size(P0);
    t.enter(P0, "Azog, Moria's Ruin");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    let a = army(&t, P1).expect("P1 amassed");
    assert_eq!(t.counters(a, "+1/+1"), 3);
    assert!(t.obj(a).chars.has_subtype("Goblin"));
    assert_eq!(t.hand_size(P0), hand);
    // P0's own Grizzly Bears: P0 amasses 2 and draws.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    let hand = t.hand_size(P0);
    t.enter(P0, "Azog, Moria's Ruin");
    t.resolve_all();
    let a = army(&t, P0).expect("P0 amassed");
    assert_eq!(t.counters(a, "+1/+1"), 2);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn unyielding_gatekeeper_returns_yours_or_gives_a_detective() {
    cr!("702.168a", "702.168d", "608.2c", "608.2h");
    supported("Unyielding Gatekeeper");
    // Disguise {1}{W}; "When this creature is turned face up, exile another target
    // nonland permanent. If you controlled it, return it to the battlefield tapped.
    // Otherwise, its controller creates a 2/2 white and blue Detective creature token."
    let setup = |t: &mut TestGame| -> ObjectId {
        t.lands(P0, "Wastes", 4);
        t.lands(P0, "Plains", 1);
        let card = t.hand(P0, "Unyielding Gatekeeper");
        let spell = t
            .cast(P0, card)
            .method(CastMethod::FaceDown(KeywordKind::Disguise))
            .go();
        t.resolve_all();
        let g = t.g.current(spell);
        assert!(t.obj(g).face_down);
        g
    };
    // P0's own Grizzly Bears: exiled and returned tapped (a new object).
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    let g = setup(&mut t);
    t.answer_targets(P0, &[obj(b)]);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, g));
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], b);
    assert!(t.obj(back[0]).tapped);
    assert!(tokens(&t, P1).is_empty() && tokens(&t, P0).is_empty());
    // The opponent's Bears: exiled, and P1 creates a Detective.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Grizzly Bears");
    let g = setup(&mut t);
    t.answer_targets(P0, &[obj(b)]);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, g));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    let d = tokens(&t, P1);
    assert_eq!(d.len(), 1);
    let o = t.obj(d[0]);
    assert!(o.chars.has_subtype("Detective") && o.is(CardType::Creature));
    assert_eq!(t.pt(d[0]), (2, 2));
}
