//! Remove Enchantments (hand-written, `src/cards/remove_enchantments.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn returns_yours_and_destroys_the_others() {
    cr!("608.2b", "701.8a");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An opponent's Aura on your creature is destroyed.
    let theirs = t.battlefield(P1, "Pacifism");
    t.g.objects[theirs.0 as usize].attached_to = Some(Entity::Object(bears));
    // Your Aura on an opponent's (non-attacking) creature: you both own and control it.
    let giant = t.battlefield(P1, "Hill Giant");
    let weak = t.battlefield(P0, "Weakness");
    t.g.objects[weak.0 as usize].attached_to = Some(Entity::Object(giant));
    // An opponent's enchantment that isn't attached to your things is untouched.
    let their_anthem = t.battlefield(P1, "Glorious Anthem");
    t.g.recompute();
    t.lands(P0, "Plains", 1);
    let s = t.hand(P0, "Remove Enchantments");
    t.cast(P0, s).go();
    t.resolve();
    assert!(!t.on_battlefield(anthem));
    assert!(!t.on_battlefield(weak));
    assert!(t.in_hand(P0, "Glorious Anthem"));
    assert!(t.in_hand(P0, "Weakness"));
    assert!(t.in_graveyard(P1, "Pacifism"));
    assert!(t.on_battlefield(their_anthem));
}

#[test]
fn auras_on_attacking_creatures_of_opponents() {
    cr!("608.2b", "701.8a");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P1, "Grizzly Bears");
    // An opponent's Aura on their attacking creature is destroyed...
    let on_attacker = t.battlefield(P1, "Holy Strength");
    t.g.objects[on_attacker.0 as usize].attached_to = Some(Entity::Object(attacker));
    // ...one on their creature that isn't attacking is untouched.
    let on_idle = t.battlefield(P1, "Holy Strength");
    t.g.objects[on_idle.0 as usize].attached_to = Some(Entity::Object(idle));
    // An Aura you own but an opponent controls on their attacking creature is returned.
    let mine = t.battlefield(P0, "Unholy Strength");
    t.g.objects[mine.0 as usize].attached_to = Some(Entity::Object(attacker));
    t.g.objects[mine.0 as usize].controller = P1;
    t.g.recompute();
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(P1, DecisionKind::Attackers, Answer::Attackers(vec![(attacker, Entity::Player(P0))]));
    t.advance_to(P1, Step::DeclareAttackers);
    t.lands(P0, "Plains", 1);
    let s = t.hand(P0, "Remove Enchantments");
    t.cast(P0, s).go();
    t.resolve();
    assert!(!t.on_battlefield(on_attacker));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(t.on_battlefield(on_idle));
    assert!(t.in_hand(P0, "Unholy Strength"));
}
