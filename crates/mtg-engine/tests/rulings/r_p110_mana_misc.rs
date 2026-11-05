//! Rulings batch P110 — more abilities that add mana: "the third time this ability has
//! resolved this turn" (CR 707.10b, 608.2b), first main phases (CR 505.1a, 505.1b),
//! delayed mana (CR 603.7), triggered mana abilities (CR 605.1b, 605.4a), mana emptying
//! (CR 106.4, 500.5), and the conditions of a few mana triggers.

use crate::r_p110_common::*;
use crate::r_s01_common::attack_with;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// How many times a log line containing `text` was written.
fn logged(t: &TestGame, text: &str) -> usize {
    t.g.log.iter().filter(|l| l.text.contains(text)).count()
}

// --- Soulbright Flamekin and Soulbright Seeker -------------------------------------------

/// Activates `src`'s trample ability targeting `target`, paying from the pool (adding
/// `cost` first), and leaves it on the stack.
fn trample(t: &mut TestGame, src: ObjectId, cost: (ManaType, u32), target: ObjectId) -> ObjectId {
    mana(t, P0, cost.0, cost.1);
    let ab = act(t, P0, src, "trample", &[obj(target)])
        .unwrap()
        .expect("on the stack");
    assert!(t.g.stack.contains(&ab));
    ab
}

#[test]
fn soulbright_abilities_use_the_stack_and_pay_off_the_third_time_only() {
    cr!("605.5a", "608.2h", "117.7");
    ruling!(
        "Soulbright Flamekin",
        "Soulbright Flamekin's ability uses the stack and can be responded to. Because it has a target, it's not a mana ability, even if you know that it will produce mana."
    );
    ruling!(
        "Soulbright Seeker",
        "Because it has a target, Soulbright Seeker's last ability is not a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Soulbright Flamekin",
        "You add eight red mana only the third time Soulbright Flamekin's ability resolves in a turn. You won't get the bonus the fourth, fifth, sixth, or any subsequent times in that same turn."
    );
    ruling!(
        "Soulbright Seeker",
        "You add {R}{R}{R}{R} only the third time Soulbright Seeker's last ability resolves in a turn. You won't get the bonus mana the fourth, fifth, sixth, or any subsequent times in the same turn."
    );
    for (card, cost, bonus) in [
        ("Soulbright Flamekin", (ManaType::C, 2), 8u32),
        ("Soulbright Seeker", (ManaType::R, 1), 4),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, card);
        let bear = t.battlefield(P0, "Grizzly Bears");
        let mut after = vec![];
        for _ in 0..4 {
            t.answer_yes(P0, true);
            trample(&mut t, src, cost, bear);
            assert_eq!(pool_total(&t, P0), 0, "{card}: nothing before it resolves");
            t.resolve();
            after.push(pool_of(&t, P0, ManaType::R));
            mtg_engine::mana_abilities::empty_pool(&mut t.g, P0);
        }
        assert_eq!(after, vec![0, 0, bonus, 0], "{card}");
        t.g.recompute();
        assert!(has_kw(&t, bear, mtg_engine::keywords::KeywordKind::Trample));
    }
}

#[test]
fn soulbright_flamekin_counts_its_own_resolutions_and_copies() {
    cr!("707.10b", "608.2b");
    ruling!(
        "Soulbright Flamekin",
        "When Soulbright Flamekin's ability resolves, it counts the number of times that same ability from that same creature has already resolved that turn. A copy of this ability (created by Rings of Brighthearth, for example) will count toward the total. Abilities from other creatures with the same name don't count towards the total. Neither does an ability that didn't resolve because its target was illegal."
    );
    // Flamekin A resolves once; Flamekin B once; A's ability targeting a Bears that dies
    // in response doesn't resolve; A's second resolution adds nothing, its third adds 8.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Soulbright Flamekin");
    let b = t.battlefield(P0, "Soulbright Flamekin");
    let target = t.battlefield(P0, "Hill Giant");
    let doomed = t.battlefield(P0, "Grizzly Bears");
    let step = |t: &mut TestGame, src: ObjectId, tgt: ObjectId, kill: bool| {
        t.answer_yes(P0, true);
        trample(t, src, (ManaType::C, 2), tgt);
        if kill {
            destroy(t, tgt);
        }
        t.resolve_all();
        let r = pool_of(t, P0, ManaType::R);
        mtg_engine::mana_abilities::empty_pool(&mut t.g, P0);
        r
    };
    assert_eq!(step(&mut t, a, target, false), 0);
    assert_eq!(step(&mut t, b, target, false), 0);
    assert_eq!(step(&mut t, a, doomed, true), 0);
    assert_eq!(step(&mut t, a, target, false), 0);
    assert_eq!(step(&mut t, a, target, false), 8);
    // With Rings of Brighthearth copying the first activation, the second activation is
    // the third resolution.
    supported("Rings of Brighthearth");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Soulbright Flamekin");
    t.battlefield(P0, "Rings of Brighthearth");
    let target = t.battlefield(P0, "Hill Giant");
    // Rings: "you may pay {2}. If you do, copy that ability."
    mana(&mut t, P0, ManaType::C, 2);
    t.answer_yes(P0, true);
    crate::r_s25_common::keep_copy_targets(&mut t, P0);
    trample(&mut t, a, (ManaType::C, 2), target);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0, "two resolutions, no mana");
    // (No mana is left to pay for the Rings this time.)
    t.clear_answers();
    t.answer_yes(P0, true);
    trample(&mut t, a, (ManaType::C, 2), target);
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::R), 8);
}

// --- First main phases -------------------------------------------------------------------

/// P0's turn from the draw step: the first main phase, then Relentless Assault ("After
/// this main phase, there is an additional combat phase followed by an additional main
/// phase.") is cast in the postcombat main phase, then on to the end step.
fn three_main_phases(card: &str) -> TestGame {
    supported(card);
    let mut t = TestGame::new(2);
    t.battlefield(P0, card);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PostcombatMain);
    lands_for_cost(&mut t, P0, "Relentless Assault");
    let ra = t.hand(P0, "Relentless Assault");
    t.cast(P0, ra).go();
    t.resolve_all();
    assert_eq!(t.g.turn.main_phases, 2);
    t.advance_to(P0, Step::End);
    assert_eq!(t.g.turn.main_phases, 3);
    t
}

#[test]
fn first_main_phase_abilities_trigger_once_a_turn() {
    cr!("505.1a", "505.1b", "500.8");
    ruling!(
        "Eladamri's Vineyard",
        "It will not trigger a second time in a turn, even if you cast a spell that gives you a third main phase."
    );
    ruling!(
        "Eladamri's Vineyard",
        "Only the first main phase each turn is considered a precombat main phase, even if other combat phases are created (with Relentless Assault, for example)."
    );
    ruling!(
        "Blinkmoth Urn",
        "The \"precombat main phase\" is the first main phase of the turn. All other main phases are \"postcombat main phases.\""
    );
    for card in ["Eladamri's Vineyard", "Blinkmoth Urn"] {
        let t = three_main_phases(card);
        assert_eq!(logged(&t, &format!("Resolving {card}")), 1, "{card}");
    }
}

// --- Delayed mana ------------------------------------------------------------------------

#[test]
fn conduit_of_storms_mana_comes_even_if_it_left() {
    cr!("603.7", "603.7c");
    ruling!(
        "Conduit of Storms // Conduit of Emrakul",
        "You’ll add mana at the beginning of the next main phase whether or not Conduit of Storms (or Conduit of Emrakul) is still on the battlefield."
    );
    supported("Conduit of Storms // Conduit of Emrakul");
    let mut t = TestGame::new(2);
    let conduit = t.battlefield(P0, "Conduit of Storms // Conduit of Emrakul");
    attack_with(&mut t, &[(conduit, Entity::Player(P1))]);
    t.resolve_all();
    destroy(&mut t, conduit);
    assert!(!t.on_battlefield(conduit));
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::R), 1);
}

#[test]
fn mana_sculpts_delayed_mana() {
    cr!("608.2b", "603.7", "113.6g");
    ruling!(
        "Mana Sculpt",
        "If the target spell is an illegal target when Mana Sculpt tries to resolve, it won't resolve and none of its effects will happen. You won't add mana at the beginning of your next main phase. If the target is legal but not countered (probably because an effect says that spell can't be countered) and you control a Wizard, you'll still add mana."
    );
    supported("Mana Sculpt");
    // On P1's turn, P1 casts `spell` ({B}{G} Abrupt Decay, which can't be countered,
    // targeting P0's Ornithopter); P0, controlling Prodigal Sorcerer (a Wizard), casts
    // Mana Sculpt targeting it. With `fizzle`, P0 also counters the target first with
    // Counterspell. Returns the mana in P0's pool in P0's next precombat main phase.
    let run = |spell: &str, fizzle: bool| {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.battlefield(P0, "Prodigal Sorcerer");
        let thopter = t.battlefield(P0, "Ornithopter");
        let targets: Vec<Entity> = if spell == "Abrupt Decay" {
            vec![obj(thopter)]
        } else {
            vec![]
        };
        let s = cast_new(&mut t, P1, spell, &targets);
        let sculpt = cast_new(&mut t, P0, "Mana Sculpt", &[obj(s)]);
        if fizzle {
            cast_new(&mut t, P0, "Counterspell", &[obj(s)]);
            t.resolve();
            assert!(!t.g.stack.contains(&s));
        }
        t.resolve_all();
        assert_eq!(resolved(&t, sculpt), !fizzle);
        t.advance_to(P0, Step::PrecombatMain);
        t.resolve_all();
        pool_total(&t, P0)
    };
    assert_eq!(
        run("Abrupt Decay", false),
        2,
        "not countered, still the mana"
    );
    assert_eq!(run("Grizzly Bears", false), 2);
    assert_eq!(run("Grizzly Bears", true), 0, "illegal target: no mana");
}

// --- Triggered mana abilities and other triggers -----------------------------------------

#[test]
fn rose_triggers_for_any_junk_sacrifice() {
    cr!("605.1b", "605.4a", "603.2", "605.5a");
    interp!("mana-trigger-from-a-mana-ability-cost");
    ruling!(
        "Rose, Cutthroat Raider",
        "Rose's last ability triggers when you sacrifice a Junk for any reason, not just to activate a Junk's activated ability."
    );
    supported("Rose, Cutthroat Raider");
    supported("Krark-Clan Ironworks");
    // Sacrificed to Krark-Clan Ironworks' mana ability ("Sacrifice an artifact: Add
    // {C}{C}."): Rose's ability triggers from a mana ability, so it is one and adds {R}
    // at once (CR 605.1b).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rose, Cutthroat Raider");
    let kci = t.battlefield(P0, "Krark-Clan Ironworks");
    let junk = create_token(&mut t, P0, "Junk");
    t.answer_choose(P0, &[obj(junk)]);
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t,
        P0,
        kci,
        "Sacrifice"
    ));
    assert!(t.g.pending_triggers.is_empty());
    t.settle();
    assert!(t.g.stack.is_empty());
    assert_eq!(pool_of(&t, P0, ManaType::C), 2);
    assert_eq!(pool_of(&t, P0, ManaType::R), 1);
    // The same mana, while paying for a spell: Rose's {R} and the Ironworks' {C}{C} cast
    // Hill Giant ({3}{R}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rose, Cutthroat Raider");
    let kci = t.battlefield(P0, "Krark-Clan Ironworks");
    let junk = create_token(&mut t, P0, "Junk");
    t.lands(P0, "Wastes", 1);
    t.answer_choose(P0, &[obj(junk)]);
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t,
        P0,
        kci,
        "Sacrifice"
    ));
    let giant = t.hand(P0, "Hill Giant");
    t.cast(P0, giant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // A trigger with a target isn't a mana ability, even from a mana ability's cost:
    // Blood Artist's, when a creature is sacrificed to Ashnod's Altar.
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let altar = t.battlefield(P0, "Ashnod's Altar");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(b)]);
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t,
        P0,
        altar,
        "Sacrifice"
    ));
    t.settle();
    assert!(ability_on_stack(&t, artist));
    // Sacrificed for its own ability: the trigger uses the stack.
    let mut t = TestGame::new(2);
    let rose = t.battlefield(P0, "Rose, Cutthroat Raider");
    let junk = create_token(&mut t, P0, "Junk");
    t.library_top(P0, "Grizzly Bears");
    act(&mut t, P0, junk, "Exile the top card", &[]).unwrap();
    t.settle();
    assert!(ability_on_stack(&t, rose));
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool_of(&t, P0, ManaType::R), 1);
}

#[test]
fn rodeo_pyromancers_doesnt_trigger_on_the_turn_its_cast() {
    cr!("603.2", "603.6a");
    ruling!(
        "Rodeo Pyromancers",
        "That ability triggers only while Rodeo Pyromancers is on the battlefield. Notably, it won’t trigger on a turn you cast Rodeo Pyromancers, no matter which spell it was."
    );
    supported("Rodeo Pyromancers");
    // Cast as the first spell, then a second spell: no trigger either time.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Rodeo Pyromancers", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1, "just the Ornithopter");
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    // Cast as the second spell: the third spell doesn't trigger it either.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Rodeo Pyromancers", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn flamecache_gecko_cares_about_life_lost_not_net_change() {
    cr!("119.3", "603.4");
    ruling!(
        "Flamecache Gecko",
        "Flamecache Gecko’s ability cares whether an opponent lost life this turn, not how their life total changed. For example, an opponent who gained 2 life and lost 1 life in the same turn still lost life."
    );
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    let salve = {
        lands_for_cost(&mut t, P0, "Healing Salve");
        t.hand(P0, "Healing Salve")
    };
    t.cast(P0, salve)
        .modes(&[0])
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
    cast_new(&mut t, P0, "Flamecache Gecko", &[]);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 2);
    // No life lost: no mana.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Flamecache Gecko", &[]);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn iridescent_tiger_triggers_if_cast_from_anywhere() {
    cr!("603.4", "601.2");
    ruling!(
        "Iridescent Tiger",
        "Iridescent Tiger’s ability triggers if you cast it from any zone. It doesn’t trigger if you put Iridescent Tiger onto the battlefield without casting it."
    );
    supported("Iridescent Tiger");
    // Put onto the battlefield: nothing.
    let mut t = TestGame::new(2);
    t.enter(P0, "Iridescent Tiger");
    t.settle();
    assert!(t.g.stack.is_empty());
    // Cast from exile with Zuko's permission: the five mana.
    let mut t = TestGame::new(2);
    let zuko = t.battlefield(P0, "Zuko, Exiled Prince");
    let tiger = t.library_top(P0, "Iridescent Tiger");
    mana(&mut t, P0, ManaType::C, 3);
    act(&mut t, P0, zuko, "Exile the top card", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(tiger), mtg_engine::object::Zone::Exile);
    t.lands(P0, "Mountain", 5);
    t.cast(P0, t.g.current(tiger)).go();
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 5);
}

#[test]
fn bounty_of_the_luxa_gone_draws_without_counters() {
    cr!("400.7", "608.2h");
    ruling!(
        "Bounty of the Luxa",
        "If Bounty of the Luxa leaves the battlefield while its triggered ability is on the stack, you can't remove any flood counters from it, even if it had one before it left the battlefield. You won't put one on it as the ability resolves, but you will draw a card."
    );
    supported("Bounty of the Luxa");
    let mut t = TestGame::new(2);
    let bounty = t.battlefield(P0, "Bounty of the Luxa");
    t.g.objects[bounty.0 as usize]
        .counters
        .insert("flood".into(), 1);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert!(ability_on_stack(&t, bounty));
    destroy(&mut t, bounty);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(pool_total(&t, P0), 0);
    assert_eq!(t.counters(bounty, "flood"), 0);
}

#[test]
fn neheb_adds_mana_for_cards_discarded_not_drawn() {
    cr!("608.2c", "121.2");
    ruling!(
        "Neheb, Dreadhorde Champion",
        "Other effects may cause you to draw more or fewer cards than the number of cards you discarded. In these cases, the amount of mana you add is equal to the number of cards you discarded, not the number of cards you drew."
    );
    // With Spirit of the Labyrinth ("Each player can't draw more than one card each
    // turn."), P0 discards two cards and draws one: {R}{R}.
    supported("Spirit of the Labyrinth");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Spirit of the Labyrinth");
    let neheb = t.battlefield(P0, "Neheb, Dreadhorde Champion");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    attack_with(&mut t, &[(neheb, Entity::Player(P1))]);
    let library = t.library_size(P0);
    t.advance_to(P0, Step::CombatDamage);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(library - t.library_size(P0), 1);
    assert_eq!(pool_of(&t, P0, ManaType::R), 2);
}

#[test]
fn radhas_mana_empties_before_blockers() {
    cr!("106.4", "500.5");
    ruling!(
        "Radha, Heir to Keld",
        "You lose the mana produced by Radha's ability before blockers are declared if you don't spend it first."
    );
    let mut t = TestGame::new(2);
    let radha = t.battlefield(P0, "Radha, Heir to Keld");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(radha, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::R), 2);
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn carpet_of_flowers_may_target_a_different_opponent_each_turn() {
    cr!("115.1", "603.3d", "603.4");
    ruling!(
        "Carpet of Flowers",
        "You can target a different opponent each turn."
    );
    supported("Carpet of Flowers");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Carpet of Flowers");
    t.lands(P1, "Island", 2);
    t.lands(P2, "Island", 1);
    let mut pools = vec![];
    t.set_step(P0, Step::Draw);
    for opp in [P1, P2] {
        t.answer_targets(P0, &[Entity::Player(opp)]);
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::PrecombatMain);
        t.resolve_all();
        pools.push(pool_total(&t, P0));
        t.advance_to(P1, Step::Upkeep);
    }
    assert_eq!(pools, vec![2, 1]);
}

#[test]
fn misty_salons_x_is_fixed_as_it_resolves() {
    cr!("608.2h", "709.5h");
    ruling!(
        "Smoky Lounge // Misty Salon",
        "The value of X is calculated only once, as Misty Salon's ability resolves. Changes in the number of unlocked doors among Rooms you control that occur after the ability resolves won't affect the power or toughness of the token."
    );
    supported("Smoky Lounge // Misty Salon");
    let mut t = TestGame::new(2);
    let room = t.enter(P0, "Smoky Lounge // Misty Salon");
    t.settle();
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
    t.g.flush_events();
    t.resolve_all();
    let spirit = tokens(&t, P0)[0];
    assert_eq!(t.pt(spirit), (1, 1));
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 0, P0));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(spirit), (1, 1));
}
