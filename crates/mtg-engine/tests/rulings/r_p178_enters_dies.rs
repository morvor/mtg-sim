//! Rulings batch P178 — enters, dies, tapped and land triggers: creatures entering or
//! dying together (Elvish Vanguard, Soul's Attendant, Species Specialist, Kindred
//! Discovery, Kavu Monarch), optional triggers (Auriok Champion), counting tokens
//! (Woodland Champion), targeting a new token (Rosie Cotton), cards you own going to
//! your graveyard (Field of Souls), lands (Sarinth Greatwurm, Dirtcowl Wurm) and
//! becoming tapped (Verity Circle).

use crate::r_p178_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn enters_triggers_see_creatures_entering_at_the_same_time() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Elvish Vanguard",
        "If Elvish Vanguard enters the battlefield at the same time as another Elf, its ability will trigger."
    );
    ruling!(
        "Soul's Attendant",
        "If Soul's Attendant and another creature enter at the same time, Soul's Attendant's ability will trigger."
    );
    supported("Elvish Vanguard");
    supported("Soul's Attendant");
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let ids = enter_together(&mut t, &[(P0, "Elvish Vanguard"), (P0, "Llanowar Elves")]);
    assert_eq!(triggers_from(&t, ids[0]), 1);
    t.resolve_all();
    assert_eq!(t.counters(ids[0], "+1/+1"), 1);

    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let ids = enter_together(&mut t, &[(P0, "Soul's Attendant"), (P1, "Grizzly Bears")]);
    assert_eq!(triggers_from(&t, ids[0]), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn auriok_champion_life_gain_is_optional() {
    cr!("603.5");
    ruling!(
        "Auriok Champion",
        "Gaining life when a creature enters is optional."
    );
    supported("Auriok Champion");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Auriok Champion");
        t.answer_yes(P0, yes);
        t.enter(P1, "Grizzly Bears");
        t.resolve_all();
        assert_eq!(t.life(P0), if yes { 21 } else { 20 });
    }
}

#[test]
fn kavu_monarch_gives_itself_trample_but_no_counter() {
    cr!("603.2", "613.1f");
    ruling!(
        "Kavu Monarch",
        "Does give itself Trample, but does not give itself a +1/+1 counter."
    );
    supported("Kavu Monarch");
    let mut t = TestGame::new(2);
    let monarch = t.enter(P0, "Kavu Monarch");
    t.resolve_all();
    assert!(has_kw(
        &t,
        monarch,
        mtg_engine::keywords::KeywordKind::Trample
    ));
    assert_eq!(t.counters(monarch, "+1/+1"), 0);
    // Another Kavu does give it one.
    t.enter(P0, "Kavu Titan");
    t.resolve_all();
    assert_eq!(t.counters(monarch, "+1/+1"), 1);
}

#[test]
fn species_specialist_sees_itself_and_creatures_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Species Specialist",
        "If Species Specialist happens to have the chosen creature type, its ability triggers when it dies."
    );
    ruling!(
        "Species Specialist",
        "If a creature of the chosen type dies at the same time as Species Specialist, its ability triggers for that creature."
    );
    supported("Species Specialist");
    // Chosen type Human: the Specialist (a Human Warrior) dies alone.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Human");
    let spec = t.enter(P0, "Species Specialist");
    t.resolve_all();
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    let h = t.hand_size(P0);
    destroy(&mut t, spec);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 1, "it saw itself die");
    // Chosen type Bear: it dies at the same time as a Bear (Wrath of God).
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Species Specialist");
    t.resolve_all();
    t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let h = t.hand_size(P0);
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Species Specialist"));
    assert_eq!(t.hand_size(P0), h + 1, "it saw the Bear die");
}

#[test]
fn kindred_discovery_type_is_chosen_before_it_enters() {
    cr!("614.12", "603.6a");
    ruling!(
        "Kindred Discovery",
        "The choice of creature type is made as Kindred Discovery enters the battlefield. Players can't take any actions between the time the choice is made and the time it enters the battlefield. Notably, this means that if it is entering the battlefield at the same time as any creatures of the chosen type, its last ability will trigger."
    );
    supported("Kindred Discovery");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    let ids = enter_together(&mut t, &[(P0, "Kindred Discovery"), (P0, "Grizzly Bears")]);
    assert_eq!(triggers_from(&t, ids[0]), 1);
    let h = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h + 1);
}

#[test]
fn kindred_discovery_without_a_chosen_type_never_triggers() {
    cr!("603.2");
    ruling!(
        "Kindred Discovery",
        "If you somehow control a Kindred Discovery with no chosen creature type, its last ability can't trigger, even if a creature with no creature types enters the battlefield or attacks."
    );
    // Put onto the battlefield without entering (no choice was made).
    let mut t = TestGame::new(2);
    let kd = t.battlefield(P0, "Kindred Discovery");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_from(&t, kd), 0);
    // A creature with no creature types: an artifact entering under March of the
    // Machines.
    t.battlefield(P0, "March of the Machines");
    let ring = t.enter(P0, "Sol Ring");
    assert!(t.obj_now(ring).is(mtg_engine::types::CardType::Creature));
    t.settle();
    assert_eq!(triggers_from(&t, kd), 0);
}

#[test]
fn woodland_champion_counts_tokens_that_already_left() {
    cr!("603.2c");
    ruling!(
        "Woodland Champion",
        "The number of +1/+1 counters you put on Woodland Champion is the number of tokens that entered the battlefield under your control, even if some or all of them leave the battlefield before the triggered ability resolves."
    );
    supported("Woodland Champion");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Woodland Champion");
    cast_resolve_no_triggers(&mut t, P0, "Raise the Alarm");
    assert_eq!(triggers_from(&t, champ), 1);
    for tok in tokens(&t, P0) {
        destroy(&mut t, tok);
    }
    t.resolve_all();
    assert_eq!(t.counters(champ, "+1/+1"), 2);
}

/// Casts `name` and resolves just the spell, leaving the triggers it caused on the stack.
fn cast_resolve_no_triggers(t: &mut TestGame, p: PlayerId, name: &str) {
    let spell = cast_targeting(t, p, name, &[]);
    assert_eq!(top_of_stack(t), spell);
    t.g.resolve_top();
    t.settle();
}

#[test]
fn rosie_cotton_can_target_the_token_you_created() {
    cr!("603.3d", "111.1");
    ruling!(
        "Rosie Cotton of South Lane",
        "If you create a creature token, you can target it with Rosie Cotton of South Lane's last ability."
    );
    supported("Rosie Cotton of South Lane");
    let mut t = TestGame::new(2);
    let rosie = t.battlefield(P0, "Rosie Cotton of South Lane");
    // Raise the Alarm makes two tokens: Rosie triggers for each; both target the first.
    let tok = create_bear_token(&mut t);
    t.answer_targets(P0, &[obj(tok)]);
    t.answer_targets(P0, &[obj(tok)]);
    t.settle();
    assert_eq!(triggers_from(&t, rosie), 2);
    t.resolve_all();
    assert_eq!(t.counters(tok, "+1/+1"), 2);
}

/// P0 creates a 1/1 creature token through a spell (Raise the Alarm makes two: returns the
/// first, with the trigger it caused not yet on the stack).
fn create_bear_token(t: &mut TestGame) -> ObjectId {
    let spell = cast_targeting(t, P0, "Raise the Alarm", &[]);
    assert_eq!(top_of_stack(t), spell);
    t.g.resolve_top();
    tokens(t, P0)[0]
}

#[test]
fn field_of_souls_sees_your_card_die_under_an_opponents_control() {
    cr!("400.3", "603.2");
    ruling!(
        "Field of Souls",
        "A creature card that you own but don't control will be put into your graveyard if that creature dies, and Field of Souls's ability will trigger."
    );
    supported("Field of Souls");
    let mut t = TestGame::new(2);
    let field = t.battlefield(P0, "Field of Souls");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    destroy(&mut t, bears);
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(triggers_from(&t, field), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn sarinth_greatwurm_triggers_on_any_players_land() {
    cr!("603.2");
    ruling!(
        "Sarinth Greatwurm",
        "Sarinth Greatwurm's second ability triggers whenever a land enters the battlefield under any player's control, not just under your control."
    );
    supported("Sarinth Greatwurm");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Sarinth Greatwurm");
    t.set_step(P1, Step::PrecombatMain);
    let forest = t.hand(P1, "Forest");
    t.play_land(P1, forest).unwrap();
    t.settle();
    assert_eq!(triggers_from(&t, wurm), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "P0 gets the Powerstone");
}

#[test]
fn dirtcowl_wurm_putting_a_land_onto_the_battlefield_isnt_playing_it() {
    cr!("305.4", "305.1");
    ruling!(
        "Dirtcowl Wurm",
        "A spell or ability that tells a player to “put a land onto the battlefield” does not count as playing a land."
    );
    supported("Dirtcowl Wurm");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Dirtcowl Wurm");
    t.set_step(P1, Step::PrecombatMain);
    t.library_top(P1, "Forest");
    cast_targeting(&mut t, P1, "Rampant Growth", &[]);
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Forest").len(),
        2,
        "one put, one paid with"
    );
    assert_eq!(t.counters(wurm, "+1/+1"), 0);
    let forest = t.hand(P1, "Forest");
    t.play_land(P1, forest).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(wurm, "+1/+1"), 1, "playing one does");
}

#[test]
fn verity_circle_needs_an_actual_change_to_tapped() {
    cr!("603.2e", "701.26a");
    ruling!(
        "Verity Circle",
        "For Verity Circle's first ability to trigger, the creature has to actually change from untapped to tapped."
    );
    ruling!(
        "Verity Circle",
        "If a creature enters the battlefield tapped, Verity Circle's first ability doesn't trigger."
    );
    supported("Verity Circle");
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Verity Circle");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, circle), 1, "untapped to tapped");
    t.resolve_all();
    // Tapping it again while tapped: no trigger.
    t.lands(P0, "Island", 5);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, circle, "Tap target").unwrap();
    t.resolve();
    assert!(tapped(&t, bears));
    assert_eq!(triggers_from(&t, circle), 0, "it was already tapped");
    // A creature entering tapped.
    let id = t.custom(
        P1,
        (*mtg_engine::card::card("Hill Giant")).clone(),
        Zone::Exile,
    );
    t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P1),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P1),
            tapped: true,
            ..Default::default()
        },
        source: None,
    })
    .expect("entered");
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, circle), 0, "entered tapped");
}
