//! Rulings batch S09 — haste (CR 702.10) cards: creatures put onto the battlefield
//! attacking, and hasty creatures that leave at the end of the turn.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::activate_containing;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn tokens_entering_attacking_dont_trigger_attack_abilities_or_pay_attack_costs() {
    cr!("508.3a", "508.4");
    ruling!(
        "Calamity, Galloping Inferno",
        "Although the tokens enter the battlefield attacking, they were never declared as attackers. Abilities that trigger whenever a creature attacks won't trigger. If there are any costs to have a creature attack, those costs won't apply to the tokens."
    );
    supported("Calamity, Galloping Inferno");
    supported("Hero of Bladehold");
    supported("Propaganda");
    let mut t = TestGame::new(2);
    let calamity = t.battlefield(P0, "Calamity, Galloping Inferno");
    // Hero of Bladehold: battle cry; "Whenever this creature attacks, create two 1/1
    // white Soldier creature tokens that are tapped and attacking."
    let hero = t.battlefield(P0, "Hero of Bladehold");
    t.answer_choose(P0, &[Entity::Object(hero)]);
    activate_containing(&mut t, P0, calamity, "Saddle").expect("saddle");
    t.resolve_all();
    // Attacking P1 costs {2} per creature (Propaganda): P0 has exactly {2}.
    t.battlefield(P1, "Propaganda");
    t.lands(P0, "Mountain", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(calamity, Entity::Player(P1))], &[]);
    // Two tapped and attacking copies of the Hero were created; neither of their attack
    // abilities triggered (no Soldiers, no battle cry bonus for Calamity), and no cost was
    // paid for them.
    assert_eq!(tapped_lands(&t, P0), 2);
    let copies: Vec<ObjectId> = tokens(&t, P0)
        .into_iter()
        .filter(|id| t.g.obj(*id).chars.name == "Hero of Bladehold")
        .collect();
    assert_eq!(copies.len(), 2);
    assert!(with_subtype(&t, P0, "Soldier").is_empty());
    assert!(tokens(&t, P0)
        .iter()
        .all(|id| t.g.obj(*id).chars.name == "Hero of Bladehold"));
    // Calamity 4 + two Hero tokens 3 each.
    assert_eq!(t.life(P1), 10);
}

#[test]
fn a_creature_put_onto_the_battlefield_attacking_doesnt_trigger_attack_abilities() {
    cr!("508.3a", "508.4");
    ruling!(
        "Raph & Mikey, Troublemakers",
        "Although the creature you put onto the battlefield is attacking, it was never declared as an attacking creature. Abilities that trigger whenever a creature attacks won't trigger when that creature enters attacking."
    );
    supported("Raph & Mikey, Troublemakers");
    supported("Hero of Bladehold");
    let mut t = TestGame::new(2);
    // "Whenever Raph & Mikey attack, reveal cards from the top of your library until you
    // reveal a creature card. Put that card onto the battlefield tapped and attacking and
    // the rest on the bottom of your library in a random order."
    stack_library(&mut t, P0, &["Lightning Bolt", "Hero of Bladehold"]);
    let rm = t.battlefield(P0, "Raph & Mikey, Troublemakers");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(rm, Entity::Player(P1))], &[]);
    let hero = t.named_on_battlefield("Hero of Bladehold");
    assert_eq!(hero.len(), 1);
    assert!(t.obj_now(hero[0]).tapped);
    // The Hero attacked, but its attack abilities (battle cry, two attacking Soldiers)
    // didn't trigger, and neither did Raph & Mikey's again.
    assert!(with_subtype(&t, P0, "Soldier").is_empty());
    assert_eq!(t.named_on_battlefield("Hero of Bladehold").len(), 1);
    // Raph & Mikey 7 + Hero of Bladehold 3.
    assert_eq!(t.life(P1), 10);
}

#[test]
fn the_end_step_return_happens_only_if_it_is_still_on_the_battlefield() {
    cr!("603.2", "400.7");
    ruling!(
        "Viashino Sandscout",
        "It is returned to its owner’s hand at the end of turn only if it is on the battlefield."
    );
    supported("Viashino Sandscout");
    // On the battlefield: it returns to its owner's hand.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Viashino Sandscout");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(scout), Zone::Hand(P0));
    // Destroyed with the trigger on the stack: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Viashino Sandscout");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, scout);
    t.resolve_all();
    assert_eq!(t.zone(scout), Zone::Graveyard(P0));
    assert!(!t.in_hand(P0, "Viashino Sandscout"));
}

#[test]
fn ball_lightning_is_sacrificed_at_the_end_of_every_turn_it_is_on_the_battlefield() {
    cr!("603.2", "513.1");
    ruling!(
        "Ball Lightning",
        "The creature is sacrificed at the end of every turn in which it is on the battlefield. There is no choice about what turn to sacrifice it."
    );
    supported("Ball Lightning");
    // It's on the battlefield during an opponent's turn: sacrificed at that turn's end.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PostcombatMain);
    let ball = t.battlefield(P0, "Ball Lightning");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(ball), Zone::Graveyard(P0));
}
