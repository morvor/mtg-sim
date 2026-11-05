//! Rulings batch P036 — which spells a counterspell can target (CR 115.1, 601.2c): mana
//! value restrictions and {X} (CR 202.3e, 118.9, 702.37 face-down), "spell that targets
//! ..." (CR 115.10 for modal spells), the second spell cast this turn (CR 707.10, 707.12),
//! conditions checked on resolution (CR 608.2), targets that change before resolution
//! (CR 608.2b), "up to" targets and spells that need a target to be cast (CR 601.2c).

use crate::r_p036_common::*;
use crate::r_s01_common::supported;
use crate::r_s07_common::resolved;
use crate::r_s21_common::castable;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The legal first-target candidates offered to `p` when casting the real card `name`
/// now (with X = `x`); the cast is attempted with `pick` as the first target (and the
/// spell, if cast, is left on the stack). Returns (candidates, the spell if it was cast).
fn offered_for(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    x: Option<i64>,
    pick: &[Entity],
) -> (Vec<Entity>, Option<ObjectId>) {
    lands_for_cost(t, p, name);
    if let Some(x) = x {
        t.lands(p, "Wastes", x as usize);
        t.answer(p, DecisionKind::X, Answer::Number(x));
    }
    let card = t.hand(p, name);
    let from = t.asked().len();
    t.answer_targets(p, pick);
    t.g.turn.priority = Some(p);
    let r = t.g.cast_spell(p, card, CastMethod::Normal);
    t.g.flush_events();
    let c = offered(t, p, from).into_iter().next().unwrap_or_default();
    t.clear_answers();
    (c, r.ok())
}

/// P1 casts Volcanic Geyser ("Volcanic Geyser deals X damage to any target.", {X}{R}{R})
/// at P0 with the given X.
fn geyser(t: &mut TestGame, x: i64) -> ObjectId {
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Wastes", x as usize);
    let c = t.hand(P1, "Volcanic Geyser");
    t.cast(P1, c).x(x).target(Entity::Player(P0)).go()
}

/// P1 casts Electrolyze ("Electrolyze deals 2 damage divided as you choose among one or
/// two targets. Draw a card.") at the two targets.
fn electrolyze(t: &mut TestGame, a: Entity, b: Entity) -> ObjectId {
    t.answer_targets(P1, &[a, b]);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    cast_new(t, P1, "Electrolyze", &[])
}

// --- Mana value -------------------------------------------------------------------------

#[test]
fn disdainful_stroke_cant_target_a_face_down_spell() {
    cr!("708.4", "202.3", "115.1");
    ruling!(
        "Disdainful Stroke",
        "A face-down spell has mana value 0 and can't be targeted by Disdainful Stroke."
    );
    supported("Disdainful Stroke");
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    // Hystrodon ({4}{G}, mana value 5) cast face down with morph for {3}.
    t.lands(P1, "Wastes", 3);
    let hystrodon = t.hand(P1, "Hystrodon");
    let fd = t.cast(P1, hystrodon).method(crate::r_s12_common::MORPH).go();
    assert!(t.obj(fd).face_down);
    // In response, P1's Volcanic Geyser with X = 2 (mana value 4).
    let g = geyser(&mut t, 2);
    let (c, s) = offered_for(&mut t, P0, "Disdainful Stroke", None, &[obj(g)]);
    assert_eq!(c, vec![obj(g)]);
    assert!(s.is_some());
}

#[test]
fn disdainful_stroke_counts_the_chosen_x() {
    cr!("202.3e", "115.1");
    ruling!(
        "Disdainful Stroke",
        "If a spell has {X} in its mana cost, use the value chosen for X when determining the spell's mana value."
    );
    // Volcanic Geyser with X = 2 has mana value 4; with X = 1, 3.
    for (x, legal) in [(2, true), (1, false)] {
        let mut t = TestGame::new(2);
        let g = geyser(&mut t, x);
        let (c, _) = offered_for(&mut t, P0, "Disdainful Stroke", None, &[obj(g)]);
        assert_eq!(c.contains(&obj(g)), legal, "X = {x}");
    }
}

#[test]
fn minor_misstep_uses_x_on_the_stack() {
    cr!("202.3e", "115.1");
    ruling!(
        "Minor Misstep",
        "While a spell with {X} in its mana cost is on the stack, use the value of X that was chosen when it was cast to determine its mana value. Normally, this means that a spell with {X} in its mana cost will have a mana value greater than 1."
    );
    supported("Minor Misstep");
    supported("Blaze");
    // Blaze ({X}{R}): mana value 1 + X on the stack.
    for (x, legal) in [(0, true), (1, false)] {
        let mut t = TestGame::new(2);
        turn_of(&mut t, P1);
        t.lands(P1, "Mountain", 1);
        t.lands(P1, "Wastes", x as usize);
        let c = t.hand(P1, "Blaze");
        let blaze = t.cast(P1, c).x(x).target(Entity::Player(P0)).go();
        let (c, s) = offered_for(&mut t, P0, "Minor Misstep", None, &[obj(blaze)]);
        assert_eq!(c.contains(&obj(blaze)), legal, "X = {x}");
        if legal {
            t.resolve();
            assert!(resolved(&t, s.unwrap()));
            assert!(!on_stack(&t, blaze));
        }
    }
}

#[test]
fn spell_blast_x_includes_the_targets_x() {
    cr!("202.3e", "107.3a", "115.1");
    ruling!(
        "Spell Blast",
        "If the target spell has {X} in its mana cost, include the value chosen for that X when determining the value of the {X} in Spell Blast's mana cost. For example, if you wish to counter Volcanic Geyser (a spell with mana cost {X}{R}{R}) whose X is 7, you'll need to choose 9 for Spell Blast's X and pay {9}{U}."
    );
    supported("Spell Blast");
    // X = 2 (the printed mana value) doesn't let it target the Geyser; X = 9 does.
    for (x, legal) in [(2, false), (7, false), (9, true)] {
        let mut t = TestGame::new(2);
        let g = geyser(&mut t, 7);
        let (c, s) = offered_for(&mut t, P0, "Spell Blast", Some(x), &[obj(g)]);
        assert_eq!(c.contains(&obj(g)), legal, "X = {x}");
        if legal {
            t.resolve();
            assert!(resolved(&t, s.unwrap()));
            assert!(!on_stack(&t, g));
            assert_eq!(t.life(P0), 20);
        }
    }
}

#[test]
fn spell_blast_ignores_alternative_and_additional_costs_and_reductions() {
    cr!("202.3", "118.9", "118.8", "601.2f");
    ruling!(
        "Spell Blast",
        "Alternative costs, additional costs, cost increases, and cost reductions don't affect a spell's mana value."
    );
    supported("Frogmite");
    supported("Burst Lightning");
    supported("Think Twice");
    // Frogmite ({4}, affinity for artifacts) cast for {0}: mana value 4.
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    for _ in 0..4 {
        t.battlefield(P1, "Ornithopter");
    }
    let fm = t.hand(P1, "Frogmite");
    let frog = t.cast(P1, fm).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 0);
    for (x, legal) in [(0, false), (4, true)] {
        let (c, _) = offered_for(&mut t, P0, "Spell Blast", Some(x), &[obj(frog)]);
        assert_eq!(c.contains(&obj(frog)), legal, "Frogmite, X = {x}");
        t.resolve_all();
        if t.g.stack.is_empty() {
            break;
        }
    }
    // Burst Lightning ({R}) kicked for {4}: mana value 1.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 4);
    let bl = t.hand(P1, "Burst Lightning");
    let burst = t.cast(P1, bl).kicked(true).target(Entity::Player(P0)).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 5);
    let (c, _) = offered_for(&mut t, P0, "Spell Blast", Some(5), &[obj(burst)]);
    assert!(!c.contains(&obj(burst)));
    let (c, _) = offered_for(&mut t, P0, "Spell Blast", Some(1), &[obj(burst)]);
    assert!(c.contains(&obj(burst)));
    // Think Twice ({1}{U}) cast with flashback ({2}{U}) from the graveyard: mana value 2.
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 1);
    t.lands(P1, "Wastes", 2);
    let tt = t.graveyard(P1, "Think Twice");
    let methods = crate::r_s08_common::legal_cast_methods(&mut t, P1, tt);
    let fb = methods
        .into_iter()
        .find(|m| *m != CastMethod::Normal)
        .expect("flashback");
    let think = t.cast(P1, tt).method(fb).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 3);
    let (c, _) = offered_for(&mut t, P0, "Spell Blast", Some(3), &[obj(think)]);
    assert!(!c.contains(&obj(think)));
    let (c, _) = offered_for(&mut t, P0, "Spell Blast", Some(2), &[obj(think)]);
    assert!(c.contains(&obj(think)));
}

// --- "Spell that targets ..." -----------------------------------------------------------

#[test]
fn intervene_needs_a_spell_targeting_at_least_one_creature() {
    cr!("115.1");
    ruling!(
        "Intervene",
        "Intervene can only target a spell that is currently targeting a creature."
    );
    ruling!(
        "Intervene",
        "It means “targeting at least one creature” not “exactly one creature”."
    );
    supported("Intervene");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let at_player = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let at_creature = cast_new(&mut t, P1, "Shock", &[obj(bears)]);
    let mixed = electrolyze(&mut t, obj(bears), Entity::Player(P0));
    let (c, s) = offered_for(&mut t, P0, "Intervene", None, &[obj(mixed)]);
    assert!(!c.contains(&obj(at_player)));
    assert!(c.contains(&obj(at_creature)));
    assert!(c.contains(&obj(mixed)));
    t.resolve();
    assert!(resolved(&t, s.unwrap()));
    assert!(!on_stack(&t, mixed));
}

#[test]
fn intervene_uses_only_the_chosen_mode_of_a_modal_spell() {
    cr!("115.1", "700.2a", "601.2b");
    ruling!(
        "Intervene",
        "If a spell is modal, use the target(s) of the current mode only for this check. Targets for the other modes are never selected."
    );
    supported("Izzet Charm");
    // Izzet Charm: "• Counter target noncreature spell unless its controller pays {2}.
    // • Izzet Charm deals 2 damage to target creature. • Draw two cards, then discard two
    // cards."
    for (mode, legal) in [(1, true), (2, false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        crate::r_s29_common::choose_modes(&mut t, P1, &[mode]);
        let charm = cast_new(&mut t, P1, "Izzet Charm", &[obj(bears)]);
        let (c, _) = offered_for(&mut t, P0, "Intervene", None, &[obj(charm)]);
        assert_eq!(c.contains(&obj(charm)), legal, "mode {mode}");
    }
}

#[test]
fn confound_needs_a_creature_target_still_there_and_still_a_creature() {
    cr!("608.2b");
    ruling!(
        "Confound",
        "Confound doesn’t resolve if the targeted spell has been countered or does not still target one or more permanents that are still creatures and are still on the battlefield. Confound is different from other such spells in that it actually specifies that the spell must target a creature, not just be able to target a creature or that the spell has a certain number of targets."
    );
    supported("Confound");
    supported("Imprisoned in the Moon");
    // `change` 0: nothing (control); 1: the spell is countered; 2: the creature leaves the
    // battlefield; 3: the creature stops being a creature. Returns cards P0 drew from
    // Confound and whether it resolved.
    let run = |change: u8| {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let shock = cast_new(&mut t, P1, "Shock", &[obj(bears)]);
        let (c, s) = offered_for(&mut t, P0, "Confound", None, &[obj(shock)]);
        assert!(c.contains(&obj(shock)));
        let s = s.unwrap();
        match change {
            1 => {
                cast_new(&mut t, P0, "Counterspell", &[obj(shock)]);
                t.resolve();
            }
            2 => crate::r_s02_common::destroy(&mut t, bears),
            3 => {
                // "Enchanted permanent is a colorless land with '{T}: Add {C}' and loses
                // all other card types and abilities."
                crate::r_s06_common::attach_new(&mut t, P1, "Imprisoned in the Moon", bears);
                assert!(!t.obj_now(bears).is(mtg_engine::types::CardType::Creature));
                assert!(t.on_battlefield(bears));
            }
            _ => {}
        }
        let hand = t.hand_size(P0);
        t.resolve();
        (t.hand_size(P0) - hand, resolved(&t, s))
    };
    assert_eq!(run(0), (1, true));
    for change in [1, 2, 3] {
        assert_eq!(run(change), (0, false), "change {change}");
    }
}

#[test]
fn spell_that_targets_you_or_your_permanent_can_have_other_targets() {
    cr!("115.1");
    ruling!(
        "Hindering Light",
        "Hindering Light can target a spell that has multiple targets, as long as at least one of those targets is you or a permanent you control."
    );
    ruling!(
        "Keep Safe",
        "Keep Safe can target a spell that has multiple targets, as long as at least one of those targets is a permanent you control."
    );
    ruling!(
        "Rebuff the Wicked",
        "Rebuff the Wicked can target a spell that has multiple targets, as long as at least one of those targets is a permanent you control."
    );
    ruling!(
        "Turn Aside",
        "Turn Aside can target a spell that has multiple targets, as long as at least one of those targets is a permanent you control."
    );
    ruling!(
        "Outwit",
        "The spell you target could also target other things, as long as it also targets a player."
    );
    // (card, whether it accepts a spell targeting P0 (a player) and P1's creature)
    let cases = [
        ("Hindering Light", true),
        ("Keep Safe", false),
        ("Rebuff the Wicked", false),
        ("Turn Aside", false),
        ("Outwit", true),
    ];
    for (name, player_ok) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        // Electrolyze at P0's Bears and P1 (both players' things).
        let mixed = electrolyze(&mut t, obj(mine), Entity::Player(P1));
        // Electrolyze at P0 and P1's Bears.
        let at_me = electrolyze(&mut t, Entity::Player(P0), obj(theirs));
        let (c, s) = offered_for(&mut t, P0, name, None, &[obj(mixed)]);
        assert!(c.contains(&obj(mixed)), "{name}");
        assert_eq!(c.contains(&obj(at_me)), player_ok, "{name}");
        t.resolve();
        assert!(resolved(&t, s.unwrap()), "{name}");
        assert!(!on_stack(&t, mixed), "{name}");
    }
}

#[test]
fn spells_targeting_only_opponents_things_arent_legal() {
    cr!("115.1");
    // Controls for the cases above: a spell targeting only P1's things.
    ruling!(
        "Hindering Light",
        "Hindering Light can target a spell that has multiple targets, as long as at least one of those targets is you or a permanent you control."
    );
    for name in ["Hindering Light", "Keep Safe", "Rebuff the Wicked", "Turn Aside"] {
        let mut t = TestGame::new(2);
        let theirs = t.battlefield(P1, "Grizzly Bears");
        let theirs2 = t.battlefield(P1, "Hill Giant");
        let only_theirs = electrolyze(&mut t, obj(theirs), obj(theirs2));
        let (c, _) = offered_for(&mut t, P0, name, None, &[obj(only_theirs)]);
        assert!(!c.contains(&obj(only_theirs)), "{name}");
    }
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let creatures_only = electrolyze(&mut t, obj(theirs), obj(mine));
    let (c, _) = offered_for(&mut t, P0, "Outwit", None, &[obj(creatures_only)]);
    assert!(!c.contains(&obj(creatures_only)));
}

/// P1's Lightning Bolt targets P0's Grizzly Bears; P0 answers with the real card `name`;
/// then `change` happens to the Bears: `leave` (bounced to hand) or it gets hexproof
/// (Ranger's Guile, so it's an illegal target for P1's Bolt but still P0's). Returns
/// (whether `name` resolved, whether the Bolt is still on the stack).
fn protect_then(name: &str, leave: bool) -> (bool, bool) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[obj(bears)]);
    let s = cast_new(&mut t, P0, name, &[obj(bolt)]);
    if leave {
        crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P0));
    } else {
        cast_new(&mut t, P0, "Ranger's Guile", &[obj(bears)]);
        t.resolve();
        assert!(crate::r_s06_common::has_kw(
            &t,
            bears,
            mtg_engine::keywords::KeywordKind::Hexproof
        ));
    }
    t.resolve();
    (resolved(&t, s), on_stack(&t, bolt))
}

#[test]
fn keep_safe_and_rebuff_need_the_permanent_still_on_the_battlefield() {
    cr!("608.2b", "702.11b");
    ruling!(
        "Keep Safe",
        "If the permanent you control targeted by the target spell leaves the battlefield, that spell is no longer a legal target for Keep Safe. On the other hand, if that target permanent becomes an illegal target for the target spell but remains on the battlefield under your control, the target spell is still a legal target for Keep Safe."
    );
    ruling!(
        "Rebuff the Wicked",
        "If the permanent you control targeted by the target spell leaves the battlefield, that spell is no longer a legal target for Rebuff the Wicked. On the other hand, if that targeted permanent becomes an illegal target for the target spell but remains on the battlefield under your control, the target spell is still a legal target for Rebuff the Wicked."
    );
    supported("Ranger's Guile");
    for name in ["Keep Safe", "Rebuff the Wicked"] {
        assert_eq!(protect_then(name, true), (false, true), "{name}: left");
        assert_eq!(protect_then(name, false), (true, false), "{name}: hexproof");
    }
}

/// P0 activates Siren Stormtamer ("{U}, Sacrifice this creature: Counter target spell or
/// ability that targets you or a creature you control.") at `target`.
fn stormtamer(t: &mut TestGame, target: ObjectId) -> Option<ObjectId> {
    let st = t.battlefield(P0, "Siren Stormtamer");
    t.lands(P0, "Island", 1);
    t.answer_targets(P0, &[obj(target)]);
    crate::r_s06_common::activate_containing(t, P0, st, "Counter target").expect("activate")
}

#[test]
fn siren_stormtamer_needs_the_creature_still_there() {
    cr!("608.2b", "702.11b");
    ruling!(
        "Siren Stormtamer",
        "If the creature you control targeted by the target spell or ability leaves the battlefield, that spell or ability is no longer a legal target for Siren Stormtamer's ability. On the other hand, if that targeted creature becomes an illegal target for the target spell but remains on the battlefield under your control, the target spell or ability is still a legal target for Siren Stormtamer's ability."
    );
    supported("Siren Stormtamer");
    // A spell (Lightning Bolt) and an ability (Prodigal Pyromancer's ping) at the Bears.
    for ability in [false, true] {
        for leave in [true, false] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            let threat = if ability {
                let pyro = t.battlefield(P1, "Prodigal Pyromancer");
                t.answer_targets(P1, &[obj(bears)]);
                crate::r_s06_common::activate_containing(&mut t, P1, pyro, "damage")
                    .unwrap()
                    .unwrap()
            } else {
                cast_new(&mut t, P1, "Lightning Bolt", &[obj(bears)])
            };
            let ab = stormtamer(&mut t, threat).expect("on the stack");
            if leave {
                crate::r_s05_common::move_to(&mut t, bears, Zone::Hand(P0));
            } else {
                cast_new(&mut t, P0, "Ranger's Guile", &[obj(bears)]);
                t.resolve();
            }
            t.resolve();
            assert!(!on_stack(&t, ab));
            assert_eq!(
                on_stack(&t, threat),
                leave,
                "ability {ability}, leave {leave}"
            );
        }
    }
}

#[test]
fn siren_stormtamer_targets_a_spell_with_several_targets() {
    cr!("115.1");
    ruling!(
        "Siren Stormtamer",
        "Siren Stormtamer's activated ability can target a spell or ability that has multiple targets, as long as at least one of those targets is you or a creature you control."
    );
    for (at_me, legal) in [(true, true), (false, false)] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        // Electrolyze at (P0 or P0's Bears) and P1's Bears; or P1's two things.
        let first = if at_me { Entity::Player(P0) } else { obj(theirs) };
        let e = electrolyze(&mut t, first, Entity::Player(P1));
        let e2 = electrolyze(&mut t, obj(mine), obj(theirs));
        let st = t.battlefield(P0, "Siren Stormtamer");
        let c = crate::r_p037_common::ability_targets(&mut t, st, "Counter target");
        assert_eq!(c.contains(&obj(e)), legal, "at me {at_me}");
        assert!(c.contains(&obj(e2)));
    }
}

// --- Second spell cast this turn ----------------------------------------------------------

#[test]
fn second_guess_only_needs_the_second_spell_on_the_stack() {
    cr!("115.1");
    ruling!(
        "Second Guess",
        "It doesn’t matter if the first spell cast that turn has resolved yet or not. It also doesn’t matter how many spells have been cast after the second one, as long as the second spell cast is still on the stack."
    );
    supported("Second Guess");
    let mut t = TestGame::new(2);
    let first = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve();
    assert!(!on_stack(&t, first));
    let second = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let third = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let fourth = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let (c, s) = offered_for(&mut t, P0, "Second Guess", None, &[obj(second)]);
    assert_eq!(c, vec![obj(second)]);
    assert!(!c.contains(&obj(third)) && !c.contains(&obj(fourth)));
    t.resolve();
    assert!(resolved(&t, s.unwrap()));
    assert!(!on_stack(&t, second));
}

#[test]
fn second_guess_counts_cast_copies_but_not_other_copies() {
    cr!("707.10", "707.12", "702.40a");
    ruling!(
        "Second Guess",
        "Most spells and abilities that copy a spell create that copy on the stack. These copies are not cast and won’t count toward the number of spells cast that turn. However, a few (like Isochron Scepter’s ability) instruct you to copy a spell and “cast the copy.” These spells are cast and will count toward the number of spells cast that turn."
    );
    supported("Isochron Scepter");
    supported("Grapeshot");
    // P1 casts Lightning Bolt (first); its storm copy aside, Grapeshot is the second.
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve();
    t.answer_yes(P1, false); // keep the storm copy's target
    let gs = cast_new(&mut t, P1, "Grapeshot", &[Entity::Player(P0)]);
    t.settle();
    t.resolve(); // storm: one copy
    let copies = crate::r_s25_common::spell_copies(&t);
    assert_eq!(copies.len(), 1);
    let (c, _) = offered_for(&mut t, P0, "Second Guess", None, &[obj(gs)]);
    assert!(c.contains(&obj(gs)));
    assert!(!c.contains(&obj(copies[0])));

    // Isochron Scepter casts its copy: P0's Shock is first, the Scepter's copy second.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve();
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    crate::r_s06_common::activate_containing(&mut t, P0, scepter, "copy the exiled card")
        .unwrap();
    t.resolve();
    // The copy of Lightning Bolt was cast: it's the second spell cast this turn.
    assert_eq!(t.stack_len(), 1);
    let copy = crate::r_s04_common::top_of_stack(&t);
    assert_eq!(t.obj(copy).chars.name, "Lightning Bolt");
    let (c, s) = offered_for(&mut t, P1, "Second Guess", None, &[obj(copy)]);
    assert_eq!(c, vec![obj(copy)]);
    t.resolve();
    assert!(resolved(&t, s.unwrap()));
    assert!(!on_stack(&t, copy));
    assert_eq!(t.life(P1), 18);
}

// --- Conditions checked on resolution -------------------------------------------------

#[test]
fn nix_targets_any_spell_and_checks_on_resolution() {
    cr!("115.1", "608.2b", "601.2h");
    ruling!(
        "Nix",
        "Nix can target any spell. The condition is checked only on resolution."
    );
    supported("Nix");
    // A spell P1 paid mana for: a legal target, not countered.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let (c, s) = offered_for(&mut t, P0, "Nix", None, &[obj(bolt)]);
    assert!(c.contains(&obj(bolt)));
    t.resolve();
    assert!(resolved(&t, s.unwrap()));
    assert!(on_stack(&t, bolt));
    // A spell cast for no mana (Ornithopter, {0}) is countered.
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let o = t.hand(P1, "Ornithopter");
    let orni = t.cast(P1, o).go();
    offered_for(&mut t, P0, "Nix", None, &[obj(orni)]);
    t.resolve();
    assert!(!on_stack(&t, orni));
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn nix_checks_whether_the_caster_spent_mana_not_the_controller() {
    cr!("608.2b", "601.2h");
    ruling!(
        "Nix",
        "Nix checks whether any player paid mana to cast the spell, not whether its current controller did. If Commandeer is cast on a spell, then Nix is cast on that spell, Nix won’t counter it unless the player who originally cast that spell spent no mana to cast it."
    );
    supported("Commandeer");
    // (P1's spell, whether P1 spent mana on it)
    for (name, paid) in [("Shock", true), ("Mox Pearl", false)] {
        let mut t = TestGame::new(2);
        turn_of(&mut t, P1);
        let spell = if paid {
            cast_new(&mut t, P1, name, &[Entity::Player(P0)])
        } else {
            let c = t.hand(P1, name);
            t.cast(P1, c).go()
        };
        t.answer_yes(P0, false);
        cast_new(&mut t, P0, "Commandeer", &[obj(spell)]);
        t.resolve();
        assert_eq!(t.obj(spell).controller, P0);
        // P1 now casts Nix on the spell P0 controls.
        cast_new(&mut t, P1, "Nix", &[obj(spell)]);
        t.resolve();
        assert_eq!(on_stack(&t, spell), paid, "{name}");
    }
}

#[test]
fn jaded_response_checks_colors_on_resolution() {
    cr!("115.1", "608.2c");
    ruling!("Jaded Response", "It does not check colors until it resolves.");
    supported("Jaded Response");
    // P0 controls no red creature when it casts Jaded Response at a red spell, and one
    // arrives before it resolves: countered. The reverse: not countered.
    for red_at_resolution in [true, false] {
        let mut t = TestGame::new(2);
        let goblin = if red_at_resolution {
            None
        } else {
            Some(t.battlefield(P0, "Raging Goblin"))
        };
        t.battlefield(P0, "Grizzly Bears");
        let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        let (c, _) = offered_for(&mut t, P0, "Jaded Response", None, &[obj(bolt)]);
        assert!(c.contains(&obj(bolt)));
        match goblin {
            None => {
                t.battlefield(P0, "Raging Goblin");
            }
            Some(g) => crate::r_s02_common::destroy(&mut t, g),
        }
        t.resolve();
        assert_eq!(on_stack(&t, bolt), !red_at_resolution);
    }
}

// --- Creature and noncreature spells -------------------------------------------------------

#[test]
fn artifact_creature_spells_are_creature_spells() {
    cr!("115.1", "205.2a");
    ruling!(
        "Essence Scatter",
        "A \"creature spell\" is any spell with the card type creature, even if it has other types such as artifact or enchantment."
    );
    ruling!(
        "Negate",
        "A \"creature spell\" is any spell with the creature type, even if it has other types such as artifact or enchantment."
    );
    supported("Essence Scatter");
    supported("Negate");
    supported("Nyxborn Courser");
    // Ornithopter (artifact creature) and Nyxborn Courser (enchantment creature), and an
    // artifact (Mind Stone).
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    for name in ["Ornithopter", "Nyxborn Courser"] {
        let spell = cast_new(&mut t, P1, name, &[]);
        let (c, _) = offered_for(&mut t, P0, "Essence Scatter", None, &[obj(spell)]);
        assert_eq!(c, vec![obj(spell)], "{name}");
        let (c, _) = offered_for(&mut t, P0, "Negate", None, &[]);
        assert!(!c.contains(&obj(spell)), "{name}");
        t.resolve_all();
    }
    let stone = cast_new(&mut t, P1, "Mind Stone", &[]);
    let (c, _) = offered_for(&mut t, P0, "Negate", None, &[obj(stone)]);
    assert_eq!(c, vec![obj(stone)]);
    let (c, _) = offered_for(&mut t, P0, "Essence Scatter", None, &[]);
    assert!(c.is_empty());
}

// --- Up to two targets / needing a target ---------------------------------------------------

#[test]
fn double_negative_needs_both_spells_on_the_stack() {
    cr!("601.2c", "115.1", "707.10", "117.3c");
    ruling!(
        "Double Negative",
        "In order to counter two target spells, both of those spells must be on the stack at the time you cast Double Negative. Two spells can be on the stack at the same time as the result of an ability (such as cascade or replicate), as the result of a copy effect (such as the one from Cloven Casting or Twincast), or because a spell has been cast in response to another spell."
    );
    supported("Double Negative");
    supported("Twincast");
    // A spell cast in response to another.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.answer_targets(P0, &[obj(bolt), obj(shock)]);
    let s = cast_new(&mut t, P0, "Double Negative", &[]);
    t.resolve();
    assert!(resolved(&t, s));
    assert!(!on_stack(&t, bolt) && !on_stack(&t, shock));
    assert_eq!(t.life(P0), 20);
    // A spell and its copy (Twincast, cast by P1 in response to its own Bolt).
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    crate::r_s25_common::keep_copy_targets(&mut t, P1);
    cast_new(&mut t, P1, "Twincast", &[obj(bolt)]);
    t.resolve();
    let copy = crate::r_s25_common::spell_copies(&t)[0];
    t.answer_targets(P0, &[obj(bolt), obj(copy)]);
    cast_new(&mut t, P0, "Double Negative", &[]);
    t.resolve();
    assert!(!on_stack(&t, bolt) && !on_stack(&t, copy));
    assert_eq!(t.life(P0), 20);
    // A spell that already resolved can't be targeted.
    let mut t = TestGame::new(2);
    let first = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve();
    let second = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    let (c, _) = offered_for(&mut t, P0, "Double Negative", None, &[obj(second)]);
    assert_eq!(c, vec![obj(second)]);
    assert!(!c.contains(&obj(first)));
}

#[test]
fn double_negative_holding_priority_versus_passing() {
    cr!("117.3c", "117.4", "608.1");
    ruling!(
        "Double Negative",
        "It’s unusual for a player to cast a spell in response to their own spell, and that will be specifically called out when it happens. Under normal circumstances, if a player casts a spell, it is assumed that the player passes priority afterward. If that player casts another spell, it is assumed that each other player has also passed priority, which caused the first spell to resolve and the first player to receive priority again. Unless the first player says otherwise, only one spell will be on the stack at a time in that scenario."
    );
    // After casting a spell, the caster receives priority again and may cast another in
    // response to its own spell: two spells are on the stack.
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    assert_eq!(t.g.turn.priority, Some(P1));
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    assert!(on_stack(&t, bolt) && on_stack(&t, shock));
    // If the caster passes instead (and so does everyone else), the first spell resolves
    // before the caster gets priority again: only one spell on the stack at a time.
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let life = t.life(P0);
    let ok = t.g.run_until(1_000, |g| g.stack.is_empty() && g.turn.priority == Some(P1));
    assert!(ok);
    assert!(!on_stack(&t, bolt));
    assert_eq!(t.life(P0), life - 3);
}

#[test]
fn double_negative_takes_zero_one_or_two_targets() {
    cr!("601.2c", "115.1a");
    ruling!(
        "Double Negative",
        "You may cast Double Negative with no targets, one target, or two targets."
    );
    // No spell on the stack: still castable with no targets.
    let mut t = TestGame::new(2);
    lands_for_cost(&mut t, P0, "Double Negative");
    let dn = t.hand(P0, "Double Negative");
    assert!(castable(&mut t, P0, dn));
    t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![]));
    let s = t.cast_with(P0, dn, &[]).unwrap();
    t.resolve();
    assert!(resolved(&t, s));
    // One target out of two spells.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![obj(shock)]));
    lands_for_cost(&mut t, P0, "Double Negative");
    let dn = t.hand(P0, "Double Negative");
    t.cast_with(P0, dn, &[]).unwrap();
    t.resolve();
    assert!(!on_stack(&t, shock));
    assert!(on_stack(&t, bolt));
}

#[test]
fn trap_essence_needs_a_creature_spell_but_not_a_creature() {
    cr!("601.2c", "115.1a");
    ruling!(
        "Trap Essence",
        "You don’t have to target a creature to cast Trap Essence, although you must target a creature spell on the stack."
    );
    supported("Trap Essence");
    // No creature spell: can't cast it (even with a creature on the battlefield).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let _ = bolt;
    lands_for_cost(&mut t, P0, "Trap Essence");
    let te = t.hand(P0, "Trap Essence");
    assert!(!castable(&mut t, P0, te));
    // A creature spell and no creatures: castable, and it counters the spell.
    let mut t = TestGame::new(2);
    let bears = bears_spell(&mut t, P1);
    t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![obj(bears)]));
    t.answer(P0, DecisionKind::Targets, Answer::Entities(vec![]));
    lands_for_cost(&mut t, P0, "Trap Essence");
    let te = t.hand(P0, "Trap Essence");
    let s = t.cast_with(P0, te, &[]).unwrap();
    t.resolve();
    assert!(resolved(&t, s));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn essence_capture_needs_a_creature_spell_and_affects_whichever_target_is_legal() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Essence Capture",
        "You can't cast Essence Capture without a target creature spell. If either target is illegal when Essence Capture tries to resolve, the other is still affected as appropriate."
    );
    // No creature spell: can't cast it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    lands_for_cost(&mut t, P0, "Essence Capture");
    let ec = t.hand(P0, "Essence Capture");
    assert!(!castable(&mut t, P0, ec));
    // P0's creature leaves: the spell is still countered.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let spell = bears_spell(&mut t, P1);
    cast_new(&mut t, P0, "Essence Capture", &[obj(spell), obj(mine)]);
    crate::r_s02_common::destroy(&mut t, mine);
    t.resolve();
    assert!(!on_stack(&t, spell));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // The spell is countered first: P0's creature still gets the counter.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let spell = bears_spell(&mut t, P1);
    cast_new(&mut t, P0, "Essence Capture", &[obj(spell), obj(mine)]);
    cast_new(&mut t, P0, "Counterspell", &[obj(spell)]);
    t.resolve();
    t.resolve();
    assert_eq!(t.counters(mine, "+1/+1"), 1);
}

#[test]
fn put_away_with_one_illegal_target_affects_the_other() {
    cr!("608.2b");
    ruling!(
        "Put Away",
        "If you choose to target a card in your graveyard, Put Away will have two targets. If either one of the targets becomes illegal, Put Away will continue to resolve and affect the other target."
    );
    supported("Put Away");
    // The spell is countered first: the card is still shuffled in.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let card = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Put Away", &[obj(bolt), obj(card)]);
    cast_new(&mut t, P0, "Counterspell", &[obj(bolt)]);
    t.resolve();
    t.resolve();
    assert_eq!(t.zone(t.g.current(card)), Zone::Library(P0));
    // The card leaves the graveyard first: the spell is still countered.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let card = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Put Away", &[obj(bolt), obj(card)]);
    crate::r_s05_common::move_to(&mut t, card, Zone::Exile);
    t.resolve();
    assert!(!on_stack(&t, bolt));
    assert_eq!(t.life(P0), 20);
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn put_away_cant_shuffle_the_spell_it_counters() {
    cr!("601.2c", "115.1");
    ruling!(
        "Put Away",
        "You must choose all the targets at the time you cast Put Away. You can’t use it to counter your own spell and then shuffle that card into your library because at the time you choose targets, the spell isn’t in your graveyard yet."
    );
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    let other = t.graveyard(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Put Away");
    let pa = t.hand(P0, "Put Away");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bolt)]);
    t.answer_targets(P0, &[obj(other)]);
    t.answer_yes(P0, true);
    t.cast_with(P0, pa, &[]).unwrap();
    let c = offered(&t, P0, from);
    assert_eq!(c.len(), 2);
    assert_eq!(c[1], vec![obj(other)]);
    t.resolve();
    // The Bolt went to the graveyard after targets were chosen, and stays there.
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.zone(t.g.current(other)), Zone::Library(P0));
}

#[test]
fn fall_of_the_gavel_and_render_silent_need_a_spell_to_target() {
    cr!("601.2c", "115.1");
    ruling!(
        "Fall of the Gavel",
        "You must be able to target a spell to cast Fall of the Gavel."
    );
    ruling!(
        "Render Silent",
        "You must be able to target a spell to cast Render Silent."
    );
    for name in ["Fall of the Gavel", "Render Silent"] {
        supported(name);
        let mut t = TestGame::new(2);
        lands_for_cost(&mut t, P0, name);
        let c = t.hand(P0, name);
        assert!(!castable(&mut t, P0, c), "{name}");
        cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        assert!(castable(&mut t, P0, c), "{name}");
    }
}
