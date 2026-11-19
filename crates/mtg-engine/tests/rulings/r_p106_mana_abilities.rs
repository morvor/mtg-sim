//! Rulings batch P106 — which of these abilities are mana abilities (no stack, no
//! responses) and which aren't (triggered abilities that add mana, loyalty abilities)
//! (CR 605.1a, 605.1b, 605.3, 606.3, 405.6c).

use crate::r_p106_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn birgis_cast_trigger_uses_the_stack_and_can_be_responded_to() {
    cr!("605.1b", "603.3", "117.1");
    ruling!(
        "Birgi, God of Storytelling // Harnfel, Horn of Bounty",
        "Birgi’s first ability uses the stack and can be responded to. It isn’t a mana ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, BIRGI);
    mana(&mut t, P0, ManaType::R, 1);
    cast_from_hand(&mut t, P0, "Lava Spike", &[Entity::Player(P1)]);
    t.settle();
    // Lava Spike and Birgi's trigger above it; no {R} yet.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(pool_total(&t, P0), 0);
    // P1 responds with Shock while the trigger waits.
    mana(&mut t, P1, ManaType::R, 1);
    let shock = t.hand(P1, "Shock");
    t.cast_with(P1, shock, &[Entity::Player(P0)]).unwrap();
    assert_eq!(t.stack_len(), 3);
    t.resolve();
    assert_eq!(t.life(P0), 18);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn chandra_dressed_to_kills_mana_ability_uses_the_stack_even_without_targets() {
    cr!("606.3", "605.1a", "115.1");
    ruling!(
        "Chandra, Dressed to Kill",
        "Chandra’s first ability uses the stack and can be responded to, even if no targets were chosen. It isn’t a mana ability."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Dressed to Kill");
    assert!(!is_mana_ability(&mut t, chandra, "Add {R}"));
    t.answer_targets(P0, &[]);
    act(&mut t, P0, chandra, "Add {R}", &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn vexing_puzzleboxs_mana_ability_rolls_without_the_stack() {
    cr!("605.1a", "605.3a", "706.2");
    ruling!(
        "Vexing Puzzlebox",
        "Even though it includes rolling a die, Vexing Puzzlebox's second ability is a mana ability. It does not use the stack and cannot be responded to."
    );
    let mut t = TestGame::new(2);
    let bx = t.battlefield(P0, "Vexing Puzzlebox");
    assert!(is_mana_ability(&mut t, bx, "Roll a d20"));
    let m = mark(&t);
    assert!(tap_for_mana(&mut t, P0, bx, "Roll a d20"));
    // The mana is added and the die rolled at once; nothing went on the stack for it.
    assert_eq!(pool_total(&t, P0), 1);
    let rolled: Vec<u32> = events_since(&t, m)
        .iter()
        .filter_map(|e| match e {
            Event::DieRolled { result, .. } => Some(*result),
            _ => None,
        })
        .collect();
    assert_eq!(rolled.len(), 1);
    assert_eq!(t.stack_len(), 0);
    // The roll triggers its first ability, which does use the stack.
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(bx, counters::CHARGE), rolled[0]);
}

#[test]
fn koths_mana_loyalty_ability_uses_the_stack_once_per_turn_at_sorcery_speed() {
    cr!("606.3", "605.1a", "606.4");
    ruling!(
        "Koth of the Hammer",
        "Loyalty abilities can’t be mana abilities. Koth’s second ability uses the stack and can be countered or otherwise responded to. Like all loyalty abilities, it can be activated only once per turn, during your main phase, when the stack is empty, and only if no other loyalty abilities of this permanent have been activated this turn."
    );
    let mut t = TestGame::new(2);
    let koth = t.battlefield(P0, "Koth of the Hammer");
    t.lands(P0, "Mountain", 3);
    assert!(!is_mana_ability(&mut t, koth, "for each Mountain"));
    act(&mut t, P0, koth, "for each Mountain", &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 3);
    // No second loyalty ability this turn.
    assert!(!can_activate_containing(&mut t, P0, koth, "Untap target Mountain"));

    // Not with something on the stack, and not outside a main phase.
    let mut t = TestGame::new(2);
    let koth = t.battlefield(P0, "Koth of the Hammer");
    t.lands(P0, "Mountain", 2);
    mana(&mut t, P0, ManaType::R, 1);
    let spike = t.hand(P0, "Lightning Bolt");
    t.cast_with(P0, spike, &[Entity::Player(P1)]).unwrap();
    assert!(!can_activate_containing(&mut t, P0, koth, "for each Mountain"));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_activate_containing(&mut t, P0, koth, "for each Mountain"));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_activate_containing(&mut t, P0, koth, "for each Mountain"));
}

#[test]
fn outcaster_trailblazers_mana_trigger_uses_the_stack() {
    cr!("605.1b", "603.3");
    ruling!(
        "Outcaster Trailblazer",
        "Outcaster Trailblazer’s first ability isn’t a mana ability. It uses the stack and can be responded to."
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Outcaster Trailblazer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve();
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn simple_mana_abilities_add_their_mana_and_side_effects_at_once() {
    cr!("605.1a", "605.3a", "605.3b");
    ruling!(
        "Powerstone Shard",
        "Powerstone Shard’s activated ability is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    ruling!(
        "Pristine Talisman",
        "Pristine Talisman has a mana ability. Its ability doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Altar of the Pantheon",
        "The last ability of Altar of the Pantheon is a mana ability. It doesn't use the stack and can't be responded to. You'll immediately add mana and gain 1 life, if applicable."
    );
    // Powerstone Shard: {C} for each Shard, right away.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Powerstone Shard");
    t.battlefield(P0, "Powerstone Shard");
    assert!(is_mana_ability(&mut t, s, "Add {C}"));
    assert!(tap_for_mana(&mut t, P0, s, "Add {C}"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 2);
    assert_eq!(t.stack_len(), 0);

    // Pristine Talisman: the life comes with the mana.
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P0, "Pristine Talisman");
    assert!(is_mana_ability(&mut t, pt, "Add {C}"));
    assert!(tap_for_mana(&mut t, P0, pt, "Add {C}"));
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.stack_len(), 0);

    // Altar of the Pantheon with a God (Birgi): mana and life at once; without: no life.
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of the Pantheon");
    t.battlefield(P0, BIRGI);
    assert!(is_mana_ability(&mut t, altar, "Add one mana"));
    assert!(tap_for_mana(&mut t, P0, altar, "Add one mana"));
    assert_eq!(pool_total(&t, P0), 1);
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.stack_len(), 0);
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of the Pantheon");
    assert!(tap_for_mana(&mut t, P0, altar, "Add one mana"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn x_mana_abilities_are_mana_abilities() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Astral Cornucopia",
        "The last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    ruling!(
        "Kami of Whispered Hopes",
        "The last ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    // Astral Cornucopia with two charge counters: two mana of one color, right away.
    let mut t = TestGame::new(2);
    let ac = t.battlefield(P0, "Astral Cornucopia");
    put_counters(&mut t, ac, counters::CHARGE, 2);
    assert!(is_mana_ability(&mut t, ac, "Choose a color"));
    assert!(tap_for_mana(&mut t, P0, ac, "Choose a color"));
    assert_eq!(pool_total(&t, P0), 2);
    assert_eq!(t.stack_len(), 0);
    // Kami of Whispered Hopes with a +1/+1 counter (two, with its own replacement
    // effect): X mana of one color, right away.
    let mut t = TestGame::new(2);
    let kami = t.battlefield(P0, "Kami of Whispered Hopes");
    put_counters(&mut t, kami, counters::PLUS1, 1);
    let power = t.pt(kami).0;
    assert!(power >= 2);
    assert!(is_mana_ability(&mut t, kami, "Add X mana"));
    assert!(tap_for_mana(&mut t, P0, kami, "Add X mana"));
    assert_eq!(pool_total(&t, P0), power as u32);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn victory_chimes_gives_mana_without_the_stack_before_it_is_needed() {
    cr!("605.1a", "605.3a", "601.2g");
    ruling!(
        "Victory Chimes",
        "The activated ability of Victory Chimes is a mana ability and doesn’t use the stack. However, you can’t normally activate mana abilities when another player is asked to pay mana. If you’d like to give someone else mana, you must give them mana before they need it."
    );
    // In P1's main phase, so that P1 can cast Sol Ring.
    let mut t = TestGame::new(2);
    let chimes = t.battlefield(P0, "Victory Chimes");
    t.set_step(P1, Step::PrecombatMain);
    assert!(is_mana_ability(&mut t, chimes, "adds {C}"));
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.g.turn.priority = Some(P0);
    let uid = crate::r_s27_common::ability_containing(&mut t, chimes, "adds {C}");
    t.g.activate_ability(P0, chimes, uid).unwrap();
    t.g.flush_events();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P1).mana_pool.count(ManaType::C), 1);
    assert_eq!(pool_total(&t, P0), 0);
    // P1 spends it on Sol Ring; P0 isn't asked anything while P1 pays.
    let from = t.asked().len();
    let ring = t.hand(P1, "Sol Ring");
    t.cast_with(P1, ring, &[]).unwrap();
    assert_eq!(pool_total(&t, P1), 0);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && !matches!(d, Decision::Priority { .. })));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
}
