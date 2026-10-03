//! Rulings batch S22 — what happens as and after a spell is cast: abilities that trigger
//! on casting a spell go on the stack above it and resolve first (CR 601.2i, 603.3);
//! a spell that constitutes a crime has committed it once it's cast (CR 700.13); after a
//! spell resolves, its controller (the active player) receives priority first
//! (CR 117.3b); abilities that trigger during a resolving spell wait until it has
//! finished resolving (CR 603.3), including while a spell is cast during that resolution
//! (CR 608.2g).

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// The uid of the first activated ability of `source` (followed to its current object).
fn first_activated(t: &mut TestGame, source: ObjectId) -> (ObjectId, u64) {
    t.g.recompute();
    let s = t.g.current(source);
    let uid = t
        .g
        .obj(s)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.uid)
        .expect("an activated ability");
    (s, uid)
}

/// Whether the object on the stack is a triggered ability.
fn is_trigger(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(id)
        .stack
        .as_ref()
        .is_some_and(|si| matches!(si.kind, StackKind::Triggered { .. }))
}

#[test]
fn brain_weevil_its_controller_gets_priority_first_after_it_resolves() {
    cr!("117.3b", "117.3a", "602.5d");
    ruling!(
        "Brain Weevil",
        "If you cast this as normal during your main phase, it will enter the battlefield and you’ll receive priority. If no abilities trigger because of this, you can activate its ability immediately, before any other player has a chance to remove it from the battlefield."
    );
    supported("Brain Weevil");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.hand(P1, "Forest");
    t.hand(P1, "Island");
    let weevil = t.hand(P0, "Brain Weevil");
    // P0 casts it through the priority loop; both players pass; it resolves.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: weevil,
            method: CastMethod::Normal,
        }),
    );
    let ok = t
        .g
        .run_until(1000, |g| !g.find_in_zone(Zone::Battlefield, "Brain Weevil").is_empty());
    assert!(ok);
    // P0 (the active player) receives priority, with an empty stack in their main phase.
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.g.turn.stage, Stage::Priority);
    assert!(t.g.stack.is_empty());
    let from = t.asked().len();
    // P0 activates it ("Sacrifice this creature: Target player discards two cards.
    // Activate only as a sorcery.") before P1 gets priority.
    let (w, uid) = first_activated(&mut t, weevil);
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Activate {
            source: w,
            ability: uid,
        }),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ok = t.g.run_until(1000, |g| g.stack.len() == 1);
    assert!(ok);
    assert!(t.in_graveyard(P0, "Brain Weevil"));
    let p1_priority_before = t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. }));
    assert!(!p1_priority_before);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn whirlwind_of_thought_players_may_respond_after_the_trigger_resolves() {
    cr!("603.3", "601.2i", "117.3b", "117.3c");
    ruling!(
        "Whirlwind of Thought",
        "Players can cast spells and activate abilities after the triggered ability resolves but before the spell that caused it to trigger does."
    );
    supported("Whirlwind of Thought");
    // "Whenever you cast a noncreature spell, draw a card." P0 casts Shock at P1's
    // Grizzly Bears; the trigger resolves first (P0 draws); then P1 responds to Shock
    // with Giant Growth, and the Bears survives.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Whirlwind of Thought");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Forest", 1);
    let growth = t.hand(P1, "Giant Growth");
    let shock = t.hand(P0, "Shock");
    let hand = t.hand_size(P0);
    // P1 passes while the trigger is on the stack, then casts Giant Growth.
    t.answer(P1, DecisionKind::Priority, Answer::Action(Action::Pass));
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: growth,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: shock,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[Entity::Object(bears)]);
    // Run until Giant Growth is on the stack.
    let ok = t.g.run_until(1000, |g| {
        g.stack
            .iter()
            .any(|s| g.obj(*s).chars.name == "Giant Growth")
    });
    assert!(ok);
    // The trigger already resolved (P0 drew); Shock is below Giant Growth.
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    let names: Vec<String> = t
        .g
        .stack
        .iter()
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect();
    assert_eq!(names, vec!["Shock", "Giant Growth"]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn opal_archangel_becomes_a_creature_before_the_spell_resolves() {
    cr!("603.2", "601.2i", "603.4");
    ruling!(
        "Opal Archangel",
        "It triggers when the spell is cast, which means it becomes a creature before that spell resolves."
    );
    supported("Opal Archangel");
    // "When an opponent casts a creature spell, if this permanent is an enchantment, it
    // becomes a 5/5 Angel creature with flying and vigilance."
    let mut t = TestGame::new(2);
    let opal = t.battlefield(P0, "Opal Archangel");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.g.resolve_top();
    t.settle();
    // The Bears spell is still on the stack; Opal Archangel is a 5/5 creature.
    assert!(t.g.stack.contains(&spell));
    t.g.recompute();
    assert!(t.obj_now(opal).is(CardType::Creature));
    assert_eq!(t.pt(opal), (5, 5));
}

#[test]
fn bontus_monument_triggers_for_any_creature_spell_but_reduces_only_black_ones() {
    cr!("601.2f", "603.2", "601.2i");
    ruling!(
        "Bontu's Monument",
        "Each Monument has one ability that reduces the cost of creature spells of a certain color, and a triggered ability that triggers whenever you cast any creature spell—not just a creature spell of that color."
    );
    supported("Bontu's Monument");
    // "Black creature spells you cast cost {1} less to cast. Whenever you cast a creature
    // spell, each opponent loses 1 life and you gain 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bontu's Monument");
    // A green creature spell: full cost, but it triggers.
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    // A black creature spell costs {1} less and triggers too.
    t.lands(P0, "Swamp", 4);
    let weevil = t.hand(P0, "Brain Weevil");
    t.cast(P0, weevil).go();
    assert_eq!(tapped_lands(&t, P0), 2 + 3);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn doomskar_oracle_counts_spells_cast_before_it_entered() {
    cr!("603.2", "603.2c");
    ruling!(
        "Doomskar Oracle",
        "The triggered ability triggers only if the creature that has it is on the battlefield as you cast your second spell. Spells you cast in a turn before that creature entered the battlefield will count. In other words, the ability won’t trigger if the creature with the ability is the second spell you cast during a turn or if you have already cast two or more spells by the time that creature enters the battlefield that turn."
    );
    supported("Doomskar Oracle");
    // "Whenever you cast your second spell each turn, you gain 2 life."
    let cast = |t: &mut TestGame, name: &str| {
        give_mana_for(t, P0, name);
        let c = t.hand(P0, name);
        t.cast(P0, c).go();
        t.resolve_all();
    };
    // The Oracle is the first spell: the second one triggers it.
    let mut t = TestGame::new(2);
    cast(&mut t, "Doomskar Oracle");
    cast(&mut t, "Grizzly Bears");
    assert_eq!(t.life(P0), 22);
    cast(&mut t, "Llanowar Elves");
    assert_eq!(t.life(P0), 22);
    // The Oracle is the second spell: it isn't on the battlefield as it's cast.
    let mut t = TestGame::new(2);
    cast(&mut t, "Grizzly Bears");
    cast(&mut t, "Doomskar Oracle");
    cast(&mut t, "Llanowar Elves");
    assert_eq!(t.life(P0), 20);
    // Two spells cast before it entered (it enters without being cast): no trigger.
    let mut t = TestGame::new(2);
    cast(&mut t, "Grizzly Bears");
    cast(&mut t, "Llanowar Elves");
    t.enter(P0, "Doomskar Oracle");
    t.settle();
    cast(&mut t, "Hill Giant");
    assert_eq!(t.life(P0), 20);
    // One spell cast before it entered: the next spell is the second.
    let mut t = TestGame::new(2);
    cast(&mut t, "Grizzly Bears");
    t.enter(P0, "Doomskar Oracle");
    t.settle();
    cast(&mut t, "Llanowar Elves");
    assert_eq!(t.life(P0), 22);
}

#[test]
fn dreamstalker_manticore_triggers_only_on_the_very_first_spell_in_an_opponents_turn() {
    cr!("603.2", "603.2c");
    ruling!(
        "Dreamstalker Manticore",
        "This ability triggers only on your very first spell during an opponent’s turn, not the first spell after the card is on the battlefield. If you cast a spell before it’s on the battlefield (including if you cast this card somehow during an opponent’s turn), the ability won’t trigger."
    );
    supported("Dreamstalker Manticore");
    // "Whenever you cast your first spell during each opponent's turn, this creature
    // deals 1 damage to any target."
    let opt = |t: &mut TestGame| {
        add_mana(t, P0, ManaType::U, 1);
        let c = t.hand(P0, "Opt");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P0, c).go();
        t.resolve_all();
        t.clear_answers();
    };
    // On the battlefield: the first spell in P1's turn triggers it, the second doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dreamstalker Manticore");
    t.set_step(P1, Step::Upkeep);
    opt(&mut t);
    assert_eq!(t.life(P1), 19);
    opt(&mut t);
    assert_eq!(t.life(P1), 19);
    // A spell cast in P1's turn before it's on the battlefield: no trigger afterward.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    opt(&mut t);
    t.enter(P0, "Dreamstalker Manticore");
    t.settle();
    opt(&mut t);
    assert_eq!(t.life(P1), 20);
    // In P0's own turn: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dreamstalker Manticore");
    opt(&mut t);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn deepmuck_desperado_a_crime_is_committed_as_the_spell_is_cast() {
    cr!("700.13", "601.2i", "603.2");
    ruling!(
        "Deepmuck Desperado",
        "The spell or ability that constituted a crime doesn’t have to have resolved yet or at all. As soon as you’re finished casting the spell, activating the ability, or putting the triggered ability on the stack, you’ve committed a crime."
    );
    supported("Deepmuck Desperado");
    // "Whenever you commit a crime, each opponent mills three cards."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deepmuck Desperado");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    // The trigger is on the stack above Shock right away.
    assert_eq!(t.stack_len(), 2);
    assert!(is_trigger(&t, *t.g.stack.last().unwrap()));
    // Shock is countered: the crime was still committed.
    assert!(t.g.counter(spell, None));
    let lib = t.library_size(P1);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 3);
    assert_eq!(t.life(P1), 20);
    // Activating an ability that targets an opponent's creature is a crime too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deepmuck Desperado");
    let capsule = t.battlefield(P0, "Executioner's Capsule");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let lib = t.library_size(P1);
    t.activate(P0, capsule, 0, &[Entity::Object(bears)]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(is_trigger(&t, *t.g.stack.last().unwrap()));
    t.resolve();
    assert_eq!(t.library_size(P1), lib - 3);
    assert!(t.on_battlefield(bears));
}

#[test]
fn vadmir_a_crime_is_committed_even_if_the_spell_never_resolves() {
    cr!("700.13", "601.2i");
    ruling!(
        "Vadmir, New Blood",
        "The spell or ability that constituted a crime doesn't have to have resolved yet or at all. As soon as you're finished casting the spell, activating the ability, or putting the triggered ability on the stack, you've committed a crime."
    );
    supported("Vadmir, New Blood");
    // "Whenever you commit a crime, put a +1/+1 counter on Vadmir."
    let mut t = TestGame::new(2);
    let vadmir = t.battlefield(P0, "Vadmir, New Blood");
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    assert!(t.g.counter(spell, None));
    t.resolve_all();
    assert_eq!(t.counters(vadmir, "+1/+1"), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn sram_expertise_triggers_wait_until_the_free_spell_is_cast() {
    cr!("603.3", "603.3b", "608.2g", "101.4");
    ruling!(
        "Sram's Expertise",
        "Any triggered abilities that trigger while performing the Expertise spell's first effect won't be put onto the stack until after you're done casting your free spell. They're put onto the stack at the same time as any abilities that triggered while casting that spell regardless of the order in which those abilities triggered."
    );
    // Soul Warden triggers for each of the three Servos; Whirlwind of Thought triggers
    // as the free noncreature spell (Shock) is cast. All go on the stack together, above
    // Shock, in the order P0 chooses.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Whirlwind of Thought");
    add_mana(&mut t, P0, ManaType::W, 4);
    let shock = t.hand(P0, "Shock");
    let expertise = t.hand(P0, "Sram's Expertise");
    t.answer_choose(P0, &[Entity::Object(shock)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, expertise).go();
    // Whirlwind of Thought's trigger for Sram's Expertise itself resolves first.
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.g.resolve_top();
    t.settle();
    // Shock at the bottom, then four triggered abilities.
    let names: Vec<(String, bool)> = t
        .g
        .stack
        .iter()
        .map(|s| (t.g.obj(*s).chars.name.to_string(), is_trigger(&t, *s)))
        .collect();
    assert_eq!(names.len(), 5, "{names:?}");
    assert_eq!(names[0], ("Shock".to_string(), false));
    assert!(names[1..].iter().all(|(_, trig)| *trig));
    // They were ordered together: one ordering decision covering all four.
    let orders: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::Order { prompt, items } if *p == P0 && prompt.contains("triggered") => {
                Some(items.len())
            }
            _ => None,
        })
        .collect();
    assert_eq!(orders, vec![4]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P0, "Shock"));
}
