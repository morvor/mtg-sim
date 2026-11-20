//! Item `rulings-residue-misc-2`, pass C — replacement and prevention effects, doubling, and
//! values fixed as an effect begins: Lich's Mirror and a dying creature, Enduring Angel on a
//! permanent that can't transform (or a copy that can), Bitter Feud with a prevention
//! shield, Obosh's doubled damage, Academy Manufactors, doubling counters (Lily Bowen,
//! Sword of Hours) and power (Neyith, Overclocked Electromancer, The Skullspore Nexus),
//! Megatron's excess damage, Unpredictable Cyclone, Agitator Ant, Unbound Flourishing.

use crate::r_residue_misc_2c_common::*;
use mtg_engine::ability::{Duration, Effect, Modification, Sel, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `what` becomes a copy of `of` until end of turn (as a resolving effect would).
fn become_copy(t: &mut TestGame, what: ObjectId, of: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![obj(what)], vec![obj(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

/// Gives the creature +p/+0 until end of turn.
fn pump(t: &mut TestGame, id: ObjectId, p: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::Const(p), Value::Const(0))],
    );
}

// --- Lich's Mirror ----------------------------------------------------------------------------

#[test]
fn lichs_mirror_the_dying_creatures_controller_chooses_and_dies_triggers_follow() {
    cr!("704.3", "400.6", "603.6c");
    ruling!(
        "Lich's Mirror",
        "If, during a check of state-based actions, you'd lose the game at the same time a creature you own would be put into your graveyard (due to an Earthquake for 10 or combat damage dealt to both you and the creature, for example), that creature's controller has a choice to make. The state-based actions rule is trying to simultaneously (a) shuffle that creature card into your library (due to Lich's Mirror's replacement effect) and (b) put it into your graveyard. Only one of those things can happen. The creature's controller chooses which one. If the creature is put into your graveyard, it isn't shuffled into your library. Abilities that trigger when that creature is put into a graveyard will trigger only if that option is chosen."
    );
    supported("Doomed Traveler");
    for to_library in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lich's Mirror");
        // P0's Doomed Traveler ("When this creature dies, create a 1/1 white Spirit
        // creature token with flying."), controlled by P1.
        let traveler = t.battlefield(P0, "Doomed Traveler");
        give_control(&mut t, traveler, P1);
        t.g.players[0].life = 0;
        let now = t.g.current(traveler);
        t.g.objects[now.0 as usize].damage = 1;
        let from = t.asked().len();
        t.answer_yes(P1, to_library);
        t.settle();
        t.resolve_all();
        assert!(!t.has_lost(P0), "{}", t.dump_log());
        assert_eq!(t.life(P0), 20);
        let choosers: Vec<PlayerId> = asked_since(&t, from)
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("state-based action")))
            .map(|(p, _)| p)
            .collect();
        assert_eq!(choosers, vec![P1], "{}", t.dump_log());
        assert_eq!(t.in_graveyard(P0, "Doomed Traveler"), !to_library);
        // The dies trigger only if it went to the graveyard.
        assert_eq!(
            tokens_with(&t, P1, "Spirit").len(),
            usize::from(!to_library),
            "to library {to_library}: {}",
            t.dump_log()
        );
    }
}

// --- Enduring Angel ---------------------------------------------------------------------------

const ANGEL: &str = "Enduring Angel // Angelic Enforcer";

#[test]
fn enduring_angel_a_single_faced_copy_cant_transform_and_loses_the_game() {
    cr!("701.27a", "701.27c", "614.1a", "707.2");
    ruling!(
        "Enduring Angel // Angelic Enforcer",
        "if a single-faced permanent has become a copy of it. In that situation, you will lose the game."
    );
    let mut t = TestGame::new(2);
    let original = t.battlefield(P1, ANGEL);
    let bears = t.battlefield(P0, "Grizzly Bears");
    become_copy(&mut t, bears, original);
    assert_eq!(named_by(&t, P0, "Enduring Angel").len(), 1);
    damage_to(&mut t, original, 25, pl(P0));
    assert!(t.has_lost(P0), "{}", t.dump_log());
}

#[test]
fn enduring_angel_a_token_copy_is_double_faced_and_transforms() {
    // The ruling's "if it's a token" is outdated (exempt): since the CR effective
    // 2023-04-14, a token copy of a double-faced permanent is a double-faced token.
    cr!("707.8a", "701.27a", "614.1a");
    let mut t = TestGame::new(2);
    let original = t.battlefield(P1, ANGEL);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![obj(original)]];
    t.g.exec(
        &Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: mtg_engine::ability::PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
    let token = named_by(&t, P0, "Enduring Angel");
    assert_eq!(token.len(), 1);
    damage_to(&mut t, original, 25, pl(P0));
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    assert_eq!(t.life(P0), 3);
    assert_eq!(t.obj_now(token[0]).chars.name, "Angelic Enforcer");
}

#[test]
fn enduring_angel_a_double_faced_copy_transforms_and_stays_a_copy() {
    cr!("701.28a", "707.2", "712.8e");
    ruling!(
        "Enduring Angel // Angelic Enforcer",
        "If a nonmodal double-faced card has somehow become a copy of Enduring Angel, the replacement effect will apply, the copy of Enduring Angel will transform, and your life total will become 3. However, because the copy effect that makes it Enduring Angel continues to apply to it after it transforms, it will still be a copy of Enduiring Angel. This means that the replacement effect will be applied again if your life total would be reduced to 0 or less a second time while that copy effect is still applied."
    );
    let mut t = TestGame::new(2);
    let original = t.battlefield(P1, ANGEL);
    let delver = t.battlefield(P0, "Delver of Secrets // Insectile Aberration");
    become_copy(&mut t, delver, original);
    let src = t.battlefield(P1, "Grizzly Bears");
    damage_to(&mut t, src, 25, pl(P0));
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    assert_eq!(t.life(P0), 3);
    assert_eq!(t.obj_now(delver).face, FaceState::Back);
    assert_eq!(t.obj_now(delver).chars.name, "Enduring Angel");
    // Again: it transforms back, and the life total becomes 3 again.
    damage_to(&mut t, src, 10, pl(P0));
    assert!(!t.has_lost(P0), "{}", t.dump_log());
    assert_eq!(t.life(P0), 3);
    assert_eq!(t.obj_now(delver).face, FaceState::Front);
    assert_eq!(t.obj_now(delver).chars.name, "Enduring Angel");
}

// --- Bitter Feud ------------------------------------------------------------------------------

#[test]
fn bitter_feud_and_a_prevention_shield_apply_in_the_order_the_damaged_player_chooses() {
    cr!("616.1", "615.1", "614.1a", "510.2");
    ruling!(
        "Bitter Feud",
        "If multiple effects modify how damage is dealt, the player being dealt damage, or the controller of the permanent being dealt damage, chooses the order in which to apply the effects."
    );
    supported("Decorated Griffin");
    let mut losses = vec![];
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.answer_choose(P0, &[pl(P0), pl(P1)]);
        t.enter(P0, "Bitter Feud");
        t.settle();
        let griffin = t.battlefield(P0, "Decorated Griffin");
        let giant = t.battlefield(P1, "Hill Giant");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P0, "Plains", 2);
        t.activate(P0, griffin, 0, &[]).expect("Griffin");
        t.resolve_all();
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
        attack_with(&mut t, &[(giant, pl(P0))]);
        t.advance_to(P1, Step::EndOfCombat);
        let asked: Vec<PlayerId> = asked_since(&t, from)
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseReplacement { .. }))
            .map(|(p, _)| p)
            .collect();
        assert_eq!(asked, vec![P0], "{}", t.dump_log());
        losses.push(20 - t.life(P0));
    }
    losses.sort();
    // Prevent 1 then double the 2: 4. Double to 6 then prevent 1: 5.
    assert_eq!(losses, vec![4, 5]);
}

// --- Obosh, the Preypiercer -------------------------------------------------------------------

#[test]
fn obosh_doubled_damage_is_dealt_by_the_original_source() {
    cr!("614.1a", "120.3f", "702.15b");
    ruling!(
        "Obosh, the Preypiercer",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by Obosh unless it was the original source of damage."
    );
    let mut t = TestGame::new(2);
    let obosh = t.battlefield(P0, "Obosh, the Preypiercer");
    let hawk = t.battlefield(P0, "Vampire Nighthawk"); // mana value 3, lifelink
    damage_to(&mut t, hawk, 2, pl(P1));
    assert_eq!(t.life(P1), 16);
    // Lifelink: the Nighthawk dealt the 4 damage.
    assert_eq!(t.life(P0), 24);
    let sources: Vec<ObjectId> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, .. } => Some(*source),
                _ => None,
            })
            .collect();
    assert_eq!(sources, vec![hawk]);
    assert!(!sources.contains(&obosh));
}

// --- Academy Manufactor -----------------------------------------------------------------------

#[test]
fn academy_manufactors_multiply_by_three_each() {
    cr!("614.5", "616.1", "111.10");
    ruling!(
        "Academy Manufactor",
        "If you control eighteen Academy Manufactors (I don't know, you figure it out) and would create some number of Clue, Food, or Treasure tokens, you will instead create 129,140,163 times that many Clue tokens, 129,140,163 times that many Food tokens, and 129,140,163 times that many Treasure tokens."
    );
    // With n Manufactors, 3^(n-1) of each kind (18: 3^17 = 129,140,163). Two and three here.
    for (n, each) in [(2usize, 3usize), (3, 9)] {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, "Academy Manufactor");
        }
        create_token(&mut t, P0, "Food");
        for kind in ["Clue", "Food", "Treasure"] {
            assert_eq!(
                tokens_with(&t, P0, kind).len(),
                each,
                "{n} Manufactors: {kind}"
            );
        }
    }
    assert_eq!(3u64.pow(17), 129_140_163);
}

// --- Doubling counters ------------------------------------------------------------------------

#[test]
fn doubling_counters_puts_that_many_more_through_replacement_effects() {
    cr!("701.10e", "614.1a", "122.6");
    ruling!(
        "Lily Bowen, Raging Grandma",
        "To double the number of +1/+1 counters on Lily Bowen, put a number of +1/+1 counters on it equal to the number it already has. Replacement effects that modify the number of counters being placed on creatures you control, such as the effect of Branching Evolution, apply to this ability as normal."
    );
    ruling!(
        "Sword of Hours",
        "To double the number of +1/+1 counters on a permanent, count the number of +1/+1 counters that are currently on that permanent and put that many more on it. Any replacement effects that change this number will apply accordingly."
    );
    supported("Lily Bowen, Raging Grandma");
    // Lily Bowen with two counters and Branching Evolution: 2 more, doubled to 4: six.
    let mut t = TestGame::new(2);
    let lily = t.battlefield(P0, "Lily Bowen, Raging Grandma");
    plus1(&mut t, lily, 2);
    t.battlefield(P0, "Branching Evolution");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(lily, counters::PLUS1), 6, "{}", t.dump_log());
    // Sword of Hours with Hardened Scales: the attack counter (1 + 1), a roll of 12, two
    // more plus one: five.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Sword of Hours");
    t.g.attach(sword, obj(bears));
    t.battlefield(P0, "Hardened Scales");
    t.g.dice.loaded.push_back(12);
    t.attack(&[(bears, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 5, "{}", t.dump_log());
}

// --- Doubling power ---------------------------------------------------------------------------

#[test]
fn doubling_power_uses_the_power_as_the_effect_begins() {
    cr!("701.10d", "608.2h", "611.2c");
    ruling!(
        "Neyith of the Dire Hunt",
        "If an effect instructs you to “double” a creature’s power, that creature gets +X/+0, where X is its power as that effect begins to apply. If its power is negative, instead it gets -X/-0 where X is how far below 0 its power is. The value of X won’t change if another effect alters the creature’s power later in the turn."
    );
    ruling!(
        "Overclocked Electromancer",
        "To double Overclocked Electromancer's power, it gets +X/+0, where X is its power when its second ability resolves."
    );
    ruling!(
        "The Skullspore Nexus",
        "To double a creature's power, that creature gets +X/+0, where X is that creature's power as The Skullspore Nexus's activated ability resolves."
    );
    // Neyith: a 2/2 at -1/-0 (power -1) gets -1/-0; a later +3/+0 doesn't change that.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neyith of the Dire Hunt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    pump(&mut t, bears, -3);
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bears), (-2, 2), "{}", t.dump_log());
    pump(&mut t, bears, 3);
    assert_eq!(t.pt(bears), (1, 2));
    // Neyith on a 3/3: +3/+0; later +2/+0 makes it 8, not 10.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neyith of the Dire Hunt");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 3));
    pump(&mut t, giant, 2);
    assert_eq!(t.pt(giant), (8, 3));
    // Overclocked Electromancer: a +1/+1 counter before its attack trigger resolves.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Overclocked Electromancer");
    attack_with(&mut t, &[(e, pl(P1))]);
    assert_eq!(triggers_of(&t, "Overclocked Electromancer").len(), 1);
    plus1(&mut t, e, 1);
    t.resolve_all();
    assert_eq!(t.pt(e), (6, 3), "{}", t.dump_log());
    // The Skullspore Nexus: +2/+0 in response.
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "The Skullspore Nexus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, nexus, "Double").expect("Nexus");
    pump(&mut t, bears, 2);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 2), "{}", t.dump_log());
}

// --- Megatron, Destructive Force --------------------------------------------------------------

#[test]
fn megatron_splits_the_damage_first_then_replacement_effects_modify_each_part() {
    cr!("120.4a", "614.1a", "701.28a");
    ruling!(
        "Megatron, Tyrant // Megatron, Destructive Force",
        "Once you've determined that there would be excess damage and how much damage is excess, Megatron simultaneously deals damage to the creature and to that creature's controller, then converts. The damage may be modified by replacement or prevention effects."
    );
    let mut t = TestGame::new(2);
    // Bitter Feud between P0 and P1 doubles Megatron's damage to each part.
    t.answer_choose(P0, &[pl(P0), pl(P1)]);
    t.enter(P0, "Bitter Feud");
    t.settle();
    let m = t.battlefield(P0, "Megatron, Tyrant // Megatron, Destructive Force");
    mtg_engine::dfc::transform(&mut t.g, m);
    t.g.dirty = true;
    let stone = t.battlefield(P0, "Mind Stone"); // mana value 2
    let elves = t.battlefield(P1, "Llanowar Elves"); // 1/1
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(stone)]);
    t.answer_targets(P0, &[obj(elves)]);
    attack_with(&mut t, &[(m, pl(P1))]);
    t.resolve_all();
    // 2 damage: 1 to the Elves and 1 excess to P1, each doubled: 2 and 2, at once.
    assert!(!t.on_battlefield(elves), "{}", t.dump_log());
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    let player_hits: Vec<i32> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage {
                    target: Entity::Player(p),
                    amount,
                    ..
                } if *p == P1 => Some(*amount as i32),
                _ => None,
            })
            .collect();
    assert_eq!(player_hits, vec![2]);
    assert_eq!(t.obj_now(m).face, FaceState::Front);
}

// --- Unpredictable Cyclone --------------------------------------------------------------------

#[test]
fn unpredictable_cyclone_casts_the_card_during_the_cycling_ability_ignoring_timing() {
    cr!("702.29a", "608.2g", "601.2");
    ruling!(
        "Unpredictable Cyclone",
        "If you wish to cast the last exiled card, you do so as part of the resolution of the cycling ability. You can’t wait to cast it later. Timing permissions based on a card’s type are ignored."
    );
    supported("Desert Cerodon");
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Unpredictable Cyclone");
        stack_library(&mut t, P0, &["Forest", "Hill Giant", "Island"]);
        let cerodon = t.hand(P0, "Desert Cerodon"); // creature, cycling {R}
                                                    // On P1's turn, during combat.
        t.set_step(P1, Step::DeclareAttackers);
        t.lands(P0, "Mountain", 1);
        t.answer_yes(P0, cast);
        t.g.turn.priority = Some(P0);
        activate_containing(&mut t, P0, cerodon, "Cycling").expect("cycle");
        t.resolve_all();
        assert_eq!(
            !t.named_on_battlefield("Hill Giant").is_empty(),
            cast,
            "cast {cast}: {}",
            t.dump_log()
        );
        if !cast {
            // On the bottom of the library: no permission to cast it later.
            let bottom = t.g.players[0].library[..2]
                .iter()
                .map(|id| t.obj_now(*id).chars.name.to_string())
                .collect::<Vec<_>>();
            assert!(bottom.contains(&"Hill Giant".to_string()), "{bottom:?}");
        }
    }
}

// --- Agitator Ant ------------------------------------------------------------------------------

#[test]
fn agitator_ant_goads_only_creatures_that_got_the_counters() {
    cr!("701.15a", "608.2d", "614.1a");
    interp!("counter-choice-excludes-what-cant-have-them");
    ruling!(
        "Agitator Ant",
        "If a creature can't have +1/+1 counters put on it, its controller can't choose it just to have it become goaded. If it can have counters put on it but replacement effects cause it not to receive +1/+1 counters, it doesn't become goaded."
    );
    supported("Agitator Ant");
    supported("Melira's Keepers");
    // Melira's Keepers can't have counters put on it: P1 can't choose it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agitator Ant");
    let keepers = t.battlefield(P1, "Melira's Keepers");
    t.advance_to(P0, Step::End);
    let from = t.asked().len();
    t.resolve_all();
    let offered_keepers = asked_since(&t, from).iter().any(|(p, d)| {
        *p == P1
            && matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.contains(&obj(keepers)))
    });
    assert!(
        !offered_keepers,
        "{}\n{:#?}",
        t.dump_log(),
        prompts_since(&t, from)
    );
    assert!(!goaded(&t, keepers));
    // Two "minus one" replacement effects leave none of the two counters: not goaded.
    let sink = || {
        custom_card(
            "Counter Sink",
            "Enchantment",
            "{1}",
            None,
            "If one or more +1/+1 counters would be put on a creature you control, that many +1/+1 counters minus one are put on it instead.",
        )
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agitator Ant");
    t.custom(P1, sink(), Zone::Battlefield);
    t.custom(P1, sink(), Zone::Battlefield);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 0, "{}", t.dump_log());
    assert!(!goaded(&t, bears));
    // One of them: one counter, goaded.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Agitator Ant");
    t.custom(P1, sink(), Zone::Battlefield);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1, "{}", t.dump_log());
    assert!(goaded(&t, bears));
}

// --- Unbound Flourishing ----------------------------------------------------------------------

#[test]
fn unbound_flourishing_doubles_x_before_or_after_a_cast_trigger_that_uses_it() {
    cr!("603.3b", "107.3m", "608.2h");
    ruling!(
        "Unbound Flourishing",
        "If a permanent has an ability that triggers when it's cast and that references the value of X, you choose whether to double the value of X before or after that ability resolves."
    );
    // Hydroid Krasis, X = 2: "When you cast this spell, you gain half X life and draw half
    // X cards, rounded down."
    let mut outcomes = vec![];
    for order in [vec![0usize, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Unbound Flourishing");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 2);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        let krasis = t.hand(P0, "Hydroid Krasis");
        let hand = t.hand_size(P0) - 1;
        t.cast(P0, krasis).x(2).go();
        t.settle();
        assert_eq!(t.stack_len(), 3, "{}", t.dump_log());
        let top = t.g.stack[2];
        let flourishing_first = matches!(
            &t.g.obj(top).stack.as_ref().unwrap().kind,
            mtg_engine::object::StackKind::Triggered { source, .. }
                if t.g.obj(*source).chars.name == "Unbound Flourishing"
        );
        t.resolve();
        t.resolve();
        let gained = t.life(P0) - 20;
        let drawn = t.hand_size(P0) - hand;
        outcomes.push((flourishing_first, gained, drawn));
        t.resolve_all();
        // The Krasis itself: a 4/4 either way (X doubled to 4).
        let k = t.named_on_battlefield("Hydroid Krasis");
        assert_eq!(k.len(), 1);
        assert_eq!(t.pt(k[0]), (4, 4));
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![(false, 1, 1), (true, 2, 2)]);
}
