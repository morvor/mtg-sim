//! Rulings batch P008 — burn that can hit planeswalkers, and triggered abilities: Annie
//! Joins Up's extra triggers (CR 603.2, 603.3) and linked abilities with several exiled
//! cards (CR 607.2), Saga chapters with reflexive triggers (CR 603.12) and base power and
//! toughness in timestamp order (CR 613.4b, 613.7), Spellgorger Weird tokens (CR 111.4),
//! and Snapdax's mutate trigger with no legal target (CR 603.3d, 608.2b).

use crate::r_p007_common::*;
use crate::r_p113_common::{spoil, Spoil};
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::run_from;
use crate::r_s19_common::add_lore;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// P0 controls Annie Joins Up (its own enters trigger not used).
fn annie(t: &mut TestGame) {
    supported("Annie Joins Up");
    t.battlefield(P0, "Annie Joins Up");
}

#[test]
fn annie_joins_up_checks_legendary_as_the_ability_triggers() {
    cr!("603.2", "603.3", "205.4d");
    ruling!(
        "Annie Joins Up",
        "Once an ability of a creature you control has triggered, making it legendary won’t cause that ability to trigger an additional time. Similarly, once an ability of a legendary creature you control has triggered an additional time due to Annie Joins Up’s last ability, turning it into a nonlegendary creature won’t remove that additional triggered ability from the stack."
    );
    // Elvish Visionary ("When this creature enters, draw a card.") enters nonlegendary;
    // then Leyline of Singularity makes it legendary: one trigger.
    let mut t = TestGame::new(2);
    annie(&mut t);
    t.enter(P0, "Elvish Visionary");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Leyline of Singularity");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Legendary as it enters (Leyline of Singularity): two triggers; it then stops being
    // legendary (the Leyline is destroyed): still two.
    let mut t = TestGame::new(2);
    annie(&mut t);
    let leyline = t.battlefield(P0, "Leyline of Singularity");
    t.enter(P0, "Elvish Visionary");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, leyline);
    assert_eq!(t.stack_len(), 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn annie_joins_up_doesnt_affect_replacement_effects() {
    cr!("614.1c", "614.12");
    ruling!(
        "Annie Joins Up",
        "Replacement effects are unaffected by Annie Joins Up’s last ability. For example, a 1/1 creature that enters the battlefield under your control with one +1/+1 counter on it won’t receive an additional +1/+1 counter."
    );
    // Hellhole Flailer (unleash), legendary thanks to Leyline of Singularity.
    let mut t = TestGame::new(2);
    annie(&mut t);
    t.battlefield(P0, "Leyline of Singularity");
    t.answer_yes(P0, true);
    let f = t.enter(P0, "Hellhole Flailer");
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(f, counters::PLUS1), 1);
}

#[test]
fn annie_joins_up_two_exiled_cards_for_a_linked_ability() {
    cr!("607.2a", "607.3");
    ruling!(
        "Annie Joins Up",
        "In some cases involving linked abilities, an ability requires information about “the exiled card.” When this happens, the ability gets multiple answers. If these answers are being used to determine the value of a variable, the sum is used. For example, if Elite Arcanist’s enters-the-battlefield ability triggers twice, two cards are exiled. The value of X in the activation cost of Elite Arcanist’s other ability is the sum of the two cards’ mana values. As the ability resolves, you create copies of both cards and can cast none, one, or both of the copies in any order."
    );
    // Elite Arcanist, legendary thanks to Leyline of Singularity: its enters trigger
    // triggers twice; Lightning Bolt (1) and Searing Spear (2) are exiled.
    let mut t = TestGame::new(2);
    annie(&mut t);
    t.battlefield(P0, "Leyline of Singularity");
    let bolt = t.hand(P0, "Lightning Bolt");
    let spear = t.hand(P0, "Searing Spear");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(spear)]);
    let arc = t.enter(P0, "Elite Arcanist");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Searing Spear"));
    t.g.objects[arc.0 as usize].summoning_sick = false;
    // X is 3: two lands aren't enough.
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    assert!(t.activate(P0, arc, 0, &[]).is_err(), "X = 1 + 2");
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.activate(P0, arc, 0, &[]).expect("Elite Arcanist");
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 3, "both copies cast");
}

#[test]
fn ral_and_the_implicit_maze_makes_a_spellgorger_weird() {
    cr!("714.2b", "111.4");
    ruling!(
        "Ral and the Implicit Maze",
        "Ral and the Implicit Maze's last ability creates a token that's a copy of the card Spellgorger Weird in the Oracle card reference. Official text for Spellgorger Weird can be found using the Gatherer card database at Gatherer.Wizards.com."
    );
    supported("Ral and the Implicit Maze");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Ral and the Implicit Maze");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 2);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    let weird = t.named_on_battlefield("Spellgorger Weird");
    assert_eq!(weird.len(), 1);
    let w = weird[0];
    assert!(t.obj_now(w).is_token());
    assert_eq!(t.pt(w), (2, 2));
    assert_eq!(t.g.mana_value_of(w), 3, "{{2}}{{R}}");
    // "Whenever you cast a noncreature spell, put a +1/+1 counter on Spellgorger Weird."
    cast_x(&mut t, P0, "Shock", NORMAL, None, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.pt(w), (3, 3));
}

#[test]
fn bears_of_littjara_base_power_and_toughness_in_timestamp_order() {
    cr!("613.4b", "613.7a");
    ruling!(
        "The Bears of Littjara",
        "The chapter II ability overwrites any previous effects that set any of the Shapeshifters’ base power and toughness to specific values. Any effects that set a Shapeshifter’s base power and toughness that start to apply after the chapter II ability will override the effect of that ability."
    );
    supported("The Bears of Littjara");
    let set = |t: &mut TestGame, id: ObjectId, p: i32, tough: i32| {
        run_from(
            t,
            P0,
            None,
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::SetPT(
                    Some(Value::Const(p)),
                    Some(Value::Const(tough)),
                )],
                duration: Duration::EndOfTurn,
            },
            &[obj(id)],
        );
    };
    let mut t = TestGame::new(2);
    let outcast = t.battlefield(P0, "Changeling Outcast");
    set(&mut t, outcast, 1, 6);
    assert_eq!(t.pt(outcast), (1, 6));
    let saga = t.battlefield(P0, "The Bears of Littjara");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    t.answer_targets(P0, &[obj(outcast)]);
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_eq!(t.pt(outcast), (4, 4), "overwrites the earlier effect");
    set(&mut t, outcast, 0, 2);
    assert_eq!(t.pt(outcast), (0, 2), "a later effect overrides it");
}

#[test]
fn invasion_of_the_giants_reflexive_trigger_can_be_responded_to() {
    cr!("603.12", "714.2b");
    ruling!(
        "Invasion of the Giants",
        "The chapter II ability goes on the stack without a target. You choose whether to reveal a Giant card from your hand after you have drawn a card. When you do, the reflexive triggered ability triggers. This is different than effects that say \"If you do\" in that players can respond after you've revealed the Giant card but before damage is dealt."
    );
    supported("Invasion of the Giants");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Invasion of the Giants");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), 1);
    t.hand(P0, "Hill Giant");
    add_lore(&mut t, saga, 1);
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    t.settle();
    assert_eq!(t.hand_size(P0), hand + 1, "drew first");
    assert_eq!(t.stack_len(), 1, "the reflexive trigger waits on the stack");
    assert_eq!(t.life(P1), 20);
    // P1 responds: Healing Salve's shield on P1 prevents the 2 damage.
    crate::r_p113_common::cast_modes(&mut t, P1, "Healing Salve", &[1], &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

/// P0 casts Snapdax for its mutate cost onto `onto`, putting it on top.
fn cast_snapdax(t: &mut TestGame, onto: ObjectId) -> ObjectId {
    let card = t.hand(P0, "Snapdax, Apex of the Hunt");
    for ty in [ManaType::B, ManaType::W, ManaType::W, ManaType::C, ManaType::C] {
        t.g.players[0].mana_pool.add_type(ty, 1);
    }
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(onto)
        .go()
}

#[test]
fn snapdax_mutate_trigger_needs_a_legal_target() {
    cr!("603.3d", "608.2b", "702.140d");
    ruling!(
        "Snapdax, Apex of the Hunt",
        "If there is no legal target for Snapdax’s triggered ability when it triggers, it won’t be put onto the stack at all and you won’t gain 4 life. Similarly, if the target creature or planeswalker is an illegal target by the time the ability tries to resolve, the ability won’t resolve and you won’t gain 4 life."
    );
    supported("Snapdax, Apex of the Hunt");
    // No creature or planeswalker of an opponent: no trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_snapdax(&mut t, bears);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
    // The target becomes illegal: no life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    cast_snapdax(&mut t, bears);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    spoil(&mut t, wurm, Spoil::Hexproof);
    t.resolve_all();
    assert_eq!((t.life(P0), dmg(&t, wurm)), (20, 0));
    // Legal: 4 damage and 4 life.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Indomitable Ancients");
    t.answer_targets(P0, &[obj(wurm)]);
    cast_snapdax(&mut t, bears);
    t.resolve_all();
    assert_eq!((t.life(P0), dmg(&t, wurm)), (24, 4));
}
