//! Rulings batch P018 — phasing (CR 702.26) with Disciple of Caelus Nin ("Permanents can't
//! phase in."), "for as long as" and "until" durations (CR 611.2b, 610.3, 610.4), "attacked
//! this turn by an Assassin" (CR 508.1, 506.3), looking at the top card of your library
//! any time (CR 401.2), a cast trigger resolving before its spell (CR 603.3), and one
//! loyalty ability per permanent per turn (CR 606.3).

use crate::r_p018_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s04_common::add_mana;
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::to_beginning_of_combat;
use mtg_engine::ability::Modification;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const DISCIPLE: &str = "Disciple of Caelus Nin";

/// P0's Disciple of Caelus Nin enters; P0 keeps `keep0` and P1 keeps `keep1` (each up to
/// five permanents they control); everything else phases out. Returns the Disciple.
fn disciple_enters(t: &mut TestGame, keep0: &[ObjectId], keep1: &[ObjectId]) -> ObjectId {
    let e0: Vec<Entity> = keep0.iter().map(|c| Entity::Object(*c)).collect();
    let e1: Vec<Entity> = keep1.iter().map(|c| Entity::Object(*c)).collect();
    t.answer_choose(P0, &e0);
    t.answer_choose(P1, &e1);
    let d = t.enter(P0, DISCIPLE);
    t.resolve_all();
    t.clear_answers();
    d
}

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

/// Advances to `p`'s next upkeep (through their untap step).
fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    let other = if p == P0 { P1 } else { P0 };
    if t.g.turn.active == p {
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
}

#[test]
fn a_for_as_long_as_effect_ends_when_its_source_phases_out() {
    cr!("702.26f", "611.2b");
    ruling!(
        "Disciple of Caelus Nin",
        "Any continuous effects with a “for as long as” duration, such as that of Vodalian Mindsinger, ignore phased-out objects. Any such effects will expire if their conditions are no longer met after ignoring the phased-out objects."
    );
    supported(DISCIPLE);
    supported("Vodalian Mindsinger");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    // Vodalian Mindsinger (1/1): "gain control of target creature with power less than
    // this creature's power for as long as you control this creature."
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    let singer = t.enter(P0, "Vodalian Mindsinger");
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P0);
    // The Disciple: P0 keeps the Ornithopter; the Mindsinger phases out.
    disciple_enters(&mut t, &[thopter], &[]);
    assert!(phased_out(&t, singer));
    t.g.recompute();
    assert_eq!(t.obj_now(thopter).controller, P1);
    // Even once the Mindsinger is back, the effect is over.
    let d = t.named_on_battlefield(DISCIPLE)[0];
    destroy(&mut t, d);
    to_upkeep(&mut t, P0);
    assert!(!phased_out(&t, singer));
    assert_eq!(t.obj_now(thopter).controller, P1);
}

#[test]
fn an_until_leaves_effect_waits_while_its_source_is_phased_out() {
    cr!("702.26d", "610.3");
    ruling!(
        "Disciple of Caelus Nin",
        "Any one-shot effects that are waiting “until [this] leaves the battlefield,” such as that of Banisher Priest, won’t happen when a permanent phases out."
    );
    supported("Banisher Priest");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let priest = t.enter(P0, "Banisher Priest");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    disciple_enters(&mut t, &[], &[]);
    assert!(phased_out(&t, priest));
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn disciple_phased_out_lets_others_phase_in_and_phases_in_with_them() {
    cr!("702.26a", "702.26b", "502.1");
    ruling!(
        "Disciple of Caelus Nin",
        "If Disciple of Caelus Nin itself phases out, other phased out permanents can phase back in normally while it is phased out. During its controller’s untap step, Caelus Nin and all other phased out permanents that player controls phase in at the same time."
    );
    for disciple_first in [true, false] {
        let mut t = TestGame::new(2);
        // P0's Bears phase out with the Disciple's ability (or, entering after it, by
        // another effect); so does P1's Hill Giant.
        let early = if disciple_first {
            None
        } else {
            Some(t.battlefield(P0, "Grizzly Bears"))
        };
        let giant = t.battlefield(P1, "Hill Giant");
        let d = disciple_enters(&mut t, &[], &[]);
        let bears = early.unwrap_or_else(|| {
            let b = t.battlefield(P0, "Grizzly Bears");
            mtg_engine::keyword_impls::phase_out(&mut t.g, vec![b]);
            b
        });
        assert!(phased_out(&t, bears) && phased_out(&t, giant));
        // The Disciple phases out too.
        mtg_engine::keyword_impls::phase_out(&mut t.g, vec![d]);
        t.g.recompute();
        assert!(phased_out(&t, d));
        // P1's untap step, while the Disciple is phased out: the Giant phases in normally.
        assert_eq!(t.g.turn.active, P0);
        to_upkeep(&mut t, P1);
        assert!(!phased_out(&t, giant), "disciple first: {disciple_first}");
        assert!(phased_out(&t, d) && phased_out(&t, bears));
        // P0's untap step: both phase in at the same time.
        to_upkeep(&mut t, P0);
        assert!(!phased_out(&t, d), "disciple first: {disciple_first}");
        assert!(!phased_out(&t, bears), "disciple first: {disciple_first}");
    }
}

#[test]
fn disciple_stops_everything_from_phasing_in() {
    cr!("702.26a", "101.2");
    ruling!(
        "Disciple of Caelus Nin",
        "The last ability means that no permanents phase in under any player’s control as long as Disciple of Caelus Nin is on the battlefield. This includes permanents that phase out in ways other than the first ability, and even permanents that are phased out already when it enters the battlefield. Once it leaves the battlefield, each phased out permanent will phase in normally during its controller’s untap step, with one exception (see below)."
    );
    let mut t = TestGame::new(2);
    // Phased out before the Disciple enters, and phased out by another effect after.
    let before = t.battlefield(P1, "Grizzly Bears");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![before]);
    let kept = t.battlefield(P0, "Hill Giant");
    let d = disciple_enters(&mut t, &[kept], &[]);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![kept]);
    let also = t.battlefield(P1, "Grizzly Bears");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![also]);
    to_upkeep(&mut t, P1);
    to_upkeep(&mut t, P0);
    for c in [before, kept, also] {
        assert!(phased_out(&t, c));
    }
    // The Disciple leaves: they phase in during their controllers' untap steps.
    destroy(&mut t, d);
    to_upkeep(&mut t, P1);
    assert!(!phased_out(&t, before) && !phased_out(&t, also));
    assert!(phased_out(&t, kept));
    to_upkeep(&mut t, P0);
    assert!(!phased_out(&t, kept));
}

#[test]
fn a_phase_out_until_event_during_disciple_holds_it_forever() {
    cr!("610.4", "610.4a", "702.26a");
    ruling!(
        "Disciple of Caelus Nin",
        "If an effect causes a permanent to phase out “until” some event occurs, and that event occurs while Disciple of Caelus Nin is still on the battlefield, the phased out permanent stays phased out forever."
    );
    supported("Oubliette");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let oubliette = t.enter(P0, "Oubliette");
    t.resolve_all();
    assert!(phased_out(&t, bears));
    let d = disciple_enters(&mut t, &[oubliette], &[]);
    destroy(&mut t, oubliette);
    assert!(phased_out(&t, bears));
    destroy(&mut t, d);
    for _ in 0..2 {
        to_upkeep(&mut t, P1);
        to_upkeep(&mut t, P0);
    }
    assert!(phased_out(&t, bears));
}

#[test]
fn phased_out_permanents_are_treated_as_though_they_dont_exist() {
    cr!("702.26b");
    ruling!(
        "Disciple of Caelus Nin",
        "While a permanent is phased out, it’s treated as though it doesn’t exist. It can’t be the target of spells or abilities, its static abilities have no effect on the game, its triggered abilities can’t trigger, it can’t attack or block, and so on."
    );
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let warden = t.battlefield(P0, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
    disciple_enters(&mut t, &[], &[giant]);
    for c in [anthem, warden, bears] {
        assert!(phased_out(&t, c));
    }
    t.g.recompute();
    // Static: no +1/+1 (the Disciple itself is 3/4).
    let d = t.named_on_battlefield(DISCIPLE)[0];
    assert_eq!(t.pt(d), (3, 4));
    // Trigger: Soul Warden doesn't see a creature entering.
    let life = t.life(P0);
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), life);
    // Target: Shock can't target the phased-out Bears.
    plenty_of_mana(&mut t, P0);
    let shock = t.hand(P0, "Shock");
    let from = t.asked().len();
    t.cast(P0, shock).go();
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(!offered[0].contains(&Entity::Object(bears)));
    assert!(offered[0].contains(&Entity::Object(giant)));
    t.g.stack.clear();
    empty_pool(&mut t, P0);
    // Attack: it can't.
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
}

// ---------------------------------------------------------------------------
// Ramses, Assassin Lord: "Whenever a player loses the game, if they were attacked this
// turn by an Assassin you controlled, you win the game."
// ---------------------------------------------------------------------------

const RAMSES: &str = "Ramses, Assassin Lord";

fn lose(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].life = 0;
    t.settle();
    t.resolve_all();
}

#[test]
fn ramses_wins_when_an_attacked_player_loses_for_any_reason() {
    cr!("104.2b", "603.4", "508.1");
    ruling!(
        "Ramses, Assassin Lord",
        "As long as that player was attacked this turn by an Assassin you controlled, Ramses, Assassin Lord’s last ability triggers when that player loses the game for any reason, not just due to combat damage. This is true even if the Assassin is no longer on the battlefield, no longer under your control, or no longer an Assassin at the time that player loses the game."
    );
    supported(RAMSES);
    supported("Royal Assassin");
    // The Assassin is gone, controlled by another player, or no longer an Assassin; P1
    // loses (not to combat damage).
    for change in ["destroyed", "stolen", "not an Assassin"] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, RAMSES);
        let assassin = t.battlefield(P0, "Royal Assassin");
        t.attack(&[(assassin, Entity::Player(P1))], &[]);
        match change {
            "destroyed" => destroy(&mut t, assassin),
            "stolen" => crate::r_s06_common::give_control(&mut t, assassin, P2),
            _ => {
                modify(&mut t, assassin, vec![Modification::RemoveAllCreatureTypes]);
                assert!(!t.obj_now(assassin).chars.has_subtype("Assassin"));
            }
        }
        t.advance_to(P0, Step::PostcombatMain);
        lose(&mut t, P1);
        assert!(t.has_lost(P1));
        assert!(
            matches!(
                &t.g.result,
                Some(mtg_engine::game::GameResult::Win(w)) if w == &vec![P0]
            ),
            "{change}"
        );
    }
}

#[test]
fn ramses_ignores_attacks_on_planeswalkers_and_late_assassins() {
    cr!("508.1", "508.2a", "506.3", "508.4");
    ruling!(
        "Ramses, Assassin Lord",
        "For the last ability to trigger, an Assassin you controlled must have attacked the player. Attacking a planeswalker the player controls won’t count."
    );
    ruling!(
        "Ramses, Assassin Lord",
        "If you attack an opponent with a non-Assassin creature and that creature becomes an Assassin after attackers were declared, that player was not “attacked” this turn by an Assassin you controlled."
    );
    ruling!(
        "Ramses, Assassin Lord",
        "Similarly, if an Assassin enters the battlefield under your control attacking a player, that creature didn’t “attack” and won’t cause this ability to trigger."
    );
    supported("Preeminent Captain");
    supported("Changeling Outcast");
    for case in ["planeswalker", "becomes", "enters"] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, RAMSES);
        match case {
            "planeswalker" => {
                let walker = t.battlefield(P1, "Ajani Goldmane");
                let assassin = t.battlefield(P0, "Royal Assassin");
                t.attack(&[(assassin, Entity::Object(walker))], &[]);
            }
            "becomes" => {
                let bears = t.battlefield(P0, "Grizzly Bears");
                attack_with(&mut t, &[(bears, Entity::Player(P1))]);
                modify(
                    &mut t,
                    bears,
                    vec![Modification::AddSubtypes(vec!["Assassin".into()])],
                );
                assert!(t.obj_now(bears).chars.has_subtype("Assassin"));
                t.advance_to(P0, Step::EndOfCombat);
            }
            _ => {
                // Preeminent Captain puts a Soldier (the changeling, an Assassin too)
                // onto the battlefield tapped and attacking.
                let captain = t.battlefield(P0, "Preeminent Captain");
                let outcast = t.hand(P0, "Changeling Outcast");
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[Entity::Object(outcast)]);
                t.answer_choose(P0, &[Entity::Player(P1)]);
                attack_with(&mut t, &[(captain, Entity::Player(P1))]);
                t.resolve_all();
                assert!(t.on_battlefield(outcast));
                assert!(t.obj_now(outcast).chars.has_subtype("Assassin"));
                assert!(t.g.is_attacking(t.g.current(outcast)));
                t.advance_to(P0, Step::EndOfCombat);
            }
        }
        t.advance_to(P0, Step::PostcombatMain);
        lose(&mut t, P1);
        assert!(t.has_lost(P1), "{case}");
        assert!(!t.g.is_over(), "{case}: Ramses didn't trigger");
    }
}

// ---------------------------------------------------------------------------
// Emperor Mihail II
// ---------------------------------------------------------------------------

const MIHAIL: &str = "Emperor Mihail II";

#[test]
fn emperor_mihail_lets_you_look_at_the_top_card_any_time() {
    cr!("401.2", "401.5");
    ruling!(
        "Emperor Mihail II",
        "Emperor Mihail II lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported(MIHAIL);
    let mut t = TestGame::new(2);
    t.battlefield(P0, MIHAIL);
    let top = t.library_top(P0, "Hill Giant");
    t.g.recompute();
    // Not P0's priority: P1 has it.
    t.g.turn.priority = Some(P1);
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, top));
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P1, top));
    // A new top card can be looked at too.
    let next = t.library_top(P0, "Grizzly Bears");
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, next));
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P0, top));
}

#[test]
fn emperor_mihails_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "603.2", "701.6a");
    ruling!(
        "Emperor Mihail II",
        "The last ability triggers whenever you cast any Merfolk spell, not just when you cast a Merfolk spell from the top of your library. It will resolve before the spell that caused it to trigger, even if that spell has been countered or otherwise left the stack."
    );
    supported("Counterspell");
    let mut t = TestGame::new(2);
    t.battlefield(P0, MIHAIL);
    add_mana(&mut t, P0, ManaType::U, 2);
    let merfolk = t.hand(P0, "Merfolk of the Pearl Trident");
    let spell = t.cast(P0, merfolk).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters the spell in response to the trigger.
    add_mana(&mut t, P1, ManaType::U, 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Merfolk of the Pearl Trident"));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.g.players[P0.idx()].mana_pool.total(), 0);
    let tokens = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(tokens.len(), 1);
    assert!(t.obj_now(tokens[0]).chars.has_subtype("Merfolk"));
}

// ---------------------------------------------------------------------------
// Liliana's Talent: "Enchanted planeswalker has "[−8]: Put all creature cards from all
// graveyards onto the battlefield under your control.""
// ---------------------------------------------------------------------------

#[test]
fn lilianas_talent_ability_shares_the_one_loyalty_activation_per_turn() {
    cr!("606.3");
    ruling!(
        "Liliana's Talent",
        "You can activate one loyalty ability of each planeswalker you control during each of your turns, whether that ability is one it always has or the one it gains because of Liliana's Talent."
    );
    supported("Liliana's Talent");
    supported("Ajani Goldmane");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    crate::r_s06_common::attach_new(&mut t, P0, "Liliana's Talent", ajani);
    put_counters(&mut t, ajani, counters::LOYALTY, 10);
    assert!(can_activate(&mut t, P0, ajani));
    // +1 (Ajani's own): then not the −8.
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(!can_activate(&mut t, P0, ajani));
    // Next turn, the gained −8.
    to_upkeep(&mut t, P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.graveyard(P1, "Hill Giant");
    let n = t
        .obj_now(ajani)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    t.activate(P0, ajani, n - 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert!(!can_activate(&mut t, P0, ajani));
}
