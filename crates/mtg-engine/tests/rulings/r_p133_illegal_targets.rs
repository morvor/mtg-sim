//! Rulings batch P133 — bounce spells and abilities whose targets become illegal before
//! they resolve (CR 608.2b): with all targets illegal they don't resolve and none of their
//! effects happen; with some targets still legal they resolve and do as much as they can.

use crate::r_p133_common::*;
use crate::r_s01_common::{give_mana_for, supported};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// One "single target becomes illegal" case: the card, extra setup (run before casting),
/// how it's cast at P1's Grizzly Bears, and the citation is on its own line in the test.
struct Case {
    card: &'static str,
    setup: fn(&mut TestGame),
    cast: fn(&mut TestGame, ObjectId, ObjectId) -> ObjectId,
}

fn plain(t: &mut TestGame, card: ObjectId, bears: ObjectId) -> ObjectId {
    t.cast(P0, card).target(bears).go()
}

fn kicked(t: &mut TestGame, card: ObjectId, bears: ObjectId) -> ObjectId {
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.cast(P0, card).kicked(true).target(bears).go()
}

fn nothing(_: &mut TestGame) {}

fn run(case: &Case) {
    supported(case.card);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    (case.setup)(&mut t);
    let card = spell(&mut t, P0, case.card);
    let s = (case.cast)(&mut t, card, bears);
    fizzles(&mut t, s, |t| kill(t, bears));
}

#[test]
fn single_target_bounce_spells_do_nothing_if_the_target_is_illegal() {
    cr!("608.2b", "115.1");
    ruling!(
        "Repulse",
        "If the target creature is an illegal target as Repulse tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    ruling!(
        "Escape Detection",
        "If the target creature is an illegal target as Escape Detection tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    ruling!(
        "Arrester's Admonition",
        "You don't draw a card if you cast it during your main phase."
    );
    ruling!(
        "Crashing Tide",
        "If the target creature is an illegal target by the time Crashing Tide tries to resolve, the spell doesn’t resolve. You won’t draw a card."
    );
    ruling!(
        "Temporal Machinations",
        "If the target creature is an illegal target by the time Temporal Machinations tries to resolve, the spell doesn’t resolve. You won’t draw a card."
    );
    ruling!(
        "Dramatic Rescue",
        "If the target creature is an illegal target when Dramatic Rescue tries to resolve, it won’t resolve and none of its effects will happen. You won’t gain 2 life."
    );
    ruling!(
        "String of Disappearances",
        "That creature’s controller can’t pay to copy String of Disappearances."
    );
    ruling!(
        "Ral's Dispersal",
        "If the target creature becomes an illegal target for Ral’s Dispersal, the spell doesn’t resolve. You don’t search for Ral, Caller of Storms."
    );
    ruling!(
        "Call to Heel",
        "If the targeted creature becomes an illegal target before Call to Heel resolves, Call to Heel doesn’t resolve. No one draws a card."
    );
    ruling!(
        "Drag Under",
        "If the targeted creature becomes an illegal target before Drag Under resolves, you won't draw a card."
    );
    ruling!(
        "Compelling Deterrence",
        "If the target nonland permanent is an illegal target as Compelling Deterrence tries to resolve, it won’t resolve and none of its effects will happen. No player will discard a card."
    );
    ruling!(
        "Mists of Lórien",
        "If the target nonland permanent is an illegal target as Mists of Lórien tries to resolve, it won't resolve and none of its effects will happen. No permanents will be returned to their owners' hands."
    );
    ruling!(
        "Blink of an Eye",
        "If the target nonland permanent is an illegal target by the time Blink of an Eye tries to resolve, the spell doesn't resolve. You won't draw a card if it was kicked."
    );
    ruling!(
        "Gust of Wind",
        "If the target nonland permanent is an illegal target by the time Gust of Wind tries to resolve, the spell won't resolve. You won't draw a card."
    );
    ruling!(
        "Into the Roil",
        "If the target nonland permanent is an illegal target by the time Into the Roil tries to resolve, the spell doesn't resolve. You don't draw a card if it was kicked."
    );
    ruling!(
        "Into the Roil",
        "If the targeted permanent is an illegal target by the time Into the Roil resolves, the entire spell doesn't resolve. You don't draw a card."
    );
    ruling!(
        "Winds of Rebuke",
        "If the target nonland permanent is an illegal target by the time Winds of Rebuke resolves, the entire spell doesn’t resolve. No player loses cards from the top of their library."
    );
    ruling!(
        "Leave in the Dust",
        "If the target permanent becomes an illegal target, Leave in the Dust doesn’t resolve and none of its effects happen. You won’t draw a card."
    );
    ruling!(
        "Eject",
        "If the target permanent is an illegal target as Eject tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    ruling!(
        "Repeal",
        "If the target permanent is an illegal target by the time Repeal tries to resolve, the spell doesn't resolve. You don't draw a card."
    );
    ruling!(
        "Johann's Stopgap",
        "If the permanent is an illegal target as Johann's Stopgap tries to resolve, Johann's Stopgap is removed from the stack and you won't draw a card."
    );
    let cases = [
        Case {
            card: "Repulse",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Escape Detection",
            setup: nothing,
            cast: plain,
        },
        // Cast during P0's main phase: it would draw.
        Case {
            card: "Arrester's Admonition",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Crashing Tide",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Temporal Machinations",
            setup: |t| {
                t.battlefield(P0, "Ornithopter");
            },
            cast: plain,
        },
        Case {
            card: "Dramatic Rescue",
            setup: nothing,
            cast: plain,
        },
        // P1 has the mana to copy it, but isn't offered the copy.
        Case {
            card: "String of Disappearances",
            setup: |t| {
                t.lands(P1, "Island", 2);
            },
            cast: plain,
        },
        Case {
            card: "Ral's Dispersal",
            setup: |t| {
                t.library_top(P0, "Ral, Caller of Storms");
                t.graveyard(P0, "Ral, Caller of Storms");
            },
            cast: plain,
        },
        Case {
            card: "Call to Heel",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Drag Under",
            setup: nothing,
            cast: plain,
        },
        // P0 controls a Zombie, and P1 has a card in hand.
        Case {
            card: "Compelling Deterrence",
            setup: |t| {
                t.battlefield(P0, "Walking Corpse");
            },
            cast: plain,
        },
        // Another permanent with the same mana value stays.
        Case {
            card: "Mists of Lórien",
            setup: |t| {
                t.battlefield(P1, "Grizzly Bears");
                t.battlefield(P0, "Grizzly Bears");
            },
            cast: plain,
        },
        Case {
            card: "Blink of an Eye",
            setup: nothing,
            cast: kicked,
        },
        Case {
            card: "Gust of Wind",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Into the Roil",
            setup: nothing,
            cast: kicked,
        },
        Case {
            card: "Into the Roil",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Winds of Rebuke",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Leave in the Dust",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Eject",
            setup: nothing,
            cast: plain,
        },
        Case {
            card: "Repeal",
            setup: |t| {
                t.lands(P0, "Wastes", 2);
            },
            cast: |t, card, bears| t.cast(P0, card).x(2).target(bears).go(),
        },
        Case {
            card: "Johann's Stopgap",
            setup: nothing,
            cast: plain,
        },
    ];
    for case in &cases {
        run(case);
    }
}

#[test]
fn galestrike_target_untapped_in_response() {
    cr!("608.2b", "115.1");
    ruling!(
        "Galestrike",
        "If the target creature is an illegal target by the time Galestrike tries to resolve, the spell doesn't resolve. You don't draw a card."
    );
    supported("Galestrike");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    let card = spell(&mut t, P0, "Galestrike");
    let s = t.cast(P0, card).target(bears).go();
    // No longer a "tapped creature": illegal, though still on the battlefield.
    fizzles(&mut t, s, |t| {
        t.g.untap(bears);
    });
    assert!(t.on_battlefield(bears));
}

#[test]
fn surgical_skullbomb_ability_target_left() {
    cr!("608.2b", "602.2");
    ruling!(
        "Surgical Skullbomb",
        "If the target of Surgical Skullbomb's last ability isn't a legal target as the ability tries to resolve (probably because it has left the battlefield), you won't draw a card."
    );
    supported("Surgical Skullbomb");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bomb = t.battlefield(P0, "Surgical Skullbomb");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, bomb, 1, &[Entity::Object(bears)]).unwrap();
    assert!(t.in_graveyard(P0, "Surgical Skullbomb"));
    kill(&mut t, bears);
    let before = snap(&t);
    t.resolve_all();
    assert_eq!(snap(&t), before);
}

#[test]
fn razorfin_abolisher_target_lost_its_counters() {
    cr!("608.2b", "122.1");
    ruling!(
        "Razorfin Abolisher",
        "If the targeted creature no longer has any counters on it when the ability resolves, the ability won’t resolve because its only target is illegal. The creature will stay on the battlefield."
    );
    ruling!(
        "Razorfin Abolisher",
        "Razorfin Abolisher’s ability can target a creature with any kind of counter on it, not just a -1/-1 counter."
    );
    supported("Razorfin Abolisher");
    // Any kind of counter makes it a legal target.
    for kind in ["+1/+1", "-1/-1", "stun"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.g.add_counters(Entity::Object(bears), kind, 1, None);
        let abolisher = t.battlefield(P0, "Razorfin Abolisher");
        t.lands(P0, "Island", 2);
        t.activate(P0, abolisher, 0, &[Entity::Object(bears)])
            .unwrap_or_else(|e| panic!("{kind}: {e:?}"));
        t.resolve_all();
        assert!(in_hand_of(&t, bears, P1), "{kind}");
    }
    // The counter is removed in response: the creature stays.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    let abolisher = t.battlefield(P0, "Razorfin Abolisher");
    t.lands(P0, "Island", 2);
    t.activate(P0, abolisher, 0, &[Entity::Object(bears)])
        .unwrap();
    t.g.remove_counters(Entity::Object(bears), "+1/+1", 1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Without counters it can't be targeted at all.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let abolisher = t.battlefield(P0, "Razorfin Abolisher");
    t.lands(P0, "Island", 2);
    assert!(t
        .activate(P0, abolisher, 0, &[Entity::Object(bears)])
        .is_err());
}

#[test]
fn cryptic_command_bounce_and_draw_target_illegal() {
    cr!("608.2b", "700.2");
    ruling!(
        "Cryptic Command",
        "For example, if you choose the second and fourth modes, and the permanent is an illegal target when Cryptic Command tries to resolve, you won’t draw a card."
    );
    supported("Cryptic Command");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Cryptic Command");
    let s = t.cast(P0, card).modes(&[1, 3]).target(bears).go();
    fizzles(&mut t, s, |t| kill(t, bears));
    // With the third and fourth modes (no targets), it resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Cryptic Command");
    let hand = t.hand_size(P0);
    t.cast(P0, card).modes(&[2, 3]).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn calamitous_tide_all_targets_illegal() {
    cr!("608.2b", "115.1");
    ruling!(
        "Calamitous Tide",
        "If all of the targets are illegal as Calamitous Tide tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw or discard any cards."
    );
    supported("Calamitous Tide");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Island");
    let card = spell(&mut t, P0, "Calamitous Tide");
    let s = t
        .cast(P0, card)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    fizzles(&mut t, s, |t| {
        kill(t, a);
        kill(t, b);
    });
}

#[test]
fn quandrix_command_targets() {
    cr!("608.2b", "700.2");
    ruling!(
        "Quandrix Command",
        "If all of Quandrix Command's targets are illegal as it tries to resolve, it will do nothing."
    );
    ruling!(
        "Quandrix Command",
        "If at least one target is still legal as Quandrix Command tries to resolve, it will resolve and do as much as it can."
    );
    supported("Quandrix Command");
    // Return a creature + two +1/+1 counters on another: both gone, nothing happens.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let card = spell(&mut t, P0, "Quandrix Command");
    let s = t
        .cast(P0, card)
        .modes(&[0, 2])
        .target(a)
        .target(b)
        .go();
    fizzles(&mut t, s, |t| {
        kill(t, a);
        kill(t, b);
    });
    // Only the creature to return is gone: the other still gets the counters.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let card = spell(&mut t, P0, "Quandrix Command");
    t.cast(P0, card).modes(&[0, 2]).target(a).target(b).go();
    kill(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 2);
}

#[test]
fn quandrix_command_shuffle_mode_target_cards_illegal() {
    cr!("608.2b", "608.2c", "701.24a");
    ruling!(
        "Quandrix Command",
        "if the fourth mode is chosen and all of the target cards are illegal, but the target player is still legal, that player will still shuffle their library."
    );
    supported("Quandrix Command");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gy = t.graveyard(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Quandrix Command");
    t.cast(P0, card)
        .modes(&[0, 3])
        .target(bears)
        .target(Entity::Player(P1))
        .targets(&[Entity::Object(gy)])
        .go();
    // Both the creature and the graveyard card become illegal targets.
    kill(&mut t, bears);
    let gy_now = t.g.current(gy);
    t.g.move_object(gy_now, Zone::Exile, events::MoveCause::Effect, None);
    t.settle();
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(crate::r_s33_common::shuffled_since(&t, from, P1));
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn mystic_confluence_targets() {
    cr!("608.2b", "700.2d");
    ruling!(
        "Mystic Confluence",
        "If you choose the first and/or second modes but all of the targets become illegal before Mystic Confluence resolves, the spell won't resolve. If you also chose the last mode once or twice, you won't draw any cards."
    );
    ruling!(
        "Mystic Confluence",
        "If the middle mode is chosen more than once, you choose the relative order to return the target creatures."
    );
    supported("Mystic Confluence");
    // Return, return, draw: both creatures gone, no card drawn.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Mystic Confluence");
    let s = t
        .cast(P0, card)
        .modes(&[1, 1, 2])
        .target(a)
        .target(b)
        .go();
    fizzles(&mut t, s, |t| {
        kill(t, a);
        kill(t, b);
    });
    // One is still legal: it's returned and the card is drawn.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Mystic Confluence");
    t.cast(P0, card).modes(&[1, 1, 2]).target(a).target(b).go();
    let hand = t.hand_size(P0);
    kill(&mut t, a);
    t.resolve_all();
    assert!(in_hand_of(&t, b, P1));
    assert_eq!(t.hand_size(P0), hand + 1);
    // The middle mode twice: both target creatures are returned, one after the other.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let card = spell(&mut t, P0, "Mystic Confluence");
    t.cast(P0, card).modes(&[1, 1, 2]).target(a).target(b).go();
    t.resolve_all();
    assert!(in_hand_of(&t, a, P1));
    assert!(in_hand_of(&t, b, P0));
}

#[test]
fn wail_of_the_forgotten_targets() {
    cr!("608.2b", "700.2", "702.177a");
    ruling!(
        "Wail of the Forgotten",
        "If all targets for the chosen modes become illegal before Wail of the Forgotten resolves, the spell will be countered and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    supported("Wail of the Forgotten");
    let setup = |t: &mut TestGame| {
        for _ in 0..8 {
            t.graveyard(P0, "Grizzly Bears");
        }
        t.hand(P1, "Island");
        t.battlefield(P1, "Grizzly Bears")
    };
    // Return + look at the top three: the only target is gone, so nothing happens.
    let mut t = TestGame::new(2);
    let bears = setup(&mut t);
    let card = spell(&mut t, P0, "Wail of the Forgotten");
    let s = t.cast(P0, card).modes(&[0, 2]).target(bears).go();
    fizzles(&mut t, s, |t| kill(t, bears));
    // Return + discard: the permanent is gone, the opponent still discards.
    let mut t = TestGame::new(2);
    let bears = setup(&mut t);
    let card = spell(&mut t, P0, "Wail of the Forgotten");
    t.cast(P0, card)
        .modes(&[0, 1])
        .target(bears)
        .target(Entity::Player(P1))
        .go();
    kill(&mut t, bears);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand - 1);
}

#[test]
fn lost_in_the_mist_targets() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Lost in the Mist",
        "If one of Lost in the Mist’s targets is illegal by the time it resolves, Lost in the Mist will still affect the remaining legal target. If both targets are illegal at this time, Lost in the Mist won’t resolve."
    );
    ruling!(
        "Lost in the Mist",
        "Lost in the Mist targets both the spell and the permanent. You can only cast it if you can choose legal targets for both parts."
    );
    supported("Lost in the Mist");
    // No spell to target: it can't be cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Lost in the Mist");
    assert!(!crate::r_s21_common::castable(&mut t, P0, card));
    let _ = bears;
    // P1's spell is countered by something else first: the permanent is still returned.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let p1_spell = spell(&mut t, P1, "Shock");
    let on_stack = t.cast(P1, p1_spell).target(Entity::Player(P0)).go();
    let card = spell(&mut t, P0, "Lost in the Mist");
    t.cast(P0, card)
        .target(on_stack)
        .target(bears)
        .go();
    let on_stack_now = t.g.current(on_stack);
    t.g.move_object(on_stack_now, Zone::Exile, events::MoveCause::Effect, None);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
    // Both targets gone: it doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let p1_spell = spell(&mut t, P1, "Shock");
    let on_stack = t.cast(P1, p1_spell).target(Entity::Player(P0)).go();
    let card = spell(&mut t, P0, "Lost in the Mist");
    let s = t.cast(P0, card).target(on_stack).target(bears).go();
    fizzles(&mut t, s, |t| {
        let on_stack_now = t.g.current(on_stack);
        t.g.move_object(on_stack_now, Zone::Exile, events::MoveCause::Effect, None);
        kill(t, bears);
    });
}

#[test]
fn two_target_bounce_one_target_illegal() {
    cr!("608.2b");
    ruling!(
        "Aether Tradewinds",
        "If one target permanent becomes an illegal target, the other will be returned to its owner's hand. If both targets become illegal, Aether Tradewinds won't resolve."
    );
    ruling!(
        "Aether Helix",
        "If only one target is still legal as Aether Helix tries to resolve, it will still resolve and perform the appropriate action on the remaining target."
    );
    supported("Aether Tradewinds");
    supported("Aether Helix");
    for first_gone in [true, false] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let card = spell(&mut t, P0, "Aether Tradewinds");
        t.cast(P0, card).target(mine).target(theirs).go();
        kill(&mut t, if first_gone { mine } else { theirs });
        t.resolve_all();
        if first_gone {
            assert!(in_hand_of(&t, theirs, P1));
        } else {
            assert!(in_hand_of(&t, mine, P0));
        }
    }
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Aether Tradewinds");
    let s = t.cast(P0, card).target(mine).target(theirs).go();
    fizzles(&mut t, s, |t| {
        kill(t, mine);
        kill(t, theirs);
    });
    // Aether Helix: the permanent to return is gone, the graveyard card still comes back.
    for permanent_gone in [true, false] {
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let gy = t.graveyard(P0, "Grizzly Bears");
        let card = spell(&mut t, P0, "Aether Helix");
        t.cast(P0, card).target(theirs).target(gy).go();
        if permanent_gone {
            kill(&mut t, theirs);
        } else {
            let g = t.g.current(gy);
            t.g.move_object(g, Zone::Exile, events::MoveCause::Effect, None);
        }
        t.resolve_all();
        if permanent_gone {
            assert!(in_hand_of(&t, gy, P0));
        } else {
            assert!(in_hand_of(&t, theirs, P1));
        }
    }
}

#[test]
fn karns_temporal_sundering_targets() {
    cr!("608.2b", "205.4d");
    ruling!(
        "Karn's Temporal Sundering",
        "If the target player or target nonland permanent is an illegal target as Karn’s Temporal Sundering resolves, the other target is affected as normal and Karn’s Temporal Sundering is exiled. If both targets are illegal, Karn’s Temporal Sundering doesn’t resolve and isn’t exiled."
    );
    supported("Karn's Temporal Sundering");
    // The permanent is gone: the extra turn still happens, and the card is exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Karn's Temporal Sundering");
    t.cast(P0, card)
        .target(Entity::Player(P0))
        .target(bears)
        .go();
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.g.extra_turns, vec![P0]);
    assert!(t.in_exile("Karn's Temporal Sundering"));
}

#[test]
fn karns_temporal_sundering_both_targets_illegal() {
    cr!("608.2b", "205.4d");
    ruling!(
        "Karn's Temporal Sundering",
        "If both targets are illegal, Karn’s Temporal Sundering doesn’t resolve and isn’t exiled."
    );
    // Three players: the target player leaves the game and the permanent leaves the
    // battlefield.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Karn's Temporal Sundering");
    let s = t
        .cast(P0, card)
        .target(Entity::Player(P2))
        .target(bears)
        .go();
    fizzles(&mut t, s, |t| {
        kill(t, bears);
        t.g.lose_game(P2);
    });
    assert!(t.g.extra_turns.is_empty());
    assert!(!t.in_exile("Karn's Temporal Sundering"));
}

#[test]
fn run_away_together_same_controller() {
    cr!("608.2b", "115.1");
    ruling!(
        "Run Away Together",
        "If both creatures are controlled by the same player as Run Away Together tries to resolve, both targets are illegal. The spell doesn't resolve."
    );
    supported("Run Away Together");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Run Away Together");
    let s = t.cast(P0, card).target(mine).target(theirs).go();
    fizzles(&mut t, s, |t| {
        crate::r_s06_common::give_control(t, theirs, P0);
    });
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
}

#[test]
fn run_away_together_one_target_illegal_uses_lki() {
    cr!("608.2b", "608.2h", "113.7a");
    ruling!(
        "Run Away Together",
        "If one of the two target creatures becomes an illegal target, Run Away Together can still determine its controller only to check whether the other creature is a legal target. If the illegal target has left the battlefield, use its last known information. If the other creature is still a legal target, it's returned to its owner's hand."
    );
    // P0's creature leaves; P1's creature is still controlled by a different player (by
    // P0's creature's last known information): it's returned.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Run Away Together");
    t.cast(P0, card).target(mine).target(theirs).go();
    kill(&mut t, mine);
    t.resolve_all();
    assert!(in_hand_of(&t, theirs, P1));
    // P1's creature leaves (last controlled by P1): P0's creature is returned.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Run Away Together");
    t.cast(P0, card).target(mine).target(theirs).go();
    kill(&mut t, theirs);
    t.resolve_all();
    assert!(in_hand_of(&t, mine, P0));
}

#[test]
fn compelling_deterrence_zombie_checked_after_return() {
    cr!("608.2c");
    ruling!(
        "Compelling Deterrence",
        "Compelling Deterrence checks to see if you control a Zombie after the target nonland permanent is returned to its owner’s hand. For example, if you target the only Zombie you control with Compelling Deterrence, you won’t discard a card."
    );
    ruling!(
        "Compelling Deterrence",
        "The card returned to its owner’s hand may be the card they discard. If it’s the only card in that player’s hand, it must be discarded if you control a Zombie."
    );
    supported("Compelling Deterrence");
    // Targeting P0's only Zombie: no discard.
    let mut t = TestGame::new(2);
    let zombie = t.battlefield(P0, "Walking Corpse");
    t.hand(P0, "Island");
    let card = spell(&mut t, P0, "Compelling Deterrence");
    t.cast(P0, card).target(zombie).go();
    t.resolve_all();
    assert!(in_hand_of(&t, zombie, P0));
    assert!(t.in_hand(P0, "Island"));
    assert_eq!(t.graveyard_size(P0), 1);
    // P1 with an empty hand: the returned card is discarded.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Walking Corpse");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = spell(&mut t, P0, "Compelling Deterrence");
    t.cast(P0, card).target(bears).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P1), 0);
}
