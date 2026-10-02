//! Rulings batch P224 — cards newly compiled for this batch: Gary Clone ("each creature
//! you control named ~"), Aeve, Progenitor Ooze ("~ isn't legendary if it's a token"),
//! Mordor on the March ("Exile a creature card from your graveyard. Create a token that's a
//! copy of it."), Charnel Serenade and Deeproot Wayfinder ("return a [kind] card from your
//! graveyard to the battlefield tapped / with a finality counter on it").

use crate::r_p224_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::next_upkeep;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn copies_of(t: &TestGame, name: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| t.g.obj(**id).kind == ObjKind::SpellCopy && t.g.obj(**id).chars.name == name)
        .count()
}

#[test]
fn gary_clone_squad_tokens_are_created_even_if_it_left_the_battlefield() {
    cr!("702.157a", "603.10");
    ruling!(
        "Gary Clone",
        "If the spell resolves but the creature with squad leaves the battlefield before its squad ability resolves, you'll still create the token copies."
    );
    supported("Gary Clone");
    // Gary Clone ({1}{W}, 1/3, squad {2}): "Whenever this creature attacks, each creature
    // you control named Gary Clone gets +1/+0 until end of turn." Squad paid twice.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let gary = t.hand(P0, "Gary Clone");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    let spell = t.cast(P0, gary).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    let gary = t.g.current(spell);
    destroy(&mut t, gary);
    assert!(t.in_graveyard(P0, "Gary Clone"));
    t.resolve_all();
    let clones = t.named_on_battlefield("Gary Clone");
    assert_eq!(clones.len(), 2);
    assert!(clones.iter().all(|c| t.obj(*c).is_token()));
}

#[test]
fn gary_clone_pumps_each_creature_named_gary_clone() {
    cr!("201.2", "508.1m");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Gary Clone");
    let b = t.battlefield(P0, "Gary Clone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(a), (2, 3));
    assert_eq!(t.pt(b), (2, 3));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn aeve_storm_copies_enter_one_by_one_counting_the_oozes_already_there() {
    cr!("702.40a", "707.10f", "614.1c");
    ruling!(
        "Aeve, Progenitor Ooze",
        "The storm copies enter the battlefield one by one, followed by the original spell. Each of them enters the battlefield with a number of +1/+1 counters equal to the number of Oozes you control as it enters the battlefield. If you don't control any Oozes at the start, the first one enters with no counters, the second one enters with one counter, and so on."
    );
    supported("Aeve, Progenitor Ooze");
    // Aeve ({2}{G}{G}{G}, 2/2): "Storm. Aeve isn't legendary if it's a token. Aeve enters
    // with a +1/+1 counter on it for each other Ooze you control."
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    t.lands(P0, "Forest", 5);
    let aeve = t.hand(P0, "Aeve, Progenitor Ooze");
    t.cast(P0, aeve).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Aeve, Progenitor Ooze"), 2);
    t.resolve(); // the first copy: no other Ooze
    t.resolve(); // the second copy: one other Ooze
    t.resolve_all(); // the original: two other Oozes
    let tokens: Vec<ObjectId> = tokens(&t, P0);
    assert_eq!(tokens.len(), 2);
    let mut counts: Vec<u32> = tokens
        .iter()
        .map(|id| t.counters(*id, counters::PLUS1))
        .collect();
    counts.sort();
    assert_eq!(counts, vec![0, 1]);
    // The tokens aren't legendary: all three stay.
    for id in &tokens {
        assert!(!t.obj(*id).chars.is_legendary());
    }
    let original = t.g.current(aeve);
    assert!(t.on_battlefield(original));
    assert!(t.obj(original).chars.is_legendary());
    assert_eq!(t.counters(original, counters::PLUS1), 2);
    assert_eq!(t.named_on_battlefield("Aeve, Progenitor Ooze").len(), 3);
}

#[test]
fn mordor_on_the_march_storm_counts_spells_from_other_zones_and_countered_ones() {
    cr!("702.40a");
    ruling!(
        "Mordor on the March",
        "Spells cast from zones other than a player's hand and spells that were countered or otherwise failed to resolve are still counted by the storm ability."
    );
    supported("Mordor on the March");
    // Mordor on the March: "Exile a creature card from your graveyard. Create a token
    // that's a copy of it. It gains haste until end of turn. Exile it at the beginning of
    // the next end step." Storm.
    let mut t = TestGame::new(2);
    spells_that_didnt_resolve_normally(&mut t);
    for _ in 0..5 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 4);
    t.lands(P0, "Mountain", 1);
    let m = t.hand(P0, "Mordor on the March");
    t.cast(P0, m).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Mordor on the March"), 4);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 5);
    assert!(bears.iter().all(|b| t.obj(*b).is_token()));
    assert!(bears.iter().all(|b| t
        .obj(*b)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste)));
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Grizzly Bears").len(), 5);
}

#[test]
fn mordor_on_the_march_copies_arent_counted_by_later_storm_spells() {
    cr!("702.40a", "707.10");
    ruling!(
        "Mordor on the March",
        "The copies of Mordor on the March created by its storm ability are put directly onto the stack. They aren't cast and won't be counted by other spells with storm cast later in the turn."
    );
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        t.graveyard(P0, "Grizzly Bears");
    }
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    t.lands(P0, "Swamp", 4);
    t.lands(P0, "Mountain", 1);
    let m = t.hand(P0, "Mordor on the March");
    t.cast(P0, m).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Mordor on the March"), 1);
    t.resolve_all();
    assert_eq!(t.g.history.spells_cast.len(), 2);
    t.lands(P0, "Forest", 2);
    let wts = t.hand(P0, "Weather the Storm");
    t.cast(P0, wts).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Weather the Storm"), 2);
}

#[test]
fn charnel_serenade_may_return_a_creature_card_that_was_already_in_the_graveyard() {
    cr!("608.2c", "701.25a", "122.1h");
    ruling!(
        "Charnel Serenade",
        "The creature card you choose doesn’t need to be a card you put in your graveyard with surveil. You may choose a creature card that was already in your graveyard before you surveilled."
    );
    supported("Charnel Serenade");
    // Charnel Serenade ({4}{B}{B}): "Surveil 3, then return a creature card from your
    // graveyard to the battlefield with a finality counter on it. Exile Charnel Serenade
    // with three time counters on it."
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Swamp", "Island"]);
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![cards[1], cards[2]], vec![cards[0]]),
    );
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.lands(P0, "Swamp", 6);
    let c = t.hand(P0, "Charnel Serenade");
    t.cast(P0, c).go();
    t.resolve_all();
    let giant = t.g.current(giant);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.counters(giant, counters::FINALITY), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // It's exiled with three time counters (suspended: it has suspend).
    let ex = t.g.find_in_zone(Zone::Exile, "Charnel Serenade")[0];
    assert_eq!(t.counters(ex, counters::TIME), 3);
    // The finality counter: the Giant is exiled instead of dying.
    destroy(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn charnel_serenade_neednt_be_cast_when_the_last_time_counter_is_removed() {
    cr!("702.62a", "702.62b");
    ruling!(
        "Charnel Serenade",
        "Due to a recent rules change to suspend, you are no longer required to cast the suspended card as the second triggered ability of suspend resolves. Instead, as the second triggered ability resolves, you may cast the card. Timing permissions based on the card’s type are ignored. If you don’t cast the card, it remains exiled with no time counters on it, and it’s no longer suspended."
    );
    // Declined: it stays exiled with no time counters, and no longer suspended.
    let mut t = TestGame::new(2);
    let card = t.exile(P0, "Charnel Serenade");
    t.g.add_counters(Entity::Object(card), counters::TIME, 1, None);
    t.g.recompute();
    t.answer_yes(P0, false);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.counters(card, counters::TIME), 0);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // Accepted: the sorcery is cast during the upkeep.
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let card = t.exile(P0, "Charnel Serenade");
    t.g.add_counters(Entity::Object(card), counters::TIME, 1, None);
    t.g.recompute();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert!(t.on_battlefield(t.g.current(giant)));
}

#[test]
fn deeproot_wayfinders_land_isnt_targeted_and_is_chosen_after_surveilling() {
    cr!("608.2c", "115.10", "701.25a");
    ruling!(
        "Deeproot Wayfinder",
        "The land card you return from your graveyard, if any, isn’t a target of the ability. You choose that card as the ability is resolving, after you surveil 1. It can be a land card you just put into your graveyard or one that was already there."
    );
    supported("Deeproot Wayfinder");
    // Deeproot Wayfinder: "Whenever this creature deals combat damage to a player or
    // battle, surveil 1, then you may return a land card from your graveyard to the
    // battlefield tapped."
    for just_surveilled in [true, false] {
        let mut t = TestGame::new(2);
        let wayfinder = t.battlefield(P0, "Deeproot Wayfinder");
        // The card already in the graveyard is a land only in the second case; in the
        // first, the Forest just surveilled is the only land card to choose.
        let old = t.graveyard(
            P0,
            if just_surveilled {
                "Grizzly Bears"
            } else {
                "Swamp"
            },
        );
        let top = t.library_top(P0, "Forest");
        t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![top]));
        t.answer_yes(P0, true);
        let pick = if just_surveilled { top } else { old };
        if !just_surveilled {
            t.answer_choose(P0, &[Entity::Object(old)]);
        }
        let from = t.asked().len();
        t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
        t.attack(&[(wayfinder, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert_eq!(t.life(P1), 18);
        let asked = &t.asked()[from..];
        assert!(!asked
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
        let land = t.g.current(pick);
        assert!(t.on_battlefield(land));
        assert!(t.obj(land).tapped);
        let other = if just_surveilled { old } else { top };
        assert_eq!(t.zone(t.g.current(other)), Zone::Graveyard(P0));
    }
}
