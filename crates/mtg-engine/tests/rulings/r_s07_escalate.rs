//! Rulings batch S07 — escalate (CR 702.120): "For each mode you choose beyond the first
//! as you cast this spell, you pay an additional [cost]."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s07_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn is_modes(d: &Decision) -> bool {
    matches!(d, Decision::ChooseModes { .. })
}

/// Untapped lands P0 controls and whether a spell is on the stack.
fn lands_and_stack(g: &Game) -> (usize, usize) {
    let lands = g
        .permanents()
        .filter(|o| o.controller == P0 && o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count();
    (lands, g.stack.len())
}

#[test]
fn all_modes_are_chosen_at_once_as_the_spell_is_cast() {
    cr!("601.2b", "700.2a", "702.120a");
    ruling!(
        "Borrowed Hostility",
        "You choose all of your modes at once. You can't wait to perform one mode's actions and then decide to choose more modes."
    );
    supported("Borrowed Hostility");
    // Borrowed Hostility ({R}, escalate {3}): "• Target creature gets +3/+0 until end of
    // turn. • Target creature gains first strike until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Borrowed Hostility");
    let seen = watch(&mut t, P0, is_modes, lands_and_stack);
    let from = t.asked().len();
    t.cast(P0, c).modes(&[1]).target(bears).go();
    // The modes were chosen once, with the spell already on the stack and before any cost
    // was paid; only {R} was paid for the one mode.
    assert_eq!(*seen.lock().unwrap(), vec![(4, 1)]);
    assert_eq!(untapped_lands(&t, P0), 3);
    // Resolving performs the chosen mode; no other mode can be added then.
    t.resolve_all();
    assert_eq!(count_asked(&t, from, is_modes), 1);
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(untapped_lands(&t, P0), 3);
}

#[test]
fn all_modes_are_chosen_at_once_collective_resistance() {
    cr!("601.2b", "700.2a", "702.120a");
    ruling!(
        "Collective Resistance",
        "You choose all of your modes at once. You can't wait to perform one mode's actions and then decide to choose more modes."
    );
    supported("Collective Resistance");
    // Collective Resistance ({1}{G}, escalate {G}): "• Destroy target artifact. • Destroy
    // target enchantment. • Target creature gains hexproof and indestructible until end of
    // turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let relic = t.battlefield(P1, "Millstone");
    t.lands(P0, "Forest", 3);
    let c = t.hand(P0, "Collective Resistance");
    let from = t.asked().len();
    t.cast(P0, c).modes(&[0]).target(relic).go();
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve_all();
    assert_eq!(count_asked(&t, from, is_modes), 1);
    assert!(t.in_graveyard(P1, "Millstone"));
    assert!(!has_kw(&t, bears, KeywordKind::Hexproof));
    assert!(!has_kw(&t, bears, KeywordKind::Indestructible));
}

#[test]
fn a_mode_cant_be_chosen_twice() {
    cr!("700.2d", "702.120a");
    ruling!(
        "Borrowed Hostility",
        "You can't choose any one mode multiple times."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Borrowed Hostility");
    let from = t.asked().len();
    // Choosing the +3/+0 mode twice isn't a legal choice: the spell gets one mode, and no
    // escalate cost is paid.
    let spell = t.cast(P0, c).modes(&[0, 0]).target(bears).target(bears).go();
    assert_eq!(chosen_modes(&t, spell), vec![0]);
    let offered = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes {
                allow_repeat, max, ..
            } => Some((*allow_repeat, *max)),
            _ => None,
        })
        .unwrap();
    assert_eq!(offered, (false, 2));
    assert_eq!(untapped_lands(&t, P0), 3);
    t.clear_answers();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn a_mode_cant_be_chosen_twice_savage_alliance() {
    cr!("700.2d", "702.120a");
    ruling!(
        "Savage Alliance",
        "You can't choose any one mode multiple times."
    );
    supported("Savage Alliance");
    // Savage Alliance ({2}{R}, escalate {1}): "• Creatures target player controls gain
    // trample until end of turn. • ~ deals 2 damage to target creature. • ~ deals 1 damage
    // to each creature target opponent controls."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Savage Alliance");
    // Dealing 2 damage twice to the Hill Giant (3/3) isn't allowed: the spell gets a
    // single mode and costs {2}{R}.
    let spell = t.cast(P0, c).modes(&[1, 1]).target(giant).target(giant).go();
    assert_eq!(chosen_modes(&t, spell).len(), 1);
    assert_eq!(untapped_lands(&t, P0), 2);
    t.clear_answers();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert!(damage_on(&t, giant) < 4);
}

#[test]
fn the_same_creature_or_different_creatures_can_be_chosen_for_each_mode() {
    cr!("115.3", "702.120a");
    ruling!(
        "Borrowed Malevolence",
        "If two of the chosen modes of an escalate spell target a creature, you may choose the same creature for each mode's target, or choose different creatures."
    );
    supported("Borrowed Malevolence");
    // Borrowed Malevolence ({B}, escalate {2}): "• Target creature gets +1/+1 until end of
    // turn. • Target creature gets -1/-1 until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Swamp", 6);
    // The same creature for both modes.
    let c = t.hand(P0, "Borrowed Malevolence");
    t.cast(P0, c).modes(&[0, 1]).target(bears).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // Different creatures.
    let c = t.hand(P0, "Borrowed Malevolence");
    t.cast(P0, c).modes(&[0, 1]).target(bears).target(elves).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn the_same_player_can_be_chosen_for_each_mode() {
    cr!("115.3", "702.120a");
    ruling!(
        "Savage Alliance",
        "If two of the chosen modes of an escalate spell target a creature, you may choose the same creature for each mode's target, or choose different creatures. The same is true if the chosen modes target a player (or opponent)."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Savage Alliance");
    // All three modes: P1 is the target player of the first and the target opponent of
    // the third; the Hill Giant is the creature of the second.
    t.cast(P0, c)
        .modes(&[0, 1, 2])
        .target(Entity::Player(P1))
        .target(giant)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    // 2 + 1 damage destroys the Hill Giant; the Elves die to 1 damage; P1's creatures
    // gained trample, P0's didn't.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(!has_kw(&t, bears, KeywordKind::Trample));
    assert_eq!(damage_on(&t, bears), 0);
}

#[test]
fn the_same_creature_can_be_chosen_by_two_modes_savage_alliance() {
    cr!("115.3", "702.120a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Savage Alliance");
    // The Craw Wurm (6/4) is the target creature and belongs to the target opponent: it's
    // dealt 2 and then 1 damage.
    t.cast(P0, c)
        .modes(&[1, 2])
        .target(giant)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 3);
}

#[test]
fn an_illegal_target_doesnt_stop_the_other_modes() {
    cr!("608.2b", "702.120a");
    ruling!(
        "Collective Brutality",
        "If one target of an escalate spell becomes illegal, the other targets will still be affected. If all of the targets become illegal, the spell won't resolve."
    );
    supported("Collective Brutality");
    // Collective Brutality ({1}{B}, escalate—discard a card): "• Target opponent reveals
    // their hand ... • Target creature gets -2/-2 until end of turn. • Target opponent
    // loses 2 life and you gain 2 life."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    t.hand(P0, "Llanowar Elves");
    let c = t.hand(P0, "Collective Brutality");
    let spell = t
        .cast(P0, c)
        .modes(&[1, 2])
        .target(giant)
        .target(Entity::Player(P1))
        .go();
    // The Hill Giant leaves before the spell resolves: P1 still loses 2 life.
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn when_every_target_is_illegal_the_spell_doesnt_resolve() {
    cr!("608.2b", "702.120a");
    ruling!(
        "Borrowed Hostility",
        "If one target of an escalate spell becomes illegal, the other targets will still be affected. If all of the targets become illegal, the spell won't resolve."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Mountain", 8);
    // Only the first target leaves: the other creature still gains first strike.
    let c = t.hand(P0, "Borrowed Hostility");
    let spell = t.cast(P0, c).modes(&[0, 1]).target(elves).target(bears).go();
    destroy(&mut t, elves);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    assert_eq!(t.pt(bears), (2, 2));
    // Both targets leave: the spell doesn't resolve.
    let hill = t.battlefield(P0, "Hill Giant");
    let c = t.hand(P0, "Borrowed Hostility");
    let spell = t.cast(P0, c).modes(&[0, 1]).target(hill).target(bears).go();
    destroy(&mut t, hill);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(t.in_graveyard(P0, "Borrowed Hostility"));
}

#[test]
fn an_illegal_target_doesnt_stop_the_other_modes_savage_alliance() {
    cr!("608.2b", "702.120a");
    ruling!(
        "Savage Alliance",
        "If one target of an escalate spell becomes illegal, the other targets will still be affected. If all of the targets become illegal, the spell won't resolve."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Savage Alliance");
    let spell = t
        .cast(P0, c)
        .modes(&[1, 2])
        .target(elves)
        .target(Entity::Player(P1))
        .go();
    // The Elves (the target of the 2 damage) leave: each creature P1 controls is still
    // dealt 1 damage.
    destroy(&mut t, elves);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert_eq!(damage_on(&t, giant), 1);
}

#[test]
fn escalate_costs_dont_change_the_mana_value() {
    cr!("202.3", "601.2f", "702.120a");
    ruling!(
        "Savage Alliance",
        "Additional costs don't affect a spell's mana value."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Savage Alliance");
    let spell = t
        .cast(P0, c)
        .modes(&[0, 1, 2])
        .target(Entity::Player(P0))
        .target(giant)
        .target(Entity::Player(P1))
        .go();
    // {2}{R} plus {1} twice was paid; the mana value is still 3.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 3);
}

#[test]
fn escalate_costs_dont_change_the_mana_value_collective_effort() {
    cr!("202.3", "601.2f", "702.120a");
    ruling!(
        "Collective Effort",
        "Additional costs don't affect a spell's mana value."
    );
    supported("Collective Effort");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let c = t.hand(P0, "Collective Effort");
    // One mode beyond the first: an untapped creature is tapped for it.
    let spell = t
        .cast(P0, c)
        .modes(&[0, 2])
        .target(wurm)
        .target(Entity::Player(P0))
        .go();
    assert!(t.obj(bears).tapped);
    assert_eq!(t.g.mana_value_of(spell), 3);
}
