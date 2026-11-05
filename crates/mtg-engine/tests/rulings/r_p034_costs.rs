//! Rulings batch P034 — counters removed (or put) as a cost (CR 118.3, 602.2b): which
//! counters can pay, what happens to the source afterwards, and abilities granted to
//! creatures that use their counters as fuel.

use crate::r_p034_common::*;
use crate::r_p038_common::{add, kill};
use crate::r_s01_common::{asked_since, supported, tokens};
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// Activates the activated ability of `src` whose text contains `needle`.
fn activate_text(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    needle: &str,
    targets: &[Entity],
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let i = t
        .obj_now(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains(needle))
        .unwrap_or_else(|| panic!("no activated ability with {needle:?}"));
    let src = t.g.current(src);
    t.activate(p, src, i, targets)
}

fn priority_asked(asked: &[(PlayerId, Decision)]) -> bool {
    asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. }))
}

#[test]
fn spinal_parasite_counters_and_its_ability() {
    cr!("122.1a", "118.3", "608.2d");
    ruling!(
        "Spinal Parasite",
        "Yes, this card’s base power/toughness is -1/-1. If it has five +1/+1 counters on it, it’s a 4/4 creature."
    );
    ruling!(
        "Spinal Parasite",
        "you can remove both counters to pay for its activated ability"
    );
    ruling!(
        "Spinal Parasite",
        "Any permanent is a legal target for Spinal Parasite’s ability, not just one with a counter on it."
    );
    supported("Spinal Parasite");
    let mut t = TestGame::new(2);
    let sp = t.battlefield(P0, "Spinal Parasite");
    put(&mut t, sp, counters::PLUS1, 5);
    assert_eq!(t.pt(sp), (4, 4));

    // A 1/1 with two counters removes both: it dies, and the ability still resolves,
    // removing the one charge counter of the target (its controller choosing which).
    let mut t = TestGame::new(2);
    let sp = t.battlefield(P0, "Spinal Parasite");
    put(&mut t, sp, counters::PLUS1, 2);
    assert_eq!(t.pt(sp), (1, 1));
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    put(&mut t, bears, "stun", 1);
    t.activate(P0, sp, 0, &[obj(bears)]).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Spinal Parasite"), "a -1/-1 creature");
    // P0 chooses the stun counter.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    let kinds_asked: Vec<_> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
        .collect();
    assert_eq!(kinds_asked.len(), 1);
    assert_eq!(kinds_asked[0].0, P0, "the Parasite's controller chooses");
    let Decision::ChooseOption { options, .. } = &kinds_asked[0].1 else {
        unreachable!()
    };
    // (Options read "+1/+1 counter", "stun counter".)
    let chosen = options[1].trim_end_matches(" counter");
    assert!(chosen == "stun" || chosen == counters::PLUS1, "{options:?}");
    let other = if chosen == "stun" {
        counters::PLUS1
    } else {
        "stun"
    };
    assert_eq!(t.counters(bears, chosen), 0, "the chosen kind was removed");
    assert_eq!(t.counters(bears, other), 1, "and only one counter");

    // A permanent without counters is a legal target.
    let mut t = TestGame::new(2);
    let sp = t.battlefield(P0, "Spinal Parasite");
    put(&mut t, sp, counters::PLUS1, 2);
    let land = t.battlefield(P1, "Forest");
    t.activate(P0, sp, 0, &[obj(land)]).unwrap();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(land));
}

#[test]
fn walking_ballista_with_damage_runs_out_of_toughness() {
    cr!("704.5g");
    ruling!(
        "Walking Ballista",
        "it will be destroyed immediately after you activate the ability a second time"
    );
    let mut t = TestGame::new(2);
    let wb = t.battlefield(P0, "Walking Ballista");
    put(&mut t, wb, counters::PLUS1, 3);
    t.g.deal_damage(wb, Entity::Object(wb), 1, false);
    t.g.flush_events();
    t.settle();
    assert!(t.on_battlefield(wb));
    t.activate(P0, wb, 1, &[Entity::Player(P1)]).unwrap();
    t.settle();
    assert!(t.on_battlefield(wb), "2/2 with 1 damage");
    t.activate(P0, wb, 1, &[Entity::Player(P1)]).unwrap();
    t.settle();
    assert!(!t.on_battlefield(wb), "1/1 with 1 damage: destroyed");
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "both activations still resolve");
}

#[test]
fn barkhide_troll_damage_can_become_lethal_after_removing_its_counter() {
    cr!("704.5g", "120.6");
    ruling!(
        "Barkhide Troll",
        "nonlethal damage dealt to Barkhide Troll may become lethal if you activate its last ability"
    );
    supported("Barkhide Troll");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Barkhide Troll");
    put(&mut t, troll, counters::PLUS1, 1);
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, shock).target(troll).go();
    t.resolve_all();
    assert!(t.on_battlefield(troll), "3/3 with 2 damage");
    pool(&mut t, P0, ManaType::C, 1);
    t.activate(P0, troll, 0, &[]).unwrap();
    t.settle();
    assert!(!t.on_battlefield(troll), "2/2 with 2 damage");
}

#[test]
fn removing_plus1_counters_from_among_your_creatures_as_a_cost() {
    cr!("118.3", "602.2b");
    ruling!(
        "Novijen Sages",
        "either remove two +1/+1 counters from a single creature you control or remove one +1/+1 counter from each of two different creatures"
    );
    ruling!(
        "Bolrac-Clan Crusher",
        "you can't activate it without removing a +1/+1 counter, and you can't remove additional counters"
    );
    supported("Novijen Sages");
    supported("Bolrac-Clan Crusher");
    // Novijen Sages: two from one creature, or one from each of two.
    for split in [false, true] {
        let mut t = TestGame::new(2);
        let ns = t.battlefield(P0, "Novijen Sages");
        let bears = t.battlefield(P0, "Grizzly Bears");
        put(&mut t, ns, counters::PLUS1, 4);
        put(&mut t, bears, counters::PLUS1, 2);
        pool(&mut t, P0, ManaType::C, 1);
        t.answer_choose(P0, &[obj(bears)]);
        t.answer_choose(P0, &[obj(if split { ns } else { bears })]);
        let hand = t.hand_size(P0);
        t.activate(P0, ns, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1);
        if split {
            assert_eq!((plus1(&t, ns), plus1(&t, bears)), (3, 1));
        } else {
            assert_eq!((plus1(&t, ns), plus1(&t, bears)), (4, 0));
        }
    }
    // Bolrac-Clan Crusher: exactly one counter, from any creature you control; none
    // means it can't be activated.
    let mut t = TestGame::new(2);
    let bc = t.battlefield(P0, "Bolrac-Clan Crusher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.activate(P0, bc, 0, &[Entity::Player(P1)]).is_err());
    assert!(!t.obj_now(bc).tapped);
    put(&mut t, bears, counters::PLUS1, 3);
    t.activate(P0, bc, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn deathbringer_thoctar_can_remove_any_plus1_counter() {
    cr!("118.3", "122.1");
    ruling!(
        "Deathbringer Thoctar",
        "You may remove any +1/+1 counter from Deathbringer Thoctar to activate its second ability"
    );
    supported("Deathbringer Thoctar");
    let mut t = TestGame::new(2);
    let dt = t.battlefield(P0, "Deathbringer Thoctar");
    give(&mut t, P0, dt);
    assert_eq!(plus1(&t, dt), 3);
    for _ in 0..3 {
        t.activate(P0, dt, 0, &[Entity::Player(P1)]).unwrap();
    }
    assert!(t.activate(P0, dt, 0, &[Entity::Player(P1)]).is_err());
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn retribution_of_the_ancients_costs_are_paid_without_anyone_getting_priority() {
    cr!("602.2b", "601.2h", "107.3a");
    ruling!(
        "Retribution of the Ancients",
        "Once you announce that you are activating the ability of Retribution of the Ancients, no player may take any actions"
    );
    supported("Retribution of the Ancients");
    let mut t = TestGame::new(2);
    let rota = t.battlefield(P0, "Retribution of the Ancients");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    pool(&mut t, P0, ManaType::B, 1);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, rota, 0, &[obj(giant)]).unwrap();
    assert!(!priority_asked(&asked_since(&t, from)));
    assert_eq!(plus1(&t, bears), 0);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
}

#[test]
fn ramos_cant_pay_for_a_spell_with_the_counters_it_gets_from_it() {
    cr!("601.2i", "603.3", "605.3a");
    ruling!(
        "Ramos, Dragon Engine",
        "there's no way to add counters with a spell and pay for that spell with the newly-added counters"
    );
    supported("Ramos, Dragon Engine");
    let mut t = TestGame::new(2);
    let ramos = t.battlefield(P0, "Ramos, Dragon Engine");
    put(&mut t, ramos, counters::PLUS1, 4);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t
        .cast(P0, bolt)
        .target(Entity::Player(P1))
        .try_go()
        .is_err());
    assert_eq!(plus1(&t, ramos), 4);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // Cast with a Mountain: the trigger gives the fifth counter only after casting.
    t.lands(P0, "Mountain", 1);
    t.clear_answers();
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert_eq!(plus1(&t, ramos), 4);
    t.resolve_all();
    assert_eq!(plus1(&t, ramos), 5);
}

#[test]
fn surge_node_can_target_itself() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Surge Node",
        "the ability will remove a counter from Surge Node and then put one onto it"
    );
    supported("Surge Node");
    let mut t = TestGame::new(2);
    let node = t.battlefield(P0, "Surge Node");
    put(&mut t, node, counters::CHARGE, 1);
    pool(&mut t, P0, ManaType::C, 1);
    t.activate(P0, node, 0, &[obj(node)]).unwrap();
    assert_eq!(t.counters(node, counters::CHARGE), 0, "removed as the cost");
    t.resolve_all();
    assert_eq!(t.counters(node, counters::CHARGE), 1);
}

#[test]
fn umezawas_jitte_mode_is_chosen_on_activation() {
    cr!("700.2a");
    ruling!(
        "Umezawa's Jitte",
        "The choice is made when the ability is activated."
    );
    supported("Umezawa's Jitte");
    let mut t = TestGame::new(2);
    let jitte = t.battlefield(P0, "Umezawa's Jitte");
    put(&mut t, jitte, counters::CHARGE, 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    let ab = t.activate(P0, jitte, 0, &[]).unwrap().unwrap();
    let si = t.obj_now(ab).stack.clone().unwrap();
    assert_eq!(
        si.chosen.iter().map(|c| c.mode).collect::<Vec<_>>(),
        vec![Some(2)]
    );
    let from = t.asked().len();
    t.resolve_all();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn sporoloth_ancient_grants_its_ability_to_every_creature_you_control() {
    cr!("613.1f");
    ruling!(
        "Sporoloth Ancient",
        "they would lose the bonus from Muraganda Petroglyphs"
    );
    ruling!(
        "Sporoloth Ancient",
        "can create a Saproling by removing two spore counters (by using this ability) or by removing three"
    );
    supported("Sporoloth Ancient");
    supported("Muraganda Petroglyphs");
    supported("Thallid");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Muraganda Petroglyphs");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (4, 4));
    t.battlefield(P0, "Sporoloth Ancient");
    assert_eq!(t.pt(bears), (2, 2), "it has an ability now");
    // An opponent's creature doesn't get it.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(theirs), (4, 4));
    // Thallid with three spore counters: either ability.
    let thallid = t.battlefield(P0, "Thallid");
    put(&mut t, thallid, "spore", 3);
    activate_text(&mut t, P0, thallid, "two spore", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(thallid, "spore"), 1);
    assert!(activate_text(&mut t, P0, thallid, "two spore", &[]).is_err());
    put(&mut t, thallid, "spore", 2);
    activate_text(&mut t, P0, thallid, "three spore", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(thallid, "spore"), 0);
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn magmatic_core_with_no_age_counters_has_no_targets() {
    cr!("601.2c", "601.2d", "603.3d");
    ruling!(
        "Magmatic Core",
        "If Magmatic Core has no age counters on it, the only number of targets the ability can have is zero."
    );
    supported("Magmatic Core");
    let mut t = TestGame::new(2);
    let core = add(&mut t, P0, "Magmatic Core");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(top_is_trigger(&t));
    let top = *t.g.stack.last().unwrap();
    let si = t.obj_now(top).stack.clone().unwrap();
    assert!(si
        .chosen
        .iter()
        .all(|c| c.targets.iter().all(|s| s.is_empty())));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.counters(core, "age"), 0);
}

#[test]
fn empowered_autogenerator_is_a_mana_ability() {
    cr!("605.1a", "605.3b");
    ruling!(
        "Empowered Autogenerator",
        "Empowered Autogenerator's activated ability is a mana ability. It doesn't use the stack"
    );
    supported("Empowered Autogenerator");
    let mut t = TestGame::new(2);
    let ea = t.battlefield(P0, "Empowered Autogenerator");
    put(&mut t, ea, counters::CHARGE, 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let r = t.activate(P0, ea, 0, &[]).unwrap();
    assert!(r.is_none(), "nothing put on the stack");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(ea, counters::CHARGE), 3);
    assert_eq!(t.g.player(P0).mana_pool.total(), 3);
}

/// A Junk token for `p`, from Junk Jet's enters trigger. Returns (Junk Jet, the token).
fn junk(t: &mut TestGame, p: PlayerId) -> (ObjectId, ObjectId) {
    supported("Junk Jet");
    let jj = crate::r_p038_common::enter(t, p, "Junk Jet");
    t.resolve_all();
    let toks = tokens(t, p);
    assert_eq!(toks.len(), 1);
    (jj, toks[0])
}

#[test]
fn a_junk_token_cant_pay_two_costs() {
    cr!("701.21a", "118.3");
    ruling!(
        "Junk Jet",
        "you can't sacrifice a Junk token to activate its own ability and also to activate the second ability of Junk Jet"
    );
    let mut t = TestGame::new(2);
    let (jj, tok) = junk(&mut t, P0);
    t.activate(P0, tok, 0, &[]).unwrap();
    pool(&mut t, P0, ManaType::C, 3);
    assert!(
        t.activate(P0, jj, 0, &[]).is_err(),
        "no other artifact left to sacrifice"
    );
    t.resolve_all();
    // With a second artifact, Junk Jet's ability can be activated.
    t.battlefield(P0, "Ornithopter");
    t.activate(P0, jj, 0, &[]).unwrap();
}

#[test]
fn a_card_played_with_a_junk_token_follows_the_normal_timing_and_costs() {
    cr!("305.1", "305.2", "601.2f");
    ruling!(
        "Junk Jet",
        "You pay all costs and follow all normal timing rules for the card played from exile with a Junk token's ability."
    );
    // A land: only in a main phase with an empty stack, and as the turn's land play.
    let mut t = TestGame::new(2);
    let (_, tok) = junk(&mut t, P0);
    let forest = t.library_top(P0, "Forest");
    t.activate(P0, tok, 0, &[]).unwrap();
    t.resolve_all();
    let forest = t.g.current(forest);
    assert_eq!(t.zone(forest), mtg_engine::object::Zone::Exile);
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    assert!(t.play_land(P0, forest).is_err(), "the stack isn't empty");
    t.resolve_all();
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest));
    // A spell: its mana cost is paid.
    let mut t = TestGame::new(2);
    let (_, tok) = junk(&mut t, P0);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, tok, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.cast(P0, bears).try_go().is_err(), "no mana");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // A land after the turn's land play: no.
    let mut t = TestGame::new(2);
    let (_, tok) = junk(&mut t, P0);
    let first = t.hand(P0, "Plains");
    t.play_land(P0, first).unwrap();
    let forest = t.library_top(P0, "Forest");
    t.activate(P0, tok, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.play_land(P0, t.g.current(forest)).is_err());
}

#[test]
fn junk_creating_spells_with_illegal_targets_create_no_junk() {
    cr!("608.2b");
    ruling!(
        "Break Down",
        "If each target chosen is illegal as that spell or ability tries to resolve, it won't resolve"
    );
    ruling!(
        "Commander Sofia Daguerre",
        "If each target chosen is illegal as that spell or ability tries to resolve, it won't resolve"
    );
    supported("Break Down");
    supported("Commander Sofia Daguerre");
    let mut t = TestGame::new(2);
    let orni = t.battlefield(P1, "Ornithopter");
    cast_card(&mut t, P0, "Break Down", &[obj(orni)]);
    kill(&mut t, orni);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert!(tokens(&t, P1).is_empty());
    // Commander Sofia Daguerre's "up to one target legendary permanent".
    let mut t = TestGame::new(2);
    let ramos = t.battlefield(P1, "Ramos, Dragon Engine");
    t.answer_targets(P0, &[obj(ramos)]);
    crate::r_p038_common::enter(&mut t, P0, "Commander Sofia Daguerre");
    assert!(top_is_trigger(&t));
    kill(&mut t, ramos);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty() && tokens(&t, P1).is_empty());
    // With the target still there, its controller gets the Junk.
    let mut t = TestGame::new(2);
    let ramos = t.battlefield(P1, "Ramos, Dragon Engine");
    t.answer_targets(P0, &[obj(ramos)]);
    crate::r_p038_common::enter(&mut t, P0, "Commander Sofia Daguerre");
    t.resolve_all();
    assert!(!t.on_battlefield(ramos));
    assert_eq!(tokens(&t, P1).len(), 1);
}

#[test]
fn camp_creates_no_junk_if_its_target_is_illegal() {
    // C.A.M.P.: "Whenever fortified land is tapped for mana, put a +1/+1 counter on target
    // creature you control. If that creature shares a color with the mana that land
    // produced, create a Junk token."
    cr!("608.2b");
    ruling!(
        "C.A.M.P.",
        "If each target chosen is illegal as that spell or ability tries to resolve, it won't resolve"
    );
    supported("C.A.M.P.");
    // (target, destroyed in response, Junk tokens): a green creature for the Forest's {G}
    // gets the counter and makes a Junk; a colorless one only gets the counter; a green
    // one destroyed in response, nothing.
    for (name, gone, junk) in [
        ("Llanowar Elves", false, 1),
        ("Ornithopter", false, 0),
        ("Llanowar Elves", true, 0),
    ] {
        let mut t = TestGame::new(2);
        let camp = t.battlefield(P0, "C.A.M.P.");
        let forest = t.battlefield(P0, "Forest");
        t.g.attach(camp, obj(forest));
        t.g.recompute();
        let c = t.battlefield(P0, name);
        t.answer_targets(P0, &[obj(c)]);
        t.activate(P0, forest, 0, &[]).unwrap();
        t.settle();
        assert!(top_is_trigger(&t), "{name}");
        if gone {
            kill(&mut t, c);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), junk, "{name} gone: {gone}");
        if !gone {
            assert_eq!(plus1(&t, c), 1, "{name}");
        }
    }
}
