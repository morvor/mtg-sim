//! Rulings residue P129 and P130 (item rulings-residue-reflexive-refund), part 2: what the
//! reflexive triggered abilities do as they resolve: counters chosen, distributed and
//! counted with last known information, Equipment attached (or not) and ordered with other
//! triggers, Aura tokens copying only copiable values, replacement effects that outlast
//! their reminder counters, and cards named as the ability resolves.

use crate::r_p129_common::*;
use crate::r_residue_reflexive_refund::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Wraps `p`'s agent so that, asked to order triggered abilities, it puts the ones whose
/// text contains `top` on top of the stack (last); every other decision is scripted.
pub fn order_on_top(t: &mut TestGame, p: PlayerId, top: &'static str) {
    struct Orderer {
        inner: Box<dyn mtg_engine::decision::Agent>,
        top: &'static str,
    }
    impl mtg_engine::decision::Agent for Orderer {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            if let Decision::Order { items, .. } = d {
                let (mut v, last): (Vec<usize>, Vec<usize>) =
                    (0..items.len()).partition(|i| !items[*i].contains(self.top));
                v.extend(last);
                return Answer::Indices(v);
            }
            self.inner.decide(g, p, d)
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Orderer { inner, top });
}

#[test]
fn etched_slith_targets_as_the_reflexive_ability_goes_on_the_stack_and_chooses_the_counter() {
    cr!("603.12", "122.8");
    ruling!(
        "Etched Slith",
        "You don't choose a target for Etched Slith's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you put a +1/+1 counter on Etched Slith this way."
    );
    ruling!(
        "Etched Slith",
        "If there's more than one type of counter on the targeted permanent or opponent, Etched Slith's controller chooses which counter to remove."
    );
    let mut t = TestGame::new(2);
    let slith = t.battlefield(P0, "Etched Slith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    put_counters(&mut t, bears, "stun", 1);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_p031_common::hit(&mut t, slith);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert_eq!(t.counters(slith, counters::PLUS1), 1);
    // P0 is asked which kind to remove; each answer removes that kind.
    let at = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    let opts: Vec<String> = t.asked()[at..]
        .iter()
        .find_map(|(q, d)| match d {
            Decision::ChooseOption { options, .. } if *q == P0 => Some(options.clone()),
            _ => None,
        })
        .expect("P0 chose the kind of counter");
    assert_eq!(opts.len(), 2, "{opts:?}");
    assert_eq!(
        t.counters(bears, counters::PLUS1) + t.counters(bears, "stun"),
        1
    );
    for (i, opt) in opts.iter().enumerate() {
        let mut t = TestGame::new(2);
        let slith = t.battlefield(P0, "Etched Slith");
        let bears = t.battlefield(P1, "Grizzly Bears");
        put_counters(&mut t, bears, counters::PLUS1, 1);
        put_counters(&mut t, bears, "stun", 1);
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_yes(P0, true);
        t.answer(P0, DecisionKind::Option, Answer::Index(i));
        crate::r_p031_common::hit(&mut t, slith);
        t.resolve_all();
        let left = (
            t.counters(bears, counters::PLUS1),
            t.counters(bears, "stun"),
        );
        let expect = if opt.to_lowercase().contains("stun") {
            (1, 0)
        } else {
            (0, 1)
        };
        assert_eq!(left, expect, "{opt}");
    }
}

#[test]
fn ratonhnhaketon_targets_after_the_token_is_created() {
    cr!("603.12", "301.5c", "701.3b");
    ruling!(
        "Ratonhnhaké꞉ton",
        "You don't choose a target for Ratonhnhaké:ton's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you create an Assassin token this way."
    );
    ruling!(
        "Ratonhnhaké꞉ton",
        "You may target an Equipment card with the reflexive triggered ability that can't legally be attached to the Assassin token. If you do, it will enter the battlefield unattached."
    );
    supported("Ratonhnhaké꞉ton");
    supported("Konda's Banner");
    for (equipment, attached) in [("Short Sword", true), ("Konda's Banner", false)] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Ratonhnhaké꞉ton");
        let eq = t.graveyard(P0, equipment);
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(eq)]);
        crate::r_p031_common::hit(&mut t, r);
        untargeted_then_reflexive(&mut t, P0, from, &[obj(eq)]);
        let assassin = tokens(&t, P0);
        assert_eq!(
            assassin.len(),
            1,
            "the token exists before the target is chosen"
        );
        assert!(is_in(&t, eq, Zone::Graveyard(P0)));
        t.resolve_all();
        assert!(t.on_battlefield(eq), "{equipment}");
        let at = t.obj_now(eq).attached_to;
        if attached {
            assert_eq!(at, Some(obj(assassin[0])), "{equipment}");
        } else {
            assert_eq!(at, None, "{equipment} enters unattached");
        }
    }
}

#[test]
fn yannik_chooses_the_creature_on_resolution_then_distributes_by_its_last_known_power() {
    cr!("603.12", "601.2d", "608.2h", "115.1d");
    ruling!(
        "Yannik, Scavenging Sentinel",
        "Yannik's triggered ability doesn't target the creature you control. That creature is chosen as the triggered ability resolves. After it's exiled, the reflexive triggered ability triggers and targets are chosen to receive the counters."
    );
    ruling!(
        "Yannik, Scavenging Sentinel",
        "Use the power of the creature as it last existed on the battlefield to determine how many +1/+1 counters to distribute."
    );
    ruling!(
        "Yannik, Scavenging Sentinel",
        "You choose how the counters will be distributed as you put the reflexive triggered ability onto the stack. Each target creature must be assigned at least one counter."
    );
    ruling!(
        "Yannik, Scavenging Sentinel",
        "Yannik can be the target of its own reflexive triggered ability."
    );
    // Grizzly Bears with two +1/+1 counters (4/4) is exiled: 4 counters, split 3 on
    // Yannik and 1 on Savannah Lions.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let yannik = t.enter(P0, "Yannik, Scavenging Sentinel");
    // (Its partner-with trigger targets a player; it resolves first.)
    t.answer_targets(P0, &[Entity::Player(P0)]);
    order_on_top(&mut t, P0, "Partner");
    t.settle();
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "exile another creature"),
        1
    );
    t.resolve();
    let first = top(&t);
    assert!(crate::r_s01_common::triggers_on_stack(&t, "exile another creature") == 1);
    assert!(
        targets_of(&t, first).is_empty(),
        "no target for the creature exiled"
    );
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(yannik), obj(lions)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 1]));
    resolve_into_reflexive(&mut t, &[obj(yannik), obj(lions)]);
    // The creature was chosen (not targeted) as the ability resolved.
    let chose = first_asked(&t, P0, from, DecisionKind::Entities).unwrap();
    assert!(first_asked(&t, P0, from, DecisionKind::Targets).unwrap() > chose);
    assert!(is_in(&t, bears, Zone::Exile));
    let divide: Vec<(u32, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total, min_each, ..
            } => Some((*total, *min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(divide, vec![(4, 1)]);
    assert_eq!(t.counters(yannik, counters::PLUS1), 0);
    t.resolve_all();
    assert_eq!(t.counters(yannik, counters::PLUS1), 3);
    assert_eq!(t.counters(lions, counters::PLUS1), 1);
}

#[test]
fn yannik_may_choose_zero_targets() {
    cr!("603.12", "115.1d");
    ruling!(
        "Yannik, Scavenging Sentinel",
        "You may choose zero target creatures for Yannik's reflexive triggered ability. If you do, no creature receives +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    let yannik = t.enter(P0, "Yannik, Scavenging Sentinel");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    order_on_top(&mut t, P0, "Partner");
    t.settle();
    t.resolve();
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[]);
    t.resolve_all();
    assert!(is_in(&t, bears, Zone::Exile));
    for c in [yannik, lions] {
        assert_eq!(t.counters(c, counters::PLUS1), 0);
    }
}

#[test]
fn gilgamesh_reflexive_and_job_select_are_ordered_by_you() {
    cr!("603.3b", "603.12", "702.182a");
    ruling!(
        "Gilgamesh, Master-at-Arms",
        "If you put an Equipment with job select onto the battlefield with Gilgamesh's ability, the job select ability and the reflexive triggered ability will both trigger, and you'll choose the order in which those abilities go on the stack."
    );
    supported("Gilgamesh, Master-at-Arms");
    supported("Samurai's Katana");
    let mut t = TestGame::new(2);
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Samurai's Katana",
            "Island",
            "Island",
            "Island",
            "Island",
            "Island",
        ],
    );
    let katana = lib[0];
    t.answer_choose(P0, &[obj(katana)]);
    order_on_top(&mut t, P0, "Samurai you control");
    t.answer_yes(P0, true);
    let gil = t.enter(P0, "Gilgamesh, Master-at-Arms");
    t.settle();
    t.resolve();
    assert!(t.on_battlefield(katana));
    assert_eq!(t.stack_len(), 2, "job select and the reflexive ability");
    // The reflexive ability resolves first: the Katana is attached to Gilgamesh.
    t.answer_choose(P0, &[obj(gil)]);
    t.resolve();
    assert_eq!(t.obj_now(katana).attached_to, Some(obj(gil)));
    // Then job select creates a Hero and attaches the Katana to it anyway.
    t.resolve_all();
    let heroes = with_subtype(&t, P0, "Hero");
    assert_eq!(heroes.len(), 1);
    assert_eq!(t.obj_now(katana).attached_to, Some(obj(heroes[0])));
}

#[test]
fn gilgamesh_cant_attach_equipment_that_cant_legally_be_attached() {
    cr!("301.5c", "701.3b");
    ruling!(
        "Gilgamesh, Master-at-Arms",
        "You can't use the reflexive triggered ability to try to attach an Equipment to a creature if that Equipment can't legally be attached to that creature."
    );
    let mut t = TestGame::new(2);
    let retainer = t.battlefield(P0, "Devoted Retainer");
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Konda's Banner",
            "Island",
            "Island",
            "Island",
            "Island",
            "Island",
        ],
    );
    let banner = lib[0];
    t.answer_choose(P0, &[obj(banner)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(retainer)]);
    t.answer_choose(P0, &[obj(retainer)]);
    t.enter(P0, "Gilgamesh, Master-at-Arms");
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(banner));
    assert_ne!(
        t.obj_now(banner).attached_to,
        Some(obj(retainer)),
        "Konda's Banner can be attached only to a legendary creature"
    );
    // (Positive control: to Gilgamesh, a legendary Samurai, it can be attached.)
    let mut t = TestGame::new(2);
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Konda's Banner",
            "Island",
            "Island",
            "Island",
            "Island",
            "Island",
        ],
    );
    let banner = lib[0];
    let gil = t.enter(P0, "Gilgamesh, Master-at-Arms");
    t.answer_choose(P0, &[obj(banner)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(gil)]);
    t.answer_choose(P0, &[obj(gil)]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(banner).attached_to, Some(obj(gil)));
}

#[test]
fn three_dog_aura_tokens_copy_only_copiable_values() {
    cr!("707.2", "707.4", "107.3b");
    ruling!(
        "Three Dog, Galaxy News DJ",
        "If the copied Aura had {X} in its mana cost, X is 0."
    );
    ruling!(
        "Three Dog, Galaxy News DJ",
        "The tokens copy exactly what was printed on the original Aura and nothing else (unless that Aura was itself copying something else; see below). It doesn't copy whether that Aura was tapped or untapped, whether it had any counters on it or Auras attached to it, or any non-copy effects that changed its types, color, and so on."
    );
    supported("Endless Scream");
    let mut t = TestGame::new(2);
    let td = t.battlefield(P0, "Three Dog, Galaxy News DJ");
    let scream = attach_new(&mut t, P0, "Endless Scream", obj(td));
    put_counters(&mut t, scream, "scream", 3);
    let s = t.g.current(scream);
    t.g.obj_mut(s).tapped = true;
    // A non-copy effect makes the Aura green until end of turn.
    let lens = t.battlefield(P0, "Distorting Lens");
    t.answer_targets(P0, &[obj(scream)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    activate_containing(&mut t, P0, lens, "color").unwrap();
    t.resolve_all();
    let before = t.obj_now(scream).chars.colors;
    assert_ne!(
        before,
        card_colors("Endless Scream"),
        "the Lens changed its color"
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[(td, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    let toks: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.name == "Endless Scream")
            .map(|o| o.id)
            .collect();
    assert_eq!(toks.len(), 1);
    let tok = t.obj_now(toks[0]);
    assert_eq!(tok.attached_to, Some(obj(bears)));
    assert_eq!(tok.chars.mana_value(), 1, "X is 0");
    assert!(!tok.tapped);
    assert_eq!(t.counters(toks[0], "scream"), 0);
    assert_eq!(tok.chars.colors, card_colors("Endless Scream"));
    assert_eq!(t.pt(bears), (2, 2), "no scream counters: +0/+0");
}

/// The colors of the real card `name`.
fn card_colors(name: &str) -> ColorSet {
    mtg_engine::card::card(name).front().chars.colors
}

#[test]
fn isareth_exile_replacement_outlasts_the_corpse_counter() {
    cr!("614.1a", "122.1");
    ruling!(
        "Isareth the Awakener",
        "If you somehow remove a corpse counter from a creature Isareth has returned to the battlefield, the replacement effect that will exile it continues to apply."
    );
    let mut t = TestGame::new(2);
    let is = t.battlefield(P0, "Isareth the Awakener");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(is, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "corpse"), 1);
    let b = t.g.current(bears);
    t.g.remove_counters(Entity::Object(b), "corpse", 1);
    t.g.recompute();
    assert_eq!(t.counters(bears, "corpse"), 0);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(is_in(&t, bears, Zone::Exile));
}

#[test]
fn bloodthirsty_adversary_originals_stay_in_exile() {
    cr!("707.12", "406.3");
    ruling!(
        "Bloodthirsty Adversary",
        "Whether or not you cast the copies this way, the original cards will remain in exile."
    );
    for cast in [true, false] {
        let mut t = TestGame::new(2);
        let bolt = t.graveyard(P0, "Lightning Bolt");
        t.lands(P0, "Mountain", 3);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(bolt)]);
        t.answer_yes(P0, cast);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.enter(P0, "Bloodthirsty Adversary");
        t.settle();
        t.resolve_all();
        assert_eq!(t.life(P1), if cast { 17 } else { 20 });
        assert!(is_in(&t, bolt, Zone::Exile), "cast: {cast}");
        // It stays there: it can't be cast from exile later.
        assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
        t.advance_to(P1, Step::Upkeep);
        assert!(is_in(&t, bolt, Zone::Exile));
    }
}

#[test]
fn cabal_therapist_names_the_card_as_the_reflexive_ability_resolves() {
    cr!("603.12", "608.2", "201.3");
    ruling!(
        "Cabal Therapist",
        "You name the card as Cabal Therapist’s reflexive triggered ability resolves, not as it’s put onto the stack. Then the targeted player reveals their hand and discards if appropriate. No player may choose to take actions between you naming the card and the targeted player revealing and discarding."
    );
    supported("Cabal Therapist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    t.battlefield(P0, "Cabal Therapist");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1)]);
    assert!(!t.on_battlefield(thopter));
    assert!(
        first_asked(&t, P0, from, DecisionKind::Name).is_none(),
        "no name yet"
    );
    let at = t.asked().len();
    t.resolve();
    // The name was chosen, then the discard happened in the same resolution: no player
    // got priority in between.
    let asked = t.asked();
    let after: Vec<&Decision> = asked[at..].iter().map(|(_, d)| d).collect();
    let named = after
        .iter()
        .position(|d| matches!(d, Decision::NameCard { .. }))
        .expect("named on resolution");
    assert!(!after[named..]
        .iter()
        .any(|d| matches!(d, Decision::Priority { .. })));
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn the_rise_of_sozin_targets_on_trigger_and_names_on_resolution() {
    cr!("603.3d", "608.2", "714.2b");
    ruling!(
        "The Rise of Sozin // Fire Lord Sozin",
        "You target an opponent when The Rise of Sozin's second chapter ability is put on the stack, but you don't name a card until it resolves. Notably, this means that players don't get to respond to the ability knowing what card you will name."
    );
    ruling!(
        "The Rise of Sozin // Fire Lord Sozin",
        "You don't have to choose the name of a card that you've already seen this game; you may guess at a card you think your opponent might have in their hand or library if you'd like."
    );
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Rise of Sozin // Fire Lord Sozin");
    put_counters(&mut t, saga, counters::LORE, 1);
    // Lightning Bolt is only in P1's library, unseen.
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.library_top(P1, "Island");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(
        P0,
        DecisionKind::Name,
        Answer::Text("Lightning Bolt".into()),
    );
    let s = t.g.current(saga);
    t.g.add_counters(Entity::Object(s), counters::LORE, 1, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(targets_of(&t, top(&t)), vec![Entity::Player(P1)]);
    assert!(first_asked(&t, P0, from, DecisionKind::Name).is_none());
    t.resolve_all();
    assert!(first_asked(&t, P0, from, DecisionKind::Name).is_some());
    assert!(is_in(&t, bolt, Zone::Exile));
}

#[test]
fn priority_boarding_reveals_for_the_roll_you_choose() {
    cr!("706.2", "603.2c", "603.12");
    ruling!(
        "Priority Boarding",
        "If you roll several dice at the same time, that will trigger the ability once for each roll. You can choose which result you want to reveal a card for."
    );
    supported("Priority Boarding");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Priority Boarding");
    t.library_top(P0, "Hill Giant");
    let src = t.battlefield(P0, "Memnite");
    t.g.dice.loaded.push_back(6);
    t.g.dice.loaded.push_back(2);
    let mut ctx = mtg_engine::eval::Ctx::new(Some(src), P0);
    let mut roll = mtg_engine::dice::DieRoll::new(6);
    roll.count = mtg_engine::ability::Value::c(2);
    mtg_engine::dice::roll(&mut t.g, &roll, &mut ctx);
    t.g.flush_events();
    t.settle();
    let triggers = crate::r_s01_common::triggers_on_stack(&t, "Whenever you roll a die");
    assert_eq!(triggers, 2, "one trigger for each die");
    // Resolve them one by one: decline the 2, reveal for the 6.
    let mut revealed = false;
    while t.stack_len() > 0 {
        let top_id = top(&t);
        let si = t.g.obj(top_id).stack.as_deref().expect("on the stack");
        let roll_trigger = matches!(&si.kind,
            mtg_engine::object::StackKind::Triggered { ability, .. }
                if ability.text.contains("roll a die"));
        let result = si.event.as_ref().map(|e| e.amount).unwrap_or(0);
        if roll_trigger {
            let yes = result == 6;
            revealed |= yes;
            t.answer_yes(P0, yes);
            if yes {
                t.answer_yes(P0, true);
            }
        }
        t.resolve();
    }
    assert!(revealed);
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn priority_boarding_doesnt_change_when_you_can_play_the_card() {
    cr!("601.3", "305.2", "307.1");
    ruling!(
        "Priority Boarding",
        "Priority Boarding doesn’t change when you can play the exiled card. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    // A sorcery: not while something is on the stack, then yes.
    let roll = |t: &mut TestGame, result: u32| {
        let src = t.battlefield(P0, "Memnite");
        let mut ctx = mtg_engine::eval::Ctx::new(Some(src), P0);
        t.g.dice.loaded.push_back(result);
        mtg_engine::dice::roll(&mut t.g, &mtg_engine::dice::DieRoll::new(20), &mut ctx);
        t.g.flush_events();
        t.settle();
    };
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Priority Boarding");
    let spike = t.library_top(P0, "Lava Spike");
    t.lands(P0, "Mountain", 2);
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    roll(&mut t, 5);
    t.resolve_all();
    assert!(is_in(&t, spike, Zone::Exile));
    let giant = t.hand(P0, "Grizzly Bears");
    let _ = giant;
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(
        !can_cast(&mut t, P0, spike, CastMethod::Normal),
        "stack not empty"
    );
    t.resolve_all();
    assert!(can_cast(&mut t, P0, spike, CastMethod::Normal));
    // During combat: no.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(
        !can_cast(&mut t, P0, spike, CastMethod::Normal),
        "not a main phase"
    );
    // A land: only with a land play remaining.
    for played in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Priority Boarding");
        let forest = t.library_top(P0, "Forest");
        if played {
            let land = t.hand(P0, "Island");
            t.play_land(P0, land).unwrap();
        }
        for _ in 0..3 {
            t.answer_yes(P0, true);
        }
        roll(&mut t, 5);
        t.resolve_all();
        assert!(is_in(&t, forest, Zone::Exile));
        assert_eq!(can_play_land(&mut t, P0, forest), !played);
    }
}
