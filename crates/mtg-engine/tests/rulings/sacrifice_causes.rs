//! "Spells and abilities your opponents control can't cause you to sacrifice permanents."
//! (Sigarda, Host of Herons; Tajuru Preserver) and "Triggered abilities you control can't
//! cause you to sacrifice or exile creature tokens you control." (The Master, Multiplied):
//! CR 701.21 (`rule_statics::sacrifice_causes`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `caster` casts Diabolic Edict ("Target player sacrifices a creature of their choice.")
/// targeting `victim`.
fn edict(t: &mut TestGame, caster: PlayerId, victim: PlayerId) {
    t.lands(caster, "Swamp", 2);
    let e = t.hand(caster, "Diabolic Edict");
    t.g.turn.priority = Some(caster);
    t.cast(caster, e).target(victim).go();
    t.resolve();
}

#[test]
fn an_opponents_spell_cant_make_you_sacrifice() {
    cr!("701.21a");
    ruling!("Sigarda, Host of Herons", "if it would force you to sacrifice a permanent, you just don't");
    ruling!("Tajuru Preserver", "if it would force you to sacrifice a permanent (as the annihilator ability does), you just don't");
    let mut t = TestGame::new(2);
    let sigarda = t.battlefield(P0, "Sigarda, Host of Herons");
    let bears = t.battlefield(P0, "Grizzly Bears");
    edict(&mut t, P1, P0);
    assert!(t.on_battlefield(sigarda));
    assert!(t.on_battlefield(bears));
    // Tajuru Preserver does the same.
    let mut t = TestGame::new(2);
    let tajuru = t.battlefield(P0, "Tajuru Preserver");
    edict(&mut t, P1, P0);
    assert!(t.on_battlefield(tajuru));
}

#[test]
fn your_own_spells_and_costs_still_can() {
    cr!("701.21a", "602.2b");
    ruling!("Tajuru Preserver", "You may still sacrifice permanents to pay the costs of spells you cast and abilities you activate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigarda, Host of Herons");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P0's own spell: P0 sacrifices one of their creatures (Bears, by default choice or
    // Sigarda).
    t.answer_choose(P0, &[Entity::Object(bears)]);
    edict(&mut t, P0, P0);
    assert!(!t.on_battlefield(bears));
    // An activation cost.
    let elder = t.battlefield(P0, "Sakura-Tribe Elder");
    t.library_top(P0, "Forest");
    t.activate(P0, elder, 0, &[]).unwrap();
    assert!(!t.on_battlefield(elder));
}

#[test]
fn an_opponents_ability_can_be_activated_by_sacrificing() {
    cr!("701.21a", "602.2b");
    ruling!("Tajuru Preserver", "even if that ability comes from a permanent an opponent controls (such as Excavation)");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tajuru Preserver");
    let exc = t.battlefield(P1, "Excavation");
    let land = t.battlefield(P0, "Forest");
    t.lands(P0, "Island", 1);
    let hand = t.hand_size(P0);
    t.activate(P0, exc, 0, &[]).unwrap();
    t.resolve();
    assert!(!t.on_battlefield(land));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn you_cant_choose_to_sacrifice_for_an_opponents_unless_cost() {
    cr!("701.21a", "118.12");
    ruling!("Sigarda, Host of Herons", "you can't choose to sacrifice a permanent");
    ruling!("Tajuru Preserver", "you can't choose to sacrifice a permanent. You must perform the action");
    let run = |with_sigarda: bool| {
        let mut t = TestGame::new(2);
        if with_sigarda {
            t.battlefield(P0, "Sigarda, Host of Herons");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Bellowing Mauler");
        t.set_step(P1, Step::PostcombatMain);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.advance_to(P1, Step::End);
        t.resolve_all();
        (t.life(P0), t.on_battlefield(bears))
    };
    // Without Sigarda P0 sacrifices the Bears and loses no life.
    assert_eq!(run(false), (20, false));
    // With Sigarda P0 can't sacrifice: they lose 4 life.
    assert_eq!(run(true), (16, true));
}

#[test]
fn game_rules_still_put_permanents_into_the_graveyard() {
    cr!("701.21a", "704.5j", "704.5g");
    ruling!("Sigarda, Host of Herons", "won't stop a permanent from being put into its owner's graveyard due to the \"legend rule.\"");
    ruling!("Tajuru Preserver", "won’t stop a permanent from being put into the graveyard due to the “legend rule.”");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sigarda, Host of Herons");
    let a = t.battlefield(P0, "Isamaru, Hound of Konda");
    let b = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.settle();
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1
    );
    // Lethal damage.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn your_triggered_abilities_cant_exile_your_creature_tokens() {
    cr!("701.21a", "702.116a");
    ruling!("The Master, Multiplied", "the part of myriad that makes you exile the creature tokens at end of combat is a triggered ability");
    let mut t = TestGame::new(3);
    let master = t.battlefield(P0, "The Master, Multiplied");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(master, Entity::Player(P1))], &[]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    // Myriad's "exile the tokens at end of combat" is a triggered ability P0 controls: the
    // token copy stays.
    let tokens: Vec<ObjectId> = t
        .named_on_battlefield("The Master, Multiplied")
        .into_iter()
        .filter(|o| *o != master)
        .collect();
    assert_eq!(tokens.len(), 1);
    // An opponent's spell still makes P0 sacrifice it.
    t.answer_choose(P0, &[Entity::Object(tokens[0])]);
    edict(&mut t, P1, P0);
    assert!(!t.on_battlefield(tokens[0]));
}

#[test]
fn a_special_action_from_an_opponents_permanent_may_sacrifice() {
    cr!("701.21a", "116.2d");
    ruling!("Tajuru Preserver", "You may sacrifice a permanent as a special action");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tajuru Preserver");
    t.battlefield(P1, "Damping Engine");
    t.lands(P0, "Forest", 2);
    t.g.turn.priority = Some(P0);
    let sa = t
        .g
        .legal_actions(P0)
        .into_iter()
        .find(|a| matches!(a, Action::Special(_)))
        .expect("Damping Engine's special action");
    let before = t.graveyard_size(P0);
    t.g.take_action(P0, sa);
    assert_eq!(t.graveyard_size(P0), before + 1);
}

#[test]
fn an_opponents_annihilator_trigger_cant_make_you_sacrifice() {
    cr!("701.21a", "702.86a");
    ruling!("Tajuru Preserver", "if it would force you to sacrifice a permanent (as the annihilator ability does), you just don't");
    let run = |with_tajuru: bool| {
        let mut t = TestGame::new(2);
        if with_tajuru {
            t.battlefield(P0, "Tajuru Preserver");
        }
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Forest");
        let crusher = t.battlefield(P1, "Ulamog's Crusher");
        t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
        t.set_step(P1, Step::BeginningOfCombat);
        t.attack(&[(crusher, Entity::Player(P0))], &[]);
        t.resolve_all();
        t.on_battlefield(a) as u32 + t.on_battlefield(b) as u32
    };
    // Without Tajuru Preserver, annihilator 2 takes both.
    assert_eq!(run(false), 0);
    assert_eq!(run(true), 2);
}

#[test]
fn an_opponents_sacrifice_all_but_spell_sacrifices_nothing() {
    cr!("701.21a");
    ruling!("Sigarda, Host of Herons", "if it would force you to sacrifice a permanent, you just don't");
    let mut t = TestGame::new(2);
    let sigarda = t.battlefield(P0, "Sigarda, Host of Herons");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forests: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P0, "Forest")).collect();
    let theirs = [
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Grizzly Bears"),
    ];
    // Cataclysm: "Each player chooses from among the permanents they control an artifact,
    // a creature, an enchantment, and a land, then sacrifices the rest."
    t.lands(P1, "Plains", 4);
    let c = t.hand(P1, "Cataclysm");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, c).go();
    t.resolve();
    assert!(t.on_battlefield(sigarda) && t.on_battlefield(bears));
    assert!(forests.iter().all(|f| t.on_battlefield(*f)));
    // P1's own spell still makes P1 sacrifice theirs.
    assert_eq!(theirs.iter().filter(|o| t.on_battlefield(**o)).count(), 1);
}
