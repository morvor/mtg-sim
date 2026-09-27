//! CR 702.190 Sneak (`src/kw/sneak.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const SNEAK: CastMethod = CastMethod::Keyword(KeywordKind::Sneak);
/// Splinter, Hamato Yoshi ({1}{B} 1/3): "Sneak {B}. Menace. Other Ninjas you control get
/// +1/+1."
const SPLINTER: &str = "Splinter, Hamato Yoshi";

/// Player 0 attacks; player 1 blocks as given; advances to player 0's priority in the
/// declare blockers step.
fn to_blockers(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
}

fn attacking(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.g.combat.as_ref().and_then(|c| c.attack_target(id))
}

#[test]
fn sneak_cards_compile() {
    assert_supported(&[
        SPLINTER,
        "Donatello's Technique",
        "Foot Ninjas",
        "Leonardo, Leader in Blue",
        "Shredder, Unrelenting",
    ]);
}

#[test]
fn cast_for_the_sneak_cost_returning_an_unblocked_attacker() {
    cr!("702.190a", "702.190b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, SPLINTER);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    let spell = t.cast(P0, c).method(SNEAK).go();
    // {B} rather than {1}{B}; the Bears are returned to their owner's hand.
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    // Splinter enters tapped and attacking the player the Bears were attacking.
    let s = named(&t, SPLINTER)[0];
    assert!(t.obj(s).tapped);
    assert_eq!(attacking(&t, s), Some(Entity::Player(P1)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn only_during_your_declare_blockers_step() {
    cr!("702.190a");
    ruling!(
        "Splinter, Hamato Yoshi",
        "Spells can only be cast for their sneak costs any time you could play an instant during the declare blockers step on your turn"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, SPLINTER);
    // During the declare attackers step, blockers haven't been declared yet.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.cast(P0, c).method(SNEAK).try_go().is_err());
    // In the declare blockers step it can.
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, mtg_engine::decision::Action::Cast { card, .. } if *card == c)));
    // Not in the combat damage step.
    t.advance_to(P0, Step::CombatDamage);
    assert!(t.cast(P0, c).method(SNEAK).try_go().is_err());
    // Not during an opponent's declare blockers step.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, SPLINTER);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(ogre, Entity::Player(P0))]),
    );
    t.set_step(P1, Step::BeginningOfCombat);
    t.advance_to(P1, Step::DeclareBlockers);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, c).method(SNEAK).try_go().is_err());
}

#[test]
fn a_blocked_attacker_cant_be_returned() {
    cr!("702.190a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    let c = t.hand(P0, SPLINTER);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(wall, bears)]);
    assert!(t.cast(P0, c).method(SNEAK).try_go().is_err());
    assert!(t.on_battlefield(bears));
    assert!(t.in_hand(P0, SPLINTER));
}

#[test]
fn a_sorcery_with_sneak_can_be_cast_then() {
    cr!("702.190a");
    // Donatello's Technique ({2}{U} sorcery): "Sneak {U}. Draw two cards."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.hand(P0, "Donatello's Technique");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    let hand = t.hand_size(P0);
    t.cast(P0, c).method(SNEAK).go();
    t.resolve_all();
    // -1 (the Technique) +1 (the Bears) +2.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_graveyard(P0, "Donatello's Technique"));
}

#[test]
fn it_attacks_the_same_planeswalker_as_the_returned_creature() {
    cr!("702.190b");
    ruling!(
        "Splinter, Hamato Yoshi",
        "the creature it becomes enters tapped and attacking the same player, planeswalker, or battle as the creature that was returned"
    );
    ruling!(
        "Splinter, Hamato Yoshi",
        "it was never declared as an attacking creature. Abilities that trigger whenever a creature attacks won't trigger"
    );
    let rally = custom_card(
        "Rally Horn",
        "{2}",
        "Artifact",
        None,
        "Whenever a creature you control attacks, you gain 1 life.",
    );
    let mut t = TestGame::new(2);
    put(&mut t, P0, rally, Zone::Battlefield);
    t.lands(P0, "Swamp", 1);
    let walker = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let c = t.hand(P0, SPLINTER);
    to_blockers(
        &mut t,
        &[
            (bears, Entity::Object(walker)),
            (giant, Entity::Player(P1)),
        ],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Return the Bears (attacking Jace).
    choose_objects(&mut t, P0, &[bears]);
    t.cast(P0, c).method(SNEAK).go();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    t.resolve_all();
    let s = named(&t, SPLINTER)[0];
    assert_eq!(attacking(&t, s), Some(Entity::Object(walker)));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn if_his_sneak_cost_was_paid() {
    cr!("702.190b");
    // Leonardo, Leader in Blue: "When Leonardo enters, if his sneak cost was paid,
    // creatures you control get +2/+0 until end of turn."
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let c = t.hand(P0, "Leonardo, Leader in Blue");
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[],
    );
    choose_objects(&mut t, P0, &[bears]);
    t.cast(P0, c).method(SNEAK).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 3));
    let leo = named(&t, "Leonardo, Leader in Blue")[0];
    assert_eq!(t.pt(leo), (4, 1));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 11);
    // Cast normally, no bonus.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let giant = t.battlefield(P0, "Hill Giant");
    let c = t.hand(P0, "Leonardo, Leader in Blue");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn tokens_enter_attacking_if_the_sneak_cost_was_paid() {
    cr!("702.190a");
    ruling!(
        "The Last Ronin's Technique",
        "You choose the player, planeswalker, or battle each token is attacking."
    );
    // The Last Ronin's Technique ({3}{W} instant): "Sneak {1}{W}. Create three 1/1 white
    // Ninja Turtle Spirit creature tokens. If this spell's sneak cost was paid, they enter
    // tapped and attacking."
    ruling!(
        "The Last Ronin's Technique",
        "those tokens were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger when those tokens enter attacking."
    );
    let rally = custom_card(
        "Rally Horn",
        "{2}",
        "Artifact",
        None,
        "Whenever a creature you control attacks, you gain 1 life.",
    );
    let mut t = TestGame::new(2);
    put(&mut t, P0, rally, Zone::Battlefield);
    t.lands(P0, "Plains", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let c = t.hand(P0, "The Last Ronin's Technique");
    to_blockers(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[],
    );
    choose_objects(&mut t, P0, &[bears]);
    t.cast(P0, c).method(SNEAK).go();
    t.resolve_all();
    let spirits: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Spirit"))
        .map(|o| o.id)
        .collect();
    assert_eq!(spirits.len(), 3);
    for s in &spirits {
        assert!(t.obj(*s).tapped);
        assert_eq!(attacking(&t, *s), Some(Entity::Player(P1)));
    }
    // Only the two declared attackers triggered the Horn.
    assert_eq!(t.life(P0), 22);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14);
    // Cast for its mana cost, they don't.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let c = t.hand(P0, "The Last Ronin's Technique");
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t
        .g
        .permanents()
        .filter(|o| o.is_token())
        .all(|o| !o.tapped));
}
