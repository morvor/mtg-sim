//! Rulings batch P155 — spells cast or cards played as a spell or ability resolves
//! (CR 608.2g): then or never, ignoring type-based timing permissions, and resolving
//! before the spell that triggered the ability; a copy of a spell isn't cast (CR 707.10,
//! 707.12); cards put onto the battlefield aren't cast; permissions to play a card
//! later keep its own timing and costs (CR 117.1a, 305.1, 601.2b); copies of a spell
//! use what was paid for the original (CR 707.10).

use crate::r_p155_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn sunbirds_invocation_resolves_before_the_spell_even_if_its_countered() {
    cr!("608.2g", "603.3", "405.5");
    ruling!(
        "Sunbird's Invocation",
        "The ability of Sunbird's Invocation resolves before the spell that caused it to trigger. It will resolve even if that spell is countered. If you cast a spell as part of the resolution of the ability, that spell resolves before the spell that caused the ability to trigger."
    );
    supported("Sunbird's Invocation");
    // Hill Giant (4) reveals four cards; Grizzly Bears among them is cast for free and
    // resolves before Hill Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunbird's Invocation");
    let bears = t.library_top(P0, "Grizzly Bears");
    for n in ["Forest", "Island", "Swamp"] {
        t.library_top(P0, n);
    }
    let giant = cast(&mut t, P0, "Hill Giant", "3R", &[]);
    assert_eq!(t.stack_len(), 2);
    assert!(top_is_trigger_of(
        &t,
        t.named_on_battlefield("Sunbird's Invocation")[0]
    ));
    t.answer_choose_moved(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(top_name(&t), "Grizzly Bears", "{}", t.dump_log());
    assert!(on_stack(&t, giant));
    t.resolve();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
    assert_eq!(count_named(&t, "Hill Giant"), 0);
    t.resolve_all();
    assert_eq!(count_named(&t, "Hill Giant"), 1);
    // The Giant countered in response to the trigger: the trigger still resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sunbird's Invocation");
    let bears = t.library_top(P0, "Grizzly Bears");
    for n in ["Forest", "Island", "Swamp"] {
        t.library_top(P0, n);
    }
    let giant = cast(&mut t, P0, "Hill Giant", "3R", &[]);
    cast(&mut t, P1, "Cancel", "1UU", &[obj(giant)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.stack_len(), 1, "the trigger: {}", t.dump_log());
    t.answer_choose_moved(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1, "{}", t.dump_log());
}

#[test]
fn baral_and_kari_zevs_free_spell_is_cast_during_resolution_ignoring_type_timing() {
    cr!("608.2g", "117.1a", "603.3");
    ruling!(
        "Baral and Kari Zev",
        "The spell you cast from your hand without paying its mana cost is cast during the resolution of the triggered ability. Timing restrictions of that spell based on card type are ignored. It will resolve before the spell that caused the ability to trigger."
    );
    supported("Baral and Kari Zev");
    supported("Lava Spike");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Baral and Kari Zev");
    let spike = t.hand(P0, "Lava Spike");
    let div = cast(&mut t, P0, "Divination", "2U", &[]);
    assert_eq!(t.stack_len(), 2);
    // Lava Spike, a sorcery, is cast while Divination is on the stack.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(spike)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert_eq!(top_name(&t), "Lava Spike", "{}", t.dump_log());
    assert!(on_stack(&t, div));
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), 0, "Divination hasn't resolved");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn counterpoint_casts_from_the_graveyard_as_it_resolves_ignoring_type_timing() {
    cr!("608.2g", "117.1a");
    ruling!(
        "Counterpoint",
        "You cast the card from your graveyard as Counterpoint resolves. You can’t wait to cast it later in the turn. Timing restrictions based on the card’s types are ignored."
    );
    supported("Counterpoint");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let courser = t.graveyard(P0, "Centaur Courser");
    t.set_step(P1, Step::PrecombatMain);
    let giant = cast(&mut t, P1, "Hill Giant", "3R", &[]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    cast(&mut t, P0, "Counterpoint", "3UB", &[obj(giant)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Grizzly Bears, a creature, was cast on P1's turn, during the resolution.
    assert_eq!(top_name(&t), "Grizzly Bears", "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
    // No permission lingers for the other creature card.
    lots(&mut t, P0);
    assert!(!can_cast(&mut t, P0, courser, CastMethod::Normal));
    assert!(!can_cast(&mut t, P0, courser, CastMethod::Free));
}

#[test]
fn renegade_bulls_copy_is_cast_while_its_ability_resolves_or_never() {
    cr!("608.2g", "707.12");
    ruling!(
        "Renegade Bull",
        "You cast the copy while Renegade Bull's last ability is resolving and still on the stack. You can't wait to cast it later in the turn."
    );
    supported("Renegade Bull");
    // Cast: Shock's copy deals 2 to P1 as the ability resolves.
    let mut t = TestGame::new(2);
    let bull = t.battlefield(P0, "Renegade Bull");
    let shock = t.graveyard(P0, "Shock");
    t.answer_targets(P0, &[obj(shock)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    attack_with(&mut t, &[(bull, pl(P1))]);
    t.resolve();
    assert!(t.in_exile("Shock"));
    // The copy is on the stack (with the Bull's trigger on casting it).
    assert!(
        t.g.stack
            .iter()
            .any(|id| t.g.obj(*id).kind == mtg_engine::object::ObjKind::CardCopy),
        "the copy, cast: {}",
        t.dump_log()
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.pt(bull), (1, 5), "the copy was cast");
    // Declined: the copy ceases to exist; nothing to cast later.
    let mut t = TestGame::new(2);
    let bull = t.battlefield(P0, "Renegade Bull");
    let shock = t.graveyard(P0, "Shock");
    t.answer_targets(P0, &[obj(shock)]);
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(bull, pl(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Shock"));
    assert_eq!(t.stack_len(), 0);
    lots(&mut t, P0);
    assert!(t
        .g
        .legal_actions(P0)
        .iter()
        .all(|a| !matches!(a, Action::Cast { .. })));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn breeches_copy_isnt_cast() {
    cr!("707.10", "603.2");
    ruling!(
        "Breeches, the Blastmaker",
        "The copy made by Breeches's delayed triggered ability is created on the stack, so it's not \"cast.\""
    );
    supported("Breeches, the Blastmaker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Breeches, the Blastmaker");
    t.battlefield(P0, "Memnite");
    let plasm = t.battlefield(P0, "Manaplasm");
    cast_resolve(&mut t, P0, "Opt", "U", &[]);
    assert_eq!(t.pt(plasm), (2, 2));
    t.g.dice.loaded_coins.push_back(true);
    t.answer_yes(P0, true);
    keep_copy_targets(&mut t, P0);
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert_eq!(t.life(P1), 16, "Shock and its copy: {}", t.dump_log());
    // Manaplasm saw two spells cast (Opt and Shock), not the copy.
    assert_eq!(t.pt(plasm), (3, 3));
}

#[test]
fn tend_the_pests_copies_use_the_power_of_the_creature_sacrificed_for_the_original() {
    cr!("707.10", "601.2b");
    ruling!(
        "Tend the Pests",
        "Copies of Tend the Pests will use the power of the creature sacrificed for the original spell to determine the number of Pests each copy creates. Because the copies aren't cast, you won't sacrifice additional creatures for them."
    );
    supported("Tend the Pests");
    supported("Twincast");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    let pests = cast(&mut t, P0, "Tend the Pests", "BG", &[]);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    cast(&mut t, P0, "Twincast", "UU", &[obj(pests)]);
    t.resolve();
    assert_eq!(
        t.stack_len(),
        2,
        "the copy and the original: {}",
        t.dump_log()
    );
    t.resolve();
    assert_eq!(creature_tokens(&t, P0), 3, "the copy's three Pests");
    assert!(t.on_battlefield(bears), "nothing sacrificed for the copy");
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0), 6);
    assert!(t.on_battlefield(bears));
}

#[test]
fn transmute_artifact_sacrifices_on_resolution_one_artifact_and_doesnt_cast_the_found_card() {
    cr!("608.2c", "601.2b", "603.2");
    ruling!(
        "Transmute Artifact",
        "The sacrifice of an artifact is an effect of the spell, not a cost, which means it happens as the spell resolves. If Transmute Artifact is countered, you do not have to sacrifice an artifact."
    );
    ruling!(
        "Transmute Artifact",
        "You can't sacrifice more than one artifact to this spell."
    );
    ruling!(
        "Transmute Artifact",
        "The one from the library enters when the spell is resolved, and this does not count as the casting of an artifact."
    );
    supported("Transmute Artifact");
    // Resolving: one artifact is sacrificed (the choice offers at most one), the found
    // Mind Stone ({2}) enters without being cast (Manaplasm grew for Transmute Artifact
    // only).
    let mut t = TestGame::new(2);
    let plasm = t.battlefield(P0, "Manaplasm");
    let m1 = t.battlefield(P0, "Mind Stone");
    t.battlefield(P0, "Mind Stone");
    let found = t.library_top(P0, "Mind Stone");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(m1)]);
    t.answer_choose(P0, &[obj(found)]);
    cast(&mut t, P0, "Transmute Artifact", "UU", &[]);
    assert!(t.on_battlefield(m1), "nothing sacrificed as a cost");
    assert_eq!(
        asked_of(&t, P0, from)
            .iter()
            .filter(|d| matches!(d, Decision::ChooseEntities { .. }))
            .count(),
        0
    );
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert_eq!(count_named(&t, "Mind Stone"), 2, "{}", t.dump_log());
    let sac = asked_of(&t, P0, from)
        .into_iter()
        .find_map(|d| match d {
            Decision::ChooseEntities { max, prompt, .. } if prompt.contains("acrifice") => {
                Some(max)
            }
            _ => None,
        })
        .expect("the sacrifice choice");
    assert_eq!(sac, 1, "at most one artifact");
    assert_eq!(
        t.pt(plasm),
        (3, 3),
        "+2/+2 for Transmute Artifact only, not for the Mind Stone"
    );
    // Countered: no sacrifice.
    let mut t = TestGame::new(2);
    let m1 = t.battlefield(P0, "Memnite");
    let from = t.asked().len();
    let ta = cast(&mut t, P0, "Transmute Artifact", "UU", &[]);
    cast(&mut t, P1, "Cancel", "1UU", &[obj(ta)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Transmute Artifact"));
    assert!(t.on_battlefield(m1));
    assert!(asked_of(&t, P0, from)
        .iter()
        .all(|d| !matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn vivien_lets_you_look_at_the_top_card_any_time_without_using_the_stack() {
    cr!("401.5");
    ruling!(
        "Vivien, Monsters' Advocate",
        "Vivien lets you look at the top card of your library whenever you want (with one restriction; see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    supported("Vivien, Monsters' Advocate");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P0, top));
    t.battlefield(P0, "Vivien, Monsters' Advocate");
    mtg_engine::zones::update_revealed_tops(&mut t.g);
    assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, top));
    assert!(!mtg_engine::zones::can_see_in_library(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
    // On P1's turn too, without priority.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P1);
    assert!(mtg_engine::zones::can_see_in_library(&t.g, P0, top));
}

#[test]
fn vivien_casting_from_the_library_pays_all_costs_including_kicker_and_mutate() {
    cr!("601.2b", "702.33a", "702.140a");
    ruling!(
        "Vivien, Monsters' Advocate",
        "You'll still pay all costs for that spell, including additional costs. You may also pay alternative costs such as mutate costs."
    );
    supported("Vivien, Monsters' Advocate");
    supported("Kavu Titan");
    supported("Pouncing Shoreshark");
    // Kavu Titan ({1}{G}, kicker {2}{G}) kicked from the top of the library: {3}{G}{G}
    // paid, a 5/5 with trample.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vivien, Monsters' Advocate");
    let titan = t.library_top(P0, "Kavu Titan");
    mana(&mut t, P0, "3GG");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, titan).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.pt(titan), (5, 5), "{}", t.dump_log());
    // Without the mana, it can't be cast at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vivien, Monsters' Advocate");
    let titan = t.library_top(P0, "Kavu Titan");
    assert!(!can_cast(&mut t, P0, titan, CastMethod::Normal));
    // Pouncing Shoreshark ({4}{U}, mutate {3}{U}) mutated from the library onto a
    // Grizzly Bears for {3}{U}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vivien, Monsters' Advocate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shark = t.library_top(P0, "Pouncing Shoreshark");
    mana(&mut t, P0, "3U");
    assert!(
        !can_cast(&mut t, P0, shark, CastMethod::Normal),
        "{{4}}{{U}} can't be paid"
    );
    assert!(can_cast(
        &mut t,
        P0,
        shark,
        CastMethod::Keyword(KeywordKind::Mutate)
    ));
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, shark)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(
        count_named(&t, "Pouncing Shoreshark"),
        1,
        "{}",
        t.dump_log()
    );
    assert_eq!(count_named(&t, "Grizzly Bears"), 0, "merged");
}

/// Sets up a permission for P0 to play the exiled `land` (a Forest from the top of P0's
/// library) in P0's precombat main phase; returns the exiled card.
type Setup = fn(&mut TestGame) -> ObjectId;

fn triple_triad(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Triple Triad");
    let land = t.library_top(P0, "Forest");
    t.library_top(P1, "Island");
    crate::r_s04_common::next_upkeep(t, P0);
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    t.g.current(land)
}

fn bell_borca(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Bell Borca, Spectral Sergeant");
    let land = t.library_top(P0, "Forest");
    crate::r_s04_common::next_upkeep(t, P0);
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    t.g.current(land)
}

fn hidetsugu(t: &mut TestGame) -> ObjectId {
    let h = t.battlefield(P0, "Hidetsugu, Devouring Chaos");
    let land = t.library_top(P0, "Forest");
    activate_resolve(t, P0, h, "Exile the top card", "2R", &[]);
    t.g.current(land)
}

fn rakdos(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Rakdos, the Muscle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let land = t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    t.answer_targets(P0, &[pl(P0)]);
    sacrifice(t, P0, bears);
    t.resolve_all();
    t.g.current(land)
}

fn memorial_vault(t: &mut TestGame) -> ObjectId {
    let vault = t.battlefield(P0, "Memorial Vault");
    let memnite = t.battlefield(P0, "Memnite");
    let land = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[obj(memnite)]);
    activate_resolve(t, P0, vault, "Exile the top", "", &[]);
    t.g.current(land)
}

#[test]
fn cards_played_from_exile_follow_the_normal_land_timing_and_land_plays() {
    cr!("117.1a", "305.1", "305.2", "505.6a");
    ruling!(
        "Triple Triad",
        "You follow all timing rules for cards played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty and only if you have an available land play remaining."
    );
    ruling!(
        "Bell Borca, Spectral Sergeant",
        "You must follow the normal timing permissions and restrictions for the card exiled by Bell Borca's last ability. If it's a land, you can't play it unless you have land plays available."
    );
    ruling!(
        "Hidetsugu, Devouring Chaos",
        "You must still pay all costs and follow all normal timing rules when playing a card this way. For example, you may play a land this way only during one of your own main phases while the stack is empty, and only if you haven't previously played a land this turn."
    );
    ruling!(
        "Rakdos, the Muscle",
        "You pay all costs and follow all normal timing rules for cards played from exile with Rakdos's triggered ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    ruling!(
        "Memorial Vault",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Memorial Vault’s ability. For example, if an exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    let cases: [(&str, Setup); 5] = [
        ("Triple Triad", triple_triad),
        ("Bell Borca, Spectral Sergeant", bell_borca),
        ("Hidetsugu, Devouring Chaos", hidetsugu),
        ("Rakdos, the Muscle", rakdos),
        ("Memorial Vault", memorial_vault),
    ];
    for (name, setup) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let land = setup(&mut t);
        assert_eq!(t.zone(land), Zone::Exile, "{name}: {}", t.dump_log());
        assert!(
            can_play_land(&mut t, P0, land),
            "{name}: main phase, empty stack"
        );
        // With a spell on the stack: not playable.
        cast(&mut t, P1, "Shock", "R", &[pl(P0)]);
        assert!(!can_play_land(&mut t, P0, land), "{name}: stack not empty");
        t.resolve_all();
        // In combat: not playable; a land played from hand uses up the land play.
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!can_play_land(&mut t, P0, land), "{name}: combat");
        t.set_step(P0, Step::PostcombatMain);
        assert!(can_play_land(&mut t, P0, land), "{name}: postcombat main");
        let other = t.hand(P0, "Mountain");
        t.play_land(P0, other).unwrap();
        assert!(
            !can_play_land(&mut t, P0, land),
            "{name}: no land play left"
        );
        assert_eq!(t.zone(land), Zone::Exile);
    }
}

#[test]
fn spark_of_creativity_pays_all_costs_additional_and_alternative() {
    cr!("601.2b", "118.8", "702.33a");
    ruling!(
        "Spark of Creativity",
        "You must pay all costs to play the exiled card. You may pay alternative or additional costs. If the card has any mandatory additional costs, you must pay those."
    );
    supported("Spark of Creativity");
    // Tormenting Voice exiled: its discard must be paid, so with an empty hand it can't
    // be cast; with a card it can, and the card is discarded.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let voice = t.library_top(P0, "Tormenting Voice");
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Spark of Creativity", "R", &[obj(bears)]);
    assert!(t.in_exile("Tormenting Voice"));
    let voice = t.g.current(voice);
    mana(&mut t, P0, "1R");
    assert!(
        !can_cast(&mut t, P0, voice, CastMethod::Normal),
        "nothing to discard"
    );
    let forest = t.hand(P0, "Forest");
    assert!(can_cast(&mut t, P0, voice, CastMethod::Normal));
    t.answer_choose(P0, &[obj(forest)]);
    t.cast(P0, voice).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P0, "Tormenting Voice"));
    assert_eq!(t.hand_size(P0), 2);
    // Burst Lightning exiled: it may be kicked for 4 damage; without mana, no cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let burst = t.library_top(P0, "Burst Lightning");
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Spark of Creativity", "R", &[obj(bears)]);
    let burst = t.g.current(burst);
    assert!(!can_cast(&mut t, P0, burst, CastMethod::Normal), "no mana");
    mana(&mut t, P0, "4R");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, burst).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn edgar_casts_artifacts_from_the_graveyard_paying_costs_at_sorcery_timing() {
    cr!("117.1a", "601.2f", "505.6a");
    ruling!(
        "Edgar, Master Machinist",
        "You must still pay all costs and follow any timing restrictions and permissions for artifact spells you cast from your graveyard this way. Normally, you'll be able to cast them only during your main phase while the stack is empty."
    );
    supported("Edgar, Master Machinist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgar, Master Machinist");
    let jugg = t.graveyard(P0, "Juggernaut");
    assert!(!can_cast(&mut t, P0, jugg, CastMethod::Normal), "no mana");
    mana(&mut t, P0, "4");
    assert!(can_cast(&mut t, P0, jugg, CastMethod::Normal));
    // Not with a spell on the stack, not in combat, not on P1's turn.
    cast(&mut t, P1, "Shock", "R", &[pl(P0)]);
    assert!(!can_cast(&mut t, P0, jugg, CastMethod::Normal));
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, jugg, CastMethod::Normal));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, jugg, CastMethod::Normal));
    t.set_step(P0, Step::PostcombatMain);
    t.cast(P0, jugg).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0, "{{4}} paid");
    assert!(
        t.on_battlefield(jugg) && t.obj_now(jugg).tapped,
        "{}",
        t.dump_log()
    );
}
