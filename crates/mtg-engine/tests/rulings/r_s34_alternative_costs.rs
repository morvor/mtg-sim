//! Rulings batch S34 — alternative costs (CR 118.9c), "without paying its mana cost"
//! (the Expertise cycle, CR 601.3e), and convoke (CR 702.51b) don't change a spell's mana
//! cost or mana value; a card's mana value comes from its printed mana cost (CR 202.3,
//! 202.3a), a split card's from both halves (CR 709.4b), a fused split spell's from both
//! halves too (CR 702.102b).

use crate::r_s01_common::*;
use crate::r_s08_common::mana_value;
use crate::r_s20_common::sram_expertise;
use crate::r_s34_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn runeflare_trap_cast_for_its_alternative_cost_is_still_mana_value_6() {
    cr!("118.9c", "202.3");
    ruling!(
        "Runeflare Trap",
        "Casting a Trap by paying its alternative cost doesn't change its mana cost or mana value. The only difference is the cost you actually pay."
    );
    supported("Runeflare Trap");
    // Runeflare Trap ({4}{R}{R}): "If an opponent drew three or more cards this turn, you
    // may pay {R} rather than pay this spell's mana cost." Cast for {R}, it's still a
    // spell with mana cost {4}{R}{R}: P1's Kaervek deals 6 damage.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.g.draw_cards(P1, 3);
    t.lands(P0, "Mountain", 6);
    let trap = t.hand(P0, "Runeflare Trap");
    let alt = alternative(&mut t, trap);
    aim_kaervek_at_p0(&mut t);
    let spell = t.cast(P0, trap).method(alt).target(Entity::Player(P1)).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    assert_eq!(
        t.obj(spell).chars.mana_cost.as_ref().unwrap().to_string(),
        "{4}{R}{R}"
    );
    assert_eq!(mana_value(&t, spell), 6);
    t.resolve();
    assert_eq!(t.life(P0), 14);
}

#[test]
fn baloth_cage_trap_cast_for_its_alternative_cost_is_still_mana_value_5() {
    cr!("118.9c", "202.3");
    ruling!(
        "Baloth Cage Trap",
        "Casting a Trap by paying its alternative cost doesn’t change its mana cost or mana value. The only difference is the cost you actually pay."
    );
    supported("Baloth Cage Trap");
    // Baloth Cage Trap ({3}{G}{G}): "If an opponent had an artifact enter the battlefield
    // under their control this turn, you may pay {1}{G} rather than pay this spell's mana
    // cost." Cast for {1}{G}: Kaervek deals 5 damage.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.enter(P1, "Ornithopter");
    t.settle();
    t.lands(P0, "Forest", 5);
    let trap = t.hand(P0, "Baloth Cage Trap");
    let alt = alternative(&mut t, trap);
    aim_kaervek_at_p0(&mut t);
    let spell = t.cast(P0, trap).method(alt).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(mana_value(&t, spell), 5);
    t.resolve();
    assert_eq!(t.life(P0), 15);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Beast").len(), 1);
}

#[test]
fn sram_s_expertise_judges_cards_by_their_printed_mana_cost() {
    cr!("202.3", "202.3a", "118.6a", "601.3e");
    ruling!(
        "Sram's Expertise",
        "A card's mana value is determined solely by the mana symbols printed in its upper right corner. The mana value is the total amount of mana in that cost, regardless of color. For example, a card with mana cost {1}{U}{U} has mana value 3. Ignore any alternative costs, additional costs, cost increases, or cost reductions that could apply to it. A card with no mana cost has a mana value of 0."
    );
    supported("Sram's Expertise");
    supported("Ancestral Vision");
    // "You may cast a spell with mana value 3 or less from your hand without paying its
    // mana cost." Offered: Ancestral Vision (no mana cost: mana value 0) and Burst
    // Lightning ({R}, kicker {4}). Not offered: Gigastorm Titan ({4}{U}, though it would
    // cost {1}{U} now that P0 has cast a spell this turn), Force of Will ({3}{U}{U},
    // though its alternative cost has no mana), or Phyrexian Obliterator ({B}{B}{B}{B}).
    let mut t = TestGame::new(2);
    let vision = t.hand(P0, "Ancestral Vision");
    let bolt = t.hand(P0, "Burst Lightning");
    t.hand(P0, "Gigastorm Titan");
    t.hand(P0, "Force of Will");
    t.hand(P0, "Phyrexian Obliterator");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let library = t.library_size(P0);
    let (offered, _) = sram_expertise(&mut t, Some(vision), 0);
    let mut offered = offered;
    offered.sort_by_key(|e| e.object());
    let mut expected = vec![Entity::Object(vision), Entity::Object(bolt)];
    expected.sort_by_key(|e| e.object());
    assert_eq!(offered, expected);
    // Ancestral Vision, which can't be cast for its (unpayable) mana cost, was cast.
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.library_size(P0), library - 3);
    assert!(t.in_graveyard(P0, "Ancestral Vision"));
}

#[test]
fn sram_s_expertise_can_cast_either_half_or_both_halves_of_a_fuse_card() {
    cr!("709.4b", "702.102a", "702.102b", "601.3e");
    ruling!(
        "Sram's Expertise",
        "The mana value of a split card is determined by the combined mana cost of its two halves. If an expertise spell allows you to cast a split card, you may cast either half or, if that split card has fuse, both halves."
    );
    supported("Sram's Expertise");
    supported("Wear // Tear");
    supported("Turn // Burn");
    // Wear // Tear ({1}{R} // {W}, fuse) has mana value 3 in hand. "Mana value 3 or less":
    // Wear, Tear, or both fused (mana value 3) can be cast. P0 casts both.
    let mut t = TestGame::new(2);
    let wt = t.hand(P0, "Wear // Tear");
    assert_eq!(mana_value(&t, wt), 3);
    let thopter = t.battlefield(P1, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    let (offered, ways) = sram_expertise(&mut t, Some(wt), 2);
    assert_eq!(offered, vec![Entity::Object(wt)]);
    assert_eq!(ways.len(), 3);
    assert_eq!(t.stack_len(), 1);
    let spell = t.g.stack[0];
    assert_eq!(mana_value(&t, spell), 3);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    // Turn // Burn ({2}{U} // {1}{R}, fuse): Turn (3) or Burn (2) can be cast this way,
    // but not both (a fused spell with mana value 5).
    let mut t = TestGame::new(2);
    let tb = t.hand(P0, "Turn // Burn");
    assert_eq!(mana_value(&t, tb), 5);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let (offered, ways) = sram_expertise(&mut t, Some(tb), 1);
    assert_eq!(offered, vec![Entity::Object(tb)]);
    assert_eq!(ways, vec!["Cast Turn", "Cast Burn"]);
    assert_eq!(t.obj(t.g.stack[0]).chars.name, "Burn");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn convoke_pays_the_total_cost_after_increases_and_keeps_the_mana_value() {
    cr!("702.51a", "702.51b", "601.2f", "202.3");
    ruling!(
        "Vote Out",
        "When calculating a spell’s total cost, include any alternative costs, additional costs, or anything else that increases or reduces the cost to cast the spell. Convoke applies after the total cost is calculated. Convoke doesn’t change a spell’s mana cost or mana value."
    );
    supported("Vote Out");
    // Vote Out ({3}{B}, convoke, "Destroy target creature."). With P1's Thalia the total
    // cost is {4}{B}: five creatures pay it all (the black Walking Corpse pays {B}).
    // Kaervek still sees mana value 4.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    let thalia = t.battlefield(P1, "Thalia, Guardian of Thraben");
    let mut convokers = vec![t.battlefield(P0, "Walking Corpse")];
    for _ in 0..4 {
        convokers.push(t.battlefield(P0, "Grizzly Bears"));
    }
    let vote = t.hand(P0, "Vote Out");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(convokers.iter().map(|c| Entity::Object(*c)).collect()),
    );
    aim_kaervek_at_p0(&mut t);
    let spell = t.cast(P0, vote).target(thalia).go();
    assert!(convokers.iter().all(|c| t.obj(*c).tapped));
    let info = t.obj(spell).stack.as_ref().unwrap().cast.clone();
    assert_eq!(info.convoked.len(), 5);
    assert_eq!(mana_value(&t, spell), 4);
    t.resolve();
    assert_eq!(t.life(P0), 16);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Thalia, Guardian of Thraben"));
}

#[test]
fn ingenious_mastery_cast_for_its_alternative_cost_has_the_mana_value_of_its_mana_cost() {
    cr!("118.9c", "202.3", "107.3b", "601.2f");
    ruling!(
        "Ingenious Mastery",
        "The mana value of a spell on the stack is determined by its mana cost, not any alternative costs you used to pay for it."
    );
    supported("Ingenious Mastery");
    supported("Thalia, Guardian of Thraben");
    // Ingenious Mastery ({X}{2}{U}): "You may pay {2}{U} rather than pay this spell's mana
    // cost. If the {2}{U} cost was paid, you draw three cards, then an opponent creates two
    // Treasure tokens and they scry 2. If that cost wasn't paid, you draw X cards." With
    // P1's Thalia the alternative cost totals {3}{U}, but the spell's mana value comes
    // from {X}{2}{U} with X = 0: no X can be chosen when paying an alternative cost
    // without X (an attempt to announce 3 is ignored), so its mana value is 3.
    let mut t = TestGame::new(2);
    kaervek(&mut t);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 6);
    let mastery = t.hand(P0, "Ingenious Mastery");
    let alt = alternative(&mut t, mastery);
    aim_kaervek_at_p0(&mut t);
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, mastery).method(alt).x(3).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(
        t.obj(spell).chars.mana_cost.as_ref().unwrap().to_string(),
        "{X}{2}{U}"
    );
    assert_eq!(mana_value(&t, spell), 3);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    // The {2}{U} cost was paid: three cards, and two Treasures for P1.
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert_eq!(with_subtype(&t, P1, "Treasure").len(), 2);
}
