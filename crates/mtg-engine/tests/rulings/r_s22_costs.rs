//! Rulings batch S22 — the total cost of a spell (CR 601.2f): X is chosen first
//! (CR 601.2b), additional costs and cost increases are applied before reductions, and a
//! generic reduction affects only generic mana (CR 118.7a); a reduction applies to an
//! alternative cost too; once the cost is determined and paid no player can act
//! (CR 601.2); what was actually spent is what counts afterward (expend, CR 700.14;
//! "if {U}{B} was spent", CR 601.2h).

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s04_common::add_mana;
use crate::r_s07_common::cast_methods;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts the card `name` from P0's hand (paying with P0's lands) and resolves the stack.
fn cast_and_resolve(t: &mut TestGame, name: &str, targets: &[Entity]) {
    let c = t.hand(P0, name);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, c).go();
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn expend_counts_only_the_mana_actually_spent() {
    cr!("700.14", "601.2f", "601.2h");
    ruling!(
        "Teapot Slinger",
        "If the cost to cast a spell is increased, decreased, or changed because of additional or alternative costs, expend counts only the mana you actually spent."
    );
    supported("Teapot Slinger");
    supported("Goblin Electromancer");
    // "Whenever you expend 4, this creature deals 2 damage to each opponent."
    // Fiery Confluence (mana value 4) costs {1} less with Goblin Electromancer: 3 mana
    // spent, not 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teapot Slinger");
    t.battlefield(P0, "Goblin Electromancer");
    t.lands(P0, "Mountain", 4);
    let conf = t.hand(P0, "Fiery Confluence");
    t.cast(P0, conf).modes(&[1, 1, 1]).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    // One more mana spent (Shock): the fourth.
    cast_and_resolve(&mut t, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 14 - 2 - 2);
    // Kicker increases what's spent: kicked Burst Lightning ({R} + {4}) expends 4.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teapot Slinger");
    t.lands(P0, "Mountain", 5);
    let bolt = t.hand(P0, "Burst Lightning");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 4 - 2);
    // A spell cast without paying its mana cost spends nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teapot Slinger");
    add_mana(&mut t, P0, ManaType::W, 4);
    let conf = t.hand(P0, "Fiery Confluence");
    let expertise = t.hand(P0, "Rishkar's Expertise");
    let _ = expertise;
    t.g.players[P0.idx()].mana_pool.mana.clear();
    add_mana(&mut t, P0, ManaType::G, 6);
    t.answer_choose(P0, &[Entity::Object(conf)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 1, 1]));
    t.cast(P0, expertise).go();
    t.resolve_all();
    // Rishkar's Expertise spent 6 (expend 4: 2 damage); the Confluence spent nothing.
    assert_eq!(t.life(P1), 20 - 2 - 6);
}

#[test]
fn wandertale_mentor_expend_4_triggers_once_as_the_fourth_mana_is_spent() {
    cr!("700.14");
    ruling!(
        "Wandertale Mentor",
        "Abilities that trigger whenever you “expend N” only trigger when you reach that specific amount of mana spent on casting spells that turn. This can only happen once per turn. For example, if you’ve spent three mana on spells so far this turn and you control a permanent with an ability that triggers “whenever you expend 4,” that ability will trigger the next time you spend at least one mana to cast a spell this turn. It won’t trigger again if you spend another four mana to cast spells later in the turn."
    );
    supported("Wandertale Mentor");
    // "Whenever you expend 4, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let mentor = t.battlefield(P0, "Wandertale Mentor");
    t.lands(P0, "Mountain", 12);
    // Three mana.
    cast_and_resolve(&mut t, "Act on Impulse", &[]);
    assert_eq!(t.counters(mentor, "+1/+1"), 0);
    // Two more (the fourth is among them): it triggers.
    cast_and_resolve(&mut t, "Lightning Strike", &[Entity::Player(P1)]);
    assert_eq!(t.counters(mentor, "+1/+1"), 1);
    // Four more: no second trigger this turn.
    cast_and_resolve(&mut t, "Hill Giant", &[]);
    cast_and_resolve(&mut t, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.counters(mentor, "+1/+1"), 1);
}

#[test]
fn bakersbane_duo_expend_4_triggers_once_per_turn() {
    cr!("700.14");
    ruling!(
        "Bakersbane Duo",
        "Abilities that trigger whenever you \"expend N\" only trigger when you reach that specific amount of mana spent on casting spells that turn. This can only happen once per turn."
    );
    supported("Bakersbane Duo");
    // "Whenever you expend 4, this creature gets +1/+1 until end of turn."
    let mut t = TestGame::new(2);
    let duo = t.battlefield(P0, "Bakersbane Duo");
    t.lands(P0, "Mountain", 12);
    cast_and_resolve(&mut t, "Act on Impulse", &[]);
    assert_eq!(t.pt(duo), (2, 2));
    cast_and_resolve(&mut t, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.pt(duo), (3, 3));
    // Eight more mana: still only once.
    cast_and_resolve(&mut t, "Hill Giant", &[]);
    cast_and_resolve(&mut t, "Hill Giant", &[]);
    assert_eq!(t.pt(duo), (3, 3));
}

#[test]
fn deadly_alliance_no_player_can_change_the_party_while_it_is_being_cast() {
    cr!("601.2", "601.2f", "700.8", "117.1b");
    ruling!(
        "Deadly Alliance",
        "If a spell has a cost reduction based on the number of creatures in your party, no player may attempt to change that number after you begin to cast the spell but before you pay the cost."
    );
    supported("Deadly Alliance");
    // "This spell costs {1} less to cast for each creature in your party." A Cleric
    // (Soul Warden) and a Rogue: {2}{B}. P1 would Shock the Cleric at their first chance:
    // that's only once Deadly Alliance has been cast and paid for.
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Silhana Ledgewalker");
    let target = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: shock,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P1, &[Entity::Object(warden)]);
    let alliance = t.hand(P0, "Deadly Alliance");
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: alliance,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[Entity::Object(target)]);
    let from = t.asked().len();
    let ok = t
        .g
        .run_until(1000, |g| g.stack.iter().any(|s| g.obj(*s).chars.name == "Shock"));
    assert!(ok);
    // Three Swamps paid; Deadly Alliance was already on the stack when P1 first got
    // priority.
    assert_eq!(tapped_lands(&t, P0), 3);
    let names: Vec<String> = t
        .g
        .stack
        .iter()
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect();
    assert_eq!(names, vec!["Deadly Alliance", "Shock"]);
    let p1_asked = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. }))
        .count();
    assert_eq!(p1_asked, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Soul Warden"));
}

#[test]
fn the_magic_mirror_opponents_cant_change_its_cost_reduction_while_it_is_cast() {
    cr!("601.2", "601.2f", "117.1b");
    ruling!(
        "The Magic Mirror",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to change by how much a relic's cost is reduced."
    );
    supported("The Magic Mirror");
    // "This spell costs {1} less to cast for each instant and sorcery card in your
    // graveyard." Five of them: {1}{U}{U}{U}. P1's Tormod's Crypt ("{T}, Sacrifice this
    // artifact: Exile target player's graveyard.") can be activated only after it's paid.
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.graveyard(P0, "Shock");
    }
    t.lands(P0, "Island", 9);
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    let uid = t
        .obj(crypt)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .unwrap()
        .uid;
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Activate {
            source: crypt,
            ability: uid,
        }),
    );
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let mirror = t.hand(P0, "The Magic Mirror");
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: mirror,
            method: CastMethod::Normal,
        }),
    );
    let ok = t.g.run_until(1000, |g| g.stack.len() == 2);
    assert!(ok);
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(
        t.g.obj(t.g.stack[0]).chars.name,
        "The Magic Mirror",
        "the Mirror was cast first"
    );
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.named_on_battlefield("The Magic Mirror").len(), 1);
}

#[test]
fn ruby_medallion_increases_apply_before_reductions() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Ruby Medallion",
        "If there are additional costs to cast a spell, or if the cost to cast a spell is increased by an effect (such as the one created by Thalia, Guardian of Thraben's ability), apply those increases before applying cost reductions."
    );
    supported("Ruby Medallion");
    supported("Thalia, Guardian of Thraben");
    // "Red spells you cast cost {1} less to cast." Kicked Burst Lightning: {R} + {4},
    // then {1} less: four mana (reducing first, {R} couldn't be reduced: five).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Medallion");
    t.lands(P0, "Mountain", 6);
    let bolt = t.hand(P0, "Burst Lightning");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // With Thalia ("Noncreature spells cost {1} more to cast."): Shock costs {1}{R},
    // then {1} less: {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Medallion");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 3);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn herald_of_kozilek_increases_apply_before_reductions() {
    cr!("601.2f", "118.7");
    ruling!(
        "Herald of Kozilek",
        "If there are additional costs to cast a spell, or if the cost to cast a spell is increased by an effect, apply those increases before applying cost reductions."
    );
    supported("Herald of Kozilek");
    // "Colorless spells you cast cost {1} less to cast." With Thalia, Lotus Petal ({0})
    // costs {1}, then {1} less: {0} (reducing first, it would cost {1}).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herald of Kozilek");
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Wastes", 1);
    let petal = t.hand(P0, "Lotus Petal");
    t.cast(P0, petal).go();
    assert_eq!(tapped_lands(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Lotus Petal").len(), 1);
}

#[test]
fn ruby_medallion_x_is_chosen_before_the_total_cost() {
    cr!("601.2b", "601.2f", "107.3a");
    ruling!(
        "Ruby Medallion",
        "If a spell you cast has {X} in its mana cost, you choose the value of X before calculating the spell's total cost."
    );
    // Blaze with X = 2: {2}{R}, then {1} less: {1}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Medallion");
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(2).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn fieldmist_borderpost_a_cost_reduction_applies_to_its_alternative_cost() {
    cr!("601.2f", "118.9d");
    ruling!(
        "Fieldmist Borderpost",
        "Effects that increase or reduce the cost to cast this card will apply to whichever cost you chose to pay."
    );
    supported("Fieldmist Borderpost");
    supported("Etherium Sculptor");
    // "You may pay {1} and return a basic land you control to its owner's hand rather
    // than pay this spell's mana cost." Etherium Sculptor: "Artifact spells you cast cost
    // {1} less to cast." The alternative cost becomes {0} and the land.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Etherium Sculptor");
    let plains = t.battlefield(P0, "Plains");
    let post = t.hand(P0, "Fieldmist Borderpost");
    let alt = cast_methods(&mut t, P0, post)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the alternative cost is available");
    t.answer_choose(P0, &[Entity::Object(plains)]);
    t.cast(P0, post).method(alt).go();
    assert!(t.in_hand(P0, "Plains"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Fieldmist Borderpost").len(), 1);
    // Its normal cost {1}{W}{U} is reduced as well.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Etherium Sculptor");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let post = t.hand(P0, "Fieldmist Borderpost");
    t.cast(P0, post).go();
    assert_eq!(tapped_lands(&t, P0), 2);
}

#[test]
fn spectral_procession_a_generic_reduction_applies_only_to_generic_halves() {
    cr!("601.2b", "601.2f", "118.7a", "107.4e");
    ruling!(
        "Spectral Procession",
        "If an effect reduces the cost to cast a spell by an amount of generic mana, it applies to a monocolored hybrid spell only if you've chosen a method of paying for it that includes generic mana."
    );
    supported("Spectral Procession");
    // Goblin Electromancer: "Instant and sorcery spells you cast cost {1} less to cast."
    // Paid as {W}{W}{W}, nothing is reduced: two Plains aren't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    t.lands(P0, "Plains", 2);
    let sp = t.hand(P0, "Spectral Procession");
    assert!(t.cast(P0, sp).try_go().is_err());
    // Paid as {2}{W}{W}, it's reduced to {1}{W}{W}: two Plains and a Wastes.
    t.lands(P0, "Wastes", 1);
    t.cast(P0, sp).go();
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 3);
    // Without the reduction, those three lands can't pay it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Wastes", 1);
    let sp = t.hand(P0, "Spectral Procession");
    assert!(!can_cast(&mut t, P0, sp, CastMethod::Normal) || t.cast(P0, sp).try_go().is_err());
}

#[test]
fn mishras_workshop_mana_can_pay_an_artifact_spells_kicker() {
    cr!("106.6", "601.2f", "601.2h");
    ruling!(
        "Mishra's Workshop",
        "This mana may be used on additional costs to cast the spell, such as Kicker."
    );
    supported("Mishra's Workshop");
    supported("Skyclave Relic");
    // Skyclave Relic {3}, kicker {3}: "When this artifact enters, if it was kicked,
    // create two tapped tokens that are copies of this artifact." Six mana from two
    // Workshops ("Spend this mana only to cast artifact spells.").
    let mut t = TestGame::new(2);
    t.lands(P0, "Mishra's Workshop", 2);
    let relic = t.hand(P0, "Skyclave Relic");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, relic).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Skyclave Relic").len(), 3);
}

/// Casts Mythos of Brokkos ("If {U}{B} was spent to cast this spell, search your library
/// for a card, put that card into your graveyard, then shuffle. Return up to two
/// permanent cards from your graveyard to your hand.") with the given lands (and Chromatic
/// Orrery, tapped: "You may spend mana as though it were mana of any color."); whether it
/// searched (a card left the library).
fn mythos_searched(lands: &[(&str, usize)], orrery: bool) -> Option<bool> {
    let mut t = TestGame::new(2);
    if orrery {
        let o = t.battlefield(P0, "Chromatic Orrery");
        t.g.objects[o.0 as usize].tapped = true;
    }
    for (name, n) in lands {
        t.lands(P0, name, *n);
    }
    let mythos = t.hand(P0, "Mythos of Brokkos");
    let lib = t.library_size(P0);
    t.cast(P0, mythos).try_go().ok()?;
    t.resolve_all();
    Some(t.library_size(P0) == lib - 1)
}

#[test]
fn mythos_of_brokkos_checks_the_mana_actually_spent() {
    cr!("601.2h", "609.4b", "106.1a");
    ruling!(
        "Mythos of Brokkos",
        "The ability checks what mana was actually spent to cast a spell. If an effect allows you to spend mana “as though it were mana” of any color or type, that allows you to spend mana you couldn’t otherwise spend, but it doesn’t change what mana you spent to cast the spell."
    );
    supported("Mythos of Brokkos");
    supported("Chromatic Orrery");
    // Paid with green only: no search.
    assert_eq!(mythos_searched(&[("Forest", 4)], false), Some(false));
    // Blue and black spent (for the generic part): search.
    assert_eq!(
        mythos_searched(&[("Forest", 2), ("Island", 1), ("Swamp", 1)], false),
        Some(true)
    );
    // Islands can't pay {G}{G} without Orrery.
    assert_eq!(mythos_searched(&[("Island", 4)], false), None);
    // With Orrery, blue mana spent as though it were green is still blue: {U} and no
    // {B} was spent, so no search; with Swamps too, {U}{B} was spent.
    assert_eq!(mythos_searched(&[("Island", 4)], true), Some(false));
    assert_eq!(
        mythos_searched(&[("Island", 2), ("Swamp", 2)], true),
        Some(true)
    );
}

#[test]
fn mythos_of_brokkos_cast_without_paying_its_mana_cost_cant_be_paid_for() {
    cr!("118.9", "601.2f");
    ruling!(
        "Mythos of Brokkos",
        "If an effect allows you to cast a spell without paying its mana cost, you can’t choose to cast it and pay unless another rule or effect allows you to cast that spell for a cost. Similarly, you can’t waive a cost reduction unless that effect says you may."
    );
    supported("Rishkar's Expertise");
    // Rishkar's Expertise ("Draw cards equal to the greatest power among creatures you
    // control. You may cast a spell with mana value 5 or less from your hand without
    // paying its mana cost."): Mythos is cast for free even with Islands and Swamps
    // untapped; nothing is spent, so it doesn't search.
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::G, 6);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    let mythos = t.hand(P0, "Mythos of Brokkos");
    let rishkar = t.hand(P0, "Rishkar's Expertise");
    t.answer_choose(P0, &[Entity::Object(mythos)]);
    let lib = t.library_size(P0);
    t.cast(P0, rishkar).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mythos of Brokkos"));
    assert_eq!(tapped_lands(&t, P0), 0);
    assert_eq!(t.library_size(P0), lib);
    // A cost reduction isn't optional: with Goblin Electromancer it costs {1}{G}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    t.lands(P0, "Forest", 4);
    let mythos = t.hand(P0, "Mythos of Brokkos");
    t.cast(P0, mythos).go();
    assert_eq!(tapped_lands(&t, P0), 3);
}

#[test]
fn mythos_of_brokkos_is_cast_for_its_mana_cost_not_an_alternative_one() {
    cr!("601.2h", "118.9");
    ruling!(
        "Mythos of Brokkos",
        "The abilities of the Mythos check what colors of mana were spent to cast the spell. It’s not an alternative cost to cast the spell."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    let mythos = t.hand(P0, "Mythos of Brokkos");
    // The only way to cast it is for its mana cost {2}{G}{G}.
    assert_eq!(cast_methods(&mut t, P0, mythos), vec![CastMethod::Normal]);
    let lib = t.library_size(P0);
    t.cast(P0, mythos).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib - 1);
}

#[test]
fn an_expertise_cant_cast_a_land_card() {
    cr!("305.9", "608.2g", "601.3");
    ruling!(
        "Sram's Expertise",
        "Effects that allow you to \"cast\" a card don't allow you to play a land card."
    );
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let shock = t.hand(P0, "Shock");
    add_mana(&mut t, P0, ManaType::W, 4);
    let expertise = t.hand(P0, "Sram's Expertise");
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let from = t.asked().len();
    t.cast(P0, expertise).go();
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, prompt, .. } if prompt.contains("hand") => {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(offered, vec![vec![Entity::Object(shock)]]);
    assert_eq!(t.zone(forest), Zone::Hand(P0));
    assert_eq!(t.named_on_battlefield("Forest").len(), 0);
}
