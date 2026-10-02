//! Rulings batch P146 — repeatable scry engines: triggers on casting resolve before the
//! spell, even if it's countered (CR 601.2i, 603.3, 405.5); "whenever ~ becomes tapped" is
//! a triggered ability, not a way to tap it (CR 603.1); simultaneous deaths and entries
//! (CR 603.10a, 603.6a); "choose one that hasn't been chosen this turn"; cost reductions
//! (CR 601.2f); casting "without paying its mana cost" (CR 118.9); last known
//! information (CR 113.7a, 608.2h); losing haste (CR 302.6, 506.4).

use crate::r_p146_common::*;
use crate::r_s02_common::can_attack;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

#[test]
fn thassa_unblockable_after_blocks_doesnt_undo_the_block() {
    cr!("509.1h", "506.4");
    ruling!(
        "Thassa, God of the Sea",
        "Activating Thassa's last ability after the target creature has been blocked won't change or undo the block."
    );
    supported("Thassa, God of the Sea");
    // "{1}{U}: Target creature you control can't be blocked this turn."
    let mut t = TestGame::new(2);
    let th = t.battlefield(P0, "Thassa, God of the Sea");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, bears)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    activate_resolve(&mut t, P0, th, 0, &[obj(bears)]);
    t.advance_to(P0, Step::EndOfCombat);
    // Still blocked: the Giant killed it and P1 took no damage.
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn becomes_tapped_abilities_dont_let_you_tap_the_creature() {
    cr!("603.1", "602.1");
    ruling!(
        "Compassionate Healer",
        "Compassionate Healer's triggered ability doesn't allow you to tap Compassionate Healer. You have to find some other way to tap it, such as paying a waterbend cost or attacking."
    );
    ruling!(
        "Samite Herbalist",
        "Samite Herbalist’s ability is a triggered ability, not an activated ability. It doesn’t allow you to tap it whenever you want; rather, you need some other way of tapping it, such as by attacking."
    );
    // "Whenever this creature becomes tapped, you gain 1 life and scry 1."
    for name in ["Compassionate Healer", "Samite Herbalist"] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        assert!(!can_activate(&mut t, P0, c), "{name}");
        // Attacking taps it.
        attack_with(&mut t, &[(c, Entity::Player(P1))]);
        let from = n_asked(&t);
        t.resolve_all();
        assert_eq!(t.life(P0), 21, "{name}");
        assert_eq!(scry_sizes(&t, P0, from), vec![1], "{name}");
    }
}

#[test]
fn samite_herbalist_tapped_while_casting_triggers_above_the_spell() {
    cr!("603.3", "702.51a", "601.2i");
    ruling!(
        "Samite Herbalist",
        "If Samite Herbalist becomes tapped while casting a spell or activating an ability, that spell or ability resolves after Samite Herbalist’s triggered ability has resolved."
    );
    supported("Stoke the Flames");
    // Stoke the Flames ({2}{R}{R}, convoke): the Herbalist pays {1}.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Samite Herbalist");
    t.lands(P0, "Mountain", 3);
    let stoke = t.hand(P0, "Stoke the Flames");
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![obj(h)]));
    let spell = t.cast(P0, stoke).target(Entity::Player(P1)).go();
    assert!(t.obj(h).tapped);
    t.settle();
    // The trigger is above the spell.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], spell);
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (21, 20));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

/// Casts `name` for P0 (lands for its cost) with the given targets, checks that a
/// triggered ability went on the stack above it, then counters the spell and resolves the
/// rest of the stack.
fn cast_and_counter(t: &mut TestGame, name: &str, targets: &[Entity]) {
    let spell = cast_new(t, P0, name, targets);
    t.settle();
    assert_eq!(t.g.stack[0], spell, "{name}");
    assert_eq!(stacked_triggers(t), 1, "{name}");
    counter(t, spell);
    t.resolve_all();
}

#[test]
fn cast_triggers_resolve_before_the_spell_even_if_its_countered() {
    cr!("601.2i", "603.3", "405.5", "701.6a");
    ruling!(
        "Djinn of the Fountain",
        "Djinn of the Fountain's triggered ability will resolve before the instant or sorcery spell that caused it to trigger resolves. It will resolve even if the spell that caused it to trigger is countered or has otherwise left the stack."
    );
    ruling!(
        "Lifecrafter's Bestiary",
        "Lifecrafter's Bestiary's second triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    ruling!(
        "Season of Growth",
        "Season of Growth’s last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Tenth District Legionnaire",
        "Tenth District Legionnaire's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Poetic Ingenuity",
        "Poetic Ingenuity's last ability resolves before the spell that caused it to trigger. You create a Dinosaur token even if the artifact spell is countered."
    );
    // Djinn of the Fountain: "Whenever you cast an instant or sorcery spell, choose one —
    // • This creature gets +1/+1 until end of turn. • ... • Scry 1."
    supported("Djinn of the Fountain");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Djinn of the Fountain");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    cast_and_counter(&mut t, "Lightning Bolt", &[Entity::Player(P1)]);
    assert_eq!(t.pt(d), (5, 5));
    assert_eq!(t.life(P1), 20);
    // Lifecrafter's Bestiary: "Whenever you cast a creature spell, you may pay {G}. If you
    // do, draw a card."
    supported("Lifecrafter's Bestiary");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lifecrafter's Bestiary");
    t.lands(P0, "Forest", 1);
    yes(&mut t, P0);
    let hand = t.hand_size(P0);
    cast_and_counter(&mut t, "Grizzly Bears", &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Season of Growth: "Whenever you cast a spell that targets a creature you control,
    // draw a card."
    supported("Season of Growth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Season of Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_and_counter(&mut t, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.pt(bears), (2, 2));
    // Tenth District Legionnaire: "Whenever you cast a spell that targets this creature,
    // put a +1/+1 counter on this creature, then scry 1."
    supported("Tenth District Legionnaire");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Tenth District Legionnaire");
    cast_and_counter(&mut t, "Giant Growth", &[obj(l)]);
    assert_eq!(t.pt(l), (3, 3));
    // Poetic Ingenuity: "Whenever you cast an artifact spell, create a 3/1 red Dinosaur
    // creature token."
    supported("Poetic Ingenuity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Poetic Ingenuity");
    cast_and_counter(&mut t, "Ornithopter", &[]);
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Dinosaur"))
            .count(),
        1
    );
    assert!(t.named_on_battlefield("Ornithopter").is_empty());
}

#[test]
fn lifecrafters_bestiary_draws_only_one_card() {
    cr!("118.12", "608.2c");
    ruling!(
        "Lifecrafter's Bestiary",
        "While resolving Lifecrafter's Bestiary's second triggered ability, you can't pay {G} multiple times to draw multiple cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lifecrafter's Bestiary");
    t.lands(P0, "Forest", 5);
    yes(&mut t, P0);
    yes(&mut t, P0);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    // One card drawn, and one Forest tapped for it (plus two for the Bears).
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
}

#[test]
fn season_of_growth_triggers_once_per_spell() {
    cr!("603.2c", "115.1");
    ruling!(
        "Season of Growth",
        "Season of Growth’s last ability triggers when you cast a spell that has multiple targets, as long as at least one of those targets is a creature you control."
    );
    supported("Rabid Bite");
    supported("Seeds of Strength");
    // Rabid Bite: "Target creature you control deals damage equal to its power to target
    // creature you don't control." One draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Season of Growth");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Rabid Bite", &[obj(giant), obj(bears)]);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Seeds of Strength targeting the same creature three times: one trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Season of Growth");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(
        &mut t,
        P0,
        "Seeds of Strength",
        &[obj(giant), obj(giant), obj(giant)],
    );
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    // Targeting two different creatures P0 controls: still one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Season of Growth");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Seeds of Strength", &[obj(a), obj(b), obj(a)]);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
}

#[test]
fn alibou_leaving_takes_haste_away() {
    cr!("702.10b", "508.1a", "506.4", "611.3a");
    ruling!(
        "Alibou, Ancient Witness",
        "If Alibou leaves the battlefield, artifact creatures you control lose haste."
    );
    supported("Alibou, Ancient Witness");
    // "Other artifact creatures you control have haste."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Alibou, Ancient Witness");
    let o = t.battlefield_sick(P0, "Ornithopter");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, o));
    destroy(&mut t, a);
    assert!(!can_attack(&mut t, o));
    // Having attacked, it stays attacking.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Alibou, Ancient Witness");
    let o = t.battlefield_sick(P0, "Ornithopter");
    t.answer(P0, DecisionKind::Any, Answer::Default);
    attack_with(&mut t, &[(o, Entity::Player(P1))]);
    t.resolve_all();
    destroy(&mut t, a);
    assert!(t.g.is_attacking(o));
    assert!(!t.obj(o).has_keyword(KeywordKind::Haste));
}

/// Brain in a Jar with `counters` charge counters, an Island and Wastes for its ability,
/// and Lightning Bolt (mana value 1) and Searing Spear (mana value 2) in P0's hand.
fn brain(counters: u32) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported("Brain in a Jar");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brain in a Jar");
    if counters > 0 {
        put_counters(&mut t, b, "charge", counters);
    }
    t.lands(P0, "Wastes", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spear = t.hand(P0, "Searing Spear");
    (t, b, bolt, spear)
}

/// The cards P0 was offered to cast (the candidates of entity choices) since `from`.
fn offered(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn brain_in_a_jar_counts_the_new_counter() {
    cr!("608.2c", "118.9", "202.3");
    ruling!(
        "Brain in a Jar",
        "When resolving the first ability of Brain in a Jar, the newly-placed charge counter will be counted when determining what spells you may cast."
    );
    // "{1}, {T}: Put a charge counter on this artifact, then you may cast an instant or
    // sorcery spell with mana value equal to the number of charge counters on this
    // artifact from your hand without paying its mana cost."
    let (mut t, b, bolt, spear) = brain(1);
    t.activate(P0, b, 0, &[]).unwrap();
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(spear)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = n_asked(&t);
    t.resolve_all();
    assert_eq!(t.counters(b, "charge"), 2);
    let o = offered(&t, from);
    assert!(o.contains(&obj(spear)) && !o.contains(&obj(bolt)));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn brain_in_a_jar_gone_uses_the_counters_it_had() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Brain in a Jar",
        "If Brain in a Jar leaves the battlefield before its first ability resolves, use the number of counters on it at the moment it left to determine what spell you may cast."
    );
    let (mut t, b, bolt, spear) = brain(1);
    t.activate(P0, b, 0, &[]).unwrap();
    move_to(&mut t, b, Zone::Exile);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(bolt)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = n_asked(&t);
    t.resolve_all();
    let o = offered(&t, from);
    assert!(o.contains(&obj(bolt)) && !o.contains(&obj(spear)));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn brain_in_a_jar_free_spell_pays_additional_costs() {
    cr!("118.9", "118.8", "601.2b");
    ruling!(
        "Brain in a Jar",
        "If you cast a card “without paying its mana cost,” you can’t choose to cast it for any alternative costs, such as awaken costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, such as that of Lightning Axe, you must pay those to cast the card."
    );
    supported("Lightning Axe");
    supported("Burst Lightning");
    supported("Coastal Discovery");
    // Lightning Axe ({R}): "As an additional cost to cast this spell, discard a card or
    // pay {5}. Lightning Axe deals 5 damage to target creature."
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brain in a Jar");
    t.lands(P0, "Wastes", 1);
    let axe = t.hand(P0, "Lightning Axe");
    let fodder = t.hand(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, b, 0, &[]).unwrap();
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(axe)]);
    t.answer_choose(P0, &[obj(fodder)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.zone(fodder), Zone::Graveyard(P0));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Burst Lightning ({R}, kicker {4}): the kicker may be paid; 4 damage.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brain in a Jar");
    t.lands(P0, "Wastes", 5);
    let burst = t.hand(P0, "Burst Lightning");
    t.activate(P0, b, 0, &[]).unwrap();
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(burst)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Coastal Discovery ({3}{U} sorcery, awaken 3—{5}{U}): with two charge counters on the
    // Brain it can't be cast (mana value 4) — and with four, only for no mana: no awaken.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brain in a Jar");
    put_counters(&mut t, b, "charge", 3);
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Island", 6);
    let land = t.battlefield(P0, "Forest");
    let cd = t.hand(P0, "Coastal Discovery");
    t.activate(P0, b, 0, &[]).unwrap();
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(cd)]);
    t.answer_targets(P0, &[obj(land)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.zone(cd), Zone::Graveyard(P0));
    // Drew two, no counters on the land, and no mana spent.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(t.counters(land, "+1/+1"), 0);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1);
}

#[test]
fn gandalf_with_a_short_library_goes_to_the_bottom() {
    cr!("401.7");
    ruling!(
        "Gandalf, White Rider",
        "If Gandalf, White Rider's owner has three or fewer cards left in their library when the last ability resolves, you may have it become the bottom card of their library."
    );
    supported("Gandalf, White Rider");
    // "When Gandalf dies, you may put it into its owner's library fifth from the top."
    let mut t = TestGame::new(2);
    let lib = t.g.player(P0).library.clone();
    for c in lib.into_iter().skip(3) {
        move_to(&mut t, c, Zone::Exile);
    }
    assert_eq!(t.library_size(P0), 3);
    let g = t.battlefield(P0, "Gandalf, White Rider");
    destroy(&mut t, g);
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 4);
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.obj(bottom).chars.name, "Gandalf, White Rider");
}

#[test]
fn leaves_the_battlefield_triggers_see_simultaneous_deaths() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Great Fierce Bee",
        "If Great Fierce Bee dies at the same time as one or more other creatures, Great Fierce Bee's ability still triggers."
    );
    ruling!(
        "Reaper of the Wilds",
        "If Reaper of the Wilds dies at the same time as another creature, its ability will trigger for that other creature."
    );
    ruling!(
        "Warteye Witch",
        "If Warteye Witch dies at the same time as one or more other creatures you control, Warteye Witch’s ability triggers for each of them."
    );
    ruling!(
        "Heirloom Auntie",
        "If Heirloom Auntie dies at the same time as one or more other creatures you control, its last ability will trigger for each of those other creatures."
    );
    ruling!(
        "Pitiless Plunderer",
        "If Pitiless Plunderer dies at the same time as one or more other creatures you control, its ability will still trigger for each of those other creatures."
    );
    // Great Fierce Bee: "Whenever one or more other creatures die, scry 1." Once.
    // Reaper of the Wilds: "Whenever another creature dies, scry 1." Once per other.
    // Warteye Witch: "Whenever this creature or another creature you control dies, scry
    // 1." For itself and each other.
    // Heirloom Auntie: "Whenever another creature you control dies, surveil 1, then remove
    // a -1/-1 counter from this creature." Pitiless Plunderer: "Whenever another creature
    // you control dies, create a Treasure token."
    for (name, expected) in [
        ("Great Fierce Bee", 1),
        ("Reaper of the Wilds", 2),
        ("Warteye Witch", 3),
        ("Heirloom Auntie", 2),
        ("Pitiless Plunderer", 2),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Grizzly Bears");
        destroy_together(&mut t, &[c, a, b]);
        assert_eq!(stacked_triggers(&t), expected, "{name}");
        t.resolve_all();
        if name == "Pitiless Plunderer" {
            assert_eq!(treasures(&t, P0), 2);
        }
    }
}

#[test]
fn lita_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a", "700.2b");
    ruling!(
        "Lita, Little Orphan Amphibian",
        "If Lita enters at the same time as one or more other creatures you control, Lita's ability will trigger for each of those other creatures."
    );
    supported("Lita, Little Orphan Amphibian");
    // "Alliance — Whenever another creature you control enters, choose one that hasn't
    // been chosen this turn. • Put a +1/+1 counter on Lita. • Create a Food token. • Scry
    // 1."
    let mut t = TestGame::new(2);
    enter_together(
        &mut t,
        P0,
        &[
            "Lita, Little Orphan Amphibian",
            "Grizzly Bears",
            "Hill Giant",
        ],
    );
    assert_eq!(stacked_triggers(&t), 2);
}

/// The modes chosen for the triggered abilities on the stack.
fn stacked_modes(t: &TestGame) -> Vec<usize> {
    let mut v: Vec<usize> =
        t.g.stack
            .iter()
            .flat_map(|id| t.g.obj(*id).stack.as_ref().unwrap().chosen.clone())
            .filter_map(|c| c.mode)
            .collect();
    v.sort();
    v
}

#[test]
fn lita_modes_differ_and_run_out_after_three() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Lita, Little Orphan Amphibian",
        "If multiple other creatures you control enter simultaneously, you must still choose different modes for each instance of the triggered ability that's put onto the stack. If more than three other creatures you control enter simultaneously, that choice is made only for the first three."
    );
    let mut t = TestGame::new(2);
    let lita = t.battlefield(P0, "Lita, Little Orphan Amphibian");
    enter_together(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Grizzly Bears",
            "Grizzly Bears",
            "Grizzly Bears",
        ],
    );
    assert_eq!(stacked_triggers(&t), 3);
    assert_eq!(stacked_modes(&t), vec![0, 1, 2]);
    t.resolve_all();
    assert_eq!(t.counters(lita, "+1/+1"), 1);
}

#[test]
fn two_litas_track_their_modes_separately() {
    cr!("700.2b", "603.2");
    ruling!(
        "Lita, Little Orphan Amphibian",
        "If you somehow control two or more Lita, Little Orphan Amphibians, track which modes have been chosen each turn for each one's ability separately."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Lita, Little Orphan Amphibian");
    let b = t.battlefield(P0, "Lita, Little Orphan Amphibian");
    // (One of them stops being legendary, so the legend rule doesn't apply.)
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![b])),
            mods: vec![Modification::RemoveSupertypes(vec![Supertype::Legendary])],
            duration: Duration::Permanent,
        },
        &[],
    );
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(stacked_modes(&t), vec![0, 0]);
    t.resolve_all();
    assert_eq!((t.counters(a, "+1/+1"), t.counters(b, "+1/+1")), (1, 1));
}

#[test]
fn valley_questcaller_entering_with_others_triggers() {
    cr!("603.6a");
    ruling!(
        "Valley Questcaller",
        "If Valley Questcaller enters at the same time as one or more other Rabbits, Bats, Birds, and/or Mice you control, its first ability will trigger."
    );
    supported("Valley Questcaller");
    // "Whenever one or more other Rabbits, Bats, Birds, and/or Mice you control enter,
    // scry 1."
    let mut t = TestGame::new(2);
    enter_together(&mut t, P0, &["Valley Questcaller", "Ornithopter"]);
    assert_eq!(stacked_triggers(&t), 0);
    let mut t = TestGame::new(2);
    enter_together(
        &mut t,
        P0,
        &["Valley Questcaller", "Suntail Hawk", "Suntail Hawk"],
    );
    assert_eq!(stacked_triggers(&t), 1);
}

#[test]
fn weatherlight_compleated_type_change_survives_losing_abilities() {
    cr!("613.1d", "613.1f", "613.6");
    ruling!(
        "Weatherlight Compleated",
        "If Weatherlight Compleated loses its abilities, the type-changing effect of its second ability will continue to apply to it."
    );
    // "As long as Weatherlight Compleated has four or more phyresis counters on it, it's a
    // Phyrexian creature in addition to its other types." (Its death trigger doesn't
    // compile; this static does.)
    let c = mtg_engine::card::card("Weatherlight Compleated");
    assert_eq!(c.unsupported_text().len(), 1);
    assert!(c.unsupported_text()[0].starts_with("Whenever a creature you control dies"));
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Weatherlight Compleated");
    put_counters(&mut t, w, "phyresis", 4);
    assert!(t.obj(w).is(CardType::Creature));
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![w])),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
    let o = t.obj(w);
    assert!(o.is(CardType::Creature) && o.chars.has_subtype("Phyrexian"));
    assert!(o.chars.has_subtype("Vehicle"));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(!o.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(w), (5, 5));
}

#[test]
fn graveyard_leave_triggers_once_per_event() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Stonebound Mentor",
        "If multiple cards leave your graveyard as part of the same event, Stonebound Mentor’s ability will trigger only once."
    );
    supported("Stonebound Mentor");
    // "Whenever one or more cards leave your graveyard, scry 1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stonebound Mentor");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    exile_together(&mut t, &[a, b]);
    assert_eq!(stacked_triggers(&t), 1);
}

/// Exiles the cards simultaneously (as "exile target player's graveyard" would), then
/// settles.
pub fn exile_together(t: &mut TestGame, ids: &[ObjectId]) {
    use mtg_engine::events::MoveCause;
    use mtg_engine::replacement::{EtbInfo, MoveEv};
    let moves = ids
        .iter()
        .map(|id| MoveEv {
            obj: t.g.current(*id),
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(P0),
            etb: EtbInfo::default(),
            source: None,
        })
        .collect();
    t.g.move_objects(moves);
    t.g.flush_events();
    t.settle();
}

#[test]
fn jaces_sanctum_reduces_the_total_cost() {
    cr!("601.2f", "118.7");
    ruling!(
        "Jace's Sanctum",
        "If there are additional costs to cast an instant or sorcery spell, apply those before applying cost reductions."
    );
    ruling!(
        "Jace's Sanctum",
        "Jace’s Sanctum can reduce alternative costs such as miracle or overload costs."
    );
    ruling!(
        "Jace's Sanctum",
        "The first ability of Jace’s Sanctum can’t reduce the colored mana requirement of an instant or sorcery spell."
    );
    supported("Jace's Sanctum");
    supported("Vandalblast");
    // "Instant and sorcery spells you cast cost {1} less to cast."
    // Burst Lightning ({R}, kicker {4}) kicked: {4}{R} - {1} = {3}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace's Sanctum");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let burst = t.hand(P0, "Burst Lightning");
    t.cast(P0, burst)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Vandalblast's overload cost {4}{R}: {3}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace's Sanctum");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Darksteel Relic");
    let v = t.hand(P0, "Vandalblast");
    assert!(can_cast(&mut t, P0, v, OVERLOAD));
    t.cast(P0, v).method(OVERLOAD).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a) && t.on_battlefield(b));
    // Lightning Bolt ({R}) still needs its red mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace's Sanctum");
    t.lands(P0, "Wastes", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
}

#[test]
fn puzzle_ward_triggers_for_each_die_showing_its_highest_result() {
    cr!("706.2", "603.2c");
    ruling!(
        "Netherese Puzzle-Ward",
        "If you're instructed to roll multiple dice (with one exception, see below) and more than one shows the highest natural result, the Perfect Illumination ability will trigger that many times."
    );
    supported("Netherese Puzzle-Ward");
    // "Perfect Illumination — Whenever you roll a die's highest natural result, draw a
    // card." An instruction to roll two d4 that show 4 and 4: two triggers.
    let roll_two_d4 = |t: &mut TestGame, results: [u32; 2]| {
        t.g.dice.loaded.extend(results);
        run_from(
            t,
            P0,
            None,
            Effect::RollDice(Box::new(mtg_engine::dice::DieRoll {
                count: Value::c(2),
                ..mtg_engine::dice::DieRoll::new(4)
            })),
            &[],
        );
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Netherese Puzzle-Ward");
    roll_two_d4(&mut t, [4, 4]);
    assert_eq!(triggers_on_stack(&t, "draw"), 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // A 4 and a 3: one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Netherese Puzzle-Ward");
    roll_two_d4(&mut t, [3, 4]);
    assert_eq!(triggers_on_stack(&t, "draw"), 1);
}

#[test]
fn iron_fist_pulverizer_counts_spells_cast_before_it() {
    cr!("603.2", "601.2i");
    ruling!(
        "Iron-Fist Pulverizer",
        "Spells that were cast before Iron-Fist Pulverizer entered the battlefield count. If Iron-Fist Pulverizer was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Iron-Fist Pulverizer");
    // "Whenever you cast your second spell each turn, this creature deals 2 damage to
    // target opponent. Scry 1."
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    t.battlefield(P0, "Iron-Fist Pulverizer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Cast as the first spell: the next one is the second.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Iron-Fist Pulverizer", &[]);
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn prognostic_sphinx_can_activate_while_tapped() {
    cr!("602.1", "118.3");
    ruling!(
        "Prognostic Sphinx",
        "You can activate Prognostic Sphinx's ability even if it's already tapped."
    );
    supported("Prognostic Sphinx");
    // "Discard a card: This creature gains hexproof until end of turn. Tap it."
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Prognostic Sphinx");
    t.g.tap(s);
    t.hand(P0, "Grizzly Bears");
    assert!(can_activate(&mut t, P0, s));
    activate_resolve(&mut t, P0, s, 0, &[]);
    assert!(t.obj(s).has_keyword(KeywordKind::Hexproof));
    assert!(t.obj(s).tapped);
}

/// Planetarium of Wan Shi Tong on P0's battlefield; P0 has `lands` Wastes.
fn planetarium(lands: usize) -> (TestGame, ObjectId) {
    supported("Planetarium of Wan Shi Tong");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Planetarium of Wan Shi Tong");
    t.lands(P0, "Wastes", lands);
    (t, p)
}

/// P0 scries 1 (as an effect would) keeping the card on top, then the game settles (the
/// Planetarium trigger goes on the stack).
fn scry1(t: &mut TestGame) {
    run_from(
        t,
        P0,
        None,
        Effect::Scry {
            who: PlayerRef::You,
            n: Value::c(1),
        },
        &[],
    );
}

#[test]
fn planetarium_once_a_card_is_cast_it_doesnt_trigger_again() {
    cr!("603.2h", "118.9");
    ruling!(
        "Planetarium of Wan Shi Tong",
        "Once you choose to cast the top card of your library, Planetarium of Wan Shi Tong's ability won't trigger again that turn."
    );
    // "Whenever you scry or surveil, look at the top card of your library. You may cast
    // that card without paying its mana cost. Do this only once each turn."
    let (mut t, _) = planetarium(0);
    // Declining: it triggers again on the next scry.
    stack_library(&mut t, P0, &["Ornithopter"]);
    scry1(&mut t);
    assert_eq!(stacked_triggers(&t), 1);
    t.answer_yes(P0, false);
    t.resolve_all();
    scry1(&mut t);
    assert_eq!(stacked_triggers(&t), 1);
    yes(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    // Cast once: no more triggers this turn.
    stack_library(&mut t, P0, &["Ornithopter"]);
    scry1(&mut t);
    assert_eq!(stacked_triggers(&t), 0);
}

#[test]
fn planetarium_cast_only_while_resolving() {
    cr!("608.2g", "118.9");
    ruling!(
        "Planetarium of Wan Shi Tong",
        "Planetarium of Wan Shi Tong's triggered ability allows you to cast the top card of your library during the resolution of the ability. It doesn't allow you to wait and cast the card later."
    );
    let (mut t, _) = planetarium(0);
    let top = stack_library(&mut t, P0, &["Ornithopter"])[0];
    scry1(&mut t);
    t.answer_yes(P0, false);
    t.resolve_all();
    // Later, it can't be cast from the library.
    assert_eq!(t.zone(top), Zone::Library(P0));
    assert!(!can_cast(&mut t, P0, top, CastMethod::Free));
    assert!(!can_cast(&mut t, P0, top, CastMethod::Normal));
}

#[test]
fn planetarium_free_cast_x_is_zero_and_no_alternative_costs() {
    cr!("118.9", "107.3b", "118.8");
    ruling!(
        "Planetarium of Wan Shi Tong",
        "You cannot choose to pay the mana cost or any alternate costs for the card if you choose to cast it, and if it has {X} in its mana cost, you must choose 0 as the value of {X}."
    );
    ruling!(
        "Planetarium of Wan Shi Tong",
        "You must pay any mandatory additional costs (such as sacrificing a creature) and you may pay any optional additional costs (such as kicker) for the card you cast."
    );
    // Walking Ballista ({X}{X}): X is 0, so it enters with no counters and dies.
    let (mut t, _) = planetarium(4);
    stack_library(&mut t, P0, &["Walking Ballista"]);
    scry1(&mut t);
    yes(&mut t, P0);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Walking Ballista"));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
    // Burst Lightning: the kicker {4} may be paid.
    let (mut t, _) = planetarium(4);
    stack_library(&mut t, P0, &["Burst Lightning"]);
    scry1(&mut t);
    yes(&mut t, P0);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Altar's Reap ("As an additional cost to cast this spell, sacrifice a creature."):
    // the sacrifice must be paid.
    supported("Altar's Reap");
    let (mut t, _) = planetarium(0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Altar's Reap"]);
    scry1(&mut t);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Altar's Reap"));
    assert_eq!(t.hand_size(P0), hand + 2);
    // Vandalblast can't be cast for its overload cost.
    let (mut t, _) = planetarium(5);
    let theirs = t.battlefield(P1, "Ornithopter");
    let theirs2 = t.battlefield(P1, "Ornithopter");
    stack_library(&mut t, P0, &["Vandalblast"]);
    scry1(&mut t);
    yes(&mut t, P0);
    t.answer_targets(P0, &[obj(theirs)]);
    let from = n_asked(&t);
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseCastingMethod { .. })));
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(theirs2));
}
