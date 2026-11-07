//! Rulings batch P013 — the Commander Storm spells ("When you cast this spell, copy it for
//! each time you've cast your commander from the command zone this game", CR 903.8,
//! 707.10): the count is taken as the trigger resolves, counts every commander and casts
//! that were countered; the copies aren't cast, keep their targets and X unless changed,
//! and resolve one at a time; Echo Storm's and Cogwork Assembler's token copies of
//! artifacts; Genesis Storm's reveal.

use crate::r_s01_common::{stack_library, supported, tokens};
use crate::r_s04_common::{stack_items, top_of_stack};
use crate::r_s13_common::{commander, commander_game};
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Records that `p`'s commander `cmdr` has been cast `n` times from the command zone.
fn set_casts(t: &mut TestGame, p: PlayerId, cmdr: ObjectId, n: u32) {
    let key = mtg_engine::kw::partner::commander_key(&t.g, cmdr);
    t.g.players[p.idx()].commander_casts.insert(key, n);
}

/// `p` casts the card `name` from hand, paying with the lands given (`(land, n)`).
fn cast_paying(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    lands: &[(&str, usize)],
    targets: &[Entity],
) -> ObjectId {
    for (land, n) in lands {
        t.lands(p, land, *n);
    }
    let card = t.hand(p, name);
    t.cast_with(p, card, targets).unwrap()
}

/// Lands paying each Storm's mana cost.
fn storm_lands(name: &str) -> Vec<(&'static str, usize)> {
    match name {
        "Empyrial Storm" => vec![("Plains", 6)],
        "Echo Storm" => vec![("Island", 5)],
        "Fury Storm" => vec![("Mountain", 4)],
        "Genesis Storm" => vec![("Forest", 6)],
        "Skull Storm" => vec![("Swamp", 9)],
        _ => unreachable!(),
    }
}

/// A Storm scenario: setup (returns the Storm's targets) and the number of times the
/// Storm's effect happened.
struct Storm {
    name: &'static str,
    setup: fn(&mut TestGame) -> Vec<Entity>,
    count: fn(&TestGame) -> usize,
}

fn storms() -> Vec<Storm> {
    vec![
        Storm {
            name: "Empyrial Storm",
            setup: |_| vec![],
            count: |t| t.named_on_battlefield("Angel Token").len(),
        },
        Storm {
            name: "Echo Storm",
            setup: |t| vec![obj(t.battlefield(P0, "Ornithopter"))],
            count: |t| tokens(t, P0).len(),
        },
        Storm {
            // A Shock at P1 is on the stack: each Fury Storm copies it.
            name: "Fury Storm",
            setup: |t| {
                let shock = cast_paying(
                    t,
                    P0,
                    "Shock",
                    &[("Mountain", 1)],
                    &[Entity::Player(P1)],
                );
                vec![obj(shock)]
            },
            count: |t| ((40 - t.life(P1)) / 2 - 1) as usize,
        },
        Storm {
            name: "Genesis Storm",
            setup: |t| {
                for _ in 0..6 {
                    t.library_top(P0, "Grizzly Bears");
                }
                vec![]
            },
            count: |t| t.named_on_battlefield("Grizzly Bears").len(),
        },
        Storm {
            name: "Skull Storm",
            setup: |t| {
                for _ in 0..6 {
                    t.battlefield(P1, "Grizzly Bears");
                }
                vec![]
            },
            count: |t| 6 - t.named_on_battlefield("Grizzly Bears").len(),
        },
    ]
}

#[test]
fn the_storms_count_commander_casts_as_the_trigger_resolves() {
    cr!("903.8", "707.10", "608.2h");
    ruling!(
        "Echo Storm",
        "Each Storm's triggered ability counts only how many times you've cast your commander from the command zone before that triggered ability resolves. If you cast your commander again later, you won't get another copy."
    );
    ruling!(
        "Empyrial Storm",
        "Each Storm’s triggered ability counts only how many times you’ve cast your commander from the command zone before that triggered ability resolves. If you cast your commander again later, you won’t get another copy."
    );
    for s in storms() {
        supported(s.name);
        let mut t = commander_game();
        // Ambush Viper ({1}{G}, flash), cast once already.
        let viper = commander(&mut t, P0, "Ambush Viper");
        set_casts(&mut t, P0, viper, 1);
        let targets = (s.setup)(&mut t);
        cast_paying(&mut t, P0, s.name, &storm_lands(s.name), &targets);
        t.settle();
        // In response to the trigger, the commander is cast again ({2} tax).
        t.lands(P0, "Forest", 4);
        t.cast(P0, viper).go();
        t.resolve();
        // The trigger resolves now: two copies.
        t.resolve_all();
        assert_eq!((s.count)(&t), 3, "{}", s.name);
        // Casting the commander again later doesn't add a copy.
        let viper = t.g.current(viper);
        t.answer_yes(P0, true);
        t.g.destroy(viper, None);
        t.settle();
        let viper = t.g.current(viper);
        assert_eq!(t.zone(viper), Zone::Command);
        t.lands(P0, "Forest", 6);
        t.cast(P0, viper).go();
        t.resolve_all();
        assert_eq!((s.count)(&t), 3, "{}", s.name);
    }
}

#[test]
fn the_storms_count_the_casts_of_every_commander() {
    cr!("903.8", "702.124d", "707.10");
    ruling!(
        "Echo Storm",
        "If you have multiple commanders, you'll copy this spell for each time you've cast either of them. For example, if you've cast one commander once and one twice, you'll copy this spell three times."
    );
    ruling!(
        "Empyrial Storm",
        "If you have multiple commanders, you’ll copy this spell for each time you’ve cast either of them. For example, if you’ve cast one commander once and one twice, you’ll copy this spell three times."
    );
    for s in storms() {
        let mut t = commander_game();
        let kraum = commander(&mut t, P0, "Kraum, Ludevic's Opus");
        let tymna = commander(&mut t, P0, "Tymna the Weaver");
        set_casts(&mut t, P0, kraum, 1);
        set_casts(&mut t, P0, tymna, 2);
        let targets = (s.setup)(&mut t);
        cast_paying(&mut t, P0, s.name, &storm_lands(s.name), &targets);
        t.resolve_all();
        assert_eq!((s.count)(&t), 4, "{}", s.name);
    }
}

#[test]
fn a_countered_commander_cast_still_counts() {
    cr!("903.8", "601.2", "701.6a");
    ruling!(
        "Empyrial Storm",
        "If your commander was countered, that time you cast it still counts."
    );
    ruling!(
        "Jyoti, Moag Ancient",
        "If your commander was countered, that time you cast it still counts."
    );
    supported("Jyoti, Moag Ancient");
    let countered_once = |t: &mut TestGame| {
        let viper = commander(t, P0, "Ambush Viper");
        t.lands(P0, "Forest", 2);
        let spell = t.cast(P0, viper).go();
        // Countered; its owner puts it back into the command zone.
        t.answer_yes(P0, true);
        assert!(t.g.counter(spell, None));
        t.settle();
        assert_eq!(t.zone(viper), Zone::Command);
    };
    let mut t = commander_game();
    countered_once(&mut t);
    cast_paying(&mut t, P0, "Empyrial Storm", &[("Plains", 6)], &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Angel Token").len(), 2);
    // Jyoti: "create a 1/1 Forest Dryad token for each time you've cast your commander
    // from the command zone this game".
    let mut t = commander_game();
    countered_once(&mut t);
    t.enter(P0, "Jyoti, Moag Ancient");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest Dryad Token").len(), 1);
}

#[test]
fn storm_copies_arent_cast() {
    cr!("707.10", "601.2i", "903.8");
    ruling!(
        "Echo Storm",
        "The copies are created on the stack, so they're not \"cast.\" Abilities that trigger when a player casts a spell (such as the Storm's ability itself) won't trigger."
    );
    ruling!(
        "Fury Storm",
        "The copies are created on the stack, so they’re not “cast.” Abilities that trigger when a player casts a spell (such as the Storm’s ability itself) won’t trigger."
    );
    // Magebane Lizard deals 1 damage to P0 for Echo Storm, none for its copies.
    let mut t = commander_game();
    let viper = commander(&mut t, P0, "Ambush Viper");
    set_casts(&mut t, P0, viper, 2);
    t.battlefield(P1, "Magebane Lizard");
    let thopter = t.battlefield(P0, "Ornithopter");
    cast_paying(&mut t, P0, "Echo Storm", &[("Island", 5)], &[obj(thopter)]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 3);
    assert_eq!(t.life(P0), 39);
}

#[test]
fn fury_storm_copies_may_target_fury_storm_without_more_copies() {
    cr!("707.10", "707.10c", "115.7", "903.8");
    ruling!(
        "Fury Storm",
        "The copy is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger. This means, for example, that Fury Storm’s copies may target Fury Storm itself, but this won’t trigger the copy’s “when you cast this spell” ability, and it won’t increase the number of copies of other spells you end up with."
    );
    let mut t = commander_game();
    let viper = commander(&mut t, P0, "Ambush Viper");
    set_casts(&mut t, P0, viper, 1);
    let shock = cast_paying(&mut t, P0, "Shock", &[("Mountain", 1)], &[Entity::Player(P1)]);
    let fury = cast_paying(&mut t, P0, "Fury Storm", &[("Mountain", 4)], &[obj(shock)]);
    t.settle();
    // The trigger's copy of Fury Storm targets Fury Storm itself.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(fury)]);
    t.resolve();
    assert_eq!(t.stack_len(), 3);
    // The copy resolves: it copies Fury Storm, keeping that copy's target (the Shock).
    t.answer_yes(P0, false);
    t.resolve();
    t.settle();
    // No "when you cast" trigger for the new copy.
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    // Three Shocks in all (from 40 life): the original, one from the copy of the copy
    // and one from Fury Storm itself.
    assert_eq!(t.life(P1), 34);
}

#[test]
fn fury_storm_copies_any_instant_or_sorcery_with_its_x() {
    cr!("707.10", "707.2", "107.3");
    ruling!(
        "Fury Storm",
        "Fury Storm can copy any instant or sorcery spell, not just one with targets."
    );
    ruling!(
        "Fury Storm",
        "If the spell that’s copied has an X whose value was determined as it was cast (like Lavalanche does), the copy will have the same value of X."
    );
    supported("Fury Storm");
    // Divination, no targets.
    let mut t = TestGame::new(2);
    let div = cast_paying(&mut t, P0, "Divination", &[("Island", 3)], &[]);
    cast_paying(&mut t, P0, "Fury Storm", &[("Mountain", 4)], &[obj(div)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    // Blaze with X = 3: the copy deals 3 too.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    let blaze = t.cast(P0, blaze).x(3).target(Entity::Player(P1)).go();
    cast_paying(&mut t, P0, "Fury Storm", &[("Mountain", 4)], &[obj(blaze)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn fury_storm_copy_keeps_its_targets_unless_new_ones_are_chosen() {
    cr!("707.10", "707.10c", "115.7");
    ruling!(
        "Fury Storm",
        "The copy will have the same targets as the spell it’s copying unless you choose new ones. You may change any number of the targets, including all of them or none of them, but not how many targets there are. If, for one of the targets, you can’t choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    // Kept: both Shocks hit P1.
    let mut t = TestGame::new(2);
    let shock = cast_paying(&mut t, P0, "Shock", &[("Mountain", 1)], &[Entity::Player(P1)]);
    cast_paying(&mut t, P0, "Fury Storm", &[("Mountain", 4)], &[obj(shock)]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Changed: the copy hits a creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = cast_paying(&mut t, P0, "Shock", &[("Mountain", 1)], &[Entity::Player(P1)]);
    cast_paying(&mut t, P0, "Fury Storm", &[("Mountain", 4)], &[obj(shock)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // An illegal new target (a land) isn't accepted: the target stays P1.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let shock = cast_paying(&mut t, P0, "Shock", &[("Mountain", 1)], &[Entity::Player(P1)]);
    cast_paying(&mut t, P0, "Fury Storm", &[("Mountain", 4)], &[obj(shock)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(forest)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn echo_storm_is_copied_even_if_countered() {
    cr!("707.10", "903.8", "603.3");
    ruling!(
        "Echo Storm",
        "Each Storm's triggered ability can copy the Storm spell even if that spell is countered before that ability resolves."
    );
    let mut t = commander_game();
    let viper = commander(&mut t, P0, "Ambush Viper");
    set_casts(&mut t, P0, viper, 1);
    let thopter = t.battlefield(P0, "Ornithopter");
    let storm = cast_paying(&mut t, P0, "Echo Storm", &[("Island", 5)], &[obj(thopter)]);
    t.settle();
    assert!(t.g.counter(storm, None));
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

/// Whether the object on top of the stack is a triggered ability.
fn trigger_on_top(t: &TestGame) -> bool {
    matches!(
        t.g.obj(top_of_stack(t)).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    )
}

#[test]
fn storm_copies_resolve_one_at_a_time_with_their_triggers_in_between() {
    cr!("608.1", "603.3", "707.10");
    ruling!(
        "Echo Storm",
        "Each copy resolves one at a time. Any abilities that trigger on the artifact token entering the battlefield resolve before the next copy resolves. The original spell is the last to resolve."
    );
    ruling!(
        "Genesis Storm",
        "Each copy resolves one at a time. Any abilities that trigger on the permanent entering the battlefield resolve before the next copy resolves. The original spell is the last to resolve."
    );
    // (Storm, setup returning its targets)
    type Setup = fn(&mut TestGame) -> Vec<Entity>;
    let cases: [(&str, Setup); 2] = [
        ("Echo Storm", |t| {
            vec![obj(t.battlefield(P0, "Ichor Wellspring"))]
        }),
        ("Genesis Storm", |t| {
            // Each Visionary's draw takes the next one.
            for _ in 0..6 {
                t.library_top(P0, "Elvish Visionary");
            }
            vec![]
        }),
    ];
    for (name, setup) in cases {
        let mut t = commander_game();
        let viper = commander(&mut t, P0, "Ambush Viper");
        set_casts(&mut t, P0, viper, 2);
        let targets = setup(&mut t);
        let storm = cast_paying(&mut t, P0, name, &storm_lands(name), &targets);
        t.resolve();
        // Two copies above the spell.
        assert_eq!(t.stack_len(), 3, "{name}");
        let hand = t.hand_size(P0);
        for i in 0..2 {
            t.resolve();
            // The enters trigger is above the next copy (or the spell).
            assert!(trigger_on_top(&t), "{name} copy {i}");
            assert_eq!(t.stack_len(), 3 - i, "{name} copy {i}");
            t.resolve();
            assert_eq!(t.hand_size(P0), hand + i + 1);
        }
        // The original is last.
        assert_eq!(stack_items(&t), vec![name.to_string()]);
        assert_eq!(top_of_stack(&t), t.g.current(storm));
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 3);
    }
}

#[test]
fn token_copies_of_artifacts_get_their_enters_abilities() {
    cr!("707.5", "614.1c", "614.12", "603.6a");
    ruling!(
        "Cogwork Assembler",
        "Any enters-the-battlefield abilities of the copied artifact will trigger when the token enters the battlefield."
    );
    supported("Cogwork Assembler");
    // (copied artifact, check of the token and P0's hand size before)
    type Check = fn(&TestGame, ObjectId, usize);
    let cases: [(&str, Check); 3] = [
        // A "when this enters" trigger.
        ("Ichor Wellspring", |t, _, hand| {
            assert_eq!(t.hand_size(P0), hand + 1)
        }),
        // "Enters with three charge counters".
        ("Tumble Magnet", |t, tok, _| {
            assert_eq!(t.counters(tok, "charge"), 3)
        }),
        // "Enters tapped" and "As this artifact enters, choose a color".
        ("Coldsteel Heart", |t, tok, _| {
            assert!(t.obj_now(tok).tapped);
            let o = t.obj_now(tok);
            assert!(
                o.choices.color.is_some() || o.linked_choices.values().any(|c| c.color.is_some())
            );
        }),
    ];
    for (artifact, check) in cases {
        for via in ["Cogwork Assembler", "Echo Storm"] {
            let mut t = TestGame::new(2);
            let a = t.battlefield(P0, artifact);
            let hand = t.hand_size(P0);
            if via == "Echo Storm" {
                cast_paying(&mut t, P0, via, &[("Island", 5)], &[obj(a)]);
            } else {
                let ca = t.battlefield(P0, via);
                t.lands(P0, "Wastes", 7);
                t.activate(P0, ca, 0, &[obj(a)]).unwrap();
            }
            t.resolve_all();
            let toks = tokens(&t, P0);
            assert_eq!(toks.len(), 1, "{artifact} via {via}");
            assert_eq!(t.obj_now(toks[0]).chars.name, artifact);
            check(&t, toks[0], hand);
        }
    }
    ruling!(
        "Echo Storm",
        "Any “as [this artifact] enters the battlefield” or “[this artifact] enters the battlefield with” abilities of the chosen artifact will also work."
    );
}

#[test]
fn a_token_copy_of_a_token_uses_the_original_tokens_characteristics() {
    cr!("707.2", "111.3", "707.5");
    ruling!(
        "Echo Storm",
        "If the copied artifact is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    ruling!(
        "Red Sun's Twilight",
        "If the copied artifact is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    supported("Red Sun's Twilight");
    supported("Ensoul Artifact");
    // A Treasure made a 5/5 creature by Ensoul Artifact (not a copy effect), with a
    // +1/+1 counter: the copies are plain Treasures.
    let setup = |t: &mut TestGame| -> ObjectId {
        let treasure = crate::r_s02_common::create_token(t, P0, "Treasure");
        let aura = t.battlefield(P0, "Ensoul Artifact");
        t.g.attach(aura, obj(treasure));
        t.g.add_counters(obj(treasure), "+1/+1", 1, None);
        t.g.flush_events();
        t.g.recompute();
        assert_eq!(t.pt(treasure), (6, 6));
        treasure
    };
    let check = |t: &TestGame, except: Option<ObjectId>| {
        let toks: Vec<ObjectId> = tokens(t, P0)
            .into_iter()
            .filter(|o| Some(*o) != except.map(|e| t.g.current(e)))
            .collect();
        assert!(!toks.is_empty());
        for tok in toks {
            let o = t.obj_now(tok);
            assert!(o.chars.has_subtype("Treasure"));
            assert!(!o.is(CardType::Creature));
            assert_eq!(o.counter("+1/+1"), 0);
        }
    };
    // Echo Storm.
    let mut t = TestGame::new(2);
    let treasure = setup(&mut t);
    cast_paying(&mut t, P0, "Echo Storm", &[("Island", 5)], &[obj(treasure)]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    check(&t, Some(treasure));
    // Red Sun's Twilight, X = 5: the Treasure is destroyed and copied.
    let mut t = TestGame::new(2);
    let treasure = setup(&mut t);
    t.lands(P0, "Mountain", 7);
    let rst = t.hand(P0, "Red Sun's Twilight");
    t.cast(P0, rst).x(5).targets(&[obj(treasure)]).go();
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    check(&t, None);
}

#[test]
fn genesis_storm_puts_a_declined_card_on_the_bottom() {
    cr!("701.20a", "401.4");
    ruling!(
        "Genesis Storm",
        "If you choose not to put the nonland permanent card you reveal onto the battlefield, you’ll put it on the bottom of your library with the other cards."
    );
    supported("Genesis Storm");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Hill Giant"]);
    let size = t.library_size(P0);
    t.answer_yes(P0, false);
    cast_paying(&mut t, P0, "Genesis Storm", &[("Forest", 6)], &[]);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.library_size(P0), size);
    // The Forest and the Bears are on the bottom; the Giant is on top.
    let lib = &t.g.player(P0).library;
    let mut bottom: Vec<String> = lib[..2]
        .iter()
        .map(|o| t.g.obj(*o).chars.name.to_string())
        .collect();
    bottom.sort();
    assert_eq!(bottom, vec!["Forest", "Grizzly Bears"]);
    assert_eq!(t.g.obj(*lib.last().unwrap()).chars.name, "Hill Giant");
}

#[test]
fn genesis_storm_with_no_nonland_permanent_card_reveals_the_whole_library() {
    cr!("701.20a", "401.4");
    ruling!(
        "Genesis Storm",
        "If your library no longer has any nonland permanent cards, you’ll reveal your library and then put it back in a random order."
    );
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Shock"]);
    let size = t.library_size(P0);
    cast_paying(&mut t, P0, "Genesis Storm", &[("Forest", 6)], &[]);
    t.resolve_all();
    assert_eq!(t.library_size(P0), size);
    assert_eq!(t.named_on_battlefield("Forest").len(), 6);
    // Nothing was put onto the battlefield; the cards went back into the library.
    assert!(t.named_on_battlefield("Shock").is_empty());
    for name in ["Forest", "Shock"] {
        assert!(t
            .g
            .player(P0)
            .library
            .iter()
            .any(|o| t.g.obj(*o).chars.name == name));
    }
}
