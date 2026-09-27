//! CR 702.185 Warp (`src/kw/warp.rs`; the void condition in
//! `oracle/patterns/k702_179_195.rs`).

use crate::common_k702_178_195::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const WARP: CastMethod = CastMethod::Keyword(KeywordKind::Warp);
/// Bygone Colossus ({9} 9/9 Artifact Creature): "Warp {3}".
const COLOSSUS: &str = "Bygone Colossus";

fn untapped_lands(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count()
}

fn exiled(t: &TestGame, name: &str) -> ObjectId {
    t.g.find_in_zone(Zone::Exile, name)[0]
}

#[test]
fn warp_cards_compile() {
    assert_supported(&[
        COLOSSUS,
        "Germinating Wurm",
        "Red Tiger Mechan",
        "Timeline Culler",
        "Insatiable Skittermaw",
        "Elegy Acolyte",
    ]);
}

#[test]
fn cast_from_hand_for_the_warp_cost_then_exiled_at_the_next_end_step() {
    cr!("702.185a");
    ruling!(
        "All-Fates Stalker",
        "If you choose to pay a spell’s warp cost rather than its mana cost, you’re still casting the spell"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let c = t.hand(P0, COLOSSUS);
    let spell = t.cast(P0, c).method(WARP).go();
    // {3} rather than {9}; its mana value is still 9.
    assert_eq!(untapped_lands(&t), 0);
    assert_eq!(t.g.mana_value_of(spell), 9);
    t.resolve_all();
    let colossus = named(&t, COLOSSUS)[0];
    // It stays until the beginning of the next end step.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.on_battlefield(colossus));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(colossus));
    assert_eq!(exiled_count(&t, COLOSSUS), 1);
}

#[test]
fn a_permanent_cast_normally_isnt_exiled() {
    cr!("702.185a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 9);
    let c = t.hand(P0, COLOSSUS);
    t.cast(P0, c).go();
    assert_eq!(untapped_lands(&t), 0);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(named(&t, COLOSSUS).len(), 1);
}

#[test]
fn warp_only_from_hand_and_only_when_it_could_be_cast() {
    cr!("702.185a");
    ruling!(
        "All-Fates Stalker",
        "You can cast a spell for its warp cost only if you could cast that spell"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let gy = t.graveyard(P0, COLOSSUS);
    assert!(t.cast(P0, gy).method(WARP).try_go().is_err());
    let ex = t.exile(P0, COLOSSUS);
    assert!(t.cast(P0, ex).method(WARP).try_go().is_err());
    // A creature spell: not during an opponent's turn, nor during combat.
    let c = t.hand(P0, COLOSSUS);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P0, c).method(WARP).try_go().is_err());
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.cast(P0, c).method(WARP).try_go().is_err());
    t.set_step(P0, Step::PostcombatMain);
    t.cast(P0, c).method(WARP).go();
}

#[test]
fn the_owner_may_cast_it_from_exile_after_the_turn_has_ended() {
    cr!("702.185a", "702.185b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 9);
    let c = t.hand(P0, "Germinating Wurm");
    t.cast(P0, c).method(WARP).go();
    t.resolve_all();
    // "When this creature enters, you gain 2 life."
    assert_eq!(t.life(P0), 22);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let wurm = exiled(&t, "Germinating Wurm");
    // Not during the same turn.
    assert!(t.cast(P0, wurm).try_go().is_err());
    // Its controller's opponent can't cast it.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(t.cast(P1, wurm).try_go().is_err());
    // On a later turn, its owner casts it from exile for its mana cost ({4}{G}).
    t.advance_to(P0, Step::PrecombatMain);
    let before = untapped_lands(&t);
    t.cast(P0, wurm).go();
    assert_eq!(before - untapped_lands(&t), 5);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // Cast normally, it's not exiled again.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(named(&t, "Germinating Wurm").len(), 1);
}

#[test]
fn a_permanent_that_left_before_the_end_step_isnt_exiled() {
    cr!("702.185a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let c = t.hand(P0, COLOSSUS);
    t.cast(P0, c).method(WARP).go();
    t.resolve_all();
    let colossus = named(&t, COLOSSUS)[0];
    // Returned to hand before the end step, it isn't exiled.
    run(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(colossus)],
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, COLOSSUS));
    assert_eq!(exiled_count(&t, COLOSSUS), 0);
}

#[test]
fn a_warp_card_exiled_another_way_isnt_warped() {
    cr!("702.185a", "702.185b");
    ruling!(
        "All-Fates Stalker",
        "that permanent will be exiled only if it’s still on the battlefield when that triggered ability resolves"
    );
    ruling!(
        "All-Fates Stalker",
        "If it goes to exile some other way, its owner won’t be able to cast it on a future turn."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 12);
    let c = t.hand(P0, COLOSSUS);
    t.cast(P0, c).method(WARP).go();
    t.resolve_all();
    let colossus = named(&t, COLOSSUS)[0];
    run(
        &mut t,
        P1,
        None,
        Effect::Exile {
            what: Sel::Target(0),
            face_down: false,
            link: false,
        },
        &[Entity::Object(colossus)],
    );
    assert_eq!(exiled_count(&t, COLOSSUS), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    t.advance_to(P0, Step::PrecombatMain);
    let card = exiled(&t, COLOSSUS);
    assert!(t.cast(P0, card).try_go().is_err());
    // A card exiled by the warp ability is a warped card; one exiled otherwise isn't.
    let f = Filter::Custom(mtg_engine::kw::warp::WARPED_CARD.into());
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(!t.g.matches(card, &f, &ctx));
    let c2 = t.hand(P0, COLOSSUS);
    t.cast(P0, c2).method(WARP).go();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let warped: Vec<ObjectId> = t
        .g
        .find_in_zone(Zone::Exile, COLOSSUS)
        .into_iter()
        .filter(|c| t.g.matches(*c, &f, &ctx))
        .collect();
    assert_eq!(warped.len(), 1);
}

#[test]
fn a_spell_was_warped_this_turn_even_if_it_was_countered() {
    cr!("702.185c");
    ruling!(
        "Alpharael, Stonechosen",
        "that spell was still warped, so the condition required by void abilities will be fulfilled for that turn"
    );
    // Insatiable Skittermaw: "Void — At the beginning of your end step, if a nonland
    // permanent left the battlefield this turn or a spell was warped this turn, put a
    // +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Insatiable Skittermaw");
    t.lands(P0, "Forest", 3);
    let c = t.hand(P0, COLOSSUS);
    let spell = t.cast(P0, c).method(WARP).go();
    run(
        &mut t,
        P1,
        None,
        Effect::CounterSpell {
            what: Sel::Target(0),
        },
        &[Entity::Object(spell)],
    );
    assert!(t.in_graveyard(P0, COLOSSUS));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(maw, "+1/+1"), 1);
    // Next turn, nothing happened: no counter.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(maw, "+1/+1"), 1);
}

#[test]
fn void_counts_nonland_permanents_leaving_but_not_lands() {
    cr!("702.185c");
    ruling!(
        "Alpharael, Stonechosen",
        "If a nonland permanent hasn’t left the battlefield during that turn or a spell wasn’t warped that turn by the time the ability would trigger, it won’t trigger at all."
    );
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Insatiable Skittermaw");
    let land = t.battlefield(P1, "Forest");
    let destroy = |t: &mut TestGame, id: ObjectId| {
        run(
            t,
            P1,
            None,
            Effect::Destroy {
                what: Sel::Target(0),
                no_regen: false,
            },
            &[Entity::Object(id)],
        );
    };
    destroy(&mut t, land);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(maw, "+1/+1"), 0);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(maw, "+1/+1"), 1);
}

#[test]
fn a_card_may_be_cast_from_the_graveyard_using_its_warp_ability() {
    cr!("702.185a");
    // Timeline Culler ({B}{B} 2/2 haste): "You may cast this card from your graveyard using
    // its warp ability. Warp—{B}, Pay 2 life."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    let c = t.graveyard(P0, "Timeline Culler");
    t.cast(P0, c).method(WARP).go();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(named(&t, "Timeline Culler").len(), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(exiled_count(&t, "Timeline Culler"), 1);
    // A card without that ability can't be warped from the graveyard.
    t.advance_to(P0, Step::PrecombatMain);
    let g = t.graveyard(P0, "Red Tiger Mechan");
    assert!(t.cast(P0, g).method(WARP).try_go().is_err());
}
