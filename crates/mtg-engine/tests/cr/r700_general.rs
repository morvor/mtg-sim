//! CR 700: general rules for game terms — events, modal spells and abilities, "dies",
//! devotion, historic, "this [something]", modified, crimes, "enters".

use crate::r700_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn lands_wbg(t: &mut TestGame) {
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
}

fn lands_rw(t: &mut TestGame) {
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
}

#[test]
fn one_happening_is_one_event_to_one_ability_and_several_to_another() {
    cr!("700.1");
    supported("Vengeful Townsfolk");
    supported("Zulaport Cutthroat");
    let mut t = TestGame::new(2);
    let town = t.battlefield(P0, "Vengeful Townsfolk");
    t.battlefield(P0, "Zulaport Cutthroat");
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    // One instruction destroys the three Bears at once.
    run(
        &mut t,
        P0,
        None,
        Effect::Destroy {
            what: Sel::All(Filter::Named("Grizzly Bears".into())),
            no_regen: false,
        },
    );
    t.settle();
    // "Whenever one or more other creatures you control die" sees one event;
    // "Whenever ~ or another creature you control dies" sees three.
    assert_eq!(t.stack_len(), 4);
    t.resolve_all();
    assert_eq!(t.counters(town, "+1/+1"), 1);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn modes_are_chosen_while_casting_and_illegal_modes_cant_be_chosen() {
    cr!("700.2", "700.2a");
    supported("Abzan Charm");
    let mut t = TestGame::new(2);
    lands_wbg(&mut t);
    // No creature has power 3 or greater: the exile mode has no legal target.
    t.battlefield(P1, "Grizzly Bears");
    let charm = t.hand(P0, "Abzan Charm");
    let hand = t.hand_size(P0);
    let s = t.cast(P0, charm).modes(&[0]).go();
    // The mode was chosen as the spell was cast, and it isn't the illegal one.
    assert_eq!(modes_of(&t, s), vec![1]);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(t.life(P0), 18);
    // With a legal target, that mode can be chosen.
    lands_wbg(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    let charm = t.hand(P0, "Abzan Charm");
    let s = t.cast(P0, charm).modes(&[0]).target(giant).go();
    assert_eq!(modes_of(&t, s), vec![0]);
    t.resolve();
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn a_modal_triggered_ability_chooses_its_mode_as_it_is_put_on_the_stack() {
    cr!("700.2b");
    supported("Aether Channeler");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.enter(P0, "Aether Channeler");
    t.settle();
    let ability = top(&t);
    assert_eq!(modes_of(&t, ability), vec![2]);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_modal_triggered_ability_with_no_legal_mode_is_removed_from_the_stack() {
    cr!("700.2b");
    let def = oracle_card(
        "Scrap Tinkerer",
        "Creature — Human Artificer",
        "{1}",
        Some((1, 1)),
        "When this creature enters, choose one —\n• Destroy target artifact.\n• Destroy target enchantment.",
    );
    let mut t = TestGame::new(2);
    let card = t.custom(P0, def.clone(), Zone::Hand(P0));
    t.g.move_object(card, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.settle();
    // Neither mode can be chosen: the ability is removed from the stack.
    assert_eq!(t.stack_len(), 0);
    // With an artifact to target, it's put on the stack.
    t.battlefield(P1, "Ornithopter");
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.g.move_object(card, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(modes_of(&t, top(&t)), vec![0]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn targets_are_chosen_only_for_the_chosen_modes() {
    cr!("700.2c");
    supported("Boros Charm");
    let mut t = TestGame::new(2);
    lands_rw(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let charm = t.hand(P0, "Boros Charm");
    let asked_before = times_asked(&t, P0, |d| matches!(d, Decision::ChooseTargets { .. }));
    // "Permanents you control gain indestructible until end of turn": no targets.
    let s = t.cast(P0, charm).modes(&[1]).go();
    assert_eq!(
        times_asked(&t, P0, |d| matches!(d, Decision::ChooseTargets { .. })),
        asked_before
    );
    assert!(targets_of(&t, s).is_empty());
    t.resolve();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn changing_a_modal_spells_target_cant_change_its_mode() {
    cr!("700.2f");
    supported("Boros Charm");
    supported("Deflection");
    let mut t = TestGame::new(2);
    lands_rw(&mut t);
    t.lands(P1, "Island", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let charm = t.hand(P0, "Boros Charm");
    // "Boros Charm deals 4 damage to target player or planeswalker."
    let s = t.cast(P0, charm).modes(&[0]).target(P1).go();
    let deflection = t.hand(P1, "Deflection");
    // P1 tries to redirect it to their opponent... or to a creature (which only the
    // double strike mode could target).
    t.answer_targets(P1, &[Entity::Object(bears)]);
    t.cast(P1, deflection).target(s).go();
    t.resolve();
    let candidates: Vec<Vec<Entity>> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseTargets {
                source, candidates, ..
            } if *p == P1 && *source == s => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(candidates.len(), 1);
    assert!(!candidates[0].contains(&Entity::Object(bears)));
    assert!(candidates[0].contains(&Entity::Player(P0)));
    // The mode is still the damage mode, now aimed at P0.
    assert_eq!(modes_of(&t, s), vec![0]);
    assert_eq!(targets_of(&t, s), vec![Entity::Player(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn a_copy_of_a_modal_spell_copies_its_modes() {
    cr!("700.2g");
    supported("Boros Charm");
    supported("Twincast");
    let mut t = TestGame::new(2);
    lands_rw(&mut t);
    t.lands(P1, "Island", 2);
    let charm = t.hand(P0, "Boros Charm");
    let s = t.cast(P0, charm).modes(&[0]).target(P1).go();
    // P1 copies it with Twincast and chooses a new target; the copy's controller can't
    // choose a different mode.
    let twincast = t.hand(P1, "Twincast");
    t.cast(P1, twincast).target(s).go();
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.resolve();
    let copy = top(&t);
    assert_ne!(copy, s);
    assert_eq!(modes_of(&t, copy), vec![0]);
    assert_eq!(targets_of(&t, copy), vec![Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn pawprint_modes_are_chosen_by_total_pawprints() {
    cr!("700.2i");
    let def = oracle_card(
        "Season of Tests",
        "Sorcery",
        "{2}{W}",
        None,
        "Choose up to five {P} worth of modes. You may choose the same mode more than once.\n\
         {P} — You gain 2 life.\n\
         {P}{P} — Draw a card.\n\
         {P}{P}{P} — Each opponent loses 3 life.",
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let s = t.custom(P0, def.clone(), Zone::Hand(P0));
    let hand = t.hand_size(P0);
    // {P}{P}{P} + {P}{P} = five: allowed.
    let id = t.cast(P0, s).modes(&[1, 2]).go();
    assert_eq!(modes_of(&t, id), vec![1, 2]);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand);
    // {P}{P}{P} + {P}{P}{P} = six: too many, so that choice is rejected.
    t.lands(P0, "Plains", 3);
    let s = t.custom(P0, def, Zone::Hand(P0));
    let id = t.cast(P0, s).modes(&[2, 2]).go();
    let total: u32 = modes_of(&t, id).iter().map(|m| *m as u32 + 1).sum();
    assert!(total <= 5);
    assert_ne!(modes_of(&t, id), vec![2, 2]);
}

#[test]
fn devotion_counts_copy_and_control_effects() {
    cr!("700.5", "700.5a");
    supported("Gray Merchant of Asphodel");
    supported("Clone");
    let mut t = TestGame::new(2);
    let obliterator = t.battlefield(P1, "Phyrexian Obliterator");
    // A Clone copying Phyrexian Obliterator has mana cost {B}{B}{B}{B}.
    t.answer_choose(P0, &[Entity::Object(obliterator)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(
        t.obj_now(clone).chars.name.as_str(),
        "Phyrexian Obliterator"
    );
    // P1's Obliterator is P0's after a control-changing effect.
    run(
        &mut t,
        P0,
        None,
        Effect::GainControl {
            what: Sel::All(Filter::Objects(vec![obliterator])),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
    );
    assert_eq!(t.obj_now(obliterator).controller, P0);
    // Gray Merchant's {B}{B} + 4 + 4.
    t.enter(P0, "Gray Merchant of Asphodel");
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.life(P0), 30);
}

#[test]
fn historic_means_legendary_artifact_or_saga() {
    cr!("700.6");
    supported("Jhoira, Weatherlight Captain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jhoira, Weatherlight Captain");
    t.lands(P0, "Plains", 5);
    t.lands(P0, "Forest", 2);
    let hand = t.hand_size(P0);
    let cast = |t: &mut TestGame, name: &str| {
        let c = t.hand(P0, name);
        t.cast(P0, c).go();
        t.resolve_all();
    };
    // Artifact, legendary, and Saga spells are historic; a Grizzly Bears isn't.
    cast(&mut t, "Ornithopter");
    cast(&mut t, "History of Benalia");
    cast(&mut t, "Isamaru, Hound of Konda");
    cast(&mut t, "Grizzly Bears");
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn this_something_refers_to_the_object_even_if_it_lost_that_quality() {
    cr!("700.7");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    // "{T}: This creature deals 1 damage to any target."
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    // It stops being a creature (but keeps its abilities).
    run(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![sorcerer])),
            mods: vec![Modification::RemoveTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
    );
    assert!(!t.obj_now(sorcerer).is_creature());
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn a_modified_permanent_has_counters_equipment_or_its_controllers_aura() {
    cr!("700.9");
    supported("Envoy of the Ancestors");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Envoy of the Ancestors");
    let plain = t.battlefield(P0, "Grizzly Bears");
    let countered = t.battlefield(P0, "Grizzly Bears");
    let equipped = t.battlefield(P0, "Grizzly Bears");
    let enchanted = t.battlefield(P0, "Grizzly Bears");
    let opp_aura = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(countered), "+1/+1", 1, None);
    let split = t.battlefield(P0, "Bonesplitter");
    t.g.attach(split, Entity::Object(equipped));
    let mine = t.battlefield(P0, "Pacifism");
    t.g.attach(mine, Entity::Object(enchanted));
    let theirs = t.battlefield(P1, "Pacifism");
    t.g.attach(theirs, Entity::Object(opp_aura));
    t.g.recompute();
    let lifelink = |t: &TestGame, id| t.obj_now(id).has_keyword(KeywordKind::Lifelink);
    assert!(!lifelink(&t, plain));
    assert!(lifelink(&t, countered));
    assert!(lifelink(&t, equipped));
    assert!(lifelink(&t, enchanted));
    // An Aura controlled by another player doesn't modify it.
    assert!(!lifelink(&t, opp_aura));
    // Being equipped does, whoever controls the Equipment.
    let theirs = t.battlefield(P1, "Bonesplitter");
    t.g.attach(theirs, Entity::Object(opp_aura));
    t.g.recompute();
    assert!(lifelink(&t, opp_aura));
}

#[test]
fn targeting_an_opponent_or_their_stuff_commits_a_crime() {
    cr!("700.13");
    supported("Hardbristle Bandit");
    supported("Shock");
    let mut t = TestGame::new(2);
    let bandit = t.battlefield(P0, "Hardbristle Bandit");
    t.g.objects[bandit.0 as usize].tapped = true;
    t.lands(P0, "Mountain", 3);
    let own = t.battlefield(P0, "Grizzly Bears");
    // Targeting your own creature isn't a crime.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(own).go();
    t.resolve_all();
    assert!(t.obj_now(bandit).tapped);
    assert_eq!(crimes(&t, P0), 0);
    // Targeting an opponent's creature is.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(theirs).go();
    t.resolve_all();
    assert!(!t.obj_now(bandit).tapped);
    assert_eq!(crimes(&t, P0), 1);
    // So is targeting an opponent, a card in their graveyard, or a spell they control,
    // with a spell, an activated ability, or a triggered ability.
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve_all();
    assert_eq!(crimes(&t, P0), 2);
    let in_gy = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(in_gy)]);
    let def = oracle_card(
        "Grave Robber",
        "Creature — Human Rogue",
        "{0}",
        Some((1, 1)),
        "When this creature enters, exile target card from a graveyard.",
    );
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.g.move_object(card, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(crimes(&t, P0), 3);
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(crimes(&t, P0), 4);
    // P1 casting a spell that targets their own creature commits no crime.
    t.lands(P1, "Mountain", 1);
    let giant = t.battlefield(P1, "Hill Giant");
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(giant).go();
    t.resolve_all();
    assert_eq!(crimes(&t, P1), 0);
}

fn crimes(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.crimes.get(&p).copied().unwrap_or(0)
}

#[test]
fn enters_means_enters_the_battlefield() {
    cr!("700.15");
    supported("Aether Channeler");
    let mut t = TestGame::new(2);
    // Put into a graveyard or hand: no "enters" trigger.
    let card = t.hand(P0, "Aether Channeler");
    t.g.move_object(card, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Onto the battlefield: "When this creature enters, ..." triggers.
    t.enter(P0, "Aether Channeler");
    t.settle();
    assert_eq!(t.stack_len(), 1);
}
