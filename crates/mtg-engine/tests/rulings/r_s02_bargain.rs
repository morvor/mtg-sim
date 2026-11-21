//! Rulings batch S02 — bargain (CR 702.166): "As an additional cost to cast this spell, you
//! may sacrifice an artifact, enchantment, or token." A spell whose bargain cost was paid
//! is "bargained", which its linked abilities check.
//!
//! Each shared ruling exists in two wordings (straight and curly quotes) on different
//! cards, so each test cites one card of each group.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The optional additional costs P0 was offered since decision `from`.
fn offered_costs(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::OptionalCost { name, .. } if *p == P0 => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// P0 casts the real card `name` from hand (with the mana for its mana cost), bargaining
/// by sacrificing `sacrifice` if given.
fn cast_bargained(
    t: &mut TestGame,
    name: &str,
    sacrifice: Option<ObjectId>,
    targets: &[Entity],
) -> ObjectId {
    supported(name);
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        Answer::Bool(sacrifice.is_some()),
    );
    if let Some(s) = sacrifice {
        t.answer_choose(P0, &[Entity::Object(s)]);
    }
    let mut b = t.cast(P0, c);
    for e in targets {
        b = b.target(*e);
    }
    b.go()
}

/// P1 attacks P0 with a Colossal Dreadmaw (6/6); it's P1's declare attackers step.
fn dreadmaw_attacks(t: &mut TestGame) -> ObjectId {
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(t, &[(dreadmaw, Entity::Player(P0))]);
    dreadmaw
}

#[test]
fn bargain_is_sacrificing_an_artifact_an_enchantment_or_a_token() {
    cr!("702.166a", "702.166b", "118.8a", "601.2b", "601.2h");
    ruling!(
        "High Fae Negotiator",
        "Bargain means “As an additional cost to cast this spell, you may sacrifice an artifact, enchantment, or token.”"
    );
    ruling!(
        "Johann's Stopgap",
        "Bargain means \"As an additional cost to cast this spell, you may sacrifice an artifact, enchantment, or token.\""
    );
    // High Fae Negotiator: "When this creature enters, if it was bargained, each opponent
    // loses 3 life and you gain 3 life."
    for kind in ["artifact", "enchantment", "creature token"] {
        let mut t = TestGame::new(2);
        let fodder = match kind {
            "artifact" => t.battlefield(P0, "Mind Stone"),
            "enchantment" => t.battlefield(P0, "Leyline of the Void"),
            _ => create_token(&mut t, P0, "Soldier"),
        };
        let from = t.asked().len();
        cast_bargained(&mut t, "High Fae Negotiator", Some(fodder), &[]);
        assert_eq!(offered_costs(&t, from), vec!["bargain".to_string()]);
        assert!(!t.g.is_live(fodder), "{kind} sacrificed");
        t.resolve_all();
        assert_eq!((t.life(P0), t.life(P1)), (23, 17), "{kind}");
    }
    // A nontoken creature can't be sacrificed for it (nor a land): it isn't offered.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_bargained(&mut t, "High Fae Negotiator", None, &[]);
    assert!(offered_costs(&t, from).is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));

    // Johann's Stopgap ("This spell costs {2} less to cast if it's bargained. Return target
    // nonland permanent to its owner's hand. Draw a card.") bargained with a Food token.
    supported("Johann's Stopgap");
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    let islands = t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = t.hand(P0, "Johann's Stopgap");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(food)]);
    t.cast(P0, c).target(bears).go();
    assert!(!t.g.is_live(food));
    assert!(islands.iter().all(|l| t.obj_now(*l).tapped));
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn only_one_permanent_is_sacrificed_to_pay_a_bargain_cost() {
    cr!("702.166a", "601.2h");
    ruling!(
        "High Fae Negotiator",
        "You may sacrifice only one artifact, enchantment, or token to pay a spell’s bargain cost."
    );
    ruling!(
        "Kellan's Lightblades",
        "You may sacrifice only one artifact, enchantment, or token to pay a spell's bargain cost."
    );
    let sacrifice_choices = |t: &TestGame, from: usize| -> Vec<(u32, u32)> {
        t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseEntities { min, max, .. } => Some((*min, *max)),
                _ => None,
            })
            .collect()
    };
    let mut t = TestGame::new(2);
    let fodder = [
        t.battlefield(P0, "Mind Stone"),
        t.battlefield(P0, "Leyline of the Void"),
        create_token(&mut t, P0, "Soldier"),
        create_token(&mut t, P0, "Food"),
    ];
    let from = t.asked().len();
    cast_bargained(&mut t, "High Fae Negotiator", Some(fodder[0]), &[]);
    assert_eq!(sacrifice_choices(&t, from), vec![(1, 1)]);
    assert!(!t.g.is_live(fodder[0]));
    assert!(fodder[1..].iter().all(|f| t.on_battlefield(*f)));
    t.resolve_all();
    // Bargained once: 3 life, not more.
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));

    // Kellan's Lightblades: "Kellan's Lightblades deals 3 damage to target attacking or
    // blocking creature. If this spell was bargained, destroy that creature instead."
    let mut t = TestGame::new(2);
    let tokens = [
        create_token(&mut t, P0, "Soldier"),
        create_token(&mut t, P0, "Soldier"),
    ];
    let dreadmaw = dreadmaw_attacks(&mut t);
    let from = t.asked().len();
    cast_bargained(
        &mut t,
        "Kellan's Lightblades",
        Some(tokens[1]),
        &[Entity::Object(dreadmaw)],
    );
    assert_eq!(sacrifice_choices(&t, from), vec![(1, 1)]);
    assert!(t.on_battlefield(tokens[0]));
    assert!(!t.g.is_live(tokens[1]));
    t.resolve();
    assert!(t.in_graveyard(P1, "Colossal Dreadmaw"));
}

#[test]
fn a_spell_cast_with_its_bargain_cost_paid_is_bargained() {
    cr!("702.166b", "702.166c", "118.8b", "601.2f");
    ruling!(
        "Candy Grapple",
        "Bargain represents an optional additional cost. A spell cast with that additional cost paid is “bargained.”"
    );
    ruling!(
        "Kellan's Lightblades",
        "Bargain represents an optional additional cost. A spell cast with that additional cost paid is \"bargained.\""
    );
    // Kellan's Lightblades: 3 damage to the attacking Dreadmaw unless bargained; destroyed
    // if it was.
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        let token = create_token(&mut t, P0, "Soldier");
        let dreadmaw = dreadmaw_attacks(&mut t);
        cast_bargained(
            &mut t,
            "Kellan's Lightblades",
            bargain.then_some(token),
            &[Entity::Object(dreadmaw)],
        );
        assert_eq!(t.on_battlefield(token), !bargain);
        t.resolve();
        assert_eq!(t.on_battlefield(dreadmaw), !bargain, "bargained: {bargain}");
        if !bargain {
            assert_eq!(t.obj_now(dreadmaw).damage, 3);
        }
    }
    // Candy Grapple: "Target creature gets -3/-3 until end of turn. If this spell was
    // bargained, that creature gets -5/-5 until end of turn instead." A 6/4 survives -3/-3.
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        let food = create_token(&mut t, P0, "Food");
        let wurm = t.battlefield(P1, "Craw Wurm");
        cast_bargained(
            &mut t,
            "Candy Grapple",
            bargain.then_some(food),
            &[Entity::Object(wurm)],
        );
        t.resolve_all();
        assert_eq!(t.on_battlefield(wurm), !bargain, "bargained: {bargain}");
        if !bargain {
            assert_eq!(t.pt(wurm), (3, 1));
        }
    }
    // Stonesplitter Bolt: "Stonesplitter Bolt deals X damage to target creature or
    // planeswalker. If this spell was bargained, it deals twice X damage to that permanent
    // instead." The spell deals the damage either way: the lifelinking target's controller
    // gains no life.
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        let food = create_token(&mut t, P0, "Food");
        let nighthawk = t.battlefield(P1, "Vampire Nighthawk");
        supported("Stonesplitter Bolt");
        t.lands(P0, "Mountain", 3);
        let c = t.hand(P0, "Stonesplitter Bolt");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(bargain));
        t.answer_choose(P0, &[Entity::Object(food)]);
        let spell = t.cast(P0, c).target(nighthawk).x(2).go();
        t.resolve_all();
        let damage: Vec<(ObjectId, u32)> = t
            .g
            .turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, amount, .. } => Some((*source, *amount)),
                _ => None,
            })
            .collect();
        assert_eq!(damage, vec![(spell, if bargain { 4 } else { 2 })]);
        assert_eq!(t.on_battlefield(nighthawk), !bargain, "bargained: {bargain}");
        assert_eq!(t.life(P1), 20);
    }
    // Ice Out ("This spell costs {1} less to cast if it's bargained. Counter target
    // spell."): with {U}{U}, only a bargained Ice Out can be cast.
    supported("Ice Out");
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        let food = create_token(&mut t, P0, "Food");
        t.lands(P0, "Island", 2);
        t.lands(P1, "Mountain", 1);
        let bolt = t.hand(P1, "Lightning Bolt");
        let bolt = t.cast(P1, bolt).target(P0).go();
        let c = t.hand(P0, "Ice Out");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(bargain));
        t.answer_choose(P0, &[Entity::Object(food)]);
        let r = t.cast(P0, c).target(bolt).try_go();
        assert_eq!(r.is_ok(), bargain, "bargained: {bargain}");
        t.resolve_all();
        assert_eq!(t.life(P0), if bargain { 20 } else { 17 });
    }
}

#[test]
fn a_copy_of_a_bargained_spell_is_bargained_but_a_copy_of_a_permanent_isnt() {
    cr!("702.166b", "707.2", "707.5", "707.10");
    ruling!(
        "Candy Grapple",
        "If you copy a bargained spell, the copy is also bargained. If a card or token enters the battlefield as a copy of a permanent that’s already on the battlefield, the new permanent isn’t bargained, even if the original was."
    );
    ruling!(
        "Kellan's Lightblades",
        "If you copy a bargained spell, the copy is also bargained. If a card or token enters the battlefield as a copy of a permanent that's already on the battlefield, the new permanent isn't bargained, even if the original was."
    );
    supported("Twincast");
    // Candy Grapple is copied by Twincast, and the copy targets a second 6/4: it gets
    // -5/-5 from the copy of a bargained Candy Grapple, -3/-3 otherwise.
    for bargain in [false, true] {
        let mut t = TestGame::new(2);
        let food = create_token(&mut t, P0, "Food");
        let first = t.battlefield(P1, "Craw Wurm");
        let second = t.battlefield(P1, "Craw Wurm");
        let grapple = cast_bargained(
            &mut t,
            "Candy Grapple",
            bargain.then_some(food),
            &[Entity::Object(first)],
        );
        t.lands(P0, "Island", 2);
        let twincast = t.hand(P0, "Twincast");
        t.cast(P0, twincast).target(grapple).go();
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(second)]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(first), !bargain);
        assert_eq!(t.on_battlefield(second), !bargain, "bargained: {bargain}");
        if !bargain {
            assert_eq!(t.pt(second), (3, 1));
        }
    }
    // The same for Kellan's Lightblades: the copy destroys a second attacker.
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P0, "Soldier");
    let first = t.battlefield(P1, "Colossal Dreadmaw");
    let second = t.battlefield(P1, "Colossal Dreadmaw");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(first, Entity::Player(P0)), (second, Entity::Player(P0))],
    );
    let blades = cast_bargained(
        &mut t,
        "Kellan's Lightblades",
        Some(token),
        &[Entity::Object(first)],
    );
    t.lands(P0, "Island", 2);
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(blades).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(second)]);
    t.resolve_all();
    assert!(!t.on_battlefield(first));
    assert!(!t.on_battlefield(second));

    // A Clone of a bargained High Fae Negotiator isn't bargained: its enters ability does
    // nothing.
    supported("Clone");
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    let negotiator = cast_bargained(&mut t, "High Fae Negotiator", Some(food), &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let negotiator = t.g.current(negotiator);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(negotiator)]);
    cast_bargained(&mut t, "Clone", None, &[]);
    t.resolve_all();
    let negotiators = t.named_on_battlefield("High Fae Negotiator");
    assert_eq!(negotiators.len(), 2);
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
    // Nor is a token copy of it (Cackling Counterpart).
    supported("Cackling Counterpart");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 3);
    let counterpart = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, counterpart).target(negotiator).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("High Fae Negotiator").len(), 3);
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
}

#[test]
fn bargain_only_targets_are_chosen_only_if_bargained() {
    cr!("702.166d", "601.2c", "603.3d", "115.1");
    ruling!(
        "Brave the Wilds",
        "Some instant and sorcery spells require additional targets if they’re bargained. You ignore those targeting requirements if those spells aren’t bargained, and you can’t bargain those spells unless you can choose the appropriate targets. On the other hand, you can bargain a permanent spell even if you won’t be able to choose targets for an enters-the-battlefield ability of that permanent once the spell resolves."
    );
    ruling!(
        "Agatha's Champion",
        "Some instant and sorcery spells require additional targets if they're bargained. You ignore those targeting requirements if those spells aren't bargained, and you can't bargain those spells unless you can choose the appropriate targets. On the other hand, you can bargain a permanent spell even if you won't be able to choose targets for an enters-the-battlefield ability of that permanent once the spell resolves."
    );
    supported("Brave the Wilds");
    // Brave the Wilds: "If this spell was bargained, target land you control becomes a 3/3
    // Elemental creature with haste that's still a land. Search your library for a basic
    // land card, reveal it, put it into your hand, then shuffle."
    // P0 controls no land (Llanowar Elves pays {G}): it can't be bargained...
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Llanowar Elves");
    let food = create_token(&mut t, P0, "Food");
    let basic = t.library_top(P0, "Forest");
    let c = t.hand(P0, "Brave the Wilds");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(food)]);
    assert!(t.cast(P0, c).try_go().is_err());
    assert!(t.on_battlefield(food));
    assert_eq!(t.zone(c), Zone::Hand(P0));
    // ... but it can be cast without bargaining: no target is needed.
    t.clear_answers();
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    let spell = t.cast(P0, c).go();
    assert!(t.g.obj(spell).stack.as_ref().unwrap().chosen[0]
        .targets
        .iter()
        .all(|v| v.is_empty()));
    t.answer_choose(P0, &[Entity::Object(basic)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.on_battlefield(food));
    // With a land, a bargained Brave the Wilds animates it.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let food = create_token(&mut t, P0, "Food");
    cast_bargained(
        &mut t,
        "Brave the Wilds",
        Some(food),
        &[Entity::Object(forest)],
    );
    t.resolve_all();
    let o = t.obj_now(forest);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert_eq!(t.pt(forest), (3, 3));

    // Troublemaker Ouphe ("When this creature enters, if it was bargained, exile target
    // artifact or enchantment an opponent controls.") can be bargained with nothing to
    // exile: the ability is removed from the stack for lack of a target.
    supported("Troublemaker Ouphe");
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    let own_stone = t.battlefield(P0, "Mind Stone");
    cast_bargained(&mut t, "Troublemaker Ouphe", Some(food), &[]);
    assert!(!t.g.is_live(food));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Troublemaker Ouphe").len(), 1);
    assert!(t.on_battlefield(own_stone));
    assert!(t.g.stack.is_empty());
    // With an opponent's artifact, the bargained Ouphe exiles it.
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    let stone = t.battlefield(P1, "Mind Stone");
    cast_bargained(&mut t, "Troublemaker Ouphe", Some(food), &[]);
    t.answer_targets(P0, &[Entity::Object(stone)]);
    t.resolve_all();
    assert_eq!(t.zone(stone), Zone::Exile);

    // Agatha's Champion ("When this creature enters, if it was bargained, it fights up to
    // one target creature you don't control.") can be bargained with no such creature.
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    cast_bargained(&mut t, "Agatha's Champion", Some(food), &[]);
    t.resolve_all();
    let champion = t.named_on_battlefield("Agatha's Champion");
    assert_eq!(champion.len(), 1);
    assert_eq!(t.obj_now(champion[0]).damage, 0);
    // With one, the bargained Champion fights it.
    let mut t = TestGame::new(2);
    let food = create_token(&mut t, P0, "Food");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_bargained(&mut t, "Agatha's Champion", Some(food), &[]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let champion = t.named_on_battlefield("Agatha's Champion")[0];
    assert_eq!(t.obj_now(champion).damage, 2);
}

#[test]
fn an_instead_clause_about_what_the_spell_deals_keeps_the_spell_as_the_source() {
    cr!("702.33d", "608.2c");
    ruling!(
        "Firebending Lesson",
        "If a spell's kicker cost was paid, the spell is \"kicked.\""
    );
    // Firebending Lesson: "Kicker {4}. Firebending Lesson deals 2 damage to target
    // creature. If this spell was kicked, it deals 5 damage to that creature instead."
    // ("It" is the spell, as in Stonesplitter Bolt: a lifelinking target that dealt the
    // damage to itself would gain its controller life.)
    supported("Firebending Lesson");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        let nighthawk = t.battlefield(P1, "Vampire Nighthawk");
        t.lands(P0, "Mountain", 5);
        let c = t.hand(P0, "Firebending Lesson");
        let spell = t.cast(P0, c).target(nighthawk).kicked(kicked).go();
        t.resolve_all();
        let damage: Vec<(ObjectId, u32)> = t
            .g
            .turn_events
            .iter()
            .filter_map(|e| match e {
                Event::Damage { source, amount, .. } => Some((*source, *amount)),
                _ => None,
            })
            .collect();
        assert_eq!(damage, vec![(spell, if kicked { 5 } else { 2 })]);
        assert_eq!(t.on_battlefield(nighthawk), !kicked, "kicked: {kicked}");
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn torch_the_tower_exiles_what_it_damaged_if_it_would_die_this_turn() {
    cr!("614.1a", "700.4", "701.22a", "702.166b", "608.2c");
    ruling!(
        "Torch the Tower",
        "Torch the Tower's last replacement effect will exile the target permanent if it would die this turn for any reason, not just due to lethal damage or having 0 loyalty."
    );
    // "Bargain. Torch the Tower deals 2 damage to target creature or planeswalker. If this
    // spell was bargained, instead it deals 3 damage to that permanent and you scry 1. If a
    // permanent dealt damage by Torch the Tower would die this turn, exile it instead."
    supported("Torch the Tower");
    let scries = |t: &TestGame| {
        t.asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::Scry { .. }))
            .count()
    };
    // Lethal damage: the Bears are exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_bargained(&mut t, "Torch the Tower", None, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(scries(&t), 0);
    // A 3/3 survives 2 damage, but dies to Murder later in the turn: it's exiled too.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_bargained(&mut t, "Torch the Tower", None, &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
    for victim in [giant, bears] {
        give_mana_for(&mut t, P0, "Murder");
        let murder = t.hand(P0, "Murder");
        t.cast(P0, murder).target(victim).go();
        t.resolve_all();
    }
    assert_eq!(t.zone(giant), Zone::Exile);
    // (A creature it didn't damage dies normally.)
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Bargained: 3 damage kills the Giant, which is exiled, and P0 scries 1.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let food = create_token(&mut t, P0, "Food");
    cast_bargained(
        &mut t,
        "Torch the Tower",
        Some(food),
        &[Entity::Object(giant)],
    );
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!(scries(&t), 1);
}
