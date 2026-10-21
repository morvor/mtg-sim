//! Rulings batch P104 — milling, then returning or casting cards from the graveyard:
//! untargeted choices may pick a milled card (Corpse Churn, Grapple with the Past, Tayam),
//! required returns (Sudden Reclamation, Roots of Wisdom), how returned creatures look
//! (Cauldron's Gift, Admiral Brass), {X} in a graveyard (The War in Heaven, Technomancer),
//! Liliana's -7, Out of the Tombs, Emry, Egon, and Gisa and Geralf.

use crate::r_p104_common::*;
use crate::r_p146_common::counter;
use crate::r_p177_common::set_loyalty;
use crate::r_p189_common::{bounce, set_library};
use crate::r_s01_common::*;
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s05_common::enter;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s19_common::add_lore;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn untargeted_returns_may_choose_a_card_just_milled() {
    cr!("608.2c", "115.1");
    ruling!(
        "Corpse Churn",
        "Because Corpse Churn doesn't target the creature card in your graveyard, you may choose one of the three cards you put there from your library if applicable."
    );
    ruling!(
        "Grapple with the Past",
        "Grapple with the Past doesn't target the card to return to your hand. After putting the top three cards of your library into your graveyard, you may choose from among any creature or land cards there."
    );
    for (spell, lands, milled) in [
        ("Corpse Churn", "Swamp", "Grizzly Bears"),
        ("Grapple with the Past", "Forest", "Island"),
    ] {
        supported(spell);
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Hill Giant");
        let lib = stack_library(&mut t, P0, &[milled, "Lightning Bolt", "Shock"]);
        t.lands(P0, lands, 2);
        let s = t.hand(P0, spell);
        t.cast(P0, s).go();
        t.answer_yes(P0, true);
        crate::r_s22_common::choose_named_when_offered(&mut t, P0, milled);
        t.resolve_all();
        assert_eq!(t.zone(lib[0]), Zone::Hand(P0), "{spell}");
        assert!(t.in_graveyard(P0, "Hill Giant"), "{spell}");
    }
}

#[test]
fn tayam_may_return_a_card_it_just_milled() {
    cr!("608.2c", "115.1");
    ruling!(
        "Tayam, Luminous Enigma",
        "Because the activated ability doesn’t target the permanent card in your graveyard, you may choose one of the three cards you put there from your library."
    );
    supported("Tayam, Luminous Enigma");
    let mut t = TestGame::new(2);
    let tayam = t.battlefield(P0, "Tayam, Luminous Enigma");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(obj(giant), "+1/+1", 3, None);
    t.graveyard(P0, "Llanowar Elves");
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Grizzly Bears", "Shock"]);
    t.lands(P0, "Wastes", 3);
    t.activate(P0, tayam, 0, &[]).unwrap();
    crate::r_s22_common::choose_named_when_offered(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert!(t.on_battlefield(lib[1]));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

#[test]
fn sudden_reclamation_returns_a_creature_even_without_a_land() {
    cr!("608.2c");
    ruling!(
        "Sudden Reclamation",
        "If there are no land cards in your graveyard, you still choose a creature card to return, and vice versa."
    );
    supported("Sudden Reclamation");
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Lightning Bolt", "Shock", "Opt", "Grizzly Bears"]);
    t.lands(P0, "Forest", 4);
    let s = t.hand(P0, "Sudden Reclamation");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // And a land without a creature.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Lightning Bolt", "Shock", "Opt", "Island"]);
    t.lands(P0, "Forest", 4);
    let s = t.hand(P0, "Sudden Reclamation");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
}

#[test]
fn roots_of_wisdom_must_return_a_card_if_it_can() {
    cr!("608.2c");
    ruling!(
        "Roots of Wisdom",
        "If there’s a land card or Elf card in your graveyard, you must return one. You can’t choose not to in order to draw a card."
    );
    supported("Roots of Wisdom");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Roots of Wisdom");
    let hand = t.hand_size(P0);
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[]);
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.hand_size(P0), hand);
    // Only the three milled cards left the library: no draw.
    assert_eq!(t.library_size(P0), lib - 3);
}

#[test]
fn cauldrons_gift_gives_a_counter_to_a_creature_without_any() {
    cr!("614.1c", "122.6");
    ruling!(
        "Cauldron's Gift",
        "If the creature would enter the battlefield with no +1/+1 counters on it, it enters with one +1/+1 counter on it."
    );
    supported("Cauldron's Gift");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 4);
    let s = t.hand(P0, "Cauldron's Gift");
    t.cast(P0, s).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn admiral_brass_base_pt_overwrites_earlier_setters_only() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Admiral Brass, Unsinkable",
        "Admiral Brass, Unsinkable's effect will overwrite any previous effects that set the returned creature's power and toughness to specific numbers. Effects that otherwise modify the returned creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Admiral Brass, Unsinkable");
    supported("Godhead of Awe");
    let mut t = TestGame::new(2);
    // An earlier "base 1/1" effect and an anthem.
    t.battlefield(P0, "Godhead of Awe");
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Admiral Brass, Unsinkable");
    let pirate = t.graveyard(P0, "Fathom Fleet Captain");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(pirate)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(pirate));
    assert_eq!(t.counters(pirate, "finality"), 1);
    assert_eq!(t.pt(pirate), (5, 5));
    t.g.add_counters(obj(t.g.current(pirate)), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(pirate), (6, 6));
}

/// The cards were returned to the battlefield: Hangarback Walker (first) entered with no
/// counters and died as a 0/0, so it's a new object in the graveyard.
fn assert_returned(t: &TestGame, cards: &[ObjectId]) {
    assert_ne!(t.g.current(cards[0]), cards[0]);
    assert!(t.in_graveyard(P0, "Hangarback Walker"));
    for c in &cards[1..] {
        assert!(t.on_battlefield(*c));
    }
}

#[test]
fn x_is_zero_for_cards_in_a_graveyard() {
    cr!("202.3e");
    ruling!(
        "The War in Heaven",
        "If a card in your graveyard has an {X} in its cost, X is zero."
    );
    ruling!(
        "Technomancer",
        "If a card in your graveyard has an {X} in its mana cost, X is zero."
    );
    supported("The War in Heaven");
    supported("Technomancer");
    supported("Hangarback Walker");
    // The War in Heaven: Hangarback Walker (0) and two Hill Giants (4 each) total 8.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The War in Heaven");
    t.g.objects[saga.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::LORE.into(), 2);
    let cards = [
        t.graveyard(P0, "Hangarback Walker"),
        t.graveyard(P0, "Hill Giant"),
        t.graveyard(P0, "Hill Giant"),
    ];
    t.answer_targets(P0, &cards.map(obj));
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    assert_returned(&t, &cards);
    // Technomancer: Hangarback Walker (0) and Wurmcoil Engine (6) total 6.
    let mut t = TestGame::new(2);
    let cards = [
        t.graveyard(P0, "Hangarback Walker"),
        t.graveyard(P0, "Wurmcoil Engine"),
    ];
    t.answer_choose(P0, &cards.map(obj));
    enter(&mut t, P0, "Technomancer");
    t.resolve_all();
    assert_returned(&t, &cards);
}

#[test]
fn liliana_ultimate_spares_zombies_with_other_types() {
    cr!("205.3m");
    ruling!(
        "Liliana, Death's Majesty",
        "A creature that is a Zombie and has other types, such as a Zombie Jackal, isn't a non-Zombie creature."
    );
    supported("Liliana, Death's Majesty");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana, Death's Majesty");
    set_loyalty(&mut t, lili, 7);
    let skaab = t.battlefield(P1, "Armored Skaab");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, lili, 2, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(skaab));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn out_of_the_tombs_replaces_each_draw_separately() {
    cr!("121.2", "614.11");
    ruling!(
        "Out of the Tombs",
        "If an effect instructs a player to draw multiple cards, each of those cards are drawn independently, and Out of the Tombs's second ability will apply to each one if appropriate."
    );
    supported("Out of the Tombs");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Out of the Tombs");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    set_library(&mut t, P0, &[]);
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.settle();
    assert!(t.on_battlefield(a));
    assert!(t.on_battlefield(b));
    assert!(!t.has_lost(P0));
}

/// Emry, Lurker of the Loch on P0's battlefield targets `card` (in P0's graveyard) with
/// its ability, which resolves.
fn emry_targets(t: &mut TestGame, card: ObjectId) {
    supported("Emry, Lurker of the Loch");
    let emry = t.battlefield(P0, "Emry, Lurker of the Loch");
    t.activate(P0, emry, 0, &[obj(card)]).unwrap();
    t.resolve_all();
}

#[test]
fn emry_permission_ends_when_the_card_becomes_a_new_object() {
    cr!("400.7", "601.2a");
    ruling!(
        "Emry, Lurker of the Loch",
        "Casting the target card causes it to leave your graveyard and become a new object. You can't cast it again if it returns to your graveyard this turn."
    );
    let mut t = TestGame::new(2);
    let thopter = t.graveyard(P0, "Ornithopter");
    emry_targets(&mut t, thopter);
    let methods = legal_cast_methods(&mut t, P0, thopter);
    assert!(!methods.is_empty());
    let spell = t.cast(P0, thopter).method(methods[0].clone()).go();
    counter(&mut t, spell);
    let again = t.g.find_in_zone(Zone::Graveyard(P0), "Ornithopter")[0];
    assert!(legal_cast_methods(&mut t, P0, again).is_empty());
}

#[test]
fn emry_cant_let_you_play_an_artifact_land() {
    cr!("305.9", "601.1a");
    ruling!(
        "Emry, Lurker of the Loch",
        "If Emry's last ability targets an artifact land card, you can't play it. Effects that allow you to \"cast\" a card don't allow you to play a land card."
    );
    let mut t = TestGame::new(2);
    let seat = t.graveyard(P0, "Seat of the Synod");
    emry_targets(&mut t, seat);
    assert!(!can_play_land(&mut t, P0, seat));
    assert!(legal_cast_methods(&mut t, P0, seat).is_empty());
}

#[test]
fn egon_trigger_still_resolves_after_egon_leaves() {
    cr!("113.7a");
    ruling!(
        "Egon, God of Death // Throne of Death",
        "If Egon leaves the battlefield before its triggered ability resolves, you'll still exile two cards from your graveyard if you can and draw a card if you can't."
    );
    supported("Egon, God of Death // Throne of Death");
    // No cards to exile: draw a card.
    let mut t = TestGame::new(2);
    let egon = t.battlefield(P0, "Egon, God of Death // Throne of Death");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    bounce(&mut t, egon);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Two cards to exile: exile them, no draw.
    let mut t = TestGame::new(2);
    let egon = t.battlefield(P0, "Egon, God of Death // Throne of Death");
    let cards = [t.graveyard(P0, "Island"), t.graveyard(P0, "Forest")];
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    bounce(&mut t, egon);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    for c in cards {
        assert_eq!(t.zone(c), Zone::Exile);
    }
}

#[test]
fn throne_of_death_exiles_its_cost_before_anyone_can_respond() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Egon, God of Death // Throne of Death",
        "Although players may respond to Throne of Death's activated ability, they can't respond to the paying of its activation cost. The creature card will already be exiled by the time anyone can respond."
    );
    let mut t = TestGame::new(2);
    let throne = crate::r_p079_common::back_face(&mut t, P0, "Egon, God of Death // Throne of Death");
    assert_eq!(t.obj_now(throne).chars.name, "Throne of Death");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.activate(P0, throne, 0, &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn gisa_and_geralf_zombie_milled_can_be_cast_first() {
    cr!("117.3b");
    ruling!(
        "Gisa and Geralf",
        "If a Zombie card is put into your graveyard during your main phase and the stack is empty, you have a chance to cast it before any player may attempt to remove that card from your graveyard."
    );
    supported("Gisa and Geralf");
    let mut t = TestGame::new(2);
    let z = t.library_top(P0, "Walking Corpse");
    enter(&mut t, P0, "Gisa and Geralf");
    t.resolve_all();
    assert_eq!(t.zone(z), Zone::Graveyard(P0));
    // P0 (the active player) gets priority first after the resolution and may cast it.
    assert_eq!(t.g.turn.priority, Some(P0));
    t.lands(P0, "Swamp", 2);
    assert!(!legal_cast_methods(&mut t, P0, z).is_empty());
}

#[test]
fn a_new_gisa_and_geralf_allows_another_zombie() {
    cr!("400.7");
    ruling!(
        "Gisa and Geralf",
        "If you cast one Zombie creature spell from your graveyard and then have a new Gisa and Geralf come under your control in the same turn, you may cast another Zombie creature spell from your graveyard that turn."
    );
    let mut t = TestGame::new(2);
    let gisa = t.battlefield(P0, "Gisa and Geralf");
    let a = t.graveyard(P0, "Walking Corpse");
    let b = t.graveyard(P0, "Walking Corpse");
    t.lands(P0, "Swamp", 4);
    let m = legal_cast_methods(&mut t, P0, a);
    t.cast(P0, a).method(m[0].clone()).go();
    t.resolve_all();
    assert!(legal_cast_methods(&mut t, P0, b).is_empty());
    destroy(&mut t, gisa);
    t.battlefield(P0, "Gisa and Geralf");
    assert!(!legal_cast_methods(&mut t, P0, b).is_empty());
}

#[test]
fn gisa_and_karador_permissions_are_chosen_as_you_cast() {
    cr!("601.2a", "601.3");
    ruling!(
        "Gisa and Geralf",
        "If multiple effects allow you to cast a Zombie creature spell from your graveyard, such as those of Gisa and Geralf and Karador, Ghost Chieftain, you must announce which permission you’re using as you begin to cast the spell."
    );
    supported("Karador, Ghost Chieftain");
    supported("Walking Corpse");
    let mut t = TestGame::new(2);
    let gisa = t.battlefield(P0, "Gisa and Geralf");
    let karador = t.battlefield(P0, "Karador, Ghost Chieftain");
    let a = t.graveyard(P0, "Walking Corpse");
    let b = t.graveyard(P0, "Walking Corpse");
    t.lands(P0, "Swamp", 4);
    // P0 announces Karador's permission for the first Zombie...
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(karador)]);
    let m = legal_cast_methods(&mut t, P0, a);
    t.cast(P0, a).method(m[0].clone()).go();
    let asked: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("permission") => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(asked.len(), 1);
    assert!(asked[0].contains(&obj(gisa)) && asked[0].contains(&obj(karador)));
    t.resolve_all();
    // ... so Gisa and Geralf's is still there for the second.
    let m = legal_cast_methods(&mut t, P0, b);
    assert!(!m.is_empty());
    t.cast(P0, b).method(m[0].clone()).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Walking Corpse").len(), 2);
}
