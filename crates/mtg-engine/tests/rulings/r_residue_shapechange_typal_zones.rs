//! Item `rulings-residue-shapechange-typal` — interaction rulings about permanents put
//! onto the battlefield by an effect (attacking without being declared, sacrificed at once,
//! returned by a delayed trigger, turned face up in exile), new objects and last known
//! information (CR 400.7, 603.10), replacement effects that keep applying (regeneration,
//! redirected damage) and player counters.

use crate::r_residue_shapechange_typal_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Ilharg attacks P1; its trigger resolves putting `card` from P0's hand onto the
/// battlefield tapped and attacking. Returns the new permanent.
fn ilharg_puts(t: &mut TestGame, ilharg: ObjectId, card: ObjectId) -> ObjectId {
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    attack_with(t, &[(ilharg, Entity::Player(P1))]);
    t.resolve_all();
    t.g.current(card)
}

#[test]
fn ilharg_creature_is_attacking_but_was_never_declared() {
    cr!("508.4", "508.3a", "603.2");
    ruling!(
        "Ilharg, the Raze-Boar",
        "Although the new creature is attacking, it was never declared as an attacking creature (for the purposes of abilities that trigger whenever a creature attacks, for example, such as those of Trusted Pegasus and Makeshift Battalion)."
    );
    supported("Ilharg, the Raze-Boar");
    supported("Trusted Pegasus");
    let mut t = main_phase(2);
    let ilharg = t.battlefield(P0, "Ilharg, the Raze-Boar");
    let peg = t.hand(P0, "Trusted Pegasus");
    let peg = ilharg_puts(&mut t, ilharg, peg);
    assert!(t.on_battlefield(peg));
    assert!(tapped(&t, peg));
    assert!(t.g.is_attacking(peg));
    // "Whenever this creature attacks, target attacking creature without flying gains
    // flying" didn't trigger: Ilharg has no flying.
    assert!(!crate::r_p119_common::has_kw(
        &mut t,
        ilharg,
        mtg_engine::keywords::KeywordKind::Flying
    ));
    assert_eq!(on_stack(&t), 0);
}

#[test]
fn ilharg_creature_that_died_before_the_end_step_stays_put() {
    cr!("400.7", "603.7c");
    ruling!(
        "Ilharg, the Raze-Boar",
        "If the new creature leaves the battlefield before the end step, most likely because it died in combat, that card remains in its current zone. It won't return to your hand."
    );
    let mut t = main_phase(2);
    let ilharg = t.battlefield(P0, "Ilharg, the Raze-Boar");
    let bears = t.hand(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = ilharg_puts(&mut t, ilharg, bears);
    block_and_finish(&mut t, P1, &[(wurm, bears)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn ilharg_creature_end_step_trigger_resolves_after_it_returns() {
    cr!("603.3b", "603.4", "603.7c");
    ruling!(
        "Ilharg, the Raze-Boar",
        "If the new creature has an ability that triggers at the beginning of the end step, that ability will trigger and resolve even if the creature is returned to your hand during the end step before that ability resolves."
    );
    supported("Platoon Dispenser");
    // Either order of the two triggers: the Dispenser's draws a card, also when the
    // Dispenser is back in hand as it resolves.
    let mut returned_first = vec![];
    for order in [[0usize, 1], [1, 0]] {
        let mut t = main_phase(2);
        let ilharg = t.battlefield(P0, "Ilharg, the Raze-Boar");
        t.battlefield(P0, "Grizzly Bears");
        let disp = t.hand(P0, "Platoon Dispenser");
        let disp = ilharg_puts(&mut t, ilharg, disp);
        assert!(t.on_battlefield(disp));
        block_and_finish(&mut t, P1, &[]);
        t.advance_to(P0, Step::End);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.to_vec()));
        t.settle();
        assert_eq!(on_stack(&t), 2);
        let hand = t.hand_size(P0);
        t.resolve();
        returned_first.push(t.in_hand(P0, "Platoon Dispenser"));
        t.resolve_all();
        assert!(t.in_hand(P0, "Platoon Dispenser"));
        // The Dispenser came back and a card was drawn.
        assert_eq!(t.hand_size(P0), hand + 2, "order {order:?}");
    }
    returned_first.sort();
    assert_eq!(returned_first, vec![false, true]);
}

#[test]
fn ilharg_creature_can_attack_another_player() {
    cr!("508.4", "802.2");
    ruling!(
        "Ilharg, the Raze-Boar",
        "You choose which player, planeswalker, or battle the new creature is attacking. It doesn't have to be attacking the same player, planeswalker, or battle that Ilharg is attacking."
    );
    let mut t = main_phase(3);
    let ilharg = t.battlefield(P0, "Ilharg, the Raze-Boar");
    let bears = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    // "Choose what the creature is attacking."
    t.answer_choose(P0, &[Entity::Player(P2)]);
    attack_with(&mut t, &[(ilharg, Entity::Player(P1))]);
    t.resolve_all();
    let bears = t.g.current(bears);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(bears), Some(Entity::Player(P2)));
    assert_eq!(c.attack_target(ilharg), Some(Entity::Player(P1)));
    assert!(!asked_since(&t, from).is_empty());
}

/// P0 casts Flash putting `card` from P0's hand onto the battlefield, paying (or not) the
/// reduced mana cost. Flash resolves; the triggers are not yet put on the stack.
fn flash_in(t: &mut TestGame, card: ObjectId, pay: bool) {
    supported("Flash");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer(P0, DecisionKind::Any, Answer::Bool(pay));
    cast_new(t, P0, "Flash", &[]);
    t.g.resolve_top();
}

#[test]
fn flash_creature_not_paid_for_is_sacrificed_at_once_and_both_triggers_go_on_the_stack() {
    cr!("608.2c", "603.3b", "603.6a", "603.10a", "117.3b");
    ruling!(
        "Flash",
        "If you choose not to pay, the creature is sacrificed immediately. No player will get priority in between the creature entering the battlefield and being sacrificed. Sacrificing the creature this way will trigger any abilities that trigger when it leaves the battlefield, and those abilities will be put onto the stack at the same time as those that triggered when it entered the battlefield."
    );
    ruling!(
        "Flash",
        "The creature enters the battlefield, so it will trigger enters-the-battlefield abilities even if you choose not to pay."
    );
    supported("Solemn Simulacrum");
    let mut t = main_phase(2);
    let sol = t.hand(P0, "Solemn Simulacrum");
    flash_in(&mut t, sol, false);
    // Already in the graveyard as Flash finishes resolving.
    assert!(t.in_graveyard(P0, "Solemn Simulacrum"));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "basic land"), 1);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    assert_eq!(on_stack(&t), 2);
}

#[test]
fn flash_x_in_the_mana_cost_is_zero() {
    cr!("107.3h", "118.7");
    ruling!(
        "Flash",
        "If the creature has {X} in its mana cost, X is considered to be 0."
    );
    supported("Ingenious Prodigy");
    let mut t = main_phase(2);
    let prodigy = t.hand(P0, "Ingenious Prodigy");
    t.lands(P0, "Island", 4);
    // {X}{U} with X = 0, reduced by {2}: {U}, though there's mana for a larger X.
    flash_in(&mut t, prodigy, true);
    t.settle();
    t.resolve_all();
    let prodigy = t.g.current(prodigy);
    assert!(t.on_battlefield(prodigy));
    assert_eq!(plus1(&t, prodigy), 0);
    // Flash's {1}{U} and the Prodigy's {U}.
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseX { .. })));
}

#[test]
fn minion_of_the_mighty_dragon_enters_attacking_without_attack_or_tap_triggers() {
    cr!("508.4", "508.3a", "603.2e", "205.3m");
    ruling!(
        "Minion of the Mighty",
        "The creature card is already tapped and attacking as it's put onto the battlefield. Any abilities that trigger whenever a creature becomes tapped or whenever a creature attacks won't trigger."
    );
    ruling!(
        "Minion of the Mighty",
        "A Dragon creature card is a creature card with the creature type Dragon, not necessarily a creature card with the word “Dragon” in its name."
    );
    supported("Minion of the Mighty");
    supported("Goldspan Dragon");
    supported("Quest for Renewal");
    supported("Dragonmaster Outcast");
    supported("Thundermaw Hellkite");
    let mut t = main_phase(2);
    let minion = t.battlefield(P0, "Minion of the Mighty");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let quest = t.battlefield(P0, "Quest for Renewal");
    let outcast = t.hand(P0, "Dragonmaster Outcast");
    let hellkite = t.hand(P0, "Thundermaw Hellkite");
    let gold_card = t.hand(P0, "Goldspan Dragon");
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(gold_card)]);
    attack_with(
        &mut t,
        &[(minion, Entity::Player(P1)), (wurm, Entity::Player(P1))],
    );
    t.resolve_all();
    let gold = t.g.current(gold_card);
    assert!(t.on_battlefield(gold));
    assert!(tapped(&t, gold));
    assert!(t.g.is_attacking(gold));
    // The Minion and the Wurm becoming tapped put two counters on the Quest; the Dragon
    // entering tapped doesn't, and its attack trigger doesn't create a Treasure.
    assert_eq!(t.counters(quest, "quest"), 2);
    assert!(t.named_on_battlefield("Treasure").is_empty());
    // The Dragon cards offered: Thundermaw Hellkite (a Dragon without the word in its
    // name), not Dragonmaster Outcast (a Human Shaman with it).
    let cands = asked_since(&t, from)
        .into_iter()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&obj(gold_card)) =>
            {
                Some(candidates)
            }
            _ => None,
        })
        .expect("asked for a Dragon creature card");
    assert!(cands.contains(&obj(hellkite)));
    assert!(!cands.contains(&obj(outcast)));
}

#[test]
fn summoners_egg_card_turned_face_up_in_exile() {
    cr!("113.6", "406.3", "603.2");
    ruling!(
        "Summoner's Egg",
        "Any “When this creature is turned face up” abilities that the exiled card may have don’t trigger because the card isn’t being turned face up while it’s on the battlefield."
    );
    ruling!(
        "Summoner's Egg",
        "If the card is not a creature card when it’s turned face up, it remains exiled face up."
    );
    supported("Summoner's Egg");
    supported("Brine Elemental");
    for (name, creature) in [("Brine Elemental", true), ("Lightning Bolt", false)] {
        let mut t = main_phase(2);
        let card = t.hand(P0, name);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(card)]);
        let egg = t.enter(P0, "Summoner's Egg");
        t.resolve_all();
        assert!(t.obj_now(card).face_down);
        assert_eq!(t.zone(card), Zone::Exile);
        destroy(&mut t, egg);
        t.resolve_all();
        let now = t.g.current(card);
        if creature {
            assert!(t.on_battlefield(now));
            assert_eq!(controller(&mut t, now), P0);
            // "When this creature is turned face up, each opponent skips their next untap
            // step" didn't trigger.
            assert_eq!(on_stack(&t), 0);
            let land = t.battlefield(P1, "Forest");
            t.g.objects[land.0 as usize].tapped = true;
            t.advance_to(P1, Step::Upkeep);
            assert!(!tapped(&t, land));
        } else {
            assert_eq!(t.zone(now), Zone::Exile);
            assert!(!t.obj_now(now).face_down);
        }
    }
}

#[test]
fn mossbridge_troll_regenerates_every_time() {
    cr!("701.19b", "614.8");
    ruling!(
        "Mossbridge Troll",
        "The first ability essentially means that Mossbridge Troll always has a regeneration shield. It retains that shield even if it's used."
    );
    supported("Mossbridge Troll");
    let mut t = main_phase(2);
    let troll = t.battlefield(P0, "Mossbridge Troll");
    for _ in 0..3 {
        cast_resolve(&mut t, P1, "Murder", &[obj(troll)]);
        assert!(t.on_battlefield(troll));
        assert_eq!(t.g.current(troll), troll);
        assert!(tapped(&t, troll));
        t.g.objects[troll.0 as usize].tapped = false;
    }
    // Lethal damage: regenerated, damage removed.
    damage(&mut t, troll, 5);
    assert!(t.on_battlefield(troll));
    assert_eq!(t.obj_now(troll).damage, 0);
}

#[test]
fn skullbriar_gets_one_counter_per_combat_damage_event_to_any_player_even_its_controller() {
    cr!("510.2", "603.2", "614.9");
    ruling!(
        "Skullbriar, the Walking Grave",
        "Skullbriar will get one +1/+1 counter each time it deals combat damage to a player (including you if its combat damage gets redirected), regardless of how much damage is dealt."
    );
    supported("Skullbriar, the Walking Grave");
    supported("Harm's Way");
    // Three damage to P1: one counter.
    let mut t = main_phase(2);
    let sk = t.battlefield(P0, "Skullbriar, the Walking Grave");
    give_plus1(&mut t, sk, 2);
    attack_with(&mut t, &[(sk, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(plus1(&t, sk), 3);
    // P1's Harm's Way redirects all of a 2-power Skullbriar's combat damage to P0: P0 is
    // the player dealt damage, and the counter still comes.
    let mut t = main_phase(2);
    let sk = t.battlefield(P0, "Skullbriar, the Walking Grave");
    give_plus1(&mut t, sk, 1);
    attack_with(&mut t, &[(sk, Entity::Player(P1))]);
    t.answer_choose(P1, &[obj(sk)]);
    cast_new(&mut t, P1, "Harm's Way", &[Entity::Player(P0)]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 20);
    assert_eq!(plus1(&t, sk), 2);
}

#[test]
fn azula_experience_counter_stays_with_the_player() {
    cr!("122.1", "400.7");
    ruling!(
        "Azula, Ruthless Firebender",
        "The experience counter goes on you, the player, not on Azula. You will keep that counter even if Azula, Ruthless Firebender dies."
    );
    supported("Azula, Ruthless Firebender");
    let mut t = main_phase(2);
    let azula = t.battlefield(P0, "Azula, Ruthless Firebender");
    let c = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(c)]);
    attack_with(&mut t, &[(azula, Entity::Player(P1))]);
    t.resolve_all();
    let xp = |t: &TestGame| {
        t.g.players[0]
            .counters
            .iter()
            .find(|(k, _)| k.as_str() == "experience")
            .map(|(_, n)| *n)
            .unwrap_or(0)
    };
    assert_eq!(xp(&t), 1);
    assert_eq!(t.counters(azula, "experience"), 0);
    destroy(&mut t, azula);
    assert!(t.in_graveyard(P0, "Azula, Ruthless Firebender"));
    assert_eq!(xp(&t), 1);
}
