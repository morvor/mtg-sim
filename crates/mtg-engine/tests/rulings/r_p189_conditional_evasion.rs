//! Rulings batch P189 — conditional "can't be blocked" abilities and the other abilities
//! of their cards: Hooded Horror (checked only as blockers are declared, CR 509.1b),
//! Undercover Butler (a condition of the trigger event, CR 603.2), Swimmer in Nightmares
//! and Ghostly Pilferer.

use crate::r_p057_common::into_upkeep;
use crate::r_p076_common::{mana, unblockable_after_blocked};
use crate::r_p189_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Hooded Horror: "This creature can't be blocked as long as defending player controls
// the most creatures or is tied for the most."
// ---------------------------------------------------------------------------

#[test]
fn hooded_horror_evaluated_as_blockers_are_declared() {
    cr!("509.1b", "611.3a");
    ruling!(
        "Hooded Horror",
        "The number of creatures each player controls is evaluated only as blocking creatures are declared. If, at that time, the defending player doesn’t control the most creatures or isn’t tied for the most, Hooded Horror can be blocked."
    );
    supported("Hooded Horror");
    // Tied (two each): it can't be blocked.
    let mut t = TestGame::new(2);
    let hh = t.battlefield(P0, "Hooded Horror");
    t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(hh, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(b1, hh)]));
    // The defending player has fewer creatures: it can be blocked.
    let mut t = TestGame::new(2);
    let hh = t.battlefield(P0, "Hooded Horror");
    t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(hh, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(b1, hh)]));
}

#[test]
fn hooded_horror_stays_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Hooded Horror",
        "Once Hooded Horror is blocked, it doesn’t matter if the defending player controls the most creatures. The block won’t be undone."
    );
    supported("Hooded Horror");
    let mut t = TestGame::new(2);
    let hh = t.battlefield(P0, "Hooded Horror");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, hh, blocker, move |t| {
        // Now each player controls one creature.
        t.g.destroy(bears, None);
        t.settle();
    });
}

// ---------------------------------------------------------------------------
// Undercover Butler: "Whenever this creature attacks the player with the most life or
// tied for most life, it can't be blocked this turn."
// ---------------------------------------------------------------------------

#[test]
fn undercover_butler_life_changes_after_triggering_dont_matter() {
    cr!("603.2", "603.4");
    ruling!(
        "Undercover Butler",
        "Once Undercover Butler's ability triggers, it doesn't matter what happens to players' life totals before the ability resolves."
    );
    supported("Undercover Butler");
    let mut t = TestGame::new(2);
    let ub = t.battlefield(P0, "Undercover Butler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(ub, Entity::Player(P1))]);
    assert_eq!(stack_triggers_from(&t, ub).len(), 1);
    t.g.lose_life(P1, 10);
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(bears, ub)]));
    // It doesn't trigger against a player with less life.
    let mut t = TestGame::new(2);
    let ub = t.battlefield(P0, "Undercover Butler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.lose_life(P1, 1);
    attack_with(&mut t, &[(ub, Entity::Player(P1))]);
    assert_eq!(triggers_of(&t, ub), 0);
    assert!(legal_blocks(&mut t, P1, &[(bears, ub)]));
}

#[test]
fn undercover_butler_attacking_a_planeswalker_doesnt_trigger() {
    cr!("603.2", "508.1b");
    ruling!(
        "Undercover Butler",
        "Undercover Butler's ability doesn't trigger if it attacks a planeswalker or battle, no matter what the life total of the planeswalker's controller or battle's protector is."
    );
    supported("Undercover Butler");
    let mut t = TestGame::new(2);
    let ub = t.battlefield(P0, "Undercover Butler");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.g.lose_life(P0, 5);
    attack_with(&mut t, &[(ub, Entity::Object(jace))]);
    assert_eq!(triggers_of(&t, ub), 0);
}

// ---------------------------------------------------------------------------
// Swimmer in Nightmares: "This creature gets +3/+0 as long as there are ten or more
// cards in a single graveyard."
// ---------------------------------------------------------------------------

#[test]
fn swimmer_in_nightmares_one_bonus_for_any_number_of_graveyards() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Swimmer in Nightmares",
        "The first ability of Swimmer in Nightmares gives it just +3/+0, no matter how many graveyards beyond the first have ten cards."
    );
    supported("Swimmer in Nightmares");
    let mut t = TestGame::new(2);
    let sw = t.battlefield(P0, "Swimmer in Nightmares");
    // Nine and nine: no single graveyard has ten.
    fill_graveyard(&mut t, P0, "Island", 9);
    fill_graveyard(&mut t, P1, "Island", 9);
    t.g.recompute();
    assert_eq!(t.pt(sw), (1, 4));
    t.graveyard(P1, "Island");
    t.g.recompute();
    assert_eq!(t.pt(sw), (4, 4));
    t.graveyard(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(sw), (4, 4));
}

#[test]
fn swimmer_in_nightmares_bonus_only_on_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!(
        "Swimmer in Nightmares",
        "The ability that modifies the power of Swimmer in Nightmares applies only while it's on the battlefield."
    );
    supported("Swimmer in Nightmares");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P1, "Island", 10);
    let in_gy = t.graveyard(P0, "Swimmer in Nightmares");
    let in_hand = t.hand(P0, "Swimmer in Nightmares");
    t.g.recompute();
    assert_eq!(t.pt(in_gy), (1, 4));
    assert_eq!(t.pt(in_hand), (1, 4));
}

// ---------------------------------------------------------------------------
// Ghostly Pilferer: "Whenever this creature becomes untapped, you may pay {2}. If you
// do, draw a card. Whenever an opponent casts a spell from anywhere other than their
// hand, draw a card. Discard a card: This creature can't be blocked this turn."
// ---------------------------------------------------------------------------

#[test]
fn ghostly_pilferer_untap_trigger_ordered_with_upkeep_triggers() {
    cr!("603.3b", "502.3", "503.1a");
    ruling!(
        "Ghostly Pilferer",
        "Ghostly Pilferer's first ability triggers during your untap step, but it's put onto the stack at the same time as abilities that trigger at the beginning of your upkeep step."
    );
    supported("Ghostly Pilferer");
    supported("Phyrexian Arena");
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Ghostly Pilferer");
    let arena = t.battlefield(P0, "Phyrexian Arena");
    t.g.objects[gp.0 as usize].tapped = true;
    into_upkeep(&mut t, P0);
    assert!(!t.obj(gp).tapped);
    assert_eq!(stack_triggers_from(&t, gp).len(), 1);
    assert_eq!(stack_triggers_from(&t, arena).len(), 1);
    // Both were put on the stack together, in an order P0 chose.
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
}

#[test]
fn ghostly_pilferer_pays_only_once() {
    cr!("603.5", "118.1");
    ruling!(
        "Ghostly Pilferer",
        "While resolving Ghostly Pilferer's first ability, you can't pay more than {2} to draw more than one card."
    );
    supported("Ghostly Pilferer");
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Ghostly Pilferer");
    t.g.objects[gp.0 as usize].tapped = true;
    mana(&mut t, P0, ManaType::C, 6);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    let hand = t.hand_size(P0);
    t.g.untap(gp);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 4);
}

#[test]
fn ghostly_pilferer_players_respond_after_the_draw() {
    cr!("603.3", "117.3b");
    ruling!(
        "Ghostly Pilferer",
        "Players can cast spells and activate abilities after the second triggered ability resolves but before the spell that caused it to trigger does."
    );
    supported("Ghostly Pilferer");
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Ghostly Pilferer");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    // P1 casts Think Twice with flashback from their graveyard.
    let tt = t.graveyard(P1, "Think Twice");
    mana(&mut t, P1, ManaType::U, 3);
    let spell = t
        .cast(P1, tt)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .go();
    t.settle();
    assert_eq!(*t.g.stack.last().unwrap(), stack_triggers_from(&t, gp)[0]);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    // The spell is still on the stack: P0 can respond to it, e.g. counter it.
    assert!(t.g.stack.contains(&spell));
    mana(&mut t, P0, ManaType::U, 2);
    let cs = t.hand(P0, "Counterspell");
    t.cast(P0, cs).target(spell).go();
    t.resolve_all();
    assert!(t.in_exile("Think Twice"), "countered, then exiled by flashback");
    // A spell cast from hand doesn't trigger it.
    let opt = t.hand(P1, "Opt");
    mana(&mut t, P1, ManaType::U, 1);
    t.cast(P1, opt).go();
    t.settle();
    assert_eq!(triggers_of(&t, gp), 1);
}

#[test]
fn ghostly_pilferer_stays_blocked() {
    cr!("509.1h", "506.4");
    ruling!(
        "Ghostly Pilferer",
        "Once Ghostly Pilferer has been blocked, activating its last ability doesn't cause it to become unblocked."
    );
    supported("Ghostly Pilferer");
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Ghostly Pilferer");
    let card = t.hand(P0, "Island");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, gp, blocker, move |t| {
        t.answer_choose(P0, &[Entity::Object(card)]);
        t.activate(P0, gp, 0, &[]).expect("discard a card");
        assert!(t.in_graveyard(P0, "Island"));
    });
}
