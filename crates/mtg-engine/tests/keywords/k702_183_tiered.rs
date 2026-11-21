//! CR 702.183 Tiered (`oracle/patterns/k702_179_195.rs`; the modes' additional costs are
//! paid as for spree, CR 601.2b, 601.2f).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn tiered_cards_compile() {
    assert_supported(&[
        "Thunder Magic",
        "Ice Magic",
        "Fire Magic",
        "Tifa's Limit Break",
        "Restoration Magic",
    ]);
    let c = mtg_engine::card::card("Thunder Magic");
    assert!(c.front().chars.has_keyword(KeywordKind::Tiered));
}

fn untapped_lands(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count()
}

#[test]
fn choose_one_mode_and_pay_its_additional_cost() {
    cr!("702.183a");
    ruling!(
        "Thunder Magic",
        "You must choose exactly one of the listed modes and pay its associated additional cost"
    );
    // "Thunder — {0} — 2 damage; Thundara — {3} — 4 damage; Thundaga — {5}{R} — 8 damage."
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    t.cast(P0, c).modes(&[0]).target(wurm).go();
    // {R} plus {0}.
    assert_eq!(untapped_lands(&t), 7);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);
    let c = t.hand(P0, "Thunder Magic");
    t.cast(P0, c).modes(&[2]).target(wurm).go();
    // {R} plus {5}{R}.
    assert_eq!(untapped_lands(&t), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn only_one_mode_and_only_if_its_cost_can_be_paid() {
    cr!("702.183a");
    let mut t = TestGame::new(2);
    // Enough mana for both Thunder and Thundara ({R} + {0} + {3}).
    t.lands(P0, "Mountain", 8);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    // Two modes can't be chosen.
    assert!(t
        .cast(P0, c)
        .modes(&[0, 1])
        .targets(&[Entity::Object(wurm), Entity::Object(wurm)])
        .try_go()
        .map_or(true, |s| t.g.obj(s).stack.as_ref().unwrap().chosen.len() == 1));
    t.clear_answers();
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    // Thundara costs {R} + {3}: four mana, with three lands it can't be cast.
    assert!(t.cast(P0, c).modes(&[1]).target(wurm).try_go().is_err());
    assert!(t.in_hand(P0, "Thunder Magic"));
    assert_eq!(untapped_lands(&t), 3);
}

#[test]
fn the_mana_value_is_that_of_the_mana_cost() {
    cr!("702.183a");
    ruling!(
        "Thunder Magic",
        "The mana value of a spell with tiered is determined only by its mana cost"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    let spell = t.cast(P0, c).modes(&[2]).target(wurm).go();
    assert_eq!(t.g.mana_value_of(spell), 1);
}

#[test]
fn cast_without_paying_its_mana_cost_still_pays_the_mode_cost() {
    cr!("702.183a");
    ruling!(
        "Thunder Magic",
        "you must still choose exactly one mode and pay the associated additional cost"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Thunder Magic");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![1]),
    );
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::Target(0),
            free: true,
            optional: false,
        },
        &[Entity::Object(c)],
    );
    // Only Thundara's {3} was paid.
    assert_eq!(untapped_lands(&t), 0);
    t.resolve_all();
    // 4 damage: the 6/4 Wurm dies (Thunder's 2 wouldn't have killed it).
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P0, "Thunder Magic"));
}

#[test]
fn a_mode_can_be_chosen_only_if_its_targets_are_available() {
    cr!("702.183a");
    ruling!(
        "Ice Magic",
        "If a mode requires a target, you can select that mode only if there's a legal target available."
    );
    // Every mode of Ice Magic targets a creature: with no creatures, no mode can be
    // chosen and it can't be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 8);
    let c = t.hand(P0, "Ice Magic");
    assert!(t.cast(P0, c).modes(&[0]).try_go().is_err());
    t.clear_answers();
    assert!(t.in_hand(P0, "Ice Magic"));
    // Fire Magic's modes don't target: it can be cast with no creatures around, and Fira
    // deals 2 damage to each creature.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Fire Magic");
    t.cast(P0, c).modes(&[1]).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fire Magic"));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let c = t.hand(P0, "Fire Magic");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, c).modes(&[1]).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(elves));
}

#[test]
fn modes_can_choose_values_for_the_spells_text() {
    cr!("702.183a");
    ruling!(
        "Vincent's Limit Break",
        "Vincent's Limit Break will overwrite any previous effects that set the creature's power and toughness to specific numbers."
    );
    ruling!(
        "Vincent's Limit Break",
        "Effects that otherwise modify its power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    // Vincent's Limit Break ({1}{B} instant): "Tiered. Until end of turn, target creature
    // you control gains "When this creature dies, return it to the battlefield tapped
    // under its owner's control" and has the chosen base power and toughness.
    // • Galian Beast — {0} — 3/2. • Death Gigas — {1} — 5/2. • Hellmasker — {3} — 7/2."
    assert_supported(&["Vincent's Limit Break"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A +1/+1 counter, an earlier +1/+0 and an earlier effect setting it to 0/1.
    t.g.objects[bears.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::PLUS1.into(), 1);
    for m in [
        Modification::SetPT(Some(Value::c(0)), Some(Value::c(1))),
        Modification::ModifyPT(Value::c(1), Value::c(0)),
    ] {
        crate::common_k702_052_066::run_effect(
            &mut t,
            None,
            P0,
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![m],
                duration: Duration::EndOfTurn,
            },
            &[Entity::Object(bears)],
        );
    }
    assert_eq!(t.pt(bears), (2, 2));
    let c = t.hand(P0, "Vincent's Limit Break");
    t.cast(P0, c).modes(&[1]).target(bears).go();
    // {1}{B} plus {1}.
    assert_eq!(untapped_lands(&t), 0);
    t.resolve_all();
    // Base 5/2 replaces the earlier 0/1; the +1/+0 and the counter still apply.
    assert_eq!(t.pt(bears), (7, 3));
    // It dies and returns tapped.
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P1,
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
        &[Entity::Object(bears)],
    );
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert!(t.obj(back[0]).tapped);
    assert_eq!(t.pt(back[0]), (2, 2));
}

#[test]
fn a_copy_keeps_the_chosen_mode() {
    cr!("702.183a");
    ruling!(
        "Thunder Magic",
        "If a spell with tiered is copied, the effect that creates the copy may allow you to choose new targets. You cannot choose a new mode."
    );
    ruling!(
        "Thunder Magic",
        "You choose the mode as you cast the spell with tiered. Once the mode is chosen, it can't be changed."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Thunder Magic");
    // Thundara: 4 damage.
    let spell = t.cast(P0, c).modes(&[1]).target(wurm).go();
    // A copy with a new target (the Hill Giant); the mode stays Thundara.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    crate::common_k702_052_066::run_effect(
        &mut t,
        None,
        P0,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: true,
        },
        &[Entity::Object(spell)],
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert!(!t.on_battlefield(giant));
}
