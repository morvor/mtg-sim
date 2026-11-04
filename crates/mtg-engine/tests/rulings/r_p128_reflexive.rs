//! Rulings batch P128 — reflexive triggered abilities ("When you do, ..."): the first
//! ability goes on the stack without a target, the action is taken as it resolves, and
//! only then does the reflexive ability trigger, with its targets chosen and players able
//! to respond before it resolves (CR 603.12); values read as it resolves, last known
//! information, illegal targets, and divided effects.

use crate::r_p128_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s25_common::name_now;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether every ability on the stack has no targets.
fn untargeted_stack(t: &TestGame) -> bool {
    t.g.stack
        .iter()
        .all(|s| crate::r_s25_common::targets_of(t, *s).is_empty())
}

/// Resolves the top of the stack (the triggered ability) and checks that a reflexive
/// ability is then waiting on the stack (players may respond to it) and `before` still
/// holds; then resolves the stack.
fn resolve_then_reflexive(t: &mut TestGame, before: impl Fn(&TestGame) -> bool, what: &str) {
    t.settle();
    assert_eq!(t.stack_len(), 1, "{what}: the first ability");
    assert!(untargeted_stack(t), "{what}: no target");
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "{what}: the reflexive ability");
    assert!(before(t), "{what}: nothing happened yet");
    t.resolve_all();
}

#[test]
fn reflexive_abilities_trigger_during_resolution_and_can_be_responded_to() {
    cr!("603.12", "603.3", "117.3b");
    ruling!(
        "Armix, Filigree Thrasher",
        "Armix's ability goes on the stack without a target. While that ability is resolving, you may discard a card. If you do, a second ability triggers and you pick one target to get -X/-X."
    );
    ruling!(
        "Bolg of the North",
        "Bolg of the North has a reflexive triggered ability. When it enters the battlefield, its triggered ability goes on the stack without a target."
    );
    ruling!(
        "Cavalier of Night",
        "Cavalier of Night’s enters-the-battlefield ability goes on the stack without a target. While that ability is resolving, you may sacrifice another creature."
    );
    ruling!(
        "Heart-Piercer Manticore",
        "Heart-Piercer Manticore features a new style of triggered ability. When it enters the battlefield, its triggered ability goes on the stack without a target."
    );
    ruling!(
        "Halana, Kessig Ranger",
        "Halana, Kessig Ranger's triggered ability goes on the stack without a target. While that ability is resolving, you may pay {2}."
    );
    ruling!(
        "Electro, Assaulting Battery",
        "Electro's last ability triggers without requiring a target. You choose the value of X as that ability resolves. If you pay the mana, a second \"reflexive\" ability triggers."
    );
    // Armix: discard an artifact card; -X/-X where X = artifacts you control (Armix) plus
    // artifact cards in your graveyard (the discarded Ornithopter).
    supported("Armix, Filigree Thrasher");
    let mut t = TestGame::new(2);
    let armix = t.battlefield(P0, "Armix, Filigree Thrasher");
    let giant = t.battlefield(P1, "Hill Giant");
    t.hand(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(armix, Entity::Player(P1))]);
    resolve_then_reflexive(
        &mut t,
        |t| t.in_graveyard(P0, "Ornithopter") && t.pt(giant) == (3, 3),
        "Armix",
    );
    assert_eq!(t.pt(giant), (1, 1));
    // Bolg: sacrifice Hill Giant (power 3); 3 damage to Grizzly Bears, 1 excess: amass 1.
    supported("Bolg of the North");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Bolg of the North");
    resolve_then_reflexive(
        &mut t,
        |t| t.in_graveyard(P0, "Hill Giant") && t.on_battlefield(bears),
        "Bolg",
    );
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let army = crate::r_s01_common::with_subtype(&t, P0, "Army");
    assert_eq!(army.len(), 1);
    assert_eq!(t.counters(army[0], counters::PLUS1), 1);
    // Cavalier of Night: sacrifice Grizzly Bears, destroy P1's Hill Giant.
    supported("Cavalier of Night");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Cavalier of Night");
    resolve_then_reflexive(
        &mut t,
        |t| t.in_graveyard(P0, "Grizzly Bears") && t.on_battlefield(giant),
        "Cavalier",
    );
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Heart-Piercer Manticore: sacrifice Hill Giant, 3 damage to P1.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Heart-Piercer Manticore");
    resolve_then_reflexive(
        &mut t,
        |t| t.in_graveyard(P0, "Hill Giant") && t.life(P1) == 20,
        "Manticore",
    );
    assert_eq!(t.life(P1), 17);
    // Halana: another creature (Hill Giant) enters; pay {2}; it deals 3 damage.
    supported("Halana, Kessig Ranger");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Halana, Kessig Ranger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Hill Giant");
    resolve_then_reflexive(
        &mut t,
        |t| tapped_lands(t, P0) == 2 && t.on_battlefield(bears),
        "Halana",
    );
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Electro: it leaves the battlefield; X = 2 is paid; 2 damage to P1.
    supported("Electro, Assaulting Battery");
    let mut t = TestGame::new(2);
    let el = t.battlefield(P0, "Electro, Assaulting Battery");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, el);
    resolve_then_reflexive(
        &mut t,
        |t| tapped_lands(t, P0) == 2 && t.life(P1) == 20,
        "Electro",
    );
    assert_eq!(t.life(P1), 18);
}

#[test]
fn reflexive_abilities_after_paying_one_mana() {
    cr!("603.12", "603.3");
    ruling!(
        "Escape Protocol",
        "Escape Protocol’s triggered ability goes on the stack without a target. While that ability is resolving, you may pay {1}. If you do, a second ability triggers and you pick a target artifact or creature you control."
    );
    ruling!(
        "Halo Forager",
        "Halo Forager's enters-the-battlefield ability triggers and goes on the stack without a target. If you pay {X}, the second \"reflexive\" triggered ability will trigger. You'll choose the target instant or sorcery card for that second ability at that time, based on the value you chose for X. X can be 0."
    );
    // Escape Protocol: cycle Tranquil Thicket, pay {1}, flicker Grizzly Bears.
    supported("Escape Protocol");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Escape Protocol");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rf = t.hand(P0, "Tranquil Thicket");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, rf, "Cycling").unwrap();
    t.settle();
    // Escape Protocol's ability is above the cycling ability.
    assert_eq!(triggers_on_stack(&t, "cycle a card"), 1);
    t.resolve();
    t.settle();
    assert_eq!(t.g.current(bears), bears, "not flickered yet");
    // The reflexive ability (with its target) above the cycling ability.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, t.g.stack[1]),
        vec![obj(bears)]
    );
    t.resolve_all();
    assert_ne!(t.g.current(bears), bears, "a new object");
    assert!(t.on_battlefield(t.g.current(bears)));
    // Halo Forager: X = 0, cast Ancestral Vision (mana value 0) from P1's graveyard.
    supported("Halo Forager");
    let mut t = TestGame::new(2);
    let av = t.graveyard(P1, "Ancestral Vision");
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.answer_targets(P0, &[obj(av)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let hand = t.hand_size(P0);
    t.enter(P0, "Halo Forager");
    resolve_then_reflexive(
        &mut t,
        |t| t.zone(av) == Zone::Graveyard(P1),
        "Halo Forager",
    );
    assert_eq!(t.hand_size(P0), hand + 3);
    assert!(is_in(&t, av, Zone::Exile));
}

#[test]
fn flaxen_intruder_sacrifices_itself_then_destroys() {
    cr!("603.12", "510.3a");
    ruling!(
        "Flaxen Intruder // Welcome Home",
        "Flaxen Intruder's triggered ability goes on the stack without a target. While that ability is resolving, you may sacrifice it. When you do, the reflexive triggered ability triggers and you pick a target artifact or enchantment to be destroyed."
    );
    supported("Flaxen Intruder // Welcome Home");
    let mut t = TestGame::new(2);
    let fi = t.battlefield(P0, "Flaxen Intruder // Welcome Home");
    let orni = t.battlefield(P1, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(orni)]);
    attack_with(&mut t, &[(fi, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 19);
    resolve_then_reflexive(
        &mut t,
        |t| !t.on_battlefield(fi) && t.on_battlefield(orni),
        "Flaxen Intruder",
    );
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn adversaries_get_their_counters_as_the_reflexive_ability_resolves() {
    cr!("603.12", "603.12a", "702.26b", "205.1b");
    ruling!(
        "Primal Adversary",
        "After you pay the {1}{G} cost one or more times, a second ability triggers and you choose the targets for it. Players may respond to the new ability at that point. As that ability resolves, Primal Adversary gets its +1/+1 counters and the target lands become Wolf creatures."
    );
    ruling!(
        "Primal Adversary",
        "Any land that becomes a Wolf creature this way will retain any supertypes, card types, subtypes, and abilities it had."
    );
    ruling!(
        "Spectral Adversary",
        "After you pay the {1}{U} cost one or more times, a second ability triggers and you choose the targets for it. Players may respond to the new ability at that point. As that ability resolves, Spectral Adversary gets its +1/+1 counters and the target permanents phase out."
    );
    ruling!(
        "Intrepid Adversary",
        "After you pay the {1}{W} cost one or more times, a second ability triggers and players may respond to it. As that ability resolves, the Intrepid Adversary gets its valor counters."
    );
    ruling!(
        "Tainted Adversary",
        "After you pay the {2}{B} cost one or more times, a second ability triggers. Players may respond to the new ability at that point. As that ability resolves, Tainted Adversary gets its counters and the tokens are created."
    );
    ruling!(
        "Bloodthirsty Adversary",
        "After you pay the {2}{R} cost one or more times, a second ability triggers and you choose the targets for it. Players may respond to the new ability at that point. As that ability resolves, Bloodthirsty Adversary gets its counters and the target cards are exiled."
    );
    // Primal Adversary: pay twice; Forest and a legendary land become 3/3 Wolves.
    supported("Primal Adversary");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let pend = t.battlefield(P0, "Pendelhaven");
    t.lands(P0, "Forest", 4);
    for yes in [true, true, true, false] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(forest), obj(pend)]);
    let pa = t.enter(P0, "Primal Adversary");
    resolve_then_reflexive(
        &mut t,
        |t| t.counters(pa, counters::PLUS1) == 0 && !is_creature(t, forest),
        "Primal",
    );
    assert_eq!(t.counters(pa, counters::PLUS1), 2);
    for land in [forest, pend] {
        let o = t.obj_now(land);
        assert!(o.is(CardType::Creature) && o.is(CardType::Land));
        assert!(o.chars.has_subtype("Wolf"));
        assert_eq!(t.pt(land), (3, 3));
        assert!(o.has_keyword(mtg_engine::keywords::KeywordKind::Haste));
        assert!(!o.chars.abilities.is_empty(), "keeps its abilities");
    }
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
    assert!(t
        .obj_now(pend)
        .chars
        .supertypes
        .contains(Supertype::Legendary));
    // Spectral Adversary: pay once; P1's Grizzly Bears phases out.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    for yes in [true, true, false] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(bears)]);
    let sa = t.enter(P0, "Spectral Adversary");
    resolve_then_reflexive(
        &mut t,
        |t| t.counters(sa, counters::PLUS1) == 0 && !t.obj_now(bears).phased_out,
        "Spectral",
    );
    assert_eq!(t.counters(sa, counters::PLUS1), 1);
    assert!(t.obj_now(bears).phased_out);
    // Intrepid Adversary: pay once; one valor counter.
    supported("Intrepid Adversary");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    for yes in [true, true, false] {
        t.answer_yes(P0, yes);
    }
    let ia = t.enter(P0, "Intrepid Adversary");
    resolve_then_reflexive(&mut t, |t| t.counters(ia, "valor") == 0, "Intrepid");
    assert_eq!(t.counters(ia, "valor"), 1);
    assert_eq!(t.pt(ia), (4, 2));
    // Tainted Adversary: pay once; a counter and two Zombies.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    for yes in [true, true, false] {
        t.answer_yes(P0, yes);
    }
    let ta = t.enter(P0, "Tainted Adversary");
    resolve_then_reflexive(
        &mut t,
        |t| t.counters(ta, counters::PLUS1) == 0 && tokens(t, P0).is_empty(),
        "Tainted",
    );
    assert_eq!(t.counters(ta, counters::PLUS1), 1);
    assert_eq!(tokens(&t, P0).len(), 2);
    // Bloodthirsty Adversary: pay once; Lightning Bolt is exiled and copied.
    supported("Bloodthirsty Adversary");
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 3);
    for yes in [true, true] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ba = t.enter(P0, "Bloodthirsty Adversary");
    resolve_then_reflexive(
        &mut t,
        |t| t.counters(ba, counters::PLUS1) == 0 && t.zone(bolt) == Zone::Graveyard(P0),
        "Bloodthirsty",
    );
    assert_eq!(t.counters(ba, counters::PLUS1), 1);
    assert!(is_in(&t, bolt, Zone::Exile));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn bloodthirsty_adversary_copies_are_cast_during_resolution_ignoring_timing() {
    cr!("608.2g", "707.12", "307.1");
    ruling!(
        "Bloodthirsty Adversary",
        "Copies cast this way are cast during the resolution of the ability. You don't have to follow normal timing requirements and you can't wait and cast them later."
    );
    // It enters during P1's turn (flashed in, say): the sorcery copy is cast anyway.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let spike = t.graveyard(P0, "Lava Spike");
    t.lands(P0, "Mountain", 3);
    for yes in [true, true] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(spike)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Bloodthirsty Adversary");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Declining to cast it: no copy is left to cast later.
    let mut t = TestGame::new(2);
    let spike = t.graveyard(P0, "Lava Spike");
    t.lands(P0, "Mountain", 3);
    for yes in [true, true] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(spike)]);
    t.answer_yes(P0, false);
    t.enter(P0, "Bloodthirsty Adversary");
    t.resolve_all();
    assert!(is_in(&t, spike, Zone::Exile));
    assert_eq!(t.life(P1), 20);
    let spikes =
        t.g.objects
            .iter()
            .filter(|o| t.g.is_live(o.id) && o.chars.name == "Lava Spike" && o.zone != Zone::Exile)
            .count();
    assert_eq!(spikes, 0);
}

#[test]
fn sacrificing_for_another_reason_doesnt_trigger_the_damage() {
    cr!("603.12", "701.21a");
    ruling!(
        "Bolg of the North",
        "Bolg of the North's damage-dealing ability triggers only when you sacrifice a creature as a result of the instruction of its triggered ability. It won't trigger if you sacrifice a creature for any other reason."
    );
    ruling!(
        "Heart-Piercer Manticore",
        "Heart-Piercer Manticore's damage-dealing ability triggers only when you sacrifice a creature as a result of the instruction of its triggered ability. It won't trigger if you sacrifice a creature for any other reason."
    );
    for card in ["Bolg of the North", "Heart-Piercer Manticore"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let giant = t.battlefield(P0, "Hill Giant");
        t.battlefield(P1, "Grizzly Bears");
        sacrifice(&mut t, giant);
        assert!(t.in_graveyard(P0, "Hill Giant"));
        assert_eq!(t.stack_len(), 0, "{card}");
        // (Positive control: its own instruction does trigger it.)
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(giant)]);
        t.answer_targets(P0, &[obj(bears)]);
        t.enter(P0, card);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"), "{card}");
    }
}

#[test]
fn go_shintai_counts_shrines_on_resolution_and_may_target_a_noncreature_shrine() {
    cr!("608.2h", "603.12", "115.1");
    ruling!(
        "Go-Shintai of Boundless Vigor",
        "Count the number of Shrines you control as the reflexive triggered ability resolves to determine how many +1/+1 counters to put on the target Shrine."
    );
    ruling!(
        "Go-Shintai of Boundless Vigor",
        "Go-Shintai of Boundless Vigor's reflexive triggered ability may target a Shrine which isn't currently a creature."
    );
    supported("Go-Shintai of Boundless Vigor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Go-Shintai of Boundless Vigor");
    let honden = t.battlefield(P0, "Honden of Seeing Winds");
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(honden)]);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the reflexive ability");
    // Another Shrine arrives before it resolves: three Shrines.
    t.battlefield(P0, "Honden of Cleansing Fire");
    t.resolve_all();
    assert_eq!(t.counters(honden, counters::PLUS1), 3);
    assert!(!is_creature(&t, honden));
}

#[test]
fn halana_uses_the_creatures_power_as_the_reflexive_ability_resolves() {
    cr!("608.2h", "603.12");
    ruling!(
        "Halana, Kessig Ranger",
        "For the creature that entered the battlefield, use its power as Halana's reflexive triggered ability resolves to determine how much damage is dealt. If it's no longer on the battlefield at that time, use its power as it last existed on the battlefield."
    );
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Halana, Kessig Ranger");
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, "Wastes", 2);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(wurm)]);
        let giant = t.enter(P0, "Hill Giant");
        t.settle();
        t.resolve();
        t.settle();
        assert_eq!(t.stack_len(), 1);
        // In response, the Giant gets +3/+3, and then maybe dies.
        cast_new(&mut t, P0, "Giant Growth", &[obj(giant)]);
        t.resolve();
        if gone {
            sacrifice(&mut t, giant);
        }
        t.resolve_all();
        // 6 damage to the 6/4 Craw Wurm either way.
        assert!(t.in_graveyard(P1, "Craw Wurm"), "gone: {gone}");
    }
    // (Control: without the pump, 3 damage doesn't kill it.)
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Halana, Kessig Ranger");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(wurm)]);
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
}

#[test]
fn inferno_triggers_again_after_its_power_drops() {
    cr!("603.2", "603.12");
    ruling!(
        "Inferno of the Star Mounts",
        "However, if you have a way to decrease its power, you can cause the triggered ability to trigger multiple times in a single turn."
    );
    supported("Inferno of the Star Mounts");
    let mut t = TestGame::new(2);
    t.g.players[1].life = 100;
    let inf = t.battlefield(P0, "Inferno of the Star Mounts");
    t.lands(P0, "Mountain", 17);
    let pump = |t: &mut TestGame, n: usize| {
        for _ in 0..n {
            t.answer_targets(P0, &[Entity::Player(P1)]);
            activate_containing(t, P0, inf, "+1/+0").unwrap();
            t.resolve_all();
        }
    };
    pump(&mut t, 14);
    assert_eq!(t.pt(inf).0, 20);
    assert_eq!(t.life(P1), 80);
    cast_new(&mut t, P0, "Startle", &[obj(inf)]);
    t.resolve_all();
    assert_eq!(t.pt(inf).0, 18);
    pump(&mut t, 2);
    assert_eq!(t.life(P1), 60);
}

#[test]
fn tapping_triggers_need_you_to_tap_the_opponents_creature() {
    cr!("603.2", "701.26a");
    ruling!(
        "Icewrought Sentry",
        "Icewrought Sentry's last ability will trigger only when an effect instructs you to tap an opponent's creature. It won't trigger if a spell or ability you control instructs an opponent to tap a creature they control."
    );
    ruling!(
        "Hylda of the Icy Crown",
        "Hylda of the Icy Crown's ability will trigger only when an effect instructs you to tap an opponent's creature. It won't trigger if a spell or ability you control instructs an opponent to tap a creature they control."
    );
    supported("Icewrought Sentry");
    supported("Hylda of the Icy Crown");
    supported("Tangle Wire");
    // Icewrought Sentry taps P1's Grizzly Bears: both trigger.
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Icewrought Sentry");
    t.battlefield(P0, "Hylda of the Icy Crown");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(sentry, Entity::Player(P1))]);
    t.settle();
    t.resolve();
    t.settle();
    t.resolve();
    t.settle();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(triggers_on_stack(&t, "tap an untapped creature"), 2);
    // Tangle Wire (P0's) makes P1 tap their own creature: neither triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Icewrought Sentry");
    t.battlefield(P0, "Hylda of the Icy Crown");
    let wire = t.battlefield(P0, "Tangle Wire");
    put_counters(&mut t, wire, "fade", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[obj(bears)]);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(triggers_on_stack(&t, "tap an untapped creature"), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn isareth_returned_creature() {
    cr!("508.1", "614.1a", "700.4", "611.2a");
    ruling!(
        "Isareth the Awakener",
        "Because all attacking creatures are chosen at once, a creature returned this way can't attack during the same combat as it returns, even if it has haste."
    );
    ruling!(
        "Isareth the Awakener",
        "Because to die means to be put into a graveyard from the battlefield, a creature that is exiled instead doesn't \"die.\" Abilities that would trigger when it dies won't trigger."
    );
    ruling!(
        "Isareth the Awakener",
        "If Isareth leaves the battlefield, the replacement effect continues to apply. If one of the creatures it returned would leave the battlefield, it'll be exiled instead."
    );
    supported("Isareth the Awakener");
    let mut t = TestGame::new(2);
    let is = t.battlefield(P0, "Isareth the Awakener");
    let hz = t.graveyard(P0, "Hobbling Zombie");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.answer_targets(P0, &[obj(hz)]);
    attack_with(&mut t, &[(is, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(hz));
    assert_eq!(t.counters(hz, "corpse"), 1);
    let attacking = t.g.is_attacking(t.g.current(hz));
    assert!(!attacking, "it isn't attacking");
    // Isareth leaves; the Zombie is destroyed: exiled instead, and no token.
    destroy(&mut t, is);
    destroy(&mut t, hz);
    t.resolve_all();

    assert!(is_in(&t, hz, Zone::Exile));
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn overseer_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Overseer of Vault 76",
        "If Overseer of Vault 76 and one or more other creatures with power 3 or less enter the battlefield under your control at the same time, Overseer of Vault 76’s first ability will trigger for each of them."
    );
    supported("Overseer of Vault 76");
    let mut t = TestGame::new(2);
    let both = crate::r_p116_common::enter_together(
        &mut t,
        P0,
        &[
            "Overseer of Vault 76",
            "Grizzly Bears",
            "Hill Giant",
            "Craw Wurm",
        ],
    );
    t.resolve_all();
    assert_eq!(t.counters(both[0], "quest"), 3);
}

#[test]
fn shire_shirriff_leaving_early_means_nothing_is_exiled() {
    cr!("610.3c", "603.12");
    ruling!(
        "Shire Shirriff",
        "If Shire Shirriff leaves the battlefield before its last ability resolves, you can still sacrifice a token. However, when you do, the target creature won't be exiled when the \"reflexive\" triggered ability resolves."
    );
    supported("Shire Shirriff");
    // 0: leaves before the first ability resolves; 1: while the reflexive one waits; 2:
    // stays (control).
    for when in 0..3 {
        let mut t = TestGame::new(2);
        let tok = create_token(&mut t, P0, "Soldier");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(tok)]);
        t.answer_targets(P0, &[obj(bears)]);
        let ss = t.enter(P0, "Shire Shirriff");
        t.settle();
        if when == 0 {
            destroy(&mut t, ss);
        }
        t.resolve();
        t.settle();
        assert!(!t.on_battlefield(tok), "the token was sacrificed ({when})");
        if when == 1 {
            destroy(&mut t, ss);
        }
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), when < 2, "{when}");
    }
}

#[test]
fn slinza_triggers_and_counters() {
    cr!("603.6a", "614.1c", "614.12", "603.2");
    ruling!(
        "Slinza, the Spiked Stampede",
        "If Slinza enters at the same time as one or more other creatures with power 4 or greater, its last ability triggers once for each of those other creatures, plus one more time for itself."
    );
    ruling!(
        "Slinza, the Spiked Stampede",
        "Any Beasts that enter at the same time as Slinza won't enter with an additional +1/+1 counter."
    );
    ruling!(
        "Slinza, the Spiked Stampede",
        "If one or more static abilities that apply to a creature entering change its power, those abilities are considered when determining whether Slinza's last ability triggers. The same is true for replacement effects that apply to it, such as entering with one or more +1/+1 counters"
    );
    supported("Slinza, the Spiked Stampede");
    let slinza_triggers = |t: &TestGame| triggers_on_stack(t, "fights");
    // Together with Ravenous Baloth (a 4/4 Beast) and Craw Wurm (6/4).
    let mut t = TestGame::new(2);
    let ids = crate::r_p116_common::enter_together(
        &mut t,
        P0,
        &[
            "Slinza, the Spiked Stampede",
            "Ravenous Baloth",
            "Craw Wurm",
        ],
    );
    t.settle();
    assert_eq!(slinza_triggers(&t), 3);
    assert_eq!(t.counters(ids[1], counters::PLUS1), 0, "no extra counter");
    // Slinza already there: a 3-power Beast (Axebane Beast) enters with a counter (4
    // power: it triggers); Hill Giant with Glorious Anthem (4 power: it triggers);
    // Grizzly Bears (2 + 1 = 3): no.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slinza, the Spiked Stampede");
    let ab = t.enter(P0, "Axebane Beast");
    t.settle();
    assert_eq!(t.counters(ab, counters::PLUS1), 1);
    assert_eq!(slinza_triggers(&t), 1);
    t.clear_answers();
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slinza, the Spiked Stampede");
    t.battlefield(P0, "Glorious Anthem");
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(slinza_triggers(&t), 1);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slinza, the Spiked Stampede");
    t.battlefield(P0, "Glorious Anthem");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(slinza_triggers(&t), 0);
}

#[test]
fn cavalier_copy_can_return_itself() {
    cr!("707.2", "202.3", "603.10a");
    ruling!(
        "Cavalier of Night",
        "If a creature card with mana value 3 or less becomes a copy of Cavalier of Night, its last ability can target itself when it dies."
    );
    supported("Phantasmal Image");
    let mut t = TestGame::new(2);
    let cav = t.battlefield(P0, "Cavalier of Night");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(cav)]);
    let img = t.enter(P0, "Phantasmal Image");
    t.clear_answers();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(name_now(&t, img), "Cavalier of Night");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(img)]);
    destroy(&mut t, img);
    let in_gy = t.g.current(img);
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&obj(in_gy))), "{cands:?}");
}

#[test]
fn the_fire_nation_drill_strips_indestructible_from_damaged_creatures_but_not_new_ones() {
    cr!("611.2c", "704.5g", "702.12b");
    ruling!(
        "The Fire Nation Drill",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, damage previously dealt to a creature with indestructible may cause it to be destroyed if The Fire Nation Drill's first activated ability resolves during that turn."
    );
    ruling!(
        "The Fire Nation Drill",
        "If a permanent enters under an opponent's control with hexproof or indestructible after The Fire Nation Drill's first activated ability resolves, it won't lose that ability unless you activate The Fire Nation Drill's ability again."
    );
    supported("The Fire Nation Drill");
    let mut t = TestGame::new(2);
    let drill = t.battlefield(P0, "The Fire Nation Drill");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let src = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, src, 2, myr);
    assert!(t.on_battlefield(myr), "indestructible");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, drill, "lose hexproof").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Darksteel Myr"));
    // A new Darksteel Myr keeps indestructible.
    let myr2 = t.battlefield(P1, "Darksteel Myr");
    damage(&mut t, src, 2, myr2);
    assert!(t.on_battlefield(myr2));
}

#[test]
fn iroh_gifts_return_when_the_opponent_leaves_and_need_legal_targets() {
    cr!("800.4a", "608.2b", "603.12");
    ruling!(
        "Iroh, Tea Master",
        "If an opponent leaves the game, all permanents you gave to them will return to your control."
    );
    ruling!(
        "Iroh, Tea Master",
        "If either the opponent or the permanent you control becomes an illegal target by the time Iroh's second ability resolves, the ability does nothing and you cannot choose to have the opponent gain control of the permanent."
    );
    supported("Iroh, Tea Master");
    for destroyed in [false, true] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Iroh, Tea Master");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        if destroyed {
            destroy(&mut t, bears);
        }
        t.resolve_all();
        let allies = crate::r_s01_common::with_subtype(&t, P0, "Ally")
            .into_iter()
            .filter(|a| t.obj(*a).is_token())
            .count();
        if destroyed {
            assert_eq!(allies, 0);
            continue;
        }
        assert_eq!(allies, 1);
        assert_eq!(t.obj_now(bears).controller, P1);
        t.g.lose_game(P1);
        t.settle();
        assert_eq!(t.obj_now(bears).controller, P0);
    }
}

#[test]
fn three_dog_aura_copies() {
    cr!("707.2", "303.4f", "702.16c", "603.6a");
    ruling!(
        "Three Dog, Galaxy News DJ",
        "Any enters-the-battlefield abilities of the copied Aura will trigger when the tokens enter the battlefield."
    );
    ruling!(
        "Three Dog, Galaxy News DJ",
        "If one of the Aura tokens being created couldn't legally be attached to the appropriate attacking creature (because of an ability like protection from enchantments), that Aura token won't be created."
    );
    supported("Three Dog, Galaxy News DJ");
    supported("Cartouche of Knowledge");
    let mut t = TestGame::new(2);
    let td = t.battlefield(P0, "Three Dog, Galaxy News DJ");
    attach_new(&mut t, P0, "Cartouche of Knowledge", obj(td));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wing = t.battlefield(P0, "Azorius First-Wing");
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    attack_with(
        &mut t,
        &[
            (td, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (wing, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    let auras: Vec<_> =
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.name == "Cartouche of Knowledge")
            .map(|o| (o.id, o.attached_to))
            .collect();
    assert_eq!(auras, vec![(auras[0].0, Some(obj(bears)))]);
    // The token's enters ability drew a card.
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn numa_and_blaster_hulk_keep_their_division_when_a_target_is_illegal() {
    cr!("608.2b", "601.2d", "115.7");
    ruling!(
        "Numa, Joraga Chieftain",
        "If some of the creatures are illegal targets as the reflexive triggered ability tries to resolve, the original distribution of counters still applies and the counters that would have been put on illegal targets are lost."
    );
    ruling!(
        "Blaster Hulk",
        "If some of the targets are illegal as Blaster Hulk's last ability tries to resolve, the original division of damage still applies and the damage that would have been dealt to the illegal targets is lost. It won't be dealt instead to a legal target. If all of the targets are illegal, the ability won't resolve and none of its effects will happen."
    );
    supported("Numa, Joraga Chieftain");
    supported("Blaster Hulk");
    // Numa: X = 3, two counters on one Elf and one on the other; the second leaves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Numa, Joraga Chieftain");
    let a = t.battlefield(P0, "Llanowar Elves");
    let b = t.battlefield(P0, "Elvish Mystic");
    t.lands(P0, "Wastes", 6);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    bounce(&mut t, b);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    // Blaster Hulk: 8 damage divided 5 / 3 between two creatures; one becomes illegal.
    for both_gone in [false, true] {
        let mut t = TestGame::new(2);
        let hulk = t.battlefield(P0, "Blaster Hulk");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let giant = t.battlefield(P1, "Hill Giant");
        t.g.players[0].counters.insert(counters::ENERGY.into(), 6);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(wurm), obj(giant)]);
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 5]));
        attack_with(&mut t, &[(hulk, Entity::Player(P1))]);
        t.resolve();
        t.settle();
        assert_eq!(t.stack_len(), 1, "the reflexive ability");
        bounce(&mut t, giant);
        if both_gone {
            bounce(&mut t, wurm);
        }
        t.resolve_all();
        if !both_gone {
            assert_eq!(t.obj_now(wurm).damage, 3, "only its share");
        }
        assert_eq!(t.life(P1), 20);
    }
}
