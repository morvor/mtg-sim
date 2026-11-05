//! Rulings batch P192 — triggered abilities of cards with unusual mana costs: cast
//! triggers (Demigod of Revenge, Gluttonous Hellkite, CR 603.6/601.2i), "if you cast it"
//! (Zacama), damage triggers (Deus of Calamity), intervening "if" (Wardens of the Cycle,
//! CR 603.4), graveyard triggers that can be responded to (Worldspine Wurm), "you may ...
//! if you do" (Iname as One), "up to one target" (Tolsimir), values locked in on
//! resolution (Eldrazi Linebreaker, Stirring Honormancer, Defenders of Humanity, CR
//! 608.2h), attack triggers (The Ur-Dragon, Tower Above, Giant Ambush Beetle) and
//! distributing counters (Broodlord, CR 601.2d).

use crate::r_p194_common::{damage, end_step, mana};
use crate::r_s01_common::*;
use crate::r_s04_common::stack_items;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use crate::r_s29_common::divide;
use mtg_engine::combat;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn named_on_bf(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .count()
}

// ---------------------------------------------------------------- cast triggers

#[test]
fn demigod_of_revenge_cast_trigger_resolves_first_and_needs_a_cast() {
    cr!("603.6a", "601.2i", "603.3");
    ruling!(
        "Demigod of Revenge",
        "The triggered ability will resolve before Demigod of Revenge will."
    );
    ruling!(
        "Demigod of Revenge",
        "This ability triggers when you cast Demigod of Revenge as a spell. It won’t trigger if Demigod of Revenge is put directly onto the battlefield."
    );
    supported("Demigod of Revenge");
    let mut t = TestGame::new(2);
    let in_gy = t.graveyard(P0, "Demigod of Revenge");
    let card = t.hand(P0, "Demigod of Revenge");
    mana(&mut t, P0, ManaType::B, 5);
    let spell = t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The trigger resolved; the spell is still on the stack.
    assert!(t.on_battlefield(in_gy));
    assert_eq!(t.zone(spell), Zone::Stack);
    t.resolve_all();
    assert_eq!(named_on_bf(&t, P0, "Demigod of Revenge"), 2);
    // Put onto the battlefield directly: no trigger.
    let mut t = TestGame::new(2);
    let in_gy = t.graveyard(P0, "Demigod of Revenge");
    t.enter(P0, "Demigod of Revenge");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(in_gy), Zone::Graveyard(P0));
}

#[test]
fn zacama_untaps_lands_if_cast_from_any_zone_but_not_if_put_onto_the_battlefield() {
    cr!("603.4", "601.2");
    ruling!(
        "Zacama, Primal Calamity",
        "Zacama's triggered ability triggers if you cast it from any zone. It doesn't trigger if you put Zacama onto the battlefield without casting it."
    );
    supported("Zacama, Primal Calamity");
    supported("Karador, Ghost Chieftain");
    /// P0 with three tapped Forests.
    fn setup() -> (TestGame, Vec<ObjectId>) {
        let mut t = TestGame::new(2);
        let lands = t.lands(P0, "Forest", 3);
        for l in &lands {
            t.g.objects[l.0 as usize].tapped = true;
        }
        (t, lands)
    }
    let untapped =
        |t: &TestGame, lands: &[ObjectId]| lands.iter().filter(|l| !t.obj_now(**l).tapped).count();
    let pay = |t: &mut TestGame| {
        mana(t, P0, ManaType::R, 1);
        mana(t, P0, ManaType::G, 1);
        mana(t, P0, ManaType::W, 1);
        mana(t, P0, ManaType::C, 6);
    };
    // From the hand.
    let (mut t, lands) = setup();
    let z = t.hand(P0, "Zacama, Primal Calamity");
    pay(&mut t);
    t.cast(P0, z).go();
    t.resolve_all();
    assert_eq!(untapped(&t, &lands), 3);
    // From the graveyard (Karador's permission).
    let (mut t, lands) = setup();
    t.battlefield(P0, "Karador, Ghost Chieftain");
    let z = t.graveyard(P0, "Zacama, Primal Calamity");
    pay(&mut t);
    t.cast(P0, z).go();
    t.resolve_all();
    assert!(t.on_battlefield(z));
    assert_eq!(untapped(&t, &lands), 3);
    // Put onto the battlefield.
    let (mut t, lands) = setup();
    t.enter(P0, "Zacama, Primal Calamity");
    t.settle();
    t.resolve_all();
    assert_eq!(untapped(&t, &lands), 0);
}

#[test]
fn gluttonous_hellkite_counts_creatures_actually_sacrificed() {
    cr!("601.2i", "614.1c", "701.21a");
    ruling!(
        "Gluttonous Hellkite",
        "Use the number of creatures that were actually sacrificed this way, not the value of X"
    );
    supported("Gluttonous Hellkite");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Gluttonous Hellkite");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 4);
    let spell = t.cast(P0, card).x(2).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(spell));
    // X = 2, but only one creature was sacrificed: two counters.
    assert_eq!(t.counters(spell, counters::PLUS1), 2);
}

// ---------------------------------------------------------------- damage, end step

#[test]
fn deus_of_calamity_triggers_on_noncombat_damage_of_six_or_more() {
    cr!("603.2", "120.1");
    ruling!(
        "Deus of Calamity",
        "This ability triggers from any damage, not just combat damage"
    );
    supported("Deus of Calamity");
    for (n, triggers) in [(6, true), (5, false)] {
        let mut t = TestGame::new(2);
        let deus = t.battlefield(P0, "Deus of Calamity");
        let land = t.battlefield(P1, "Forest");
        t.answer_targets(P0, &[obj(land)]);
        damage(&mut t, deus, Entity::Player(P1), n);
        assert_eq!(t.stack_len(), triggers as usize, "{n} damage");
        t.resolve_all();
        assert_eq!(t.on_battlefield(land), !triggers, "{n} damage");
    }
}

#[test]
fn wardens_of_the_cycle_doesnt_trigger_if_nothing_died() {
    cr!("603.4", "702.165a");
    ruling!(
        "Wardens of the Cycle",
        "If none did, the ability won't trigger at all."
    );
    supported("Wardens of the Cycle");
    for died in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Wardens of the Cycle");
        if died {
            let bears = t.battlefield(P1, "Grizzly Bears");
            crate::r_s02_common::destroy(&mut t, bears);
        }
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        end_step(&mut t);
        let modes_asked = t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseModes { .. }));
        assert_eq!(modes_asked, died, "died: {died}");
        assert_eq!(t.life(P0), if died { 22 } else { 20 });
    }
}

// ---------------------------------------------------------------- graveyard triggers

#[test]
fn worldspine_wurm_shuffle_is_a_trigger_that_can_be_responded_to() {
    cr!("603.2");
    ruling!(
        "Worldspine Wurm",
        "Players can respond to this ability, for example, by trying to exile Worldspine Wurm from the graveyard before it's shuffled into a library."
    );
    supported("Worldspine Wurm");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.hand(P0, "Worldspine Wurm");
        let lib = t.library_size(P0);
        let in_gy = move_to(&mut t, wurm, Zone::Graveyard(P0)).unwrap();
        // The trigger waits on the stack with the Wurm in the graveyard.
        assert_eq!(t.stack_len(), 1);
        assert_eq!(t.zone(in_gy), Zone::Graveyard(P0));
        if respond {
            move_to(&mut t, in_gy, Zone::Exile);
        }
        t.resolve_all();
        if respond {
            assert!(t.in_exile("Worldspine Wurm"));
            assert_eq!(t.library_size(P0), lib);
        } else {
            assert_eq!(t.zone(in_gy), Zone::Library(P0));
            assert_eq!(t.library_size(P0), lib + 1);
        }
    }
}

#[test]
fn iname_as_one_returns_nothing_unless_it_is_exiled() {
    cr!("603.5", "608.2c");
    ruling!(
        "Iname as One",
        "You aren't required to exile Iname. If you don't, the target Spirit card isn't returned to the battlefield."
    );
    supported("Iname as One");
    for exile in [false, true] {
        let mut t = TestGame::new(2);
        let iname = t.battlefield(P0, "Iname as One");
        let spirit = t.graveyard(P0, "Kami of Lunacy");
        t.answer_targets(P0, &[obj(spirit)]);
        t.answer_yes(P0, exile);
        crate::r_s02_common::destroy(&mut t, iname);
        t.resolve_all();
        assert_eq!(t.on_battlefield(spirit), exile, "exile: {exile}");
        assert_eq!(t.in_exile("Iname as One"), exile);
        assert_eq!(t.in_graveyard(P0, "Iname as One"), !exile);
    }
}

// ---------------------------------------------------------------- Tolsimir

#[test]
fn tolsimir_wolf_trigger_with_no_target_or_an_illegal_target() {
    cr!("608.2b", "701.14a");
    ruling!(
        "Tolsimir, Friend to Wolves",
        "You may put Tolsimir's last ability onto the stack without choosing any target. You'll just gain 3 life."
    );
    supported("Tolsimir, Friend to Wolves");
    supported("Timber Wolves");
    // (choose a target, remove it before resolution, life afterwards)
    for (target, remove, life) in [(false, false, 23), (true, true, 20), (true, false, 23)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Tolsimir, Friend to Wolves");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let chosen: Vec<Entity> = if target { vec![obj(bears)] } else { vec![] };
        t.answer_targets(P0, &chosen);
        let wolf = t.enter(P0, "Timber Wolves");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        if remove {
            move_to(&mut t, bears, Zone::Hand(P1));
        }
        t.resolve_all();
        assert_eq!(t.life(P0), life, "target {target} removed {remove}");
        let fought = target && !remove;
        // Timber Wolves (1/1) and Grizzly Bears (2/2) fight: the Wolves die.
        assert_eq!(
            t.on_battlefield(wolf),
            !fought,
            "target {target} removed {remove}"
        );
        if fought {
            assert_eq!(t.obj_now(bears).damage, 1);
        }
    }
}

#[test]
fn both_tolsimirs_and_both_vojas_can_be_controlled_together() {
    cr!("704.5j", "201.2");
    ruling!(
        "Tolsimir, Friend to Wolves",
        "Since the \"legend rule\" only cares about exact English names, this means you may control all four of these creatures at the same time."
    );
    let mut t = TestGame::new(2);
    let wolfblood = t.battlefield(P0, "Tolsimir Wolfblood");
    activate_containing(&mut t, P0, wolfblood, "Voja").unwrap();
    t.resolve_all();
    t.answer_targets(P0, &[]);
    t.enter(P0, "Tolsimir, Friend to Wolves");
    t.settle();
    t.resolve_all();
    for name in [
        "Tolsimir Wolfblood",
        "Voja",
        "Tolsimir, Friend to Wolves",
        "Voja, Friend to Elves",
    ] {
        assert_eq!(named_on_bf(&t, P0, name), 1, "{name}");
    }
}

// ---------------------------------------------------------------- values on resolution

#[test]
fn eldrazi_linebreaker_counts_eldrazi_once_on_resolution() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Eldrazi Linebreaker",
        "The value of X is calculated only once, as Eldrazi Linebreaker's last ability resolves."
    );
    supported("Eldrazi Linebreaker");
    supported("Eldrazi Devastator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eldrazi Linebreaker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A second Eldrazi arrives in response: X is 2 on resolution.
    t.battlefield(P0, "Eldrazi Devastator");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    // A third one afterwards doesn't change it.
    t.battlefield(P0, "Eldrazi Devastator");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    // Losing Eldrazi doesn't either.
    for d in t.named_on_battlefield("Eldrazi Devastator") {
        move_to(&mut t, d, Zone::Graveyard(P0));
    }
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn stirring_honormancer_counts_creatures_once_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Stirring Honormancer",
        "The value of X is determined only once, as Stirring Honormancer's triggered ability resolves."
    );
    supported("Stirring Honormancer");
    let mut t = TestGame::new(2);
    t.enter(P0, "Stirring Honormancer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, two more creatures: X is 3.
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let (hand, gy, lib) = (t.hand_size(P0), t.graveyard_size(P0), t.library_size(P0));
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib - 3);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.graveyard_size(P0), gy + 2);
}

#[test]
fn defenders_of_humanity_x_values_are_separate() {
    cr!("107.3", "107.3f", "602.2b");
    ruling!(
        "Defenders of Humanity",
        "X in the first ability is always calculated separately from X in the second ability"
    );
    supported("Defenders of Humanity");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Defenders of Humanity");
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 4);
    let ench = t.cast(P0, card).x(2).go();
    t.resolve();
    t.settle();
    // The enters trigger (X = 2) is on the stack and P0 controls no creatures yet.
    assert_eq!(t.stack_len(), 1);
    mana(&mut t, P0, ManaType::W, 1);
    mana(&mut t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    activate_containing(&mut t, P0, ench, "Exile ~").unwrap();
    assert!(t.in_exile("Defenders of Humanity"));
    t.resolve();
    assert_eq!(with_subtype(&t, P0, "Astartes").len(), 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Astartes").len(), 3);
}

// ---------------------------------------------------------------- attack triggers

/// P0 attacks P1 with `attackers` and the triggers resolve; the block options of P1.
fn attack_and_resolve(t: &mut TestGame, attackers: &[ObjectId]) -> Vec<(ObjectId, Vec<ObjectId>)> {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attack_with(t, &decl);
    t.resolve_all();
    t.g.recompute();
    combat::block_options(&t.g, &[P1])
}

#[test]
fn the_ur_dragon_draws_for_dragons_that_left_and_puts_one_permanent() {
    cr!("603.2c", "608.2h");
    ruling!(
        "The Ur-Dragon",
        "You draw one card for each Dragon you controlled that attacked, even if some of them left the battlefield"
    );
    ruling!(
        "The Ur-Dragon",
        "You may only put one permanent card from your hand onto the battlefield, no matter how many Dragons you attacked with"
    );
    supported("The Ur-Dragon");
    let mut t = TestGame::new(2);
    let ur = t.battlefield(P0, "The Ur-Dragon");
    let shivan = t.battlefield(P0, "Shivan Dragon");
    let b1 = t.hand(P0, "Grizzly Bears");
    let b2 = t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(ur, Entity::Player(P1)), (shivan, Entity::Player(P1))],
    );
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, shivan, Zone::Graveyard(P0));
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(b1), obj(b2)]);
    t.resolve_all();
    // Two cards drawn, then one Bears put onto the battlefield.
    assert_eq!(named_on_bf(&t, P0, "Grizzly Bears"), 1);
    assert_eq!(t.hand_size(P0), hand + 2 - 1);
    let max_offered = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates, max, ..
            } if candidates.contains(&obj(b1)) => Some(*max),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(max_offered, vec![1]);
}

#[test]
fn tower_above_trigger_does_nothing_if_the_target_cant_block() {
    cr!("509.1c", "608.2b");
    ruling!(
        "Tower Above",
        "if the creature you target can't block the creature affected by Tower Above"
    );
    supported("Tower Above");
    // (attacker has flying, legal block of the other attacker by the target afterwards)
    for (flier, free) in [(true, true), (false, false)] {
        let mut t = TestGame::new(2);
        let big = t.battlefield(P0, if flier { "Serra Angel" } else { "Hill Giant" });
        let other = t.battlefield(P0, "Raging Goblin");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let own = t.battlefield(P0, "Llanowar Elves");
        mana(&mut t, P0, ManaType::G, 3);
        let tower = t.hand(P0, "Tower Above");
        t.cast(P0, tower).target(big).go();
        t.resolve_all();
        // The attack trigger may target any creature, including P0's own.
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(bears)]);
        let options = attack_and_resolve(&mut t, &[big, other]);
        let cands = crate::r_s02_common::target_candidates(&t, P0, from);
        assert!(cands.iter().any(|c| c.contains(&obj(own))), "{flier}");
        let reqs = combat::block_declaration_legal(&t.g, &options, &[(bears, other)]);
        assert_eq!(reqs, free, "flier {flier}");
        assert!(!combat::block_declaration_legal(&t.g, &options, &[]) || free);
    }
}

#[test]
fn giant_ambush_beetle_block_choice_is_made_on_resolution() {
    cr!("603.5", "509.1c");
    ruling!(
        "Giant Ambush Beetle",
        "You decide whether to have the targeted creature block Giant Ambush Beetle this turn if able at the time the \"enters\" ability resolves"
    );
    supported("Giant Ambush Beetle");
    for yes in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        let beetle = t.enter(P0, "Giant Ambush Beetle");
        t.settle();
        // No yes/no before the ability resolves.
        let from = t.asked().len();
        assert_eq!(t.stack_len(), 1);
        t.answer_yes(P0, yes);
        t.resolve_all();
        let asked: Vec<_> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .cloned()
            .collect();
        assert_eq!(asked.len(), 1);
        // Attacking later asks nothing more.
        let options = attack_and_resolve(&mut t, &[beetle]);
        let none_ok = combat::block_declaration_legal(&t.g, &options, &[]);
        assert_eq!(none_ok, !yes, "yes: {yes}");
        assert!(combat::block_declaration_legal(
            &t.g,
            &options,
            &[(bears, beetle)]
        ));
    }
}

// ---------------------------------------------------------------- Broodlord

#[test]
fn broodlord_divides_counters_as_the_trigger_is_put_on_the_stack() {
    cr!("601.2d", "603.3d");
    ruling!(
        "Broodlord",
        "You choose how many targets Broodlord's triggered ability has and how the counters are distributed as you put the ability onto the stack. Each target must receive at least one counter."
    );
    supported("Broodlord");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Broodlord");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 6);
    let lord = t.cast(P0, card).x(3).go();
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, P0, &[2, 1]);
    let from = t.asked().len();
    t.resolve();
    let divides: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total, min_each, ..
            } => Some((*total, *min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(divides, vec![(3, 1)]);
    // The division is fixed while the trigger is on the stack.
    assert!(stack_items(&t)
        .iter()
        .any(|s| s.contains("Brood Telepathy") || s.contains("distribute")));
    t.resolve_all();
    assert_eq!(t.counters(lord, counters::PLUS1), 3);
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}
