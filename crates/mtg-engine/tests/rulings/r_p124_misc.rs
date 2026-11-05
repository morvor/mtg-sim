//! Rulings batch P124 — the other abilities of the batch's protective cards: Bello's
//! animated permanents (CR 613.6, 704.5m, 704.5n), delayed triggers (CR 603.7), "you
//! gained life this turn" (CR 603.4), counting while on the stack (CR 608.2h), countering
//! a spell that targets you, granted unearth (CR 113.7, 702.84), double-faced cards'
//! characteristics (CR 712.8), mana spent to cast (CR 601.2h), illegal targets (CR
//! 608.2b), naming split cards (CR 709.3a), and cards that restate their own text.

use crate::r_p124_common::*;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s17_common::face;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters::{LOYALTY, PLUS1};
use mtg_engine::types::CardType;
use mtg_engine::*;

fn is_creature(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).is(CardType::Creature)
}

#[test]
fn bello_animated_permanents_keep_counters_and_shed_attachments() {
    cr!("122.1", "704.5m", "704.5n");
    ruling!(
        "Bello, Bard of the Brambles",
        "Any counters that are put on an artifact or enchantment while it's a creature because of Bello's ability remain on that artifact or enchantment while it's not a creature"
    );
    ruling!(
        "Bello, Bard of the Brambles",
        "If an Equipment becomes attached to an artifact or enchantment that's currently a creature because of Bello's ability, it'll become unattached during the next upkeep that isn't yours."
    );
    supported("Bello, Bard of the Brambles");
    // Jayemdae Tome (an artifact with mana value 4) is a 4/4 creature during P0's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bello, Bard of the Brambles");
    let tome = t.battlefield(P0, "Jayemdae Tome");
    assert!(is_creature(&mut t, tome));
    assert_eq!(t.pt(tome), (4, 4));
    give_plus1(&mut t, tome, 1);
    let splitter = attach_new(&mut t, P0, "Bonesplitter", tome);
    let aura = attach_new(&mut t, P0, "Holy Strength", tome);
    t.g.recompute();
    assert_eq!(t.pt(tome), (8, 7));
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert!(!is_creature(&mut t, tome));
    assert_eq!(t.counters(tome, PLUS1), 1);
    assert_eq!(attached_to(&t, splitter), None);
    assert!(t.on_battlefield(splitter));
    assert_ne!(t.zone(aura), Zone::Battlefield);
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn bello_losing_its_abilities_still_animates() {
    cr!("613.6", "613.1d", "613.1f");
    ruling!(
        "Bello, Bard of the Brambles",
        "If an effect causes Bello to lose all abilities during your turn, its effect will still apply to non-Equipment artifacts and non-Aura enchantments you control."
    );
    let mut t = TestGame::new(2);
    let bello = t.battlefield(P0, "Bello, Bard of the Brambles");
    let tome = t.battlefield(P0, "Jayemdae Tome");
    lose_abilities(&mut t, bello);
    assert!(is_creature(&mut t, tome));
    assert_eq!(t.pt(tome), (4, 4));
    assert!(has_kw(&mut t, tome, KeywordKind::Indestructible));
    assert!(has_kw(&mut t, tome, KeywordKind::Haste));
}

#[test]
fn archangel_avacyn_transforms_at_the_next_upkeep_whoever_it_belongs_to() {
    cr!("603.7a", "603.7c", "701.27a");
    ruling!(
        "Archangel Avacyn // Avacyn, the Purifier",
        "Archangel Avacyn's delayed triggered ability triggers at the beginning of the next upkeep regardless of whose turn it is."
    );
    supported("Archangel Avacyn // Avacyn, the Purifier");
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P0, "Archangel Avacyn // Avacyn, the Purifier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(face(&t, avacyn), FaceState::Front);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(face(&t, avacyn), FaceState::Back);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn crested_sunmare_cares_only_whether_you_gained_life_before_the_end_step() {
    cr!("603.4", "513.1");
    ruling!(
        "Crested Sunmare",
        "Crested Sunmare’s triggered ability cares only whether you gained life in the turn, even if Crested Sunmare wasn’t on the battlefield when that happened."
    );
    ruling!(
        "Crested Sunmare",
        "Crested Sunmare’s triggered ability won’t trigger unless you’ve gained life in the turn before the end step began."
    );
    // P0 gains 1 and loses 5, then Sunmare enters: a Horse at the end step.
    let mut t = TestGame::new(2);
    gain_life(&mut t, P0, 1);
    t.g.lose_life(P0, 5);
    t.battlefield(P0, "Crested Sunmare");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Horse").len(), 2);
    // Wall of Reverence ("At the beginning of your end step, you may gain life equal to
    // the power of target creature you control.") gains life only in the end step.
    supported("Wall of Reverence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crested Sunmare");
    let wall = t.battlefield(P0, "Wall of Reverence");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(wall)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Horse").len(), 1);
}

#[test]
fn dark_dabbling_doesnt_count_itself() {
    cr!("608.2h", "701.19a");
    ruling!(
        "Dark Dabbling",
        "Dark Dabbling is still on the stack while you count the number of instant and/or sorcery cards in your graveyard. It doesn’t count itself."
    );
    supported("Dark Dabbling");
    // With one instant in the graveyard (two once Dark Dabbling is there), only the target
    // is regenerated; with two, the Hill Giant is too.
    for (instants, giant_survives) in [(1, false), (2, true)] {
        let mut t = TestGame::new(2);
        for _ in 0..instants {
            t.graveyard(P0, "Shock");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        cast_resolve(&mut t, P0, "Dark Dabbling", &[obj(bears)]);
        destroy(&mut t, bears);
        destroy(&mut t, giant);
        assert!(t.on_battlefield(bears));
        assert_eq!(t.on_battlefield(giant), giant_survives, "{instants} instants");
    }
}

#[test]
fn dawn_charm_counters_a_spell_with_several_targets_one_of_them_you() {
    cr!("115.1", "701.6a", "700.2");
    ruling!(
        "Dawn Charm",
        "Dawn Charm's third mode can target a spell that has multiple targets, as long as at least one of those targets is you."
    );
    supported("Dawn Charm");
    // P1's Electrolyze ("deals 2 damage divided as you choose among one or two targets")
    // at P0 and P0's Grizzly Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Island", 1);
    t.answer_targets(P1, &[Entity::Player(P0), obj(bears)]);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let electrolyze = t.hand(P1, "Electrolyze");
    let spell = t.cast(P1, electrolyze).go();
    lands_for_cost(&mut t, P0, "Dawn Charm");
    let charm = t.hand(P0, "Dawn Charm");
    t.cast(P0, charm).modes(&[2]).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Electrolyze"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn mishras_unearth_is_an_ability_of_the_artifact_card() {
    cr!("113.6", "702.84a", "113.7a");
    ruling!(
        "Mishra, Tamer of Mak Fawa",
        "the unearth abilities that Mishra, Tamer of Mak Fawa grants are activated abilities of each individual artifact card in your graveyard. They're not activated abilities of Mishra."
    );
    supported("Mishra, Tamer of Mak Fawa");
    // Activated from Ornithopter, it resolves even once Mishra is gone.
    let mut t = TestGame::new(2);
    let mishra = t.battlefield(P0, "Mishra, Tamer of Mak Fawa");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    t.activate(P0, thopter, 0, &[]).unwrap();
    let ability = *t.g.stack.last().unwrap();
    assert!(
        matches!(
            t.obj(ability).stack.as_ref().unwrap().kind,
            mtg_engine::object::StackKind::Activated { source, .. } if source == thopter
        ),
        "the source is the card"
    );
    exile_now(&mut t, mishra);
    t.resolve_all();
    let back = t.g.current(thopter);
    assert!(t.on_battlefield(back));
    assert!(has_kw(&mut t, back, KeywordKind::Haste));
}

#[test]
fn liliana_has_only_the_characteristics_of_its_face_up_face() {
    cr!("712.8a", "712.8d", "712.8e");
    ruling!(
        "Liliana, Heretical Healer // Liliana, Defiant Necromancer",
        "While a double-faced card is on the battlefield, consider only the characteristics of the face that’s currently up."
    );
    supported("Liliana, Heretical Healer // Liliana, Defiant Necromancer");
    let mut t = TestGame::new(2);
    let liliana = t.battlefield(P0, "Liliana, Heretical Healer // Liliana, Defiant Necromancer");
    assert!(is_creature(&mut t, liliana));
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    let pw = t.g.current(liliana);
    assert!(t.on_battlefield(pw));
    t.g.recompute();
    let o = t.obj(pw);
    assert_eq!(o.chars.name.as_str(), "Liliana, Defiant Necromancer");
    assert!(o.is(CardType::Planeswalker) && !o.is(CardType::Creature));
    assert_eq!(t.counters(pw, LOYALTY), 3);
    // In the graveyard: the front face only.
    let pw = t.g.current(pw);
    t.g.remove_counters(Entity::Object(pw), LOYALTY, 3);
    t.g.dirty = true;
    t.settle();
    let card = t.g.current(pw);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    t.g.recompute();
    let o = t.obj(card);
    assert_eq!(o.chars.name.as_str(), "Liliana, Heretical Healer");
    assert!(o.is(CardType::Creature) && !o.is(CardType::Planeswalker));
}

#[test]
fn boromir_counters_only_spells_cast_with_no_mana() {
    cr!("601.2h", "603.4", "702.34a");
    ruling!(
        "Boromir, Warden of the Tower",
        "If a spell is cast with an alternative cost that includes mana (such as flashback), Boromir's second ability won't trigger and won't counter that spell."
    );
    supported("Boromir, Warden of the Tower");
    // P1's Think Twice cast with flashback ({2}{U}) isn't countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boromir, Warden of the Tower");
    t.lands(P1, "Island", 3);
    let tt = t.graveyard(P1, "Think Twice");
    let hand = t.hand_size(P1);
    t.cast(P1, tt).method(CastMethod::Keyword(KeywordKind::Flashback)).go();
    t.settle();
    assert_eq!(t.stack_len(), 1, "no trigger");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    // P1's Invigorate for its alternative cost (P0 gains 3 life) is countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boromir, Warden of the Tower");
    t.lands(P1, "Forest", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let invigorate = t.hand(P1, "Invigorate");
    let alt = t
        .cast_options(P1, invigorate)
        .into_iter()
        .map(|o| o.method)
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .unwrap();
    t.cast(P1, invigorate).method(alt).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Invigorate"));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn resistance_reunited_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Resistance Reunited",
        "If the target creature is no longer a legal target as Resistance Reunited tries to resolve, it won't resolve and no creatures will gain indestructible."
    );
    supported("Resistance Reunited");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Bonesplitter", bears);
    let giant = t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Resistance Reunited", &[obj(giant)]);
    bounce(&mut t, giant);
    t.resolve_all();
    assert!(!indestructible(&mut t, bears));
}

#[test]
fn veil_of_summer_draws_one_card_however_many_spells() {
    cr!("608.2c");
    ruling!(
        "Veil of Summer",
        "If your opponents have cast more than one blue or black spell, you still draw only one card as Veil of Summer resolves."
    );
    supported("Veil of Summer");
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 2);
    for _ in 0..2 {
        let opt = t.hand(P1, "Opt");
        t.cast(P1, opt).go();
        t.resolve_all();
    }
    lands_for_cost(&mut t, P0, "Veil of Summer");
    let veil = t.hand(P0, "Veil of Summer");
    let hand = t.hand_size(P0);
    t.cast(P0, veil).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn gallifrey_falls_a_name_chosen_matches_either_half() {
    cr!("709.4", "709.4a", "701.9b");
    ruling!(
        "Gallifrey Falls // No More",
        "A split card has the chosen name if one of its two names matches the chosen name."
    );
    // Cabal Therapy naming "No More": P1 discards Gallifrey Falls // No More.
    supported("Gallifrey Falls // No More");
    for name in ["No More", "Gallifrey Falls"] {
        let mut t = TestGame::new(2);
        t.hand(P1, "Gallifrey Falls // No More");
        t.hand(P1, "Grizzly Bears");
        t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
        cast_resolve(&mut t, P0, "Cabal Therapy", &[Entity::Player(P1)]);
        assert!(t.in_graveyard(P1, "Gallifrey Falls // No More"), "{name}");
        assert!(t.in_hand(P1, "Grizzly Bears"));
    }
}
