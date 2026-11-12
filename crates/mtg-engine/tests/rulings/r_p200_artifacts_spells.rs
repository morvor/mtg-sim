//! Rulings batch P200 — artifacts and spells: Jandor's Ring's "last card drawn", Peter
//! Parker's Camera's copies (CR 707.10, 607.2), Urza's Armor's prevention (615), Throne of
//! Geth and Urza's Chalice, Library of Leng's discards (614, 701.9), Counterbore, Legion's
//! End, Maelstrom Pulse and Ancient Vendetta's searches by name (701.23, 608.2b),
//! Scalpelexis's repeated exile.

use crate::r_p194_common::mana;
use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use crate::r_s31_common::revealed_cards;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn exiled_named(t: &TestGame, name: &str) -> usize {
    t.g.exile
        .iter()
        .filter(|c| t.g.obj(**c).chars.name == name)
        .count()
}

fn in_zone_named(t: &TestGame, ids: &[ObjectId], name: &str) -> usize {
    ids.iter()
        .filter(|c| t.g.obj(**c).chars.name == name)
        .count()
}

// ---------------------------------------------------------------------------------------
// Jandor's Ring, Urza's Armor, Throne of Geth, Urza's Chalice
// ---------------------------------------------------------------------------------------

#[test]
fn jandors_ring_discards_the_last_of_several_cards_drawn() {
    cr!("121.2", "601.2h");
    ruling!(
        "Jandor's Ring",
        "If you draw more than one card due to a spell or ability, you must discard the last one of those drawn."
    );
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Jandor's Ring");
    stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant", "Craw Wurm"]);
    let div = t.hand(P0, "Divination");
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, div).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Hill Giant"));
    mana(&mut t, P0, ManaType::C, 2);
    // Whatever P0 answers, the cost is the Hill Giant (drawn last).
    let bears = t.g.find_in_zone(Zone::Hand(P0), "Grizzly Bears")[0];
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, ring, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Craw Wurm"));
}

#[test]
fn urzas_armor_is_a_prevention_effect() {
    cr!("615.1a", "615.12");
    ruling!(
        "Urza's Armor",
        "It is a prevention effect applied when damage would be dealt."
    );
    supported("Skullcrack");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urza's Armor");
    crate::r_p194_common::shock(&mut t, P1, Entity::Player(P0));
    assert_eq!(t.life(P0), 19);
    // Damage that can't be prevented: all of it is dealt.
    t.set_step(P1, Step::PrecombatMain);
    let sc = t.hand(P1, "Skullcrack");
    mana(&mut t, P1, ManaType::R, 1);
    mana(&mut t, P1, ManaType::C, 1);
    t.cast(P1, sc).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn throne_of_geth_can_sacrifice_itself() {
    cr!("602.2b", "701.34a");
    ruling!(
        "Throne of Geth",
        "You may sacrifice Throne of Geth to pay the cost of its own ability."
    );
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, "Throne of Geth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    t.answer_choose(P0, &[obj(throne)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, throne, 0, &[]).unwrap();
    assert_eq!(t.zone(throne), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn urzas_chalice_doesnt_see_its_own_casting() {
    cr!("603.2", "113.6");
    ruling!(
        "Urza's Chalice",
        "This will not trigger on its own casting. It must be on the battlefield at the time the artifact is cast."
    );
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Urza's Chalice");
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, c).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(c));
    // The next artifact spell: it triggers, and P0 pays {1} to gain 1.
    let o = t.hand(P0, "Ornithopter");
    t.cast(P0, o).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

// ---------------------------------------------------------------------------------------
// Peter Parker's Camera
// ---------------------------------------------------------------------------------------

fn camera(t: &mut TestGame) -> ObjectId {
    let camera = t.battlefield(P0, "Peter Parker's Camera");
    t.g.add_counters(obj(camera), "film", 3, None);
    camera
}

#[test]
fn camera_copies_the_ability_without_granting_one() {
    cr!("707.10", "707.10b");
    ruling!(
        "Peter Parker's Camera",
        "Peter Parker's Camera's ability targets an ability that is on the stack and creates another instance of that ability on the stack. It doesn't cause any object to gain an ability."
    );
    supported("Wall of Omens");
    let mut t = TestGame::new(2);
    let cam = camera(&mut t);
    let hand = t.hand_size(P0);
    let wall = t.enter(P0, "Wall of Omens");
    t.settle();
    let trigger = top_of_stack(&t);
    let abilities = t.obj_now(wall).chars.abilities.len();
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, cam, 0, &[obj(trigger)]).unwrap();
    t.resolve();
    // The copy is on the stack above the original; the Wall has no new ability.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.obj_now(wall).chars.abilities.len(), abilities);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.counters(cam, "film"), 2);
}

#[test]
fn camera_copy_of_a_linked_exile_gives_several_exiled_cards() {
    cr!("607.2a", "707.10", "607.3");
    ruling!(
        "Peter Parker's Camera",
        "In some cases involving linked abilities, an ability requires information about \"the exiled card.\" When this happens, the ability gets multiple answers. If these answers are being used to determine the value of a variable, the sum is used. If an ability attempts to create a token that is a copy of \"the exiled card,\" it creates a token for each card exiled this way that is a copy of that exiled card."
    );
    supported("Prototype Portal");
    // Prototype Portal: "Imprint — When this artifact enters, you may exile an artifact
    // card from your hand. / {X}, {T}: Create a token that's a copy of the exiled card. X
    // is the mana value of the exiled card."
    let mut t = TestGame::new(2);
    let cam = camera(&mut t);
    let stone = t.hand(P0, "Mind Stone");
    let thopter = t.hand(P0, "Ornithopter");
    let portal = t.enter(P0, "Prototype Portal");
    t.settle();
    let trigger = top_of_stack(&t);
    mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, cam, 0, &[obj(trigger)]).unwrap();
    // The copy resolves first: Ornithopter; then the original: Mind Stone.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(stone)]);
    t.resolve_all();
    assert_eq!(t.zone(stone), Zone::Exile);
    assert_eq!(t.zone(thopter), Zone::Exile);
    // X = 2 + 0: {1} isn't enough; {2} makes a copy of each.
    mana(&mut t, P0, ManaType::C, 1);
    assert!(activate_containing(&mut t, P0, portal, "Create a token").is_err());
    t.g.players[P0.idx()].mana_pool = Default::default();
    mana(&mut t, P0, ManaType::C, 2);
    activate_containing(&mut t, P0, portal, "Create a token").unwrap();
    t.resolve_all();
    let toks: Vec<String> = tokens(&t, P0)
        .into_iter()
        .map(|id| t.g.obj(id).chars.name.to_string())
        .collect();
    assert_eq!(toks.len(), 2, "{toks:?}");
    assert!(toks.contains(&"Mind Stone".to_string()));
    assert!(toks.contains(&"Ornithopter".to_string()));
}

// ---------------------------------------------------------------------------------------
// Library of Leng
// ---------------------------------------------------------------------------------------

#[test]
fn library_of_leng_applies_to_any_effect_discard_and_is_still_a_discard() {
    cr!("614.1a", "701.9a", "701.9c");
    ruling!(
        "Library of Leng",
        "The ability applies any time a spell or ability has you discard as part of its effect. It does not matter if you or your opponent control the spell or ability. The discard is forced because it is an effect."
    );
    ruling!(
        "Library of Leng",
        "The ability replaces the normal discard action with a discard action that puts the card on the library instead of the graveyard."
    );
    ruling!(
        "Library of Leng",
        "Since the card goes directly to the library, the card is not revealed unless the spell or ability requiring the discard specifically says it is."
    );
    supported("Careful Study");
    supported("Megrim");
    // P0's own Careful Study ("Draw two cards, then discard two cards."), with P1's
    // Megrim ("Whenever an opponent discards a card, Megrim deals 2 damage to that
    // player."): both cards go on top of the library, and each is still discarded.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Library of Leng");
    t.battlefield(P1, "Megrim");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    let study = t.hand(P0, "Careful Study");
    mana(&mut t, P0, ManaType::U, 1);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let from = t.g.turn_events.len();
    t.cast(P0, study).go();
    t.resolve_all();
    for c in [a, b] {
        assert!(matches!(t.zone(c), Zone::Library(_)), "{c:?}");
    }
    let discards = t.g.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, Event::Discarded { player, .. } if *player == P0))
        .count();
    assert_eq!(discards, 2);
    assert_eq!(t.life(P0), 16);
    assert!(revealed_cards(&t)
        .iter()
        .all(|c| t.g.current(*c) != t.g.current(a) && t.g.current(*c) != t.g.current(b)));
    // P1's Mind Rot: the same.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Library of Leng");
    let a = t.hand(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let rot = t.hand(P1, "Mind Rot");
    mana(&mut t, P1, ManaType::B, 1);
    mana(&mut t, P1, ManaType::C, 2);
    t.answer_yes(P0, true);
    t.cast(P1, rot).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(matches!(t.zone(a), Zone::Library(_)));
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(top, t.g.current(a));
}

#[test]
fn library_of_leng_doesnt_affect_milling() {
    cr!("701.17a", "701.9a");
    ruling!(
        "Library of Leng",
        "This effect has no effect on the cards being put into the graveyard from a library, because they are not \"discarded\"."
    );
    supported("Thought Scour");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Library of Leng");
    let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    let scour = t.hand(P0, "Thought Scour");
    mana(&mut t, P0, ManaType::U, 1);
    let from = t.asked().len();
    t.cast(P0, scour).target(Entity::Player(P0)).go();
    t.resolve_all();
    for c in cards {
        assert_eq!(t.zone(c), Zone::Graveyard(P0));
    }
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
}

#[test]
fn library_of_leng_random_discard_seen_before_deciding() {
    cr!("701.9b", "614.1a");
    ruling!(
        "Library of Leng",
        "You can look at a randomly discarded card before deciding where it goes."
    );
    supported("Hymn to Tourach");
    let mut t = TestGame::new(2);
    let leng = t.battlefield(P0, "Library of Leng");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let hymn = t.hand(P1, "Hymn to Tourach");
    mana(&mut t, P1, ManaType::B, 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    t.cast(P1, hymn).target(Entity::Player(P0)).go();
    t.resolve_all();
    // Each question names the card it's about.
    let prompts: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::YesNo { source, prompt } if *p == P0 && *source == Some(leng) => {
                Some(prompt.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(prompts.len(), 2, "{prompts:?}");
    let first_on_top = prompts[0].contains("Grizzly Bears");
    assert!(
        first_on_top || prompts[0].contains("Hill Giant"),
        "{prompts:?}"
    );
    let (top, gy) = if first_on_top { (a, b) } else { (b, a) };
    assert!(matches!(t.zone(top), Zone::Library(_)));
    assert_eq!(t.zone(gy), Zone::Graveyard(P0));
}

// ---------------------------------------------------------------------------------------
// Counterbore
// ---------------------------------------------------------------------------------------

/// P1 casts Counterbore targeting `spell`, choosing `chosen` among the cards found.
fn counterbore(t: &mut TestGame, spell: ObjectId, chosen: Option<Vec<Entity>>) {
    let cb = t.hand(P1, "Counterbore");
    mana(t, P1, ManaType::U, 2);
    mana(t, P1, ManaType::C, 3);
    if let Some(c) = chosen {
        t.answer_choose(P1, &c);
    }
    t.cast(P1, cb).target(spell).go();
    t.resolve_all();
}

#[test]
fn counterbore_exiles_the_spell_and_the_cards_with_its_name() {
    cr!("701.6a", "701.23a", "400.7");
    ruling!(
        "Counterbore",
        "In most cases, Counterbore will exile the targeted spell. Its first sentence counters the spell, which puts it into its owner’s graveyard. Its second sentence then removes it from the graveyard."
    );
    ruling!(
        "Counterbore",
        "Counterbore has no effect on cards on the battlefield that have the same name as the targeted spell."
    );
    let mut t = TestGame::new(2);
    let on_bf = t.battlefield(P0, "Grizzly Bears");
    let gy = t.graveyard(P0, "Grizzly Bears");
    let lib = t.library_top(P0, "Grizzly Bears");
    let hand = t.hand(P0, "Grizzly Bears");
    let spell_card = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, spell_card).go();
    counterbore(&mut t, spell, None);
    assert_eq!(t.zone(spell_card), Zone::Exile);
    for c in [gy, lib, hand] {
        assert_eq!(t.zone(c), Zone::Exile, "{c:?}");
    }
    assert!(t.on_battlefield(on_bf));
    assert_eq!(exiled_named(&t, "Grizzly Bears"), 4);
}

#[test]
fn counterbore_on_a_stolen_spell_searches_its_controller() {
    cr!("701.6a", "608.2c");
    ruling!(
        "Counterbore",
        "If the spell’s controller isn’t the same as its owner (due to Commandeer, for example), it won’t be exiled. The spell is countered and put into its owner’s graveyard. Then the spell’s controller’s graveyard, hand, and library are searched."
    );
    supported("Commandeer");
    let mut t = TestGame::new(3);
    // P2 casts Divination; P0 Commandeers it; P1 Counterbores it.
    t.set_step(P2, Step::PrecombatMain);
    let div = t.hand(P2, "Divination");
    let p2_gy = t.graveyard(P2, "Divination");
    mana(&mut t, P2, ManaType::U, 1);
    mana(&mut t, P2, ManaType::C, 2);
    let spell = t.cast(P2, div).go();
    let p0_gy = t.graveyard(P0, "Divination");
    let p0_hand = t.hand(P0, "Divination");
    let cmd = t.hand(P0, "Commandeer");
    mana(&mut t, P0, ManaType::U, 2);
    mana(&mut t, P0, ManaType::C, 5);
    t.answer_yes(P0, false);
    t.g.turn.priority = Some(P0);
    t.cast(P0, cmd).target(spell).go();
    t.resolve();
    assert_eq!(t.g.obj(spell).controller, P0);
    counterbore(&mut t, spell, None);
    // The countered spell is in P2's graveyard; P0's Divinations are exiled; P2's other
    // Divination stays.
    assert_eq!(t.zone(div), Zone::Graveyard(P2));
    assert_eq!(t.zone(p2_gy), Zone::Graveyard(P2));
    assert_eq!(t.zone(p0_gy), Zone::Exile);
    assert_eq!(t.zone(p0_hand), Zone::Exile);
}

#[test]
fn counterbore_on_an_uncounterable_spell_still_searches() {
    cr!("113.6g", "608.2c");
    ruling!(
        "Counterbore",
        "If the targeted spell can’t be countered (it’s Vexing Shusher, for example), that spell will remain on the stack. Counterbore will continue to resolve. You still get to search for and exile all other cards with that name."
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, "Vexing Shusher");
    let hand = t.hand(P0, "Vexing Shusher");
    let shusher = t.hand(P0, "Vexing Shusher");
    mana(&mut t, P0, ManaType::R, 2);
    let spell = t.cast(P0, shusher).go();
    counterbore(&mut t, spell, None);
    assert!(t.on_battlefield(shusher));
    assert_eq!(t.zone(gy), Zone::Exile);
    assert_eq!(t.zone(hand), Zone::Exile);
}

#[test]
fn counterbore_must_find_graveyard_cards_but_may_leave_hidden_ones() {
    cr!("701.23b", "400.2");
    ruling!(
        "Counterbore",
        "The cards you’re searching for must be found if they’re in the graveyard, because that’s a zone everyone can see. Finding those cards in the hand and library is optional, though."
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, "Grizzly Bears");
    let lib = t.library_top(P0, "Grizzly Bears");
    let hand = t.hand(P0, "Grizzly Bears");
    let spell_card = t.hand(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let spell = t.cast(P0, spell_card).go();
    // P1 chooses to find none of them.
    let from = t.asked().len();
    counterbore(&mut t, spell, Some(vec![]));
    assert_eq!(t.zone(gy), Zone::Exile);
    assert_eq!(t.zone(spell_card), Zone::Exile);
    assert_eq!(t.zone(hand), Zone::Hand(P0));
    assert!(matches!(t.zone(lib), Zone::Library(_)));
    // The choice offered was optional.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { min: 0, .. })));
}

// ---------------------------------------------------------------------------------------
// Legion's End, Maelstrom Pulse
// ---------------------------------------------------------------------------------------

/// P1's Grizzly Bears: the target, another one given hexproof by Ranger's Guile, one in
/// hand and one in the graveyard; P1 has Leyline of Sanctity.
fn bears_with_hexproof(t: &mut TestGame) -> [ObjectId; 4] {
    supported("Ranger's Guile");
    supported("Leyline of Sanctity");
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Leyline of Sanctity");
    let hand = t.hand(P1, "Grizzly Bears");
    let gy = t.graveyard(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let guile = t.hand(P1, "Ranger's Guile");
    mana(t, P1, ManaType::G, 1);
    t.cast(P1, guile).target(other).go();
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    [target, other, hand, gy]
}

#[test]
fn legions_end_exiles_hexproof_namesakes() {
    cr!("115.1", "702.11b", "608.2c");
    ruling!(
        "Legion's End",
        "Legion's End targets only one creature. Other creatures will be exiled even if they have hexproof, and cards in its controller's hand and graveyard will be exiled even if that player has hexproof."
    );
    let mut t = TestGame::new(2);
    let ids = bears_with_hexproof(&mut t);
    let le = t.hand(P0, "Legion's End");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, le).target(ids[0]).go();
    t.resolve_all();
    for id in ids {
        assert_eq!(t.zone(id), Zone::Exile, "{id:?}");
    }
}

#[test]
fn legions_end_does_nothing_if_its_target_is_gone() {
    cr!("608.2b");
    ruling!(
        "Legion's End",
        "If the target creature is an illegal target by the time Legion's End tries to resolve, the spell doesn't resolve. No player reveals their hand, and nothing is exiled."
    );
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand(P1, "Grizzly Bears");
    let le = t.hand(P0, "Legion's End");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, le).target(target).go();
    move_to(&mut t, target, Zone::Hand(P1));
    let before = revealed_cards(&t).len();
    t.resolve_all();
    assert!(t.on_battlefield(other));
    assert_eq!(t.zone(hand), Zone::Hand(P1));
    assert_eq!(exiled_named(&t, "Grizzly Bears"), 0);
    assert_eq!(revealed_cards(&t).len(), before);
}

#[test]
fn maelstrom_pulse_destroys_hexproof_and_protected_namesakes() {
    cr!("115.1", "702.11b", "702.16b", "608.2c");
    ruling!(
        "Maelstrom Pulse",
        "Maelstrom Pulse has only one target. Other permanents with the same name will be destroyed even if they have hexproof or protection."
    );
    supported("Gods Willing");
    let mut t = TestGame::new(2);
    let ids = bears_with_hexproof(&mut t);
    // A third Bears with protection from black (Gods Willing, choosing black).
    let prot = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let gw = t.hand(P1, "Gods Willing");
    mana(&mut t, P1, ManaType::W, 1);
    t.answer(P1, DecisionKind::Option, Answer::Index(2));
    t.cast(P1, gw).target(prot).go();
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    assert!(spell_targets(&mut t, P0, "Maelstrom Pulse")
        .iter()
        .all(|e| *e != obj(prot) && *e != obj(ids[1])));
    let pulse = t.hand(P0, "Maelstrom Pulse");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, pulse).target(ids[0]).go();
    t.resolve_all();
    for id in [ids[0], ids[1], prot] {
        assert_eq!(t.zone(id), Zone::Graveyard(P1), "{id:?}");
    }
}

#[test]
fn maelstrom_pulse_needs_a_legal_target_but_not_a_destroyed_one() {
    cr!("608.2b", "702.12b", "608.2c");
    ruling!(
        "Maelstrom Pulse",
        "If the target permanent is an illegal target by the time Maelstrom Pulse tries to resolve, the spell won't resolve. You won't destroy any permanents at all. If the target is legal but not destroyed (most likely because it has indestructible), you do destroy other permanents with the same name."
    );
    supported("Darksteel Relic");
    // Gone: nothing destroyed.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let pulse = t.hand(P0, "Maelstrom Pulse");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, pulse).target(target).go();
    move_to(&mut t, target, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.on_battlefield(other));
    // Indestructible target (Darksteel Plate): the other one is destroyed.
    supported("Darksteel Plate");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let plate = t.battlefield(P1, "Darksteel Plate");
    assert!(t.g.attach(plate, obj(target)));
    t.g.recompute();
    let pulse = t.hand(P0, "Maelstrom Pulse");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, pulse).target(target).go();
    t.resolve_all();
    assert!(t.on_battlefield(target));
    assert_eq!(t.zone(other), Zone::Graveyard(P1));
}

// ---------------------------------------------------------------------------------------
// Ancient Vendetta
// ---------------------------------------------------------------------------------------

/// P0 casts Ancient Vendetta targeting P1, naming `name` and choosing `chosen` (all found
/// cards if `None`) as it resolves. The decisions asked are logged from `from`.
fn vendetta(t: &mut TestGame, name: &str, chosen: Option<Vec<Entity>>) -> usize {
    let av = t.hand(P0, "Ancient Vendetta");
    mana(t, P0, ManaType::B, 1);
    mana(t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::Name, Answer::Text(name.into()));
    if let Some(c) = chosen {
        t.answer_choose(P0, &c);
    }
    let from = t.asked().len();
    t.cast(P0, av).target(Entity::Player(P1)).go();
    from
}

#[test]
fn ancient_vendetta_names_on_resolution_any_name_up_to_four() {
    cr!("608.2c", "201.4", "701.23b");
    ruling!(
        "Ancient Vendetta",
        "You target a player as this spell is put on the stack, but you don’t name a card until it resolves. Notably, this means that players don’t get to respond to the spell knowing what card you will name."
    );
    ruling!(
        "Ancient Vendetta",
        "Unlike similar cards, Ancient Vendetta allows you to choose the name of a basic land. Note that you may only search for up to four cards with the chosen name, even if you find more of them."
    );
    ruling!(
        "Ancient Vendetta",
        "You don’t have to choose the name of a card that you’ve already seen this game; you may guess at a card you think your opponent might have in their hand or library, if you’d like."
    );
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P1, "Island");
    }
    t.hand(P1, "Island");
    t.graveyard(P1, "Island");
    let from = vendetta(&mut t, "Island", None);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::NameCard { .. })));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::NameCard { .. })));
    assert_eq!(exiled_named(&t, "Island"), 4);
    // A name P0 hasn't seen (and nobody has): nothing found, the library shuffled.
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P1, "Grizzly Bears");
    vendetta(&mut t, "Black Lotus", None);
    t.resolve_all();
    assert_eq!(t.zone(gy), Zone::Graveyard(P1));
    assert!(t.g.exile.is_empty());
}

#[test]
fn ancient_vendetta_may_leave_found_cards() {
    // "Up to four": which of the cards found are exiled is chosen as the spell resolves,
    // even among those in the (public) graveyard.
    cr!("608.2d");
    ruling!(
        "Ancient Vendetta",
        "You don’t have to exile each card you find with that name. If you want to leave one in their graveyard, for example, you may."
    );
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Grizzly Bears");
    vendetta(&mut t, "Grizzly Bears", Some(vec![obj(a)]));
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Graveyard(P1));
}

// ---------------------------------------------------------------------------------------
// Scalpelexis
// ---------------------------------------------------------------------------------------

#[test]
fn scalpelexis_repeats_until_no_pair() {
    cr!("608.2c", "603.2");
    ruling!(
        "Scalpelexis",
        "The “repeat this process” means to repeat as often as it takes to not get a matching pair."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Scalpelexis");
    // P1's library from the top: four cards with a pair, four with a pair, four with no
    // pair, then two Forests that are never reached.
    let mut top_first = vec!["Island", "Island", "Forest", "Swamp"];
    top_first.extend(["Mountain", "Plains", "Mountain", "Swamp"]);
    top_first.extend(["Grizzly Bears", "Hill Giant", "Craw Wurm", "Shock"]);
    top_first.extend(["Forest", "Forest"]);
    stack_library(&mut t, P1, &top_first);
    let lib = t.library_size(P1);
    t.attack(&[(s, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib - 12);
    assert_eq!(t.g.exile.len(), 12);
    let ex = t.g.exile.clone();
    assert_eq!(in_zone_named(&t, &ex, "Craw Wurm"), 1);
    assert_eq!(in_zone_named(&t, &ex, "Forest"), 1);
}
