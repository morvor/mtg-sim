//! Rulings batch P165 — blue synergies: "if you've cast another blue spell this turn"
//! (Dream Thief), devotion to blue (Master of Waves, Thassa's Oracle, Threnody Singer),
//! Tidal Influence, Paragon of Gathering Mists, Floodgate's state trigger, Gadwick, Thassa,
//! Deep-Dwelling, Artistic Refusal, Temporal Cleansing, Complete the Circuit, Sedraxis
//! Alchemist, Statute of Denial, Brine Seer and Scent of Brine, Abjure, and Harmonized
//! Crescendo.

use crate::r_p108_common::put_counters;
use crate::r_p189_common::{bounce, triggers_of};
use crate::r_s01_common::{asked_since, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s25_common::{cast_new, change_copy_targets, lands_for_cost, spell_copies};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn p1() -> Entity {
    Entity::Player(P1)
}

fn cast_opt(t: &mut TestGame, p: PlayerId) {
    cast_new(t, p, "Opt", &[]);
    t.resolve_all();
}

// ---------------------------------------------------------------------------------------
// Dream Thief
// ---------------------------------------------------------------------------------------

#[test]
fn dream_thief_checks_on_resolution_even_if_it_wasnt_cast() {
    cr!("603.4", "608.2c");
    ruling!(
        "Dream Thief",
        "Although Dream Thief says it looks for \"another blue spell,\" there's no requirement that Dream Thief actually be cast as a spell (or be blue) for this part of its ability to work."
    );
    supported("Dream Thief");
    for blue_spell in [false, true] {
        let mut t = TestGame::new(2);
        if blue_spell {
            cast_opt(&mut t, P0);
        }
        let hand = t.hand_size(P0);
        // Put onto the battlefield directly (as Zombify would).
        t.enter(P0, "Dream Thief");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + usize::from(blue_spell));
    }
}

#[test]
fn dream_thief_counts_a_blue_spell_cast_in_response_to_its_trigger() {
    cr!("608.2c", "603.2");
    ruling!(
        "Dream Thief",
        "Dream Thief doesn't check whether you've cast another blue spell until its \"enters\" ability resolves. If Dream Thief's \"enters\" ability triggers, then you cast a blue instant spell in response, you'll get the bonus."
    );
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, "Dream Thief", &[]);
        t.resolve();
        // The trigger is on the stack; Dream Thief itself doesn't count.
        assert_eq!(t.stack_len(), 1);
        let hand = t.hand_size(P0);
        if respond {
            lands_for_cost(&mut t, P0, "Opt");
            let opt = t.hand(P0, "Opt");
            t.cast(P0, opt).go();
            // Opt resolves first (scry 1, draw a card).
            t.resolve();
        }
        let after_opt = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), after_opt + usize::from(respond));
        assert!(after_opt >= hand);
    }
}

// ---------------------------------------------------------------------------------------
// Tidal Influence
// ---------------------------------------------------------------------------------------

#[test]
fn tidal_influence_changes_blue_creatures_power_as_soon_as_counters_change() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Tidal Influence",
        "As soon as the number of counters on this card changes, so does the power of blue creatures."
    );
    supported("Tidal Influence");
    let mut t = TestGame::new(2);
    let merfolk = t.battlefield(P1, "Coral Merfolk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ti = t.battlefield(P0, "Tidal Influence");
    assert_eq!(t.pt(merfolk), (2, 1));
    put_counters(&mut t, ti, "tide", 1);
    assert_eq!(t.pt(merfolk), (0, 1));
    assert_eq!(t.pt(bears), (2, 2));
    put_counters(&mut t, ti, "tide", 1);
    assert_eq!(t.pt(merfolk), (2, 1));
    put_counters(&mut t, ti, "tide", 1);
    assert_eq!(t.pt(merfolk), (4, 1));
    assert_eq!(t.pt(bears), (2, 2));
}

// ---------------------------------------------------------------------------------------
// Master of Waves
// ---------------------------------------------------------------------------------------

fn elementals(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    crate::r_s01_common::with_subtype(t, p, "Elemental")
}

#[test]
fn master_of_waves_tokens_are_two_one_and_die_without_it() {
    cr!("613.4c", "704.5f", "704.5d");
    ruling!(
        "Master of Waves",
        "Barring other effects, the Elemental tokens will enter the battlefield as 2/1 creatures."
    );
    ruling!(
        "Master of Waves",
        "If Master of Waves leaves the battlefield, and nothing else is raising the toughness of the Elemental tokens above 0, the Elemental tokens will die and subsequently cease to exist."
    );
    supported("Master of Waves");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coral Merfolk");
    let master = t.enter(P0, "Master of Waves");
    t.resolve_all();
    // Devotion 2: Master's {U} and Coral Merfolk's {U}.
    let els = elementals(&t, P0);
    assert_eq!(els.len(), 2);
    for e in &els {
        assert_eq!(t.pt(*e), (2, 1));
    }
    bounce(&mut t, master);
    t.settle();
    assert!(elementals(&t, P0).is_empty());
    for e in els {
        assert!(!t.g.is_live(e) || t.obj_now(e).zone != Zone::Battlefield);
    }
}

#[test]
fn master_of_waves_uses_devotion_as_its_trigger_resolves() {
    cr!("608.2h", "700.5");
    ruling!(
        "Master of Waves",
        "Use your devotion to blue as the triggered ability resolves to determine how many tokens are put onto the battlefield. If Master of Waves is still on the battlefield at that time, its mana cost will count toward your devotion to blue."
    );
    // More devotion by resolution: Coral Merfolk enters in response.
    let mut t = TestGame::new(2);
    t.enter(P0, "Master of Waves");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Coral Merfolk");
    t.resolve_all();
    assert_eq!(elementals(&t, P0).len(), 2);
    // Master leaves in response: it doesn't count; only Coral Merfolk's {U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coral Merfolk");
    let master = t.enter(P0, "Master of Waves");
    t.settle();
    bounce(&mut t, master);
    t.resolve_all();
    // The 1/0 token dies without Master.
    assert_eq!(triggers_of(&t, master), 1);
    let made = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, mtg_engine::events::Event::TokenCreated { .. }))
        .count();
    assert_eq!(made, 1);
}

// ---------------------------------------------------------------------------------------
// Paragon of Gathering Mists
// ---------------------------------------------------------------------------------------

#[test]
fn gaining_flying_after_being_blocked_doesnt_undo_the_block() {
    cr!("509.1h", "506.4");
    ruling!(
        "Paragon of Gathering Mists",
        "Causing a creature to gain flying after it’s been blocked won’t change or undo that block."
    );
    supported("Paragon of Gathering Mists");
    let mut t = TestGame::new(2);
    let paragon = t.battlefield(P0, "Paragon of Gathering Mists");
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(merfolk, p1())], &[(bears, merfolk)]);
    t.lands(P0, "Island", 1);
    t.activate(P0, paragon, 0, &[o(merfolk)]).expect("paragon");
    t.resolve_all();
    assert!(crate::r_s06_common::has_kw(
        &t,
        merfolk,
        mtg_engine::keywords::KeywordKind::Flying
    ));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(merfolk));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // Coral Merfolk (3/2 with Paragon) and the Bears trade.
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Floodgate
// ---------------------------------------------------------------------------------------

#[test]
fn floodgates_state_trigger_retriggers_after_being_countered() {
    cr!("603.8");
    ruling!(
        "Floodgate",
        "Floodgate’s first triggered ability is a state trigger. It won’t trigger again while the ability is on the stack, but if the ability is countered and Floodgate still has flying, it will trigger again immediately."
    );
    supported("Floodgate");
    supported("Stifle");
    let mut t = TestGame::new(2);
    let fg = t.battlefield(P0, "Floodgate");
    // It gains flying until end of turn.
    crate::r_p117_common::modify(
        &mut t,
        fg,
        vec![mtg_engine::ability::Modification::AddKeyword(
            mtg_engine::keywords::KeywordKind::Flying.into(),
        )],
    );
    assert_eq!(t.stack_len(), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    // Stifle it: it triggers again.
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(trig).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_ne!(t.g.stack[0], trig);
    assert_eq!(triggers_of(&t, fg), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Floodgate"));
}

// ---------------------------------------------------------------------------------------
// Gadwick, the Wizened
// ---------------------------------------------------------------------------------------

#[test]
fn gadwicks_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Gadwick, the Wizened",
        "Gadwick's second ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Gadwick, the Wizened");
    let mut t = TestGame::new(2);
    let gad = t.battlefield(P0, "Gadwick, the Wizened");
    let land = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(land)]);
    let opt = cast_new(&mut t, P0, "Opt", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], opt);
    assert_eq!(triggers_of(&t, gad), 1);
    // P1 counters Opt in response.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(opt).go();
    t.resolve(); // Counterspell
    assert!(t.in_graveyard(P0, "Opt"));
    t.resolve_all();
    assert!(t.obj_now(land).tapped);
}

#[test]
fn gadwick_doesnt_trigger_for_itself_and_x_is_zero_if_not_cast() {
    cr!("603.6a", "107.3m");
    ruling!(
        "Gadwick, the Wizened",
        "Gadwick's triggered ability won't trigger when you cast it because it's not on the battlefield yet."
    );
    ruling!(
        "Gadwick, the Wizened",
        "If Gadwick enters the battlefield without being cast, or if it was cast for a cost other than its mana cost, the value of X for its first ability is 0."
    );
    // Cast with X = 2: no tap trigger, draws two.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Gadwick, the Wizened");
    t.lands(P0, "Wastes", 2);
    let gad = t.hand(P0, "Gadwick, the Wizened");
    let hand = t.hand_size(P0) - 1;
    let spell = t.cast(P0, gad).x(2).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.stack[0], spell);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Put onto the battlefield: X is 0.
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    t.enter(P0, "Gadwick, the Wizened");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

// ---------------------------------------------------------------------------------------
// Artistic Refusal
// ---------------------------------------------------------------------------------------

#[test]
fn artistic_refusal_with_both_modes_does_nothing_if_its_target_is_gone() {
    cr!("608.2b");
    ruling!(
        "Artistic Refusal",
        "If you choose both modes, and the target of the first mode is an illegal target at the time Artistic Refusal tries to resolve (probably because that spell has been countered by something else), Artistic Refusal won't resolve and none of its effects will happen. You won't draw or discard any cards."
    );
    supported("Artistic Refusal");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = cast_new(&mut t, P1, "Grizzly Bears", &[]);
    // P0 responds with Artistic Refusal (both modes), then counters the Bears first.
    lands_for_cost(&mut t, P0, "Artistic Refusal");
    let ar = t.hand(P0, "Artistic Refusal");
    let ar = t.cast(P0, ar).modes(&[0, 1]).target(bears).go();
    let cs = cast_new(&mut t, P0, "Counterspell", &[o(bears)]);
    let hand = t.hand_size(P0);
    t.resolve(); // Counterspell
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let from = t.asked().len();
    t.resolve(); // Artistic Refusal
    assert!(t.in_graveyard(P0, "Artistic Refusal"));
    assert_eq!(t.hand_size(P0), hand);
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    let _ = (ar, cs);
}

// ---------------------------------------------------------------------------------------
// Thassa, Deep-Dwelling
// ---------------------------------------------------------------------------------------

/// P0 controls P1's Grizzly Bears until end of turn (Act of Treason) with Thassa out.
fn borrowed_bears(players: usize) -> TestGame {
    let mut t = TestGame::new(players);
    supported("Thassa, Deep-Dwelling");
    supported("Act of Treason");
    t.battlefield(P0, "Thassa, Deep-Dwelling");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Act of Treason", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.answer_targets(P0, &[o(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t
}

#[test]
fn thassa_blinks_a_creature_borrowed_until_end_of_turn_and_you_keep_it() {
    cr!("611.2c", "400.7", "610.3");
    ruling!(
        "Thassa, Deep-Dwelling",
        "If you gain control of a creature “until end of turn,” you’ll control it during your end step."
    );
    ruling!(
        "Thassa, Deep-Dwelling",
        "When an effect returns the exiled card “under your control,” you control it indefinitely after that."
    );
    let mut t = borrowed_bears(2);
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert_eq!(t.obj_now(bears[0]).owner, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears[0]).controller, P0);
}

#[test]
fn thassas_creature_you_dont_own_is_exiled_when_you_leave_the_game() {
    cr!("800.4a");
    ruling!(
        "Thassa, Deep-Dwelling",
        "If you leave the game, any creatures you control from Thassa’s effect but don’t own are exiled."
    );
    let mut t = borrowed_bears(3);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj_now(bears).controller, P0);
    t.g.lose_game(P0);
    t.settle();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn thassas_returned_creature_is_a_new_object() {
    cr!("400.7", "704.5m", "704.5n");
    ruling!(
        "Thassa, Deep-Dwelling",
        "When the card returns to the battlefield, it will be a new object with no connection to the card that was exiled. Auras attached to the exiled creature will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled creature will cease to exist."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thassa, Deep-Dwelling");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bears);
    let sword = t.battlefield(P0, "Bonesplitter");
    t.g.attach(sword, o(bears));
    put_counters(&mut t, bears, "+1/+1", 2);
    assert_eq!(t.pt(bears), (7, 6));
    t.answer_targets(P0, &[o(bears)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let now = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(now.len(), 1);
    assert_ne!(now[0], bears);
    assert_eq!(t.pt(now[0]), (2, 2));
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
}

// ---------------------------------------------------------------------------------------
// Thassa's Oracle
// ---------------------------------------------------------------------------------------

#[test]
fn thassas_oracle_with_zero_devotion_and_no_library_wins() {
    cr!("700.5", "104.2b");
    ruling!(
        "Thassa's Oracle",
        "If your devotion to blue is zero at the time the triggered ability of Thassa's Oracle resolves, you don't look at or move any cards in your library. If you have no cards in your library, you win the game."
    );
    supported("Thassa's Oracle");
    // Devotion 0 (Oracle bounced in response) with a library: nothing happens.
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Island");
    let oracle = t.enter(P0, "Thassa's Oracle");
    t.settle();
    bounce(&mut t, oracle);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.g.player(P0).has_won);
    assert_eq!(t.g.player(P0).library.last(), Some(&top));
    assert!(asked_since(&t, from).is_empty());
    // Devotion 0 and no library: X (0) >= 0 cards: P0 wins.
    let mut t = TestGame::new(2);
    let lib: Vec<ObjectId> = t.g.player(P0).library.clone();
    for id in lib {
        t.g.move_object(id, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    let oracle = t.enter(P0, "Thassa's Oracle");
    t.settle();
    bounce(&mut t, oracle);
    t.resolve_all();
    assert!(t.g.player(P0).has_won || t.has_lost(P1));
}

// ---------------------------------------------------------------------------------------
// Temporal Cleansing
// ---------------------------------------------------------------------------------------

#[test]
fn temporal_cleansings_owner_chooses_and_everyone_knows() {
    cr!("401.4");
    ruling!(
        "Temporal Cleansing",
        "The owner of the permanent chooses where the card ends up. All players get to know that choice."
    );
    supported("Temporal Cleansing");
    for bottom in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let from = t.asked().len();
        t.answer(
            P1,
            DecisionKind::Option,
            Answer::Index(usize::from(bottom)),
        );
        cast_new(&mut t, P0, "Temporal Cleansing", &[o(bears)]);
        t.resolve_all();
        let choosers: Vec<PlayerId> = asked_since(&t, from)
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(choosers, vec![P1]);
        let lib = &t.g.player(P1).library;
        let card = t.g.current(bears);
        let pos = lib.iter().position(|id| *id == card).expect("in library");
        if bottom {
            assert_eq!(pos, 0);
        } else {
            assert_eq!(pos, lib.len() - 2);
        }
        // The choice is public: the log says where it went.
        let log = t.dump_log();
        assert!(log.contains("Grizzly Bears"), "{log}");
    }
}

#[test]
fn temporal_cleansing_puts_all_cards_of_a_melded_permanent_together() {
    cr!("401.4", "712.4c");
    ruling!(
        "Temporal Cleansing",
        "In some unusual cases—though getting less unusual all the time—a permanent will be comprised of multiple cards, such as a melded permanent or a mutated creature. In those cases, the owner of the permanent puts all those cards second from the top (that is, just under the top card) or on the bottom in any order."
    );
    for bottom in [false, true] {
        let mut t = TestGame::new(2);
        let bruna = t.battlefield(P1, "Bruna, the Fading Light");
        let gisela = t.battlefield(P1, "Gisela, the Broken Blade");
        let a = t.g.exile_object(bruna, None).unwrap();
        let b = t.g.exile_object(gisela, None).unwrap();
        let brisela = mtg_engine::merge::meld(
            &mut t.g,
            a,
            b,
            mtg_engine::card::card("Brisela, Voice of Nightmares"),
            P1,
        )
        .expect("melded");
        t.g.recompute();
        let top = *t.g.player(P1).library.last().unwrap();
        t.answer(
            P1,
            DecisionKind::Option,
            Answer::Index(usize::from(bottom)),
        );
        cast_new(&mut t, P0, "Temporal Cleansing", &[o(brisela)]);
        t.resolve_all();
        let lib = t.g.player(P1).library.clone();
        let n = lib.len();
        let names: Vec<String> = lib
            .iter()
            .map(|id| t.g.obj(*id).chars.name.to_string())
            .collect();
        assert_eq!(n, 32, "{names:?}");
        assert_eq!(lib[n - 1], top);
        let melded = if bottom { &names[0..2] } else { &names[n - 3..n - 1] };
        let mut melded = melded.to_vec();
        melded.sort();
        assert_eq!(
            melded,
            vec!["Bruna, the Fading Light", "Gisela, the Broken Blade"],
            "bottom: {bottom}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Complete the Circuit
// ---------------------------------------------------------------------------------------

#[test]
fn complete_the_circuit_flash_is_only_for_casting_sorceries() {
    cr!("601.3", "602.5d");
    ruling!(
        "Complete the Circuit",
        "The \"as though they had flash\" effect applies only to casting sorcery spells. It does not, for example, change when you may activate abilities that can be activated \"only as a sorcery.\""
    );
    supported("Complete the Circuit");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.set_step(P0, Step::BeginningOfCombat);
    cast_new(&mut t, P0, "Complete the Circuit", &[]);
    t.resolve_all();
    // A sorcery during combat: fine.
    let div = cast_new(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Divination"));
    let _ = div;
    // Equip ("only as a sorcery"): not now.
    t.lands(P0, "Wastes", 1);
    assert!(t.activate(P0, sword, 0, &[o(bears)]).is_err());
}

#[test]
fn complete_the_circuit_copies_may_have_different_new_targets() {
    cr!("707.10c", "603.7c");
    ruling!(
        "Complete the Circuit",
        "You may choose different new targets for each of the copies."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    cast_new(&mut t, P0, "Complete the Circuit", &[]);
    t.resolve_all();
    // Shock at P1; the copies at the Bears and the Elves.
    change_copy_targets(&mut t, P0, &[Some(o(a))]);
    change_copy_targets(&mut t, P0, &[Some(o(b))]);
    cast_new(&mut t, P0, "Shock", &[p1()]);
    t.settle();
    // The "when you next cast" trigger resolves, copying the Shock twice.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

// ---------------------------------------------------------------------------------------
// Threnody Singer
// ---------------------------------------------------------------------------------------

#[test]
fn threnody_singers_x_is_locked_in_on_resolution() {
    cr!("608.2h", "611.2b");
    ruling!(
        "Threnody Singer",
        "The value of X is determined only as the triggered ability resolves. Once that happens, the value of X won’t change later in the turn even if your devotion to blue changes."
    );
    supported("Threnody Singer");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[o(giant)]);
    t.enter(P0, "Threnody Singer");
    t.settle();
    // Coral Merfolk arrives before resolution: devotion 2.
    t.battlefield(P0, "Coral Merfolk");
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 3));
    // More devotion later doesn't change it.
    t.battlefield(P0, "Coral Merfolk");
    t.settle();
    assert_eq!(t.pt(giant), (1, 3));
}

// ---------------------------------------------------------------------------------------
// Sedraxis Alchemist
// ---------------------------------------------------------------------------------------

#[test]
fn sedraxis_alchemist_must_target_your_own_permanent_if_thats_all_there_is() {
    cr!("603.3d", "115.1");
    ruling!(
        "Sedraxis Alchemist",
        "This ability isn't optional. When Sedraxis Alchemist enters, if you control a blue permanent and no other player controls a nonland permanent, you'll have to target a nonland permanent you control."
    );
    supported("Sedraxis Alchemist");
    let mut t = TestGame::new(2);
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    t.battlefield(P1, "Island");
    t.answer_targets(P0, &[o(merfolk)]);
    t.enter(P0, "Sedraxis Alchemist");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let targets = crate::r_s25_common::targets_of(&t, t.g.stack[0]);
    assert_eq!(targets.len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Coral Merfolk"));
    // Answering "no target" isn't an option: the default still picks one of ours.
    let mut t = TestGame::new(2);
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Sedraxis Alchemist");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let targets = crate::r_s25_common::targets_of(&t, t.g.stack[0]);
    assert_eq!(targets.len(), 1);
    let _ = merfolk;
}

// ---------------------------------------------------------------------------------------
// Statute of Denial
// ---------------------------------------------------------------------------------------

#[test]
fn statute_of_denial_checks_for_a_blue_creature_on_resolution_and_loots() {
    cr!("608.2c");
    ruling!(
        "Statute of Denial",
        "Whether you control a blue creature is checked only when Statute of Denial resolves. If you do, drawing and discarding a card is mandatory. You can’t choose to not do it."
    );
    supported("Statute of Denial");
    for (merfolk_at_cast, merfolk_at_resolve) in [(true, false), (false, true)] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let merfolk = merfolk_at_cast.then(|| t.battlefield(P0, "Coral Merfolk"));
        let bears = cast_new(&mut t, P1, "Grizzly Bears", &[]);
        cast_new(&mut t, P0, "Statute of Denial", &[o(bears)]);
        if let Some(m) = merfolk {
            bounce(&mut t, m);
        }
        if merfolk_at_resolve {
            t.battlefield(P0, "Coral Merfolk");
        }
        let lib = t.library_size(P0);
        let gy = t.graveyard_size(P0);
        // Try to decline: nothing optional is offered.
        t.answer_yes(P0, false);
        t.resolve();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        let looted = merfolk_at_resolve;
        assert_eq!(t.library_size(P0), lib - usize::from(looted));
        // Statute itself plus the discarded card.
        assert_eq!(t.graveyard_size(P0), gy + 1 + usize::from(looted));
    }
}

// ---------------------------------------------------------------------------------------
// Brine Seer, Scent of Brine
// ---------------------------------------------------------------------------------------

#[test]
fn brine_seer_can_reveal_zero_cards_and_the_controller_may_pay_zero() {
    cr!("118.5", "118.12");
    ruling!(
        "Brine Seer",
        "You can reveal zero cards, and the controller of the spell can pay {0} to stop this or not pay {0} and let it be countered."
    );
    supported("Brine Seer");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let seer = t.battlefield(P0, "Brine Seer");
        t.hand(P0, "Coral Merfolk");
        let bears = cast_new(&mut t, P1, "Grizzly Bears", &[]);
        lands_for_cost(&mut t, P0, "Brine Seer");
        t.answer_choose(P0, &[]);
        t.answer_yes(P1, pay);
        t.activate(P0, seer, 0, &[o(bears)]).expect("seer");
        t.resolve();
        assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), !pay, "pay: {pay}");
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), usize::from(pay));
    }
}

#[test]
fn scent_of_brine_reveals_during_resolution() {
    cr!("608.2c", "701.20a");
    ruling!("Scent of Brine", "You reveal cards during resolution.");
    supported("Scent of Brine");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let merfolk = t.hand(P0, "Coral Merfolk");
    let bears = cast_new(&mut t, P1, "Grizzly Bears", &[]);
    let from = t.asked().len();
    t.answer_choose(P0, &[o(merfolk)]);
    cast_new(&mut t, P0, "Scent of Brine", &[o(bears)]);
    // Nothing revealed while casting.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
    // P1 has no mana left to pay {1}.
    t.resolve();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------------
// Abjure
// ---------------------------------------------------------------------------------------

#[test]
fn abjure_sacrifices_exactly_one_blue_permanent() {
    cr!("601.2h", "118.8");
    ruling!("Abjure", "You can't sacrifice more than one permanent.");
    supported("Abjure");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let m1 = t.battlefield(P0, "Coral Merfolk");
    let m2 = t.battlefield(P0, "Coral Merfolk");
    let bears = cast_new(&mut t, P1, "Grizzly Bears", &[]);
    t.answer_choose(P0, &[o(m1), o(m2)]);
    cast_new(&mut t, P0, "Abjure", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Coral Merfolk").len(), 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------------
// Harmonized Crescendo
// ---------------------------------------------------------------------------------------

#[test]
fn harmonized_crescendo_chooses_the_type_on_resolution() {
    cr!("608.2c", "608.2f");
    ruling!(
        "Harmonized Crescendo",
        "You choose the creature type as Harmonized Crescendo resolves. Once Harmonized Crescendo starts to resolve, players can't respond to the choice or take any actions until Harmonized Crescendo finishes resolving."
    );
    supported("Harmonized Crescendo");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Coral Merfolk");
    t.battlefield(P0, "Coral Merfolk");
    t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    lands_for_cost(&mut t, P0, "Harmonized Crescendo");
    let hc = t.hand(P0, "Harmonized Crescendo");
    crate::r_s24_common::choose_creature_type(&mut t, P0, "Merfolk");
    t.cast(P0, hc).go();
    // No type chosen while casting.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. } | Decision::ChooseOption { .. })));
    let hand = t.hand_size(P0);
    let mid = t.asked().len();
    t.resolve();
    let during: Vec<_> = asked_since(&t, mid);
    // The type was chosen during resolution, with no priority in between.
    assert!(during
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. } | Decision::ChooseOption { .. })));
    assert!(!during
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    // Two Merfolk: two cards.
    assert_eq!(t.hand_size(P0), hand + 2);
}
