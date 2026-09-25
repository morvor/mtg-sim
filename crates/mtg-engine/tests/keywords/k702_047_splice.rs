//! CR 702.47 Splice.

use crate::common_k702_011_017::assert_supported;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

fn splice_offers(t: &TestGame) -> Vec<ObjectId> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::OptionalCost { source, name, .. } if name == "splice" => Some(source),
            _ => None,
        })
        .collect()
}

fn spell_ability_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Spell(_)))
        .count()
}

fn splice(t: &mut TestGame, p: PlayerId, yes: bool) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(yes));
}

fn tokens_named(t: &TestGame, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

#[test]
fn a_spliced_card_adds_its_effects_and_its_cost_is_paid() {
    cr!("702.47", "702.47a");
    assert_supported("Kodama's Might");
    assert_supported("Glacial Ray");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let might = t.hand(P0, "Kodama's Might");
    let ray = t.hand(P0, "Glacial Ray");
    splice(&mut t, P0, true);
    let spell = t.cast(P0, might).target(bears).target(P1).go();
    assert_eq!(splice_offers(&t), vec![ray]);
    assert_eq!(spell_ability_count(&t, spell), 2);
    // {G} plus the splice cost {1}{R}; the spliced card stays in hand.
    assert!(t.g.permanents().all(|o| !o.chars.is_land() || o.tapped));
    assert!(t.in_hand(P0, "Glacial Ray"));
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn splice_onto_instant_or_sorcery() {
    cr!("702.47a");
    ruling!(
        "Everdream",
        "The abilities spliced onto the spell happen last, after all of that spell’s other effects."
    );
    assert_supported("Everdream");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.hand(P0, "Everdream");
    splice(&mut t, P0, true);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Drew a card; Everdream is still in hand.
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_hand(P0, "Everdream"));
}

#[test]
fn the_main_spells_effects_happen_before_the_spliced_ones() {
    cr!("702.47b");
    ruling!(
        "Splicer's Skill",
        "Notably, the Golem won’t be on the battlefield yet if the spell has an effect that affects all creatures"
    );
    assert_supported("Splicer's Skill");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 4);
    let pyro = t.hand(P0, "Pyroclasm");
    t.hand(P0, "Splicer's Skill");
    splice(&mut t, P0, true);
    t.cast(P0, pyro).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // The Golem was created after the damage was dealt.
    assert_eq!(tokens_named(&t, "Golem"), 1);
}

#[test]
fn splice_costs_can_be_other_than_mana() {
    cr!("702.47a");
    ruling!(
        "Torrent of Stone",
        "You can sacrifice any two lands that have the subtype Mountain to splice Torrent of Stone onto an Arcane spell, not just lands named Mountain."
    );
    assert_supported("Torrent of Stone");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Stomping Ground");
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Torrent of Stone");
    splice(&mut t, P0, true);
    t.cast(P0, might).target(bears).target(giant).go();
    // Both Mountains (one of them a Stomping Ground) were sacrificed.
    assert!(t.in_graveyard(P0, "Mountain"));
    assert!(t.in_graveyard(P0, "Stomping Ground"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn a_splice_cost_can_be_an_action() {
    cr!("702.47a");
    let mut t = TestGame::new(2);
    // Roar of Jukai: "Splice onto Arcane—An opponent gains 5 life."
    let roar = card("Roar of Jukai");
    let splice_kw = roar
        .front()
        .chars
        .abilities
        .iter()
        .find_map(|a| a.keyword().filter(|k| k.kind == KeywordKind::Splice).cloned())
        .expect("Roar of Jukai has splice");
    assert!(splice_kw.cost.is_some());
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Roar of Jukai");
    splice(&mut t, P0, true);
    t.cast(P0, might).target(bears).go();
    assert_eq!(t.life(P1), 25);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn a_card_whose_targets_cant_be_chosen_cant_be_spliced() {
    cr!("702.47b");
    assert_supported("Blessed Breath");
    let mut t = TestGame::new(2);
    // Blessed Breath needs a target creature you control; P0 controls none.
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Blessed Breath");
    t.cast(P0, might).target(giant).go();
    assert!(splice_offers(&t).is_empty());
}

#[test]
fn a_card_cant_be_spliced_onto_itself_but_one_with_the_same_name_can() {
    cr!("702.47b");
    ruling!(
        "Glacial Ray",
        "A card with a splice ability can’t be spliced onto itself because the spell is on the stack (and not in your hand) when you reveal the cards you want to splice onto it."
    );
    ruling!(
        "Everdream",
        "Each individual card can be spliced only once onto any one spell, although multiple cards with the same name may be spliced onto one spell."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let ray1 = t.hand(P0, "Glacial Ray");
    let ray2 = t.hand(P0, "Glacial Ray");
    splice(&mut t, P0, true);
    t.cast(P0, ray1).target(P1).target(P1).go();
    // Only the other Glacial Ray was offered, once.
    assert_eq!(splice_offers(&t), vec![ray2]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn the_order_of_several_spliced_cards_is_chosen() {
    cr!("702.47b");
    ruling!(
        "Glacial Ray",
        "You reveal all cards you intend to splice at the same time. Each individual card can be spliced only once onto any one spell."
    );
    assert_supported("Hideous Laughter");
    assert_supported("Spiritual Visit");
    // Main spell Kodama's Might; spliced Hideous Laughter and Spiritual Visit. If the
    // Spirit is created first, Hideous Laughter's -2/-2 kills it.
    for (order, spirit_survives) in [(vec![0usize, 1], true), (vec![1, 0], false)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Swamp", 5);
        t.lands(P0, "Plains", 1);
        let might = t.hand(P0, "Kodama's Might");
        t.hand(P0, "Hideous Laughter");
        t.hand(P0, "Spiritual Visit");
        splice(&mut t, P0, true);
        splice(&mut t, P0, true);
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        let spell = t.cast(P0, might).target(bears).go();
        assert_eq!(spell_ability_count(&t, spell), 3);
        t.resolve_all();
        assert_eq!(tokens_named(&t, "Spirit") == 1, spirit_survives);
        // Kodama's Might's own effect happened: the Bears survive as a 2/2.
        assert_eq!(t.pt(bears), (2, 2));
    }
}

#[test]
fn the_spell_keeps_its_own_characteristics_and_named_text_refers_to_it() {
    cr!("702.47c");
    let mut t = TestGame::new(2);
    // Silver Knight has protection from red. The spell Glacial Ray's text is spliced onto
    // is green: "Glacial Ray deals 2 damage" means the green spell deals it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let knight = t.battlefield(P1, "Silver Knight");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 2);
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Glacial Ray");
    splice(&mut t, P0, true);
    let spell = t.cast(P0, might).target(bears).target(knight).go();
    let o = t.obj_now(spell);
    assert_eq!(o.chars.name, "Kodama's Might");
    assert!(o.chars.colors.contains(Color::Green));
    assert!(!o.chars.colors.contains(Color::Red));
    assert_eq!(o.chars.mana_cost.as_ref().unwrap().to_string(), "{G}");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Silver Knight"));
}

#[test]
fn targets_for_the_spliced_text_are_chosen_and_checked_normally() {
    cr!("702.47d");
    ruling!(
        "Glacial Ray",
        "You choose all targets for the spell after revealing cards you want to splice, including any targets required by the text of any of those cards."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Mountain", 1);
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Glacial Ray");
    splice(&mut t, P0, true);
    t.cast(P0, might).target(bears).target(giant).go();
    // In response, the Giant becomes an illegal target: the rest still resolves.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn a_spell_whose_targets_are_all_illegal_does_nothing() {
    cr!("702.47d");
    ruling!(
        "Glacial Ray",
        "If all of the spell’s targets are illegal when the spell tries to resolve, it won’t resolve and none of its effects will happen."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Mountain", 1);
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Spiritual Visit");
    splice(&mut t, P0, true);
    t.cast(P0, might).target(bears).go();
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    // Its only target is gone: no Spirit is created either.
    assert!(!t.on_battlefield(bears));
    assert_eq!(tokens_named(&t, "Spirit"), 0);
}

#[test]
fn the_spell_loses_the_spliced_text_when_it_leaves_the_stack() {
    cr!("702.47e");
    ruling!(
        "Everdream",
        "If the spell is countered, any cards you spliced onto it remain in your hand."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 2);
    t.lands(P1, "Island", 2);
    let might = t.hand(P0, "Kodama's Might");
    t.hand(P0, "Glacial Ray");
    splice(&mut t, P0, true);
    let spell = t.cast(P0, might).target(bears).target(P1).go();
    assert_eq!(spell_ability_count(&t, spell), 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_hand(P0, "Glacial Ray"));
    let gy = t.g.find_in_zone(Zone::Graveyard(P0), "Kodama's Might")[0];
    assert_eq!(spell_ability_count(&t, gy), 1);
}

#[test]
fn a_copy_of_the_spell_has_the_spliced_text_too() {
    cr!("702.47c", "702.47e");
    ruling!(
        "Everdream",
        "If a spell is copied, choices made while casting it are copied, so the copy will have the same abilities spliced onto it as the original."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Island", 4);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    // Grapeshot with Everdream spliced onto it; its storm trigger copies it once.
    let shot = t.hand(P0, "Grapeshot");
    t.hand(P0, "Everdream");
    splice(&mut t, P0, true);
    t.cast(P0, shot).target(P1).go();
    t.resolve_all();
    // Two Grapeshots and two draws.
    assert_eq!(t.life(P1), 20 - 3 - 1 - 1);
    assert_eq!(t.hand_size(P0), 3);
}
