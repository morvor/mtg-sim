//! Rulings on scheme cards (CR 314, 904): "target opponent chooses self or others", a
//! scheme with up to three targets, and casting a spell from hand without paying its mana
//! cost (I Am Duskmourn).

use crate::r_s01_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::variants;
use mtg_engine::*;

/// An Archenemy game: P0 is the archenemy, facing the other players as a team.
fn archenemy_game(n: usize) -> TestGame {
    let mut teams = vec![0u8];
    teams.extend(std::iter::repeat_n(1u8, n - 1));
    TestGame::with_config(
        n,
        GameConfig {
            variant: Variant::Archenemy,
            teams: Some(teams),
            ..Default::default()
        },
    )
}

/// Puts the scheme `name` on top of `p`'s scheme deck (face down in the command zone).
fn scheme(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    let id = t.custom(p, (*card(name)).clone(), Zone::Command);
    t.g.objects[id.0 as usize].face_down = true;
    t.g.recompute();
    id
}

/// `p` sets the top scheme of their scheme deck in motion; its "When you set this scheme
/// in motion" ability goes on the stack.
fn set_in_motion(t: &mut TestGame, p: PlayerId) {
    variants::set_in_motion(&mut t.g, p);
    t.g.flush_events();
    t.settle();
}

/// The "self or others" options offered to `p` since decision `from`.
fn self_or_others_asked(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseOption { options, .. } if *q == p => Some(options.clone()),
            _ => None,
        })
        .collect()
}

const SELF: usize = 0;
const OTHERS: usize = 1;

#[test]
fn the_targeted_player_may_choose_self_even_if_it_does_nothing() {
    cr!("314.5", "608.2d");
    ruling!(
        "Feed the Machine",
        "The targeted player may choose “self” even if they can’t perform the resulting action."
    );
    // Feed the Machine: "When you set this scheme in motion, target opponent chooses self
    // or others. If that player chooses self, the player sacrifices two creatures of their
    // choice. If the player chooses others, each of your other opponents sacrifices a
    // creature of their choice."
    let mut t = archenemy_game(3);
    let p2_bears = t.battlefield(P2, "Grizzly Bears");
    scheme(&mut t, P0, "Feed the Machine");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(SELF));
    let from = t.asked().len();
    set_in_motion(&mut t, P0);
    t.resolve_all();
    assert_eq!(
        self_or_others_asked(&t, P1, from),
        vec![vec!["self".to_string(), "others".to_string()]]
    );
    // P1 controls no creatures: nothing happens; P2 isn't affected.
    assert!(t.on_battlefield(p2_bears));
    // With creatures, "self": P1 sacrifices two.
    let mut t = archenemy_game(3);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Savannah Lions");
    let p2_bears = t.battlefield(P2, "Grizzly Bears");
    scheme(&mut t, P0, "Feed the Machine");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(SELF));
    set_in_motion(&mut t, P0);
    t.resolve_all();
    assert_eq!(creatures(&t, P1).len(), 1);
    assert!(t.on_battlefield(p2_bears));
}

#[test]
fn the_targeted_player_may_choose_others_even_if_there_are_none() {
    cr!("314.5", "608.2d");
    ruling!(
        "The Fate of the Flammable",
        "The targeted player may choose “others” even if there are no others"
    );
    // The Fate of the Flammable: "... If that player chooses self, this scheme deals 6
    // damage to that player. If the player chooses others, this scheme deals 3 damage to
    // each of your other opponents." With one opponent, "others" is no one.
    let mut t = archenemy_game(2);
    let before = (t.life(P0), t.life(P1));
    scheme(&mut t, P0, "The Fate of the Flammable");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(OTHERS));
    set_in_motion(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), before);
    // With another opponent, "others" deals 3 damage to them; "self" 6 to the chooser.
    for (choice, p1, p2) in [(OTHERS, 0, 3), (SELF, 6, 0)] {
        let mut t = archenemy_game(3);
        let before = (t.life(P1), t.life(P2));
        scheme(&mut t, P0, "The Fate of the Flammable");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer(P1, DecisionKind::Option, Answer::Index(choice));
        set_in_motion(&mut t, P0);
        t.resolve_all();
        assert_eq!((before.0 - t.life(P1), before.1 - t.life(P2)), (p1, p2));
    }
    // May Civilization Collapse ("... each of your other opponents sacrifices a land of
    // their choice"): "others" may be chosen when they have no lands.
    let mut t = archenemy_game(3);
    let lands = t.lands(P1, "Forest", 2);
    scheme(&mut t, P0, "May Civilization Collapse");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(OTHERS));
    set_in_motion(&mut t, P0);
    t.resolve_all();
    assert!(lands.iter().all(|l| t.on_battlefield(*l)));
}

#[test]
fn in_a_supervillain_rumble_others_are_everyone_but_the_archenemy_and_the_target() {
    cr!("904.12", "314.5");
    ruling!(
        "Surrender Your Thoughts",
        "In a Supervillain Rumble game, the targeted player may still choose “others.” Each player who isn't the active player or the targeted player will thus be affected."
    );
    // Surrender Your Thoughts: "... If that player chooses self, that player discards four
    // cards. If the player chooses others, each of your other opponents discards two
    // cards."
    let mut t = TestGame::with_config(4, GameConfig::supervillain_rumble());
    for p in [P0, P1, P2, P3] {
        for _ in 0..3 {
            t.hand(p, "Island");
        }
    }
    scheme(&mut t, P0, "Surrender Your Thoughts");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(OTHERS));
    set_in_motion(&mut t, P0);
    t.resolve_all();
    assert_eq!(
        [P0, P1, P2, P3].map(|p| t.hand_size(p)),
        [3, 3, 1, 1],
        "P2 and P3 discard two cards each"
    );
}

#[test]
fn tooth_claw_and_tail_may_have_zero_to_three_targets() {
    cr!("115.1", "601.2c", "603.3d");
    ruling!(
        "Tooth, Claw, and Tail",
        "You may choose zero, one, two, or three targets."
    );
    // "When you set this scheme in motion, destroy up to three target nonland permanents."
    for n in [0usize, 1, 3] {
        let mut t = archenemy_game(2);
        let perms: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        scheme(&mut t, P0, "Tooth, Claw, and Tail");
        let targets: Vec<Entity> = perms[..n].iter().map(|p| Entity::Object(*p)).collect();
        t.answer_targets(P0, &targets);
        set_in_motion(&mut t, P0);
        t.resolve_all();
        let destroyed = perms.iter().filter(|p| !t.on_battlefield(**p)).count();
        assert_eq!(destroyed, n);
    }
}

/// P0's end step with I Am Duskmourn face up ("At the beginning of your end step, you may
/// cast a spell from your hand without paying its mana cost. If you do, abandon this
/// scheme."): the trigger is on the stack.
fn duskmourn_end_step(t: &mut TestGame) {
    scheme(t, P0, "I Am Duskmourn");
    variants::set_in_motion(&mut t.g, P0);
    t.settle();
    assert_eq!(variants::face_up_schemes(&t.g).len(), 1);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.settle();
}

#[test]
fn a_spell_cast_without_paying_its_mana_cost_may_have_additional_but_not_alternative_costs() {
    cr!("118.9a", "118.9d", "608.2g", "701.33a");
    ruling!(
        "I Am Duskmourn",
        "If you cast a spell for another cost \"rather than pay its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the spell has any mandatory additional costs, such as that of Abhorrent Oculus, those must be paid to cast the spell."
    );
    // An optional additional cost: Burst Lightning's kicker {4} ("Burst Lightning deals 2
    // damage to any target. If this spell was kicked, it deals 4 damage instead.").
    let mut t = archenemy_game(2);
    t.lands(P0, "Mountain", 4);
    let burst = t.hand(P0, "Burst Lightning");
    let life = t.life(P1);
    duskmourn_end_step(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(burst)]);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), life - 4);
    assert_eq!(tapped_lands(&t, P0), 4);
    // "If you do, abandon this scheme."
    assert!(variants::face_up_schemes(&t.g).is_empty());
    // A mandatory additional cost must be paid: Village Rites ("As an additional cost to
    // cast this spell, sacrifice a creature. Draw two cards.") can't be cast without a
    // creature to sacrifice — nothing is cast, and the scheme isn't abandoned.
    let mut t = archenemy_game(2);
    let rites = t.hand(P0, "Village Rites");
    duskmourn_end_step(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(rites)]);
    t.resolve_all();
    assert_eq!(t.zone(rites), Zone::Hand(P0));
    assert_eq!(variants::face_up_schemes(&t.g).len(), 1);
    // With one, it's sacrificed.
    let mut t = archenemy_game(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rites = t.hand(P0, "Village Rites");
    duskmourn_end_step(&mut t);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(rites)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    // No alternative cost: Mulldrifter isn't evoked (it stays), Fireblast's "sacrifice two
    // Mountains" isn't paid.
    let mut t = archenemy_game(2);
    let mountains = t.lands(P0, "Mountain", 2);
    let fireblast = t.hand(P0, "Fireblast");
    let life = t.life(P1);
    duskmourn_end_step(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(fireblast)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), life - 4);
    assert!(mountains.iter().all(|m| t.on_battlefield(*m)));
    let mut t = archenemy_game(2);
    let drifter = t.hand(P0, "Mulldrifter");
    duskmourn_end_step(&mut t);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(drifter)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mulldrifter").len(), 1);
}

#[test]
fn an_until_your_next_turn_effect_lasts_until_just_before_your_next_untap_step() {
    cr!("611.2b", "603.7b", "502.1");
    ruling!(
        "A Display of My Dark Power",
        "The effect doesn't wear off until just before your next untap step (even if an effect will cause that untap step to be skipped)."
    );
    supported("Stasis");
    // A Display of My Dark Power: "When you set this scheme in motion, until your next
    // turn, whenever a player taps a land for mana, that player adds one mana of any type
    // that land produced." Stasis: "Players skip their untap steps."
    let mut t = archenemy_game(2);
    let forests = t.lands(P0, "Forest", 2);
    let theirs = t.battlefield(P1, "Forest");
    t.battlefield(P0, "Stasis");
    scheme(&mut t, P0, "A Display of My Dark Power");
    set_in_motion(&mut t, P0);
    t.resolve_all();
    let pool = |t: &TestGame, p: PlayerId| t.g.player(p).mana_pool.total();
    t.activate(P0, forests[0], 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), 2);
    // Through the opponent's turn.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    t.activate(P1, theirs, 0, &[]).unwrap();
    assert_eq!(pool(&t, P1), 2);
    // It ended as P0's next turn began, though P0's untap step was skipped.
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj(forests[0]).tapped, "the untap step was skipped");
    t.activate(P0, forests[1], 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), 1);
}
