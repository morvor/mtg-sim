//! CR 115.1, 601.2c, 608.2b: targets that must have a relationship with each other —
//! "with different controllers", "with equal toughness", "with total mana value N or
//! less", "with different names" (`TargetSpec::together`); targets chosen for each player
//! ("for each opponent, ... up to one target creature that player controls",
//! `TargetSpec::per_player`); and "each mode must target a different player"
//! (`Modal::different_players`, CR 700.2).

use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::eval::Ctx;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Executes `effect` as if a spell or ability `p` controls with these targets (one per
/// slot) resolved.
fn run(t: &mut TestGame, p: PlayerId, effect: Effect, targets: &[Entity]) {
    let mut ctx = Ctx::new(None, p);
    ctx.targets = targets.iter().map(|e| vec![*e]).collect();
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

/// The candidates and maximum of each target choice asked since `from`.
fn target_choices(t: &TestGame, from: usize) -> Vec<(Vec<Entity>, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                candidates, max, ..
            } => Some((candidates.clone(), *max)),
            _ => None,
        })
        .collect()
}

fn tap(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = true;
}

#[test]
fn targets_chosen_for_each_opponent() {
    cr!("601.2c", "115.1", "115.3");
    // Mass Mutiny with two opponents: one choice for each, of up to one creature that
    // opponent controls.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, bears);
    let giant = t.battlefield(P2, "Hill Giant");
    let elves = t.battlefield(P2, "Llanowar Elves");
    t.battlefield(P0, "Runeclaw Bear");
    mana(&mut t, P0, ManaType::R, 5);
    let spell = t.hand(P0, "Mass Mutiny");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let from = t.asked().len();
    let id = t.cast(P0, spell).go();
    let choices = target_choices(&t, from);
    assert_eq!(
        choices,
        vec![
            (vec![Entity::Object(bears)], 1),
            (vec![Entity::Object(giant), Entity::Object(elves)], 1)
        ]
    );
    assert_eq!(
        targets_of(&t, id),
        vec![Entity::Object(bears), Entity::Object(giant)]
    );
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(elves).controller, P2);

    // Two creatures from one opponent aren't one per opponent: the answer isn't legal,
    // and no creature of that opponent's is chosen ("up to one").
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P2, "Hill Giant");
    let elves = t.battlefield(P2, "Llanowar Elves");
    mana(&mut t, P0, ManaType::R, 5);
    let spell = t.hand(P0, "Mass Mutiny");
    t.answer_targets(P0, &[Entity::Object(giant), Entity::Object(elves)]);
    let id = t.cast(P0, spell).go();
    assert!(targets_of(&t, id).len() <= 1);
}

#[test]
fn a_target_chosen_for_a_player_must_still_be_theirs() {
    cr!("608.2b", "601.2c");
    // Blatant Thievery: "For each opponent, gain control of target permanent that player
    // controls." P2 gains control of the permanent targeted for P1 before it resolves:
    // it's no longer a permanent that player (P1) controls, so it's an illegal target,
    // even though P2 is also an opponent.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    mana(&mut t, P0, ManaType::U, 7);
    let spell = t.hand(P0, "Blatant Thievery");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, spell).go();
    run(
        &mut t,
        P2,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(bears)],
    );
    assert_eq!(t.obj_now(bears).controller, P2);
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P2);
    assert_eq!(t.obj_now(giant).controller, P0);
}

#[test]
fn a_player_with_nothing_to_target_gets_no_target() {
    cr!("601.2c", "115.6");
    // Mass Mutiny is cast even if no opponent controls a creature.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::R, 5);
    let spell = t.hand(P0, "Mass Mutiny");
    let id = t.cast(P0, spell).go();
    assert!(targets_of(&t, id).is_empty());
    t.resolve();
    assert!(t.in_graveyard(P0, "Mass Mutiny"));
}

#[test]
fn targets_with_different_controllers() {
    cr!("601.2c", "115.1", "608.2b");
    // Cloud's Limit Break, Blade Beam: "Destroy any number of target tapped creatures
    // with different controllers." Two of P1's creatures can't both be chosen: the
    // second is dropped.
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P2, "Llanowar Elves");
    for x in [a, b, c] {
        tap(&mut t, x);
    }
    mana(&mut t, P0, ManaType::W, 3);
    let spell = t.hand(P0, "Cloud's Limit Break");
    let id = t
        .cast(P0, spell)
        .modes(&[1])
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    assert_eq!(targets_of(&t, id), vec![Entity::Object(a), Entity::Object(c)]);
    // P2 gains control of P1's creature: the targets no longer have different
    // controllers, so neither is destroyed.
    run(
        &mut t,
        P2,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(a)],
    );
    t.resolve();
    assert!(t.on_battlefield(a));
    assert!(t.on_battlefield(b));
    assert!(t.on_battlefield(c));
}

#[test]
fn targets_with_equal_toughness() {
    cr!("601.2c", "608.2b");
    // V.A.T.S.: "Choose any number of target creatures with equal toughness." Of a 2/2,
    // a 3/3 and another 2/2, the 3/3 is dropped.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let runeclaw = t.battlefield(P1, "Runeclaw Bear");
    mana(&mut t, P0, ManaType::B, 4);
    let spell = t.hand(P0, "V.A.T.S.");
    let id = t
        .cast(P0, spell)
        .targets(&[
            Entity::Object(bears),
            Entity::Object(giant),
            Entity::Object(runeclaw),
        ])
        .go();
    assert_eq!(
        targets_of(&t, id),
        vec![Entity::Object(bears), Entity::Object(runeclaw)]
    );
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(runeclaw));
    assert!(t.on_battlefield(giant));
}

#[test]
fn targets_with_a_total_mana_value() {
    cr!("601.2c", "115.1");
    // Patch Up: "Return up to three target creature cards with total mana value 3 or
    // less from your graveyard to the battlefield." Bears (2), Elves (1) and Hill Giant
    // (4): the Giant doesn't fit.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let giant = t.graveyard(P0, "Hill Giant");
    mana(&mut t, P0, ManaType::W, 3);
    let spell = t.hand(P0, "Patch Up");
    let id = t
        .cast(P0, spell)
        .targets(&[
            Entity::Object(bears),
            Entity::Object(giant),
            Entity::Object(elves),
        ])
        .go();
    assert_eq!(
        targets_of(&t, id),
        vec![Entity::Object(bears), Entity::Object(elves)]
    );
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.in_graveyard(P0, "Hill Giant"));

    // One card over the total on its own can't be chosen.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    mana(&mut t, P0, ManaType::W, 3);
    let spell = t.hand(P0, "Patch Up");
    let id = t.cast(P0, spell).targets(&[Entity::Object(giant)]).go();
    assert!(targets_of(&t, id).is_empty());
}

#[test]
fn targets_with_different_names() {
    cr!("601.2c", "201.2b");
    // Behold the Sinister Six!: "Return up to six target creature cards with different
    // names from your graveyard to the battlefield." Two Grizzly Bears can't both be
    // chosen.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Grizzly Bears");
    let c = t.graveyard(P0, "Llanowar Elves");
    mana(&mut t, P0, ManaType::B, 7);
    let spell = t.hand(P0, "Behold the Sinister Six!");
    let id = t
        .cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    assert_eq!(targets_of(&t, id), vec![Entity::Object(a), Entity::Object(c)]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

/// Puts Vindictive Lich onto the battlefield and has it die; returns its dies trigger's
/// stack object.
fn lich_dies(t: &mut TestGame) {
    let lich = t.battlefield(P0, "Vindictive Lich");
    t.g.destroy(lich, None);
    t.g.flush_events();
    t.settle();
}

#[test]
fn each_mode_must_target_a_different_player() {
    cr!("700.2", "700.2b", "601.2c", "603.3c");
    // Vindictive Lich with two opponents: two modes, each targeting a different one.
    // Answering P1 for both: the second mode's target must be P2.
    let mut t = TestGame::new(3);
    for p in [P1, P2] {
        for _ in 0..3 {
            t.hand(p, "Grizzly Bears");
        }
    }
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    lich_dies(&mut t);
    let si = t.obj(*t.g.stack.last().unwrap()).stack.clone().unwrap();
    let players: Vec<Entity> = si
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect();
    assert_eq!(players, vec![Entity::Player(P1), Entity::Player(P2)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P2), 15);
    assert_eq!(t.hand_size(P2), 3);

    // With one opponent, only one mode can be chosen: asking for two isn't legal, and
    // only one of them happens.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    lich_dies(&mut t);
    let si = t.obj(*t.g.stack.last().unwrap()).stack.clone().unwrap();
    assert_eq!(si.chosen.len(), 1);
    t.resolve_all();
    let discarded = t.hand_size(P1) == 1;
    let lost = t.life(P1) == 15;
    assert!(!(discarded && lost));
}

#[test]
fn changing_a_target_keeps_the_player_it_was_chosen_for() {
    cr!("115.7", "115.7a", "608.2b");
    // Mass Mutiny targets P1's Bears; its target may be changed only to another creature
    // P1 controls, not to P2's Giant.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P2, "Hill Giant");
    mana(&mut t, P0, ManaType::R, 5);
    let spell = t.hand(P0, "Mass Mutiny");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[]);
    let id = t.cast(P0, spell).go();
    assert_eq!(targets_of(&t, id), vec![Entity::Object(bears)]);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    let from = t.asked().len();
    let changed = mtg_engine::target_rules::change_targets(
        &mut t.g,
        P1,
        id,
        TargetChange::One,
        None,
    );
    assert!(changed);
    let offered = target_choices(&t, from);
    assert_eq!(offered[0].0, vec![Entity::Object(elves)]);
    assert_eq!(targets_of(&t, id), vec![Entity::Object(elves)]);
    t.resolve();
    assert_eq!(t.obj_now(elves).controller, P0);
    assert_eq!(t.obj_now(giant).controller, P2);
    assert_eq!(t.zone(bears), Zone::Battlefield);
}

#[test]
fn a_target_related_to_an_earlier_target() {
    cr!("601.2c", "115.1");
    // Bioshift: "Move any number of +1/+1 counters from target creature onto another
    // target creature with the same controller." P0's lone Bears has no partner, so it
    // isn't offered as the first target: the first is one of P1's creatures, and the
    // second must be P1's other one.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::G, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let spell = t.hand(P0, "Bioshift");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let from = t.asked().len();
    let id = t.cast(P0, spell).go();
    let choices = target_choices(&t, from);
    assert_eq!(
        choices[0].0,
        vec![Entity::Object(giant), Entity::Object(elves)]
    );
    let chosen = targets_of(&t, id);
    let first = chosen[0];
    let other = if first == Entity::Object(giant) {
        Entity::Object(elves)
    } else {
        Entity::Object(giant)
    };
    assert_eq!(choices[1].0, vec![other]);
    assert_eq!(chosen, vec![first, other]);
}

/// A sorcery compiled from oracle text with the real compiler.
fn sorcery(name: &str, text: &str) -> mtg_engine::card::CardDef {
    use mtg_engine::oracle::{self, CompileContext};
    let tl = TypeLine::parse("Sorcery");
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    mtg_engine::card::CardDef::custom(mtg_engine::object::Characteristics {
        name: smol_str::SmolStr::new(name),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: std::sync::Arc::from(text),
        ..Default::default()
    })
}

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p).iter().any(
        |a| matches!(a, mtg_engine::decision::Action::Cast { card: c, .. } if *c == card),
    )
}

#[test]
fn another_target_must_be_a_different_object() {
    cr!("115.3", "601.2c");
    // "Put a +1/+1 counter on target creature, two +1/+1 counters on another target
    // creature, and three +1/+1 counters on a third target creature." (Incremental
    // Growth): three different creatures are needed; with two, the spell can't be cast.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Incremental Growth");
    t.lands(P0, "Forest", 5);
    assert!(!castable(&mut t, P0, spell));
    assert!(t.cast(P0, spell).try_go().is_err());
    let c = t.battlefield(P1, "Llanowar Elves");
    assert!(castable(&mut t, P0, spell));
    // The same creature can't be chosen again: answering an earlier target again is
    // ignored, and a different creature is chosen.
    let from = t.asked().len();
    let id = t.cast(P0, spell).target(b).target(b).target(b).go();
    let choices = target_choices(&t, from);
    assert_eq!(choices.len(), 3);
    assert!(!choices[1].0.contains(&Entity::Object(b)));
    let chosen = targets_of(&t, id);
    assert_eq!(chosen.len(), 3);
    assert_eq!(chosen[0], Entity::Object(b));
    assert!(!choices[2].0.contains(&chosen[0]) && !choices[2].0.contains(&chosen[1]));
    assert!([a, b, c].iter().all(|x| chosen.contains(&Entity::Object(*x))));
    t.resolve();
    let n = |e: Entity| t.counters(e.object().unwrap(), "+1/+1");
    assert_eq!((n(chosen[0]), n(chosen[1]), n(chosen[2])), (1, 2, 3));
}

#[test]
fn a_target_that_would_leave_another_target_without_a_choice_isnt_offered() {
    cr!("115.3", "601.2c");
    // "another target creature you control" must be a different creature from the first
    // target: P0's only creature is needed for it, so the first target is P1's.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let spell = t.custom(
        P0,
        sorcery(
            "Probe",
            "Put a +1/+1 counter on target creature and two +1/+1 counters on another target creature you control.",
        ),
        Zone::Hand(P0),
    );
    assert!(castable(&mut t, P0, spell));
    let from = t.asked().len();
    let id = t.cast(P0, spell).target(mine).target(mine).go();
    let choices = target_choices(&t, from);
    assert_eq!(choices[0].0, vec![Entity::Object(theirs)]);
    assert_eq!(
        targets_of(&t, id),
        vec![Entity::Object(theirs), Entity::Object(mine)]
    );
    t.resolve();
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
    assert_eq!(t.counters(mine, "+1/+1"), 2);

    // With only P0's creature, there's no second creature for it: can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let spell = t.custom(
        P0,
        sorcery(
            "Probe",
            "Put a +1/+1 counter on target creature and two +1/+1 counters on another target creature you control.",
        ),
        Zone::Hand(P0),
    );
    assert!(!castable(&mut t, P0, spell));
}

#[test]
fn two_targets_of_one_instance_fight_each_other() {
    cr!("701.14a", "701.14b", "608.2b", "115.1");
    // Rivals' Duel: "Choose two target creatures that share no creature types. Those
    // creatures fight each other."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    mana(&mut t, P0, ManaType::R, 4);
    let spell = t.hand(P0, "Rivals' Duel");
    let id = t
        .cast(P0, spell)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    assert_eq!(
        targets_of(&t, id),
        vec![Entity::Object(bears), Entity::Object(giant)]
    );
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.obj_now(elves).damage, 0);
}
