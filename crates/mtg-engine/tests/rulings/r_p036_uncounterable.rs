//! Rulings batch P036 — counterspells aimed at spells that can't be countered (CR 101.2,
//! 701.6b: the spell is still a legal target, isn't countered, and the counterspell's other
//! effects still happen, CR 608.2c), and counterspells whose only target became illegal
//! (CR 608.2b: they don't resolve and none of their effects happen).

use crate::r_p036_common::*;
use crate::r_s01_common::supported;
use crate::r_s07_common::resolved;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s30_common::set_life;
use mtg_engine::decision::Action;
use mtg_engine::turn::Step;
use mtg_engine::testing::*;
use mtg_engine::*;

/// What happened when P0 cast the real counterspell `name` at an uncounterable spell of P1.
struct Outcome {
    t: TestGame,
    /// Whether the uncounterable spell was among the legal first targets offered.
    offered: bool,
    /// Cards P0 drew while the counterspell resolved.
    drew: i32,
    /// Whether the uncounterable spell was still on the stack after the counterspell
    /// resolved.
    still_on_stack: bool,
    /// Whether the counterspell itself resolved.
    resolved: bool,
}

/// P1 casts Abrupt Decay at P0's Grizzly Bears (or, with `creature`, casts Carnage Tyrant);
/// `prep` sets up and returns P0's extra targets; P0 casts `name` at the spell, it
/// resolves, then the rest of the stack resolves.
fn at_uncounterable(
    name: &str,
    creature: bool,
    prep: impl FnOnce(&mut TestGame) -> Vec<Entity>,
) -> Outcome {
    supported(name);
    let mut t = TestGame::new(2);
    let (spell, bears) = if creature {
        (uncounterable_tyrant(&mut t), None)
    } else {
        let (d, b) = uncounterable_decay(&mut t);
        (d, Some(b))
    };
    let extra = prep(&mut t);
    lands_for_cost(&mut t, P0, name);
    let card = t.hand(P0, name);
    let from = t.asked().len();
    let mut targets = vec![obj(spell)];
    targets.extend(extra);
    let cs = t.cast_with(P0, card, &targets).expect("cast");
    let offered = offered(&t, P0, from)
        .first()
        .is_some_and(|c| c.contains(&obj(spell)));
    let hand = t.hand_size(P0) as i32;
    t.resolve();
    let drew = t.hand_size(P0) as i32 - hand;
    let still_on_stack = on_stack(&t, spell);
    let resolved = resolved(&t, cs);
    t.resolve_all();
    // The uncounterable spell resolved.
    match bears {
        Some(b) => assert!(!t.on_battlefield(b), "{name}: Abrupt Decay didn't resolve"),
        None => assert_eq!(
            t.named_on_battlefield("Carnage Tyrant").len(),
            1,
            "{name}: Carnage Tyrant didn't resolve"
        ),
    }
    Outcome {
        t,
        offered,
        drew,
        still_on_stack,
        resolved,
    }
}

fn check(o: &Outcome, name: &str) {
    assert!(o.offered, "{name}: uncounterable spell not offered as a target");
    assert!(o.resolved, "{name}: didn't resolve");
    assert!(o.still_on_stack, "{name}: the spell was countered");
}

#[test]
fn counterspells_with_a_rider_still_apply_it_to_an_uncounterable_spell() {
    cr!("101.2", "701.6b", "608.2c");
    ruling!(
        "Absorb",
        "A spell that can't be countered is a legal target for Absorb. The spell won't be countered when Absorb resolves, but you'll still gain 3 life."
    );
    ruling!(
        "Contradict",
        "A spell that can’t be countered by spells and abilities is a legal target for Contradict. The spell won’t be countered when Contradict resolves, but you’ll draw a card."
    );
    ruling!(
        "Ionize",
        "A spell that can’t be countered is a legal target for Ionize. The spell won’t be countered when Ionize resolves, but Ionize still deals 2 damage to that spell’s controller."
    );
    ruling!(
        "Keep Safe",
        "A spell that can’t be countered is a legal target for Keep Safe. That spell won’t be countered when Keep Safe resolves, but you’ll still draw a card."
    );
    ruling!(
        "Hindering Light",
        "You may choose to target a spell that “can’t be countered.” If you do, the first part of Hindering Light’s effect won’t do anything, but you’ll still get to draw a card."
    );
    ruling!(
        "Countersquall",
        "A spell that can’t be countered is a legal target for Countersquall. The spell won’t be countered when Countersquall resolves, but its controller still loses 2 life."
    );
    // (card, P0's life change, P1's life change, cards P0 drew). Abrupt Decay targets
    // P0's Bears, so Keep Safe and Hindering Light can target it.
    let cases: [(&str, i32, i32, i32); 6] = [
        ("Absorb", 3, 0, 0),
        ("Contradict", 0, 0, 1),
        ("Ionize", 0, -2, 0),
        ("Keep Safe", 0, 0, 1),
        ("Hindering Light", 0, 0, 1),
        ("Countersquall", 0, -2, 0),
    ];
    for (name, life0, life1, draw) in cases {
        let o = at_uncounterable(name, false, |_| vec![]);
        check(&o, name);
        assert_eq!(o.t.life(P0) - 20, life0, "{name}");
        assert_eq!(o.t.life(P1) - 20, life1, "{name}");
        assert_eq!(o.drew, draw, "{name}");
    }
}

#[test]
fn spell_snuff_draws_at_five_life_even_if_the_spell_cant_be_countered() {
    cr!("101.2", "701.6b", "608.2c");
    ruling!(
        "Spell Snuff",
        "A spell that can’t be countered is a legal target for Spell Snuff. The spell won’t be countered when Spell Snuff resolves, but if you have 5 or less life, you’ll still draw a card."
    );
    // At 5 life P0 draws; at 6 life it doesn't.
    for (life, draw) in [(5, 1), (6, 0)] {
        let o = at_uncounterable("Spell Snuff", false, |t| {
            set_life(t, P0, life);
            vec![]
        });
        check(&o, "Spell Snuff");
        assert_eq!(o.drew, draw, "at {life} life");
    }
}

#[test]
fn creature_counterspells_with_a_rider_still_apply_it_to_an_uncounterable_creature() {
    cr!("101.2", "701.6b", "608.2c");
    ruling!(
        "Bone to Ash",
        "A creature spell that can’t be countered is a legal target for Bone to Ash. The creature spell won’t be countered when Bone to Ash resolves, but you’ll still draw a card."
    );
    ruling!(
        "Exclude",
        "Exclude can target a creature spell that can't be countered. When Exclude resolves, that spell won't be countered, but you'll still draw a card."
    );
    ruling!(
        "Geist Snatch",
        "If Geist Snatch resolves but the creature spell can’t be countered, you’ll still put the Spirit token onto the battlefield."
    );
    for name in ["Bone to Ash", "Exclude"] {
        let o = at_uncounterable(name, true, |_| vec![]);
        check(&o, name);
        assert_eq!(o.drew, 1, "{name}");
    }
    let o = at_uncounterable("Geist Snatch", true, |_| vec![]);
    check(&o, "Geist Snatch");
    let spirits = crate::r_s01_common::with_subtype(&o.t, P0, "Spirit");
    assert_eq!(spirits.len(), 1);
    assert_eq!(o.t.pt(spirits[0]), (1, 1));
}

#[test]
fn essence_capture_still_puts_the_counter_on_your_creature() {
    cr!("101.2", "701.6b", "608.2c", "115.1b");
    ruling!(
        "Essence Capture",
        "A creature spell that can't be countered is a legal target for Essence Capture. The spell won't be countered when Essence Capture resolves, but you'll still put a +1/+1 counter on the target creature."
    );
    let mut bears = None;
    let o = at_uncounterable("Essence Capture", true, |t| {
        let b = t.battlefield(P0, "Grizzly Bears");
        bears = Some(b);
        vec![obj(b)]
    });
    check(&o, "Essence Capture");
    assert_eq!(o.t.counters(bears.unwrap(), "+1/+1"), 1);
}

#[test]
fn put_away_still_shuffles_the_graveyard_card_in() {
    cr!("101.2", "701.6b", "608.2c");
    ruling!(
        "Put Away",
        "If the targeted spell can’t be countered, that spell will remain on the stack. Put Away will continue to resolve. If you targeted a card in your graveyard, you still get to shuffle it into your library."
    );
    let mut card = None;
    let o = at_uncounterable("Put Away", false, |t| {
        let c = t.graveyard(P0, "Lightning Bolt");
        card = Some(c);
        t.answer_yes(P0, true);
        vec![obj(c)]
    });
    check(&o, "Put Away");
    assert!(!o.t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(o.t.zone(o.t.g.current(card.unwrap())), mtg_engine::object::Zone::Library(P0));
}

#[test]
fn fold_into_aether_does_nothing_if_the_spell_isnt_countered() {
    cr!("101.2", "701.6b", "608.2c");
    ruling!(
        "Fold into Aether",
        "If the spell isn’t countered (because it’s a spell that can’t be countered, for example), nothing happens."
    );
    // P1 has a creature card in hand and would put it onto the battlefield if offered.
    let o = at_uncounterable("Fold into Aether", false, |t| {
        t.hand(P1, "Hill Giant");
        t.answer_yes(P1, true);
        vec![]
    });
    check(&o, "Fold into Aether");
    assert!(o.t.named_on_battlefield("Hill Giant").is_empty());
    assert!(o.t.in_hand(P1, "Hill Giant"));
}

#[test]
fn fold_into_aether_countering_your_own_spell_lets_you_put_a_creature_in() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Fold into Aether",
        "If your Fold into Aether counters a spell you control, you get to put a creature card from your hand onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.hand(P1, "Shivan Dragon");
    t.answer_yes(P0, true);
    let giant = t.hand(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    cast_new(&mut t, P0, "Fold into Aether", &[obj(bolt)]);
    t.resolve();
    assert!(!on_stack(&t, bolt));
    assert_eq!(t.life(P1), 20);
    let giants = t.named_on_battlefield("Hill Giant");
    assert_eq!(giants.len(), 1);
    assert_eq!(t.obj(giants[0]).controller, P0);
    // P1, who didn't control the countered spell, doesn't get to put a creature in.
    assert!(t.in_hand(P1, "Shivan Dragon"));
}

#[test]
fn sages_dousing_draws_with_a_wizard_however_the_spell_fares() {
    cr!("608.2c", "118.12");
    ruling!(
        "Sage's Dousing",
        "If you control a Wizard, you draw a card regardless of whether the targeted spell was countered or {3} was paid."
    );
    supported("Sage's Dousing");
    // (P1 can pay, P1 pays, P0 controls a Wizard, the spell is countered, P0 draws)
    let cases = [
        (true, true, true, false, 1),
        (false, false, true, true, 1),
        (true, true, false, false, 0),
        (false, false, false, true, 0),
    ];
    for (can_pay, pay, wizard, countered, draw) in cases {
        let mut t = TestGame::new(2);
        if can_pay {
            t.lands(P1, "Wastes", 3);
        }
        let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        if wizard {
            // Siren Stormtamer: a Siren Pirate Wizard.
            t.battlefield(P0, "Siren Stormtamer");
        }
        t.answer_yes(P1, pay);
        cast_new(&mut t, P0, "Sage's Dousing", &[obj(bolt)]);
        let hand = t.hand_size(P0);
        t.resolve();
        assert_eq!(!on_stack(&t, bolt), countered, "case {can_pay} {pay} {wizard}");
        assert_eq!(t.hand_size(P0) - hand, draw, "case {can_pay} {pay} {wizard}");
    }
    // An uncounterable spell: still a draw.
    let o = at_uncounterable("Sage's Dousing", false, |t| {
        t.battlefield(P0, "Siren Stormtamer");
        vec![]
    });
    check(&o, "Sage's Dousing");
    assert_eq!(o.drew, 1);
}

#[test]
fn flip_the_switch_creates_the_zombie_even_if_the_tax_is_paid() {
    cr!("608.2c", "118.12");
    ruling!(
        "Flip the Switch",
        "You will create a Zombie creature token even if the spell's controller pays {4}."
    );
    supported("Flip the Switch");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P1, "Wastes", 4);
        let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        t.answer_yes(P1, pay);
        cast_new(&mut t, P0, "Flip the Switch", &[obj(bolt)]);
        t.resolve();
        assert_eq!(on_stack(&t, bolt), pay);
        let zombies = crate::r_s01_common::with_subtype(&t, P0, "Zombie");
        assert_eq!(zombies.len(), 1, "pay {pay}");
        assert_eq!(t.pt(zombies[0]), (2, 2));
    }
}

// --- Illegal target: the counterspell doesn't resolve ------------------------------------

/// P1 casts `victim` (Lightning Bolt at P0, or a creature spell); P0 casts the real
/// counterspell `name` at it (after `prep`, which returns extra targets); then P0's
/// Counterspell counters the victim first, and `name` tries to resolve.
fn fizzle(
    name: &str,
    creature: bool,
    prep: impl FnOnce(&mut TestGame) -> Vec<Entity>,
) -> (TestGame, ObjectId, i32) {
    supported(name);
    let mut t = TestGame::new(2);
    let victim = if creature {
        bears_spell(&mut t, P1)
    } else {
        cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)])
    };
    let mut targets = vec![obj(victim)];
    targets.extend(prep(&mut t));
    let cs = cast_new(&mut t, P0, name, &targets);
    cast_new(&mut t, P0, "Counterspell", &[obj(victim)]);
    t.resolve();
    assert!(!on_stack(&t, victim));
    let hand = t.hand_size(P0) as i32;
    t.resolve();
    let drew = t.hand_size(P0) as i32 - hand;
    assert!(!resolved(&t, cs), "{name} resolved");
    assert!(t.in_graveyard(P0, name), "{name} not in the graveyard");
    (t, cs, drew)
}

#[test]
fn counterspells_whose_target_is_gone_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Contradict",
        "If the spell becomes an illegal target (because it’s left the stack, for example) when Contradict tries to resolve, Contradict won’t resolve and none of its effects will happen. You won’t draw a card."
    );
    ruling!(
        "Fall of the Gavel",
        "If the target spell is an illegal target when Fall of the Gavel tries to resolve (perhaps because it’s been countered by another spell), Fall of the Gavel won’t resolve and none of its effects will happen. You won’t gain 5 life."
    );
    ruling!(
        "Geist Snatch",
        "If the target creature spell is an illegal target when Geist Snatch tries to resolve, Geist Snatch won’t resolve and none of its effects will happen. You won’t put a Spirit token onto the battlefield."
    );
    ruling!(
        "Summoner's Bane",
        "If the targeted spell is an illegal target by the time Summoner’s Bane resolves, the entire spell doesn’t resolve. You won’t get a creature token."
    );
    let (_, _, drew) = fizzle("Contradict", false, |_| vec![]);
    assert_eq!(drew, 0);
    let (t, _, _) = fizzle("Fall of the Gavel", false, |_| vec![]);
    assert_eq!(t.life(P0), 20);
    for name in ["Geist Snatch", "Summoner's Bane"] {
        let (t, _, _) = fizzle(name, true, |_| vec![]);
        assert!(crate::r_s01_common::tokens(&t, P0).is_empty(), "{name}");
    }
    // Controls: with a legal target, each does its thing.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    cast_new(&mut t, P0, "Fall of the Gavel", &[obj(bolt)]);
    t.resolve();
    assert_eq!(t.life(P0), 25);
    let mut t = TestGame::new(2);
    let bears = bears_spell(&mut t, P1);
    cast_new(&mut t, P0, "Summoner's Bane", &[obj(bears)]);
    t.resolve();
    let illusions = crate::r_s01_common::with_subtype(&t, P0, "Illusion");
    assert_eq!(illusions.len(), 1);
    assert_eq!(t.pt(illusions[0]), (2, 2));
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
}

#[test]
fn summoners_bane_gives_you_the_token() {
    cr!("608.2c", "111.2");
    ruling!(
        "Summoner's Bane",
        "You get the token creature, not the controller of the creature spell."
    );
    supported("Summoner's Bane");
    let mut t = TestGame::new(2);
    let bears = bears_spell(&mut t, P1);
    cast_new(&mut t, P0, "Summoner's Bane", &[obj(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let illusions = crate::r_s01_common::with_subtype(&t, P0, "Illusion");
    assert_eq!(illusions.len(), 1);
    assert!(crate::r_s01_common::tokens(&t, P1).is_empty());
}

#[test]
fn render_silent_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Render Silent",
        "If the spell is an illegal target when Render Silent tries to resolve (because it’s been countered by another spell, for example), Render Silent won’t resolve and none of its effects will happen."
    );
    let (mut t, _, _) = fizzle("Render Silent", false, |_| vec![]);
    // P1 can still cast spells this turn.
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    assert!(crate::r_s21_common::castable(&mut t, P1, shock));
    // Control: when Render Silent resolves, P1 can't.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    cast_new(&mut t, P0, "Render Silent", &[obj(bolt)]);
    t.resolve();
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    assert!(!crate::r_s21_common::castable(&mut t, P1, shock));
}

#[test]
fn lay_bare_with_an_illegal_target_doesnt_look_at_the_hand() {
    cr!("608.2b");
    ruling!(
        "Lay Bare",
        "If the targeted spell is an illegal target by the time Lay Bare resolves, Lay Bare doesn’t resolve. You won’t look at its controller’s hand."
    );
    let looked = |t: &TestGame| t.dump_log().contains("looks at");
    let (t, _, _) = fizzle("Lay Bare", false, |_| vec![]);
    assert!(!looked(&t));
    // Control: with a legal target P0 looks at P1's hand.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    cast_new(&mut t, P0, "Lay Bare", &[obj(bolt)]);
    t.resolve();
    assert!(!on_stack(&t, bolt));
    assert!(looked(&t));
}

#[test]
fn repel_intruders_with_an_illegal_target_makes_no_tokens() {
    cr!("608.2b", "601.2h");
    ruling!(
        "Repel Intruders",
        "If you choose to target a creature spell, Repel Intruders has a target. If that target becomes illegal by the time Repel Intruders resolves, the entire spell won’t resolve. You won’t get any tokens."
    );
    supported("Repel Intruders");
    // P0 pays {3}{W/U} with {W}{U}{2} from its pool: both parts would apply.
    let run = |countered_first: bool| {
        let mut t = TestGame::new(2);
        let bears = bears_spell(&mut t, P1);
        crate::r_p037_common::pool(&mut t, P0, mtg_engine::mana::ManaType::W, 1);
        crate::r_p037_common::pool(&mut t, P0, mtg_engine::mana::ManaType::U, 1);
        crate::r_p037_common::pool(&mut t, P0, mtg_engine::mana::ManaType::C, 2);
        let card = t.hand(P0, "Repel Intruders");
        t.cast_with(P0, card, &[obj(bears)]).unwrap();
        if countered_first {
            cast_new(&mut t, P0, "Counterspell", &[obj(bears)]);
            t.resolve();
        }
        t.resolve();
        t
    };
    let t = run(true);
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    let t = run(false);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Kithkin").len(), 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn render_silent_doesnt_stop_responses_or_abilities_or_lands() {
    cr!("608.2c", "117.3c", "305.1", "702.86a");
    ruling!(
        "Render Silent",
        "Render Silent doesn’t stop any player from casting spells in response to Render Silent before it resolves."
    );
    ruling!(
        "Render Silent",
        "The affected player can still activate abilities, including abilities of cards in their hand (such as bloodrush abilities), and the player can still play lands."
    );
    supported("Render Silent");
    supported("Rubblebelt Maaka");
    // It's P1's turn; P0 answers P1's Lightning Bolt with Render Silent.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    cast_new(&mut t, P0, "Render Silent", &[obj(bolt)]);
    // P1 can respond with another spell while Render Silent is on the stack.
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    assert!(crate::r_s21_common::castable(&mut t, P1, shock));
    t.resolve();
    assert!(!on_stack(&t, bolt));
    assert!(t.stack.is_empty());
    assert!(!crate::r_s21_common::castable(&mut t, P1, shock));
    // P1 can still activate a permanent's ability and play a land.
    assert!(crate::r_s02_common::can_activate(&mut t, P1, pyro));
    let land = t.hand(P1, "Forest");
    assert!(crate::r_s02_common::can_play_land(&mut t, P1, land));
    t.play_land(P1, land).unwrap();
    // And activate bloodrush (an ability of a card in hand) for an attacking creature.
    let maaka = t.hand(P1, "Rubblebelt Maaka");
    crate::r_s01_common::attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    t.answer_targets(P1, &[obj(giant)]);
    t.g.turn.priority = Some(P1);
    let actions = t.g.legal_actions(P1);
    let uid = actions
        .iter()
        .find_map(|a| match a {
            Action::Activate { source, ability } if *source == maaka => Some(*ability),
            _ => None,
        })
        .expect("bloodrush offered");
    t.g.activate_ability(P1, maaka, uid).unwrap();
    t.resolve();
    assert_eq!(t.pt(giant), (6, 6));
    // P1 still can't cast Shock this turn.
    assert!(!crate::r_s21_common::castable(&mut t, P1, shock));
}
