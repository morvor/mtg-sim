//! Rulings batch P182 — characteristics that depend on the game: Karametra's devotion
//! (CR 700.5, 613.1d, 614.12), and Knight of the Reliquary's and Fiend Artisan's
//! self-pumping static abilities, which apply only on the battlefield (CR 611.3, 113.6).

use crate::r_p182_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const KARAMETRA: &str = "Karametra, God of Harvests";

/// A game in which P0 controls `n` Elvish Mystics (devotion to green `n`).
fn with_mystics(n: usize) -> (TestGame, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let m = (0..n).map(|_| t.battlefield(P0, "Elvish Mystic")).collect();
    (t, m)
}

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(t.g.current(id)).is(CardType::Creature)
}

#[test]
fn karametra_is_a_creature_off_the_battlefield_and_on_the_stack() {
    cr!("700.5", "611.3a");
    ruling!(
        "Karametra, God of Harvests",
        "The type-changing ability that can make Karametra not be a creature functions only on the battlefield. It's always a creature card in other zones, regardless of your devotion to its color. It's always a creature spell while it's on the stack."
    );
    let mut t = TestGame::new(2);
    let k = t.hand(P0, KARAMETRA);
    assert!(is_creature(&t, k));
    let g = t.graveyard(P0, KARAMETRA);
    assert!(is_creature(&t, g));
    let spell = cast(&mut t, P0, KARAMETRA, "3GW", &[]);
    assert!(is_creature(&t, spell), "a creature spell");
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(spell)));
    assert!(!is_creature(&t, spell), "devotion 2 on the battlefield");
}

#[test]
fn karametra_as_it_enters_counts_devotion_without_itself_and_after_with_itself() {
    cr!("614.12", "603.6a", "700.5");
    ruling!(
        "Karametra, God of Harvests",
        "As Karametra enters the battlefield, your devotion to green and white will determine whether any replacement effects that affect creatures entering the battlefield apply. Because replacement effects are considered before Karametra is on the battlefield, the mana symbols in its mana cost won't be counted when determining this."
    );
    ruling!(
        "Karametra, God of Harvests",
        "When Karametra enters the battlefield, your devotion to green and white will determine if a creature entered the battlefield or not for abilities that trigger whenever a creature enters the battlefield. The mana symbols in Karametra's own mana cost are counted when determining this."
    );
    // Five Mystics and Soul Warden ({W}): devotion 6 without Karametra, 8 with it. P1's
    // Thalia, Heretic Cathar taps entering creatures; Soul Warden sees a creature enter.
    let (mut t, _) = with_mystics(5);
    t.battlefield(P1, "Thalia, Heretic Cathar");
    t.battlefield(P0, "Soul Warden");
    let spell = cast(&mut t, P0, KARAMETRA, "3GW", &[]);
    t.resolve_all();
    let k = t.g.current(spell);
    assert!(t.on_battlefield(k) && is_creature(&t, k));
    assert!(!t.g.obj(k).tapped, "not a creature as it entered: Thalia didn't apply");
    assert_eq!(t.life(P0), 21, "a creature entered: Soul Warden triggered");
    // Seven Mystics: a creature already as it enters (six wouldn't do: it's not counted).
    let (mut t, _) = with_mystics(7);
    t.battlefield(P1, "Thalia, Heretic Cathar");
    let spell = cast(&mut t, P0, KARAMETRA, "3GW", &[]);
    t.resolve_all();
    assert!(t.g.obj(t.g.current(spell)).tapped);
    // Three Mystics and Soul Warden: devotion 6 with Karametra, never a creature, so Soul
    // Warden doesn't trigger.
    let (mut t, _) = with_mystics(3);
    t.battlefield(P0, "Soul Warden");
    cast(&mut t, P0, KARAMETRA, "3GW", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn karametra_not_a_creature_keeps_counters_types_and_abilities() {
    cr!("205.1b", "122.1", "506.4", "613.1d", "613.1f", "700.5");
    ruling!(
        "Karametra, God of Harvests",
        "Counters put on Karametra remain on it while it's not a creature, even if they have no effect."
    );
    ruling!(
        "Karametra, God of Harvests",
        "If Karametra stops being a creature, it loses the type creature and the creature type God. It continues to be a legendary enchantment."
    );
    ruling!(
        "Karametra, God of Harvests",
        "If Karametra is attacking or blocking and it stops being a creature, it will be removed from combat. It won't rejoin combat if it resumes being a creature later during that combat."
    );
    ruling!(
        "Karametra, God of Harvests",
        "Karametra's abilities function as long as it's on the battlefield, regardless of whether it's a creature."
    );
    ruling!(
        "Karametra, God of Harvests",
        "If an effect causes Karametra to lose all abilities, its ability that causes it to stop being a creature still applies if appropriate."
    );
    let (mut t, mystics) = with_mystics(5);
    let k = t.battlefield(P0, KARAMETRA);
    assert!(is_creature(&t, k));
    crate::r_p003_common::put(&mut t, k, counters::PLUS1, 2);
    attack_with(&mut t, &[(k, pl(P1))]);
    assert!(attacking(&t, k));
    // A Mystic leaves: devotion 6.
    destroy(&mut t, mystics[0]);
    t.g.recompute();
    let o = t.g.obj(k);
    assert!(!o.is(CardType::Creature));
    assert!(o.is(CardType::Enchantment) && o.chars.supertypes.contains(Supertype::Legendary));
    assert!(!o.chars.has_subtype("God"));
    assert_eq!(t.counters(k, counters::PLUS1), 2);
    assert!(!attacking(&t, k), "removed from combat");
    // Devotion is back: a creature again, but not attacking.
    t.battlefield(P0, "Elvish Mystic");
    t.g.recompute();
    assert!(is_creature(&t, k));
    assert!(!attacking(&t, k));
    // Not a creature, its last ability still triggers.
    let (mut t, _) = with_mystics(0);
    let lib = set_library(&mut t, P0, &["Forest"]);
    let k = t.battlefield(P0, KARAMETRA);
    assert!(!is_creature(&t, k));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    cast(&mut t, P0, "Grizzly Bears", "1G", &[]);
    t.resolve_all();
    assert!(tapped(&t, lib[0]));
    // Losing all abilities doesn't make it a creature.
    let (mut t, _) = with_mystics(0);
    let k = t.battlefield(P0, KARAMETRA);
    crate::r_p117_common::modify(
        &mut t,
        k,
        vec![mtg_engine::ability::Modification::RemoveAllAbilities],
    );
    assert!(!t.obj_now(k).has_keyword(KeywordKind::Indestructible), "it lost its abilities");
    assert!(!is_creature(&t, k));
}

#[test]
fn karametra_doesnt_see_a_bestowed_aura_spell() {
    cr!("702.103b", "603.2");
    ruling!(
        "Karametra, God of Harvests",
        "If you cast a creature card with bestow for its bestow cost, it becomes an Aura spell and not a creature spell. Karametra's last ability won't trigger."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, KARAMETRA);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let satyr = t.hand(P0, "Boon Satyr");
    mana(&mut t, P0, "3GG");
    t.cast(P0, satyr)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(obj(bears))
        .go();
    t.settle();
    assert_eq!(crate::r_s11_common::triggered_from(&t, k), 0);
    t.resolve_all();
    // Cast normally, it triggers.
    let satyr = t.hand(P0, "Boon Satyr");
    mana(&mut t, P0, "1GG");
    t.answer_yes(P0, false);
    t.cast(P0, satyr).go();
    t.settle();
    assert_eq!(crate::r_s11_common::triggered_from(&t, k), 1);
}

#[test]
fn self_pumps_apply_only_on_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!(
        "Knight of the Reliquary",
        "Knight of the Reliquary's first ability applies only while it's on the battlefield. In all other zones, it's a 2/2 creature."
    );
    ruling!(
        "Fiend Artisan",
        "Fiend Artisan's first ability applies only while it's on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Plains");
    t.graveyard(P0, "Grizzly Bears");
    let knight = t.hand(P0, "Knight of the Reliquary");
    let artisan = t.hand(P0, "Fiend Artisan");
    assert_eq!(t.pt(knight), (2, 2));
    assert_eq!(t.pt(artisan), (1, 1));
    let knight_gy = t.graveyard(P0, "Knight of the Reliquary");
    let artisan_gy = t.graveyard(P0, "Fiend Artisan");
    assert_eq!(t.pt(knight_gy), (2, 2));
    assert_eq!(t.pt(artisan_gy), (1, 1));
    let k = t.battlefield(P0, "Knight of the Reliquary");
    let a = t.battlefield(P0, "Fiend Artisan");
    assert_eq!(t.pt(k), (4, 4), "two land cards");
    // Grizzly Bears, Knight of the Reliquary and Fiend Artisan are creature cards.
    assert_eq!(t.pt(a), (4, 4));
}

#[test]
fn fiend_artisan_damage_becomes_lethal_as_creature_cards_leave() {
    cr!("704.5g", "120.6");
    ruling!(
        "Fiend Artisan",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Fiend Artisan may become lethal if creature cards leave your graveyard during that turn."
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Grizzly Bears");
    let artisan = t.battlefield(P0, "Fiend Artisan");
    let src = t.battlefield(P1, "Mind Stone");
    crate::r_p003_common::damage(&mut t, src, artisan, 2);
    assert!(t.on_battlefield(artisan), "3/3 with 2 damage");
    for c in [a, b] {
        t.g.move_object(c, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.flush_events();
    t.settle();
    assert!(t.in_graveyard(P0, "Fiend Artisan"));
}
