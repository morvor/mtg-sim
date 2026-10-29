//! Rulings batch P152 — the other abilities of permanents that sacrifice themselves:
//! quest counters (one per trigger), reflexive triggers (CR 603.12), "you may pay" once
//! (CR 118.12), effects whose affected set is locked in on resolution (CR 611.2c),
//! combat damage triggers whose source can't be sacrificed, and costs (CR 118, 602.2b).

use crate::r_p152_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn with_subtype_token(t: &TestGame, ty: &str) -> ObjectId {
    t.g.permanents()
        .find(|o| o.is_token() && o.chars.has_subtype(ty))
        .map(|o| o.id)
        .expect("token")
}

fn energy(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counters.get("energy").copied().unwrap_or(0)
}

// ---------------------------------------------------------------------------------------
// Quests: a counter per trigger
// ---------------------------------------------------------------------------------------

#[test]
fn quest_for_the_gravelord_counts_any_creature_dying() {
    cr!("700.4", "603.2");
    ruling!(
        "Quest for the Gravelord",
        "Quest for the Gravelord’s first ability triggers regardless of who controlled the creature"
    );
    supported("Quest for the Gravelord");
    let mut t = TestGame::new(2);
    let quest = t.battlefield(P0, "Quest for the Gravelord");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    for b in [theirs, mine] {
        destroy(&mut t, b);
        t.answer_yes(P0, true);
        t.resolve_all();
    }
    assert_eq!(t.counters(quest, "quest"), 2);
}

#[test]
fn quest_for_the_gemblades_triggers_per_creature_dealt_damage() {
    cr!("510.2", "603.2c");
    ruling!(
        "Quest for the Gemblades",
        "The first ability will trigger multiple times if a creature you control deals combat damage to multiple creatures"
    );
    ruling!(
        "Quest for the Gemblades",
        "When the first ability resolves, you may put just one quest counter on Quest for the Gemblades. It doesn’t matter how much damage the source dealt."
    );
    supported("Quest for the Gemblades");
    let mut t = TestGame::new(2);
    let quest = t.battlefield(P0, "Quest for the Gemblades");
    let giant = t.battlefield(P0, "Craw Wurm");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer(
        P0,
        DecisionKind::Damage,
        Answer::Numbers(vec![2, 4]),
    );
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    block_and_finish(&mut t, P1, &[(a, giant), (b, giant)]);
    t.resolve_all();
    assert_eq!(t.counters(quest, "quest"), 2, "one counter per trigger");
}

#[test]
fn quest_for_pure_flame_triggers_per_opponent_and_adds_one_counter() {
    cr!("120.3", "603.2c");
    ruling!(
        "Quest for Pure Flame",
        "or if a single source you control deals damage to multiple opponents at once."
    );
    ruling!(
        "Quest for Pure Flame",
        "When the first ability resolves, you may put just one quest counter on Quest for Pure Flame. It doesn’t matter how much damage the source dealt."
    );
    supported("Quest for Pure Flame");
    let mut t = TestGame::new(3);
    let quest = t.battlefield(P0, "Quest for Pure Flame");
    // Flame Rift: 4 damage to each player (P0 isn't an opponent).
    cast_new(&mut t, P0, "Flame Rift", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "quest counter"), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(quest, "quest"), 2);
}

#[test]
fn quest_for_pure_flame_doubles_all_your_damage_this_turn() {
    cr!("614.1a", "611.2c");
    ruling!(
        "Quest for Pure Flame",
        "The second ability affects all sources you control that would deal damage to a permanent or player at any point in the rest of the turn."
    );
    supported("Quest for Pure Flame");
    let mut t = TestGame::new(2);
    let quest = t.battlefield(P0, "Quest for Pure Flame");
    put_counters(&mut t, quest, "quest", 4);
    act(&mut t, P0, quest, "Remove four quest counters", &[]).expect("Quest");
    t.resolve_all();
    let giant = t.battlefield(P1, "Hill Giant");
    // A source that entered after the ability resolved.
    let shocker = t.battlefield(P0, "Prodigal Pyromancer");
    t.activate(P0, shocker, 0, &[Entity::Player(P1)]).expect("ping");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    cast_new(&mut t, P0, "Shock", &[obj(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant), "4 damage to the 3/3");
    // The opponent's sources aren't affected.
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn quest_for_the_holy_relic_attaches_without_targeting_or_to_nothing() {
    cr!("301.5c", "701.3b");
    ruling!(
        "Quest for the Holy Relic",
        "The activated ability doesn’t target the creature the Equipment will be attached to. It may be attached to a creature with shroud"
    );
    ruling!(
        "Quest for the Holy Relic",
        "You may activate Quest for the Holy Relic’s activated ability even if you control no creatures"
    );
    supported("Quest for the Holy Relic");
    for with_creature in [true, false] {
        let mut t = TestGame::new(2);
        let quest = t.battlefield(P0, "Quest for the Holy Relic");
        put_counters(&mut t, quest, "quest", 5);
        let shroud = with_creature.then(|| t.battlefield(P0, "Argothian Enchantress"));
        let eq = t.library_top(P0, "Bonesplitter");
        t.answer_choose(P0, &[obj(eq)]);
        if let Some(c) = shroud {
            t.answer_choose(P0, &[obj(c)]);
        }
        act(&mut t, P0, quest, "Remove five quest counters", &[]).expect("Quest");
        t.resolve_all();
        let b = t.named_on_battlefield("Bonesplitter");
        assert_eq!(b.len(), 1);
        match shroud {
            Some(c) => {
                assert_eq!(t.g.obj(b[0]).attached_to, Some(Entity::Object(c)));
                assert_eq!(t.pt(c), (2, 1));
            }
            None => assert_eq!(t.g.obj(b[0]).attached_to, None),
        }
    }
}

#[test]
fn ordeal_of_nylea_triggers_when_sacrificed_for_any_reason() {
    cr!("701.21a", "603.6c");
    ruling!(
        "Ordeal of Nylea",
        "Ordeal of Nylea's last ability will trigger when you sacrifice it for any reason"
    );
    supported("Ordeal of Nylea");
    for sacrificed in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let ordeal = crate::r_s06_common::attach_new(&mut t, P0, "Ordeal of Nylea", bears);
        let forests: Vec<ObjectId> = (0..2).map(|_| t.library_top(P0, "Forest")).collect();
        if sacrificed {
            sacrifice(&mut t, P0, ordeal);
        } else {
            destroy(&mut t, ordeal);
        }
        assert_eq!(
            triggers_on_stack(&t, "search your library"),
            sacrificed as usize
        );
        t.answer_choose(P0, &[obj(forests[0]), obj(forests[1])]);
        t.resolve_all();
        assert_eq!(count_named(&t, "Forest"), if sacrificed { 2 } else { 0 });
    }
}

// ---------------------------------------------------------------------------------------
// "You may pay {1}": once
// ---------------------------------------------------------------------------------------

#[test]
fn durable_handicraft_and_era_of_innovation_take_one_payment() {
    cr!("118.12", "603.5");
    ruling!(
        "Durable Handicraft",
        "You can pay {1} only once each time Durable Handicraft's triggered ability resolves."
    );
    ruling!(
        "Durable Handicraft",
        "You don't choose whether to pay {1} until Durable Handicraft's triggered ability is resolving."
    );
    ruling!(
        "Era of Innovation",
        "You can pay {1} only once each time Era of Innovation's first ability resolves."
    );
    supported("Durable Handicraft");
    supported("Era of Innovation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Durable Handicraft");
    t.battlefield(P0, "Era of Innovation");
    t.lands(P0, "Wastes", 4);
    let n = t.asked().len();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert!(!asked_yes_no(&t, P0, n), "not asked as it triggers");
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    t.enter(P0, "Ornithopter");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    let ornithopter = t.named_on_battlefield("Ornithopter")[0];
    assert_eq!(t.counters(ornithopter, "+1/+1"), 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
}

// ---------------------------------------------------------------------------------------
// Deadly Designs: its controller's trigger, up to two targets
// ---------------------------------------------------------------------------------------

#[test]
fn deadly_designs_trigger_is_its_controllers_with_up_to_two_targets() {
    cr!("603.3a", "601.2c", "115.6");
    ruling!(
        "Deadly Designs",
        "You control the last ability of Deadly Designs, no matter who activated the ability"
    );
    ruling!(
        "Deadly Designs",
        "You must sacrifice Deadly Designs as its last ability resolves. However, you can choose zero creatures as targets if you want."
    );
    supported("Deadly Designs");
    for zero in [false, true] {
        let mut t = TestGame::new(2);
        let dd = t.battlefield(P0, "Deadly Designs");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        put_counters(&mut t, dd, "plot", 4);
        pool(&mut t, P1, &[ManaType::C], 2);
        t.activate(P1, dd, 0, &[]).expect("any player");
        let targets = if zero { vec![] } else { vec![obj(theirs)] };
        t.answer_targets(P0, &targets);
        t.resolve_all();
        assert!(!t.on_battlefield(dd), "sacrificed ({zero})");
        assert_eq!(t.on_battlefield(theirs), zero);
        let asked_p0 = t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. }));
        assert!(asked_p0, "P0 chose the targets");
    }
}

// ---------------------------------------------------------------------------------------
// Reflexive triggers: targets chosen once you sacrifice
// ---------------------------------------------------------------------------------------

#[test]
fn blood_spatter_analysis_targets_with_its_reflexive_trigger() {
    cr!("603.12");
    ruling!(
        "Blood Spatter Analysis",
        "a second \"reflexive\" ability triggers when you sacrifice Blood Spatter Analysis this way. You choose a target for that ability as it goes on the stack."
    );
    supported("Blood Spatter Analysis");
    let mut t = TestGame::new(2);
    let bsa = t.battlefield(P0, "Blood Spatter Analysis");
    put_counters(&mut t, bsa, "bloodstain", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    // The Bears that just died is a legal target for the reflexive ability.
    t.resolve();
    assert!(!t.on_battlefield(bsa));
    assert_eq!(crate::r_p151_common::abilities_on_stack(&t), 1, "the reflexive trigger");
    let card = t.g.current(bears);
    t.answer_targets(P0, &[obj(card)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn the_falcon_targets_with_its_reflexive_trigger() {
    cr!("603.12");
    ruling!(
        "The Falcon, Airship Restored",
        "You don't choose a target for The Falcon's second ability at the time it triggers."
    );
    supported("The Falcon, Airship Restored");
    let mut t = TestGame::new(2);
    let falcon = t.battlefield(P0, "The Falcon, Airship Restored");
    let crew = t.battlefield(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s04_common::crew(&mut t, P0, falcon, &[crew]));
    t.resolve_all();
    attack_with(&mut t, &[(falcon, Entity::Player(P1))]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you may sacrifice it"), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(falcon));
    assert_eq!(crate::r_p151_common::abilities_on_stack(&t), 1, "the reflexive trigger");
    t.resolve_all();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
}

// ---------------------------------------------------------------------------------------
// Combat damage triggers whose source can't be sacrificed
// ---------------------------------------------------------------------------------------

#[test]
fn flamespeakers_will_must_be_there_to_destroy_the_artifact() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Flamespeaker's Will",
        "If it's not on the battlefield at that time, you can't sacrifice it and destroy the target artifact."
    );
    supported("Flamespeaker's Will");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let will = crate::r_s06_common::attach_new(&mut t, P0, "Flamespeaker's Will", bears);
        let art = t.battlefield(P1, "Ornithopter");
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.answer_targets(P0, &[obj(art)]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "destroy target artifact"), 1);
        if gone {
            bounce(&mut t, will);
        }
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.on_battlefield(art), gone, "{gone}");
    }
}

#[test]
fn kathari_bomber_makes_goblins_even_if_it_cant_be_sacrificed() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Kathari Bomber",
        "You create the tokens even if you can’t sacrifice Kathari Bomber."
    );
    supported("Kathari Bomber");
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        let bomber = t.battlefield(P0, "Kathari Bomber");
        attack_with(&mut t, &[(bomber, Entity::Player(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        if gone {
            bounce(&mut t, bomber);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), 2, "{gone}");
        assert_eq!(
            t.zone(bomber),
            if gone { Zone::Hand(P0) } else { Zone::Graveyard(P0) }
        );
    }
}

#[test]
fn vexing_devil_is_sacrificed_as_the_damage_is_dealt() {
    cr!("608.2c", "117.3b");
    ruling!(
        "Vexing Devil",
        "No player may take actions between the time an opponent chooses to be dealt damage by Vexing Devil and the time you sacrifice Vexing Devil."
    );
    supported("Vexing Devil");
    let mut t = TestGame::new(2);
    let devil = t.enter(P0, "Vexing Devil");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P1, true);
    t.resolve();
    // Within that one resolution: damage dealt, Devil sacrificed.
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Vexing Devil"));
    assert!(!t.on_battlefield(devil));
}

#[test]
fn time_elemental_sacrifice_and_damage_happen_at_end_of_combat() {
    cr!("603.7", "511.2");
    ruling!(
        "Time Elemental",
        "You both sacrifice it and deal the 5 damage at end of combat."
    );
    supported("Time Elemental");
    let mut t = TestGame::new(2);
    let te = t.battlefield(P0, "Time Elemental");
    attack_with(&mut t, &[(te, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(te), "not yet");
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Time Elemental"));
    assert_eq!(t.life(P0), 15);
}

// ---------------------------------------------------------------------------------------
// "If this ability has been activated four or more times this turn"
// ---------------------------------------------------------------------------------------

#[test]
fn dragon_whelp_makes_a_delayed_trigger_from_the_fourth_activation_on() {
    cr!("603.7a", "602.1");
    ruling!(
        "Dragon Whelp",
        "if you activate Dragon Whelp's ability five times, for example, two different delayed triggered abilities will trigger"
    );
    ruling!(
        "Dragon Whelp",
        "There's no limit to the number of times you can activate Dragon Whelp's ability in a turn."
    );
    supported("Dragon Whelp");
    for n in [3, 5] {
        let mut t = TestGame::new(2);
        let whelp = t.battlefield(P0, "Dragon Whelp");
        t.lands(P0, "Mountain", n);
        for _ in 0..n {
            t.activate(P0, whelp, 0, &[]).expect("firebreathing");
            t.resolve_all();
        }
        assert_eq!(t.pt(whelp).0, 2 + n as i32);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(crate::r_p151_common::abilities_on_stack(&t), n - 3, "{n}");
        t.resolve_all();
        assert_eq!(t.on_battlefield(whelp), n == 3);
    }
}

#[test]
fn initiates_of_the_ebon_hand_mana_has_a_new_source() {
    cr!("106.3", "605.1a");
    ruling!(
        "Initiates of the Ebon Hand",
        "Mana which “changes color” as it goes through the Hand forgets its original source"
    );
    supported("Initiates of the Ebon Hand");
    let mut t = TestGame::new(2);
    let ini = t.battlefield(P0, "Initiates of the Ebon Hand");
    let snow = t.battlefield(P0, "Snow-Covered Island");
    t.activate(P0, ini, 0, &[]).expect("Initiates");
    let pool = &t.g.player(P0).mana_pool.mana;
    assert!(t.obj_now(snow).tapped, "paid with the snow land");
    assert_eq!(pool.len(), 1);
    assert_eq!(pool[0].ty, ManaType::B);
    assert_eq!(pool[0].source, Some(ini));
    assert!(!pool[0].snow, "not snow mana any more");
}

// ---------------------------------------------------------------------------------------
// Costs
// ---------------------------------------------------------------------------------------

#[test]
fn a_killer_among_us_cant_be_activated_by_a_new_controller() {
    cr!("118.3", "602.2b");
    ruling!(
        "A Killer Among Us",
        "If another player gains control of A Killer Among Us, they will be unable to activate its last ability."
    );
    supported("A Killer Among Us");
    for stolen in [false, true] {
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Option, Answer::Index(2)); // Goblin
        let killer = t.enter(P0, "A Killer Among Us");
        t.resolve_all();
        // An attacking Goblin token to target.
        let goblin = with_subtype_token(&t, "Goblin");
        t.g.objects[goblin.0 as usize].summoning_sick = false;
        attack_with(&mut t, &[(goblin, Entity::Player(P1))]);
        let p = if stolen { P1 } else { P0 };
        if stolen {
            steal_for_good(&mut t, P1, killer);
        }
        assert_eq!(can_activate(&mut t, p, killer), !stolen, "{stolen}");
    }
}

#[test]
fn vessel_of_malignity_the_opponent_chooses_the_cards() {
    cr!("608.2c", "406.3");
    ruling!(
        "Vessel of Malignity",
        "That player chooses two cards, exiles them face up, and then you get to see what was exiled."
    );
    supported("Vessel of Malignity");
    let mut t = TestGame::new(2);
    let vessel = t.battlefield(P0, "Vessel of Malignity");
    pool(&mut t, P0, &[ManaType::B, ManaType::C], 1);
    let cards = [
        t.hand(P1, "Grizzly Bears"),
        t.hand(P1, "Shock"),
        t.hand(P1, "Hill Giant"),
    ];
    t.answer_choose(P1, &[obj(cards[1]), obj(cards[2])]);
    let n = t.asked().len();
    act(&mut t, P0, vessel, "exiles two cards", &[Entity::Player(P1)]).expect("Vessel");
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_exile("Shock") && t.in_exile("Hill Giant"));
    let p0_chose = t.asked()[n..].iter().any(|(p, d)| {
        *p == P0 && matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })
    });
    assert!(!p0_chose);
    let shock = t.g.current(cards[1]);
    assert!(!t.g.obj(shock).face_down, "face up");
}

#[test]
fn connecting_the_dots_can_discard_an_empty_hand() {
    cr!("118.3", "701.9b");
    ruling!(
        "Connecting the Dots",
        "You can pay the cost of \"discard your hand\" even if your hand contains zero cards."
    );
    supported("Connecting the Dots");
    let mut t = TestGame::new(2);
    let dots = t.battlefield(P0, "Connecting the Dots");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let exiled = t.g.exile.len();
    assert_eq!(exiled, 1);
    assert_eq!(t.hand_size(P0), 0);
    pool(&mut t, P0, &[ManaType::R, ManaType::C], 1);
    act(&mut t, P0, dots, "Discard your hand", &[]).expect("empty hand");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn sinister_concoction_pays_all_its_costs() {
    cr!("601.2h", "602.2b");
    ruling!(
        "Sinister Concoction",
        "You may pay Sinister Concoction’s costs in any order."
    );
    supported("Sinister Concoction");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Sinister Concoction");
    t.lands(P0, "Swamp", 1);
    t.hand(P0, "Shock");
    let target = t.battlefield(P1, "Grizzly Bears");
    let lib = t.library_size(P0);
    act(&mut t, P0, sc, "Destroy target creature", &[obj(target)]).expect("Concoction");
    assert_eq!(t.library_size(P0), lib - 1);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P0), 19);
    assert!(t.in_graveyard(P0, "Sinister Concoction"));
    t.resolve_all();
    assert!(!t.on_battlefield(target));
}

#[test]
fn seal_of_removal_is_sacrificed_as_the_cost() {
    cr!("602.2b", "117.1c");
    ruling!(
        "Seal of Removal",
        "Sacrificing Seal of Removal is the cost to activate its ability."
    );
    supported("Seal of Removal");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let seal = cast_new(&mut t, P0, "Seal of Removal", &[]);
    t.resolve_all();
    let target = t.battlefield(P1, "Grizzly Bears");
    // P0 has priority right after the Seal resolves.
    assert_eq!(t.g.turn.priority, Some(P0));
    let seal = t.g.current(seal);
    act(&mut t, P0, seal, "Return target creature", &[obj(target)]).expect("Seal");
    assert!(t.in_graveyard(P0, "Seal of Removal"));
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------------
// Affected sets locked in on resolution
// ---------------------------------------------------------------------------------------

#[test]
fn night_of_the_sweets_revenge_affects_creatures_you_control_then() {
    cr!("611.2c");
    ruling!(
        "Night of the Sweets' Revenge",
        "Night of the Sweets' Revenge's last ability affects only creatures you control at the time it resolves."
    );
    supported("Night of the Sweets' Revenge");
    let mut t = TestGame::new(2);
    let night = t.battlefield(P0, "Night of the Sweets' Revenge");
    let early = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::create_token(&mut t, P0, "Food");
    pool(&mut t, P0, &[ManaType::G], 2);
    pool(&mut t, P0, &[ManaType::C], 5);
    act(&mut t, P0, night, "Creatures you control get", &[]).expect("Night");
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(early), (3, 3));
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn lightshield_array_affects_creatures_you_control_then() {
    cr!("611.2c");
    ruling!(
        "Invasion of Gobakhan // Lightshield Array",
        "The set of creatures affected by Lightshield Array’s last ability is determined as the ability resolves."
    );
    supported("Invasion of Gobakhan // Lightshield Array");
    let mut t = TestGame::new(2);
    let array = t.battlefield(P0, "Invasion of Gobakhan // Lightshield Array");
    // On the battlefield transformed.
    mtg_engine::dfc::transform(&mut t.g, array);
    t.g.recompute();
    assert_eq!(t.obj_now(array).chars.name, "Lightshield Array");
    let early = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, array, "Creatures you control gain", &[]).expect("Array");
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    use mtg_engine::keywords::KeywordKind;
    t.g.recompute();
    assert!(t.obj_now(early).chars.has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(late).chars.has_keyword(KeywordKind::Indestructible));
}

#[test]
fn suicidal_charge_affects_creatures_the_opponents_control_then() {
    cr!("611.2c", "508.1d");
    ruling!(
        "Suicidal Charge",
        "Suicidal Charge’s ability affects only the creatures your opponents control at the time it resolves."
    );
    supported("Suicidal Charge");
    let mut t = TestGame::new(2);
    let charge = t.battlefield(P0, "Suicidal Charge");
    let early = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::Upkeep);
    t.activate(P0, charge, 0, &[]).expect("Charge");
    t.resolve_all();
    let late = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(early), (2, 2));
    assert_eq!(t.pt(late), (3, 3));
    // P1 declares no attackers; the requirement makes the early Giant attack anyway.
    t.advance_to(P1, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().expect("combat");
    let attacking: Vec<ObjectId> = c.attackers.iter().map(|a| a.id).collect();
    assert!(attacking.contains(&early));
    assert!(!attacking.contains(&late));
}

// ---------------------------------------------------------------------------------------
// Other sacrifice effects
// ---------------------------------------------------------------------------------------

#[test]
fn last_light_of_durins_day_puts_a_dragon_permanent_card_onto_the_battlefield() {
    cr!("701.23a");
    ruling!(
        "Last Light of Durin's Day",
        "Only a Dragon permanent card can be put onto the battlefield this way."
    );
    supported("Last Light of Durin's Day");
    let mut t = TestGame::new(2);
    let ll = t.battlefield(P0, "Last Light of Durin's Day");
    put_counters(&mut t, ll, "quest", 5);
    let dragon = t.hand(P0, "Shivan Dragon");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(dragon)]);
    t.enter(P0, "Mountain");
    t.resolve_all();
    assert!(!t.on_battlefield(ll));
    assert_eq!(count_named(&t, "Shivan Dragon"), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn legacys_allure_rechecks_the_power_on_resolution() {
    cr!("608.2b", "115.1");
    ruling!(
        "Legacy's Allure",
        "If the creature’s power changes to be greater than this before the effect resolves, then the Legacy’s Allure triggered ability doesn’t resolve"
    );
    supported("Legacy's Allure");
    for pumped in [false, true] {
        let mut t = TestGame::new(2);
        let allure = t.battlefield(P0, "Legacy's Allure");
        put_counters(&mut t, allure, "treasure", 2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        act(&mut t, P0, allure, "Gain control", &[obj(bears)]).expect("Allure");
        if pumped {
            crate::r_p119_common::giant_growth(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(t.obj_now(bears).controller, if pumped { P1 } else { P0 });
        assert!(t.in_graveyard(P0, "Legacy's Allure"));
    }
}

#[test]
fn morcants_eyes_and_midnight_snack_count_x_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Morcant's Eyes",
        "In most cases, Morcant's Eyes will be in your graveyard at that time, so it will count toward the value of X."
    );
    ruling!(
        "Midnight Snack",
        "The value of X is determined only once, as Midnight Snack's last ability resolves."
    );
    supported("Morcant's Eyes");
    supported("Midnight Snack");
    let mut t = TestGame::new(2);
    let eyes = t.battlefield(P0, "Morcant's Eyes");
    t.graveyard(P0, "Llanowar Elves");
    pool(&mut t, P0, &[ManaType::G], 2);
    pool(&mut t, P0, &[ManaType::C], 4);
    act(&mut t, P0, eyes, "Create X", &[]).expect("Eyes");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2, "the Elves and Morcant's Eyes itself");
    let mut t = TestGame::new(2);
    let snack = t.battlefield(P0, "Midnight Snack");
    gain(&mut t, P0, 2);
    pool(&mut t, P0, &[ManaType::B], 1);
    pool(&mut t, P0, &[ManaType::C], 2);
    act(&mut t, P0, snack, "loses X life", &[Entity::Player(P1)]).expect("Snack");
    gain(&mut t, P0, 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 15, "life gained in response counts");
}

#[test]
fn gaeas_touch_and_uro_put_lands_without_playing_them() {
    cr!("305.4", "305.2");
    ruling!(
        "Gaea's Touch",
        "This means it does not count towards the number of lands played this turn"
    );
    ruling!(
        "Uro, Titan of Nature's Wrath",
        "Uro's effect doesn't count as playing a land."
    );
    supported("Gaea's Touch");
    supported("Uro, Titan of Nature's Wrath");
    let mut t = TestGame::new(2);
    let touch = t.battlefield(P0, "Gaea's Touch");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    act(&mut t, P0, touch, "basic Forest card", &[]).expect("Gaea's Touch");
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 0);
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).expect("land play still available");
    // Uro: after the land play, and on the opponent's turn.
    for opp_turn in [false, true] {
        let mut t = TestGame::new(2);
        if opp_turn {
            t.set_step(P1, Step::PrecombatMain);
        } else {
            let l = t.hand(P0, "Swamp");
            t.play_land(P0, l).expect("land");
        }
        let land = t.hand(P0, "Forest");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(land)]);
        t.enter(P0, "Uro, Titan of Nature's Wrath");
        t.resolve_all();
        assert!(t.on_battlefield(land), "{opp_turn}");
    }
}

#[test]
fn festival_of_embers_exiles_itself_and_stops_dies_triggers() {
    cr!("614.6", "700.4");
    ruling!(
        "Festival of Embers",
        "When Festival of Embers’s last ability resolves, it will be exiled."
    );
    ruling!(
        "Festival of Embers",
        "abilities that trigger whenever a creature you own dies won’t trigger"
    );
    supported("Festival of Embers");
    let mut t = TestGame::new(2);
    let festival = t.battlefield(P0, "Festival of Embers");
    t.battlefield(P1, "Blood Artist");
    let mine = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, mine);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.stack_len(), 0, "no dies trigger");
    // P1's creature dies normally.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, theirs);
    assert_eq!(triggers_on_stack(&t, "dies"), 1);
    t.resolve_all();
    pool(&mut t, P0, &[ManaType::R, ManaType::C], 1);
    act(&mut t, P0, festival, "Sacrifice", &[]).expect("Festival");
    t.resolve_all();
    assert!(t.in_exile("Festival of Embers"));
}

#[test]
fn call_to_arms_cant_choose_colorless() {
    cr!("105.1", "105.2c");
    ruling!("Call to Arms", "You can't choose \"colorless\" as a color.");
    supported("Call to Arms");
    let mut t = TestGame::new(2);
    let n = t.asked().len();
    t.enter(P0, "Call to Arms");
    let opts: Vec<Vec<String>> = t.asked()[n..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    let colors = opts
        .iter()
        .find(|o| o.iter().any(|s| s.contains("White") || s.contains("white")))
        .expect("a color choice");
    assert_eq!(colors.len(), 5, "{colors:?}");
    assert!(colors.iter().all(|s| !s.to_lowercase().contains("colorless")));
}

#[test]
fn contested_game_ball_in_two_headed_giant_you_choose_the_attacker() {
    cr!("810.2", "506.2");
    ruling!(
        "Contested Game Ball",
        "In a Two-Headed Giant game, each player on the attacking team is an attacking player."
    );
    supported("Contested Game Ball");
    let mut seen = vec![];
    for pick in [P2, P3] {
        let mut t = two_headed_giant();
        let ball = t.battlefield(P0, "Contested Game Ball");
        t.set_step(P2, Step::BeginningOfCombat);
        let attacker = t.battlefield(P2, "Grizzly Bears");
        attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
        t.answer_choose(P0, &[Entity::Player(pick)]);
        t.answer(P0, DecisionKind::Option, Answer::Index(if pick == P2 { 0 } else { 1 }));
        block_and_finish(&mut t, P0, &[]);
        t.resolve_all();
        let c = t.obj_now(ball).controller;
        assert!(c == P2 || c == P3, "an attacking player gained control");
        seen.push(c);
    }
    seen.sort();
    assert_eq!(seen, vec![P2, P3], "the choice is honored");
}
