//! Rulings batch P219 — mill (CR 701.17) under replacement and prevention effects: Bruvac
//! the Grandiloquent, The Water Crystal, Angel of Suffering, The Mindskinner, and what
//! "milled this way" means when a replacement changes the milled cards.

use crate::r_p210_common::deal;
use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s05_common::enter;
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::empty_library;
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` mills `n` cards as a resolving effect controlled by `controller` would.
fn mill(t: &mut TestGame, controller: PlayerId, p: PlayerId, n: i32) {
    crate::r_s05_common::run_from(
        t,
        controller,
        None,
        Effect::Mill {
            who: PlayerRef::Target(0),
            n: Value::Const(n),
        },
        &[Entity::Player(p)],
    );
}

/// P0 activates Persistent Petitioners' "{1}, {T}: Target player mills a card." at `p`.
fn petition(t: &mut TestGame, p: PlayerId) {
    let pet = t.battlefield(P0, "Persistent Petitioners");
    t.lands(P0, "Island", 1);
    t.answer_targets(P0, &[Entity::Player(p)]);
    activate_containing(t, P0, pet, "mills a card").expect("activated");
    t.resolve_all();
}

#[test]
fn two_or_three_bruvacs_double_the_milling_again() {
    cr!("701.17a", "614.5", "616.1");
    ruling!(
        "Bruvac the Grandiloquent",
        "Because Bruvac is legendary, it’s unlikely that one player will control two. However, if that happens, each opponent mills four times as many cards as originally instructed. If they control three, their opponents mill eight times as many, and so on."
    );
    supported("Bruvac the Grandiloquent");
    supported("Persistent Petitioners");
    supported("Mirror Gallery");
    for (bruvacs, milled) in [(1, 2), (2, 4), (3, 8)] {
        let mut t = TestGame::new(2);
        // Mirror Gallery: "The "legend rule" doesn't apply."
        t.battlefield(P0, "Mirror Gallery");
        for _ in 0..bruvacs {
            t.battlefield(P0, "Bruvac the Grandiloquent");
        }
        petition(&mut t, P1);
        assert_eq!(t.named_on_battlefield("Bruvac the Grandiloquent").len(), bruvacs);
        assert_eq!(t.graveyard_size(P1), milled, "{bruvacs} Bruvacs");
    }
}

#[test]
fn two_water_crystals_add_eight() {
    cr!("701.17a", "614.5", "616.1");
    ruling!(
        "The Water Crystal",
        "In the rare case where you control two of The Water Crystal, each opponent mills eight more cards than originally instructed. If you control three, your opponents mill twelve more, and so on."
    );
    supported("The Water Crystal");
    for (crystals, milled) in [(1, 5), (2, 9), (3, 13)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mirror Gallery");
        for _ in 0..crystals {
            t.battlefield(P0, "The Water Crystal");
        }
        petition(&mut t, P1);
        assert_eq!(t.graveyard_size(P1), milled, "{crystals} Crystals");
    }
}

#[test]
fn putting_cards_into_a_graveyard_without_milling_isnt_affected() {
    cr!("701.17a", "701.25a", "614.1a");
    ruling!(
        "Bruvac the Grandiloquent",
        "If a player is instructed to put a card into their graveyard without using the word “mill,” Bruvac’s ability doesn’t apply."
    );
    ruling!(
        "The Water Crystal",
        "If an opponent is instructed to put a card into their graveyard without using the word \"mill,\" The Water Crystal's second ability doesn't apply."
    );
    supported("Consider");
    // P1 casts Consider ("Surveil 1. Draw a card.") and puts the surveilled card into
    // their graveyard: just that card, with either permanent around.
    for watcher in ["Bruvac the Grandiloquent", "The Water Crystal"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, watcher);
        let top = t.library_top(P1, "Hill Giant");
        let consider = in_hand_with_mana(&mut t, P1, "Consider");
        t.answer(
            P1,
            DecisionKind::Surveil,
            mtg_engine::decision::Answer::Split(vec![], vec![top]),
        );
        t.cast(P1, consider).go();
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Hill Giant"), "{watcher}");
        // The Hill Giant and Consider itself.
        assert_eq!(t.graveyard_size(P1), 2, "{watcher}");
        // A mill instruction is replaced: two cards (Bruvac), five (The Water Crystal).
        mill(&mut t, P0, P1, 1);
        let milled = if watcher == "The Water Crystal" { 5 } else { 2 };
        assert_eq!(t.graveyard_size(P1), 2 + milled, "{watcher}");
    }
}

#[test]
fn angel_of_suffering_prevents_even_with_a_short_library() {
    cr!("615.1a", "701.17b", "614.1a");
    ruling!(
        "Angel of Suffering",
        "Damage that would be dealt to you will be prevented even if you can't mill twice that many cards."
    );
    ruling!(
        "Angel of Suffering",
        "If you would mill more cards than are in your library, you mill all cards in your library."
    );
    supported("Angel of Suffering");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angel of Suffering");
    let bolt = t.battlefield(P1, "Prodigal Sorcerer");
    empty_library(&mut t, P0);
    stack_library(&mut t, P0, &["Forest", "Forest", "Forest"]);
    // 3 damage: mill six, but only three are there.
    deal(&mut t, bolt, 3, P0);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
    assert!(!t.has_lost(P0));
    // More damage with an empty library is still prevented.
    deal(&mut t, bolt, 2, P0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn angel_of_suffering_still_mills_if_the_damage_cant_be_prevented() {
    cr!("615.12", "614.1a", "616.1");
    ruling!(
        "Angel of Suffering",
        "If the damage can't be prevented for some reason, you'll still mill twice that many cards."
    );
    supported("Skullcrack");
    // Skullcrack: "Players can't gain life this turn. Damage can't be prevented this turn.
    // Skullcrack deals 3 damage to target player or planeswalker."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angel of Suffering");
    let skullcrack = in_hand_with_mana(&mut t, P1, "Skullcrack");
    t.cast(P1, skullcrack).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.graveyard_size(P0), 6);
}

#[test]
fn two_mindskinners_replace_the_damage_only_once() {
    cr!("615.1a", "616.1");
    ruling!(
        "The Mindskinner",
        "If you somehow control more than one of The Mindskinner, the multiple replacement effects will have no effect. Damage dealt to an opponent by a source you control will be replaced only once with that opponent milling that many cards."
    );
    supported("The Mindskinner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, "The Mindskinner");
    t.battlefield(P0, "The Mindskinner");
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    deal(&mut t, sorcerer, 3, P1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.graveyard_size(P1), 3);
}

#[test]
fn bruvac_lets_you_play_each_card_milled_by_ark_or_tablet() {
    cr!("701.17d", "614.1a");
    ruling!(
        "Ark of Hunger",
        "If more than one card is milled by Ark of Hunger's last ability (because of the effect of Bruvac the Grandiloquent's ability, for example), you may play any of the milled cards that turn."
    );
    ruling!(
        "Tablet of Discovery",
        "If more than one card is milled by Tablet of Discovery's first ability (because of the effect of Bruvac the Grandiloquent's ability, for example), you may play any of the milled cards that turn."
    );
    supported("Ark of Hunger");
    supported("Tablet of Discovery");
    for source in ["Ark of Hunger", "Tablet of Discovery"] {
        let mut t = TestGame::new(2);
        // P1's Bruvac: P0 is its controller's opponent.
        t.battlefield(P1, "Bruvac the Grandiloquent");
        let cards = stack_library(&mut t, P0, &["Grizzly Bears", "Shock"]);
        if source == "Ark of Hunger" {
            let ark = t.battlefield(P0, source);
            activate_containing(&mut t, P0, ark, "Mill a card").expect("activated");
        } else {
            enter(&mut t, P0, source);
        }
        t.resolve_all();
        let cards: Vec<ObjectId> = cards.iter().map(|c| t.g.current(*c)).collect();
        assert!(cards.iter().all(|c| t.zone(*c) == Zone::Graveyard(P0)), "{source}");
        t.lands(P0, "Forest", 2);
        t.lands(P0, "Mountain", 1);
        assert!(can_cast(&mut t, P0, cards[0], CastMethod::Normal), "{source}");
        assert!(can_cast(&mut t, P0, cards[1], CastMethod::Normal), "{source}");
        t.cast(P0, cards[1]).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert!(can_cast(&mut t, P0, cards[0], CastMethod::Normal), "{source}");
    }
}

/// Chooses the Hill Giant among an entity choice's candidates.
pub fn choose_hill_giant(
    g: &mtg_engine::game::Game,
    d: &mtg_engine::decision::Decision,
) -> Option<mtg_engine::decision::Answer> {
    let mtg_engine::decision::Decision::ChooseEntities { candidates, .. } = d else {
        return None;
    };
    let giant = candidates
        .iter()
        .find(|e| e.object().is_some_and(|o| g.obj(o).chars.name == "Hill Giant"))?;
    Some(mtg_engine::decision::Answer::Entities(vec![*giant]))
}

#[test]
fn gigamole_can_take_a_creature_card_milled_into_exile() {
    cr!("701.17c", "614.1a");
    ruling!(
        "Ravenous Gigamole",
        "If a replacement effect causes the milled cards to be exiled rather than go to the graveyard, this effect can still apply to the cards \"milled this way\" in exile."
    );
    supported("Ravenous Gigamole");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let cards = stack_library(&mut t, P0, &["Forest", "Hill Giant", "Shock"]);
    crate::r_s03_common::respond(&mut t, P0, choose_hill_giant);
    let mole = enter(&mut t, P0, "Ravenous Gigamole");
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.zone(cards[0]), Zone::Exile);
    assert_eq!(t.counters(mole, "+1/+1"), 0);
}

#[test]
fn not_taking_a_card_puts_a_counter_even_with_nothing_to_take() {
    cr!("608.2c", "603.12");
    ruling!(
        "Ravenous Gigamole",
        "You put a +1/+1 counter on Ravenous Gigamole if you don't put a creature card into your hand with its ability, even if there wasn't a creature card among the milled cards."
    );
    ruling!(
        "Tomakul Scrapsmith",
        "You put a +1/+1 counter on Tomakul Scrapsmith if you don't put an artifact card into your hand with its ability, even if there wasn't an artifact card among the milled cards."
    );
    supported("Tomakul Scrapsmith");
    for name in ["Ravenous Gigamole", "Tomakul Scrapsmith"] {
        // No creature or artifact card among the milled cards.
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Forest", "Island", "Shock"]);
        let c = enter(&mut t, P0, name);
        t.resolve_all();
        assert_eq!(t.counters(c, "+1/+1"), 1, "{name}");
        assert_eq!(t.graveyard_size(P0), 3, "{name}");
        // There was one, but P0 doesn't take it.
        let mut t = TestGame::new(2);
        stack_library(&mut t, P0, &["Ornithopter", "Island", "Shock"]);
        t.answer_choose(P0, &[]);
        let c = enter(&mut t, P0, name);
        t.resolve_all();
        assert_eq!(t.counters(c, "+1/+1"), 1, "{name}");
        assert!(t.in_graveyard(P0, "Ornithopter"), "{name}");
    }
}

#[test]
fn grolnok_exiles_permanent_cards_milled_by_anything() {
    cr!("603.2", "701.17a");
    ruling!(
        "Grolnok, the Omnivore",
        "Grolnok, the Omnivore's second ability exiles all permanent cards that are put into your graveyard from your library, not just those that you mill when a Frog you control attacks."
    );
    supported("Grolnok, the Omnivore");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grolnok, the Omnivore");
    stack_library(&mut t, P0, &["Grizzly Bears", "Shock"]);
    mill(&mut t, P1, P0, 2);
    t.resolve_all();
    let bears = t.g.find_in_zone(Zone::Exile, "Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.counters(bears[0], "croak"), 1);
    assert!(t.in_graveyard(P0, "Shock"));
}

/// P1 casts Skullcrack at P0: damage can't be prevented this turn.
fn skullcrack(t: &mut TestGame) {
    let skullcrack = in_hand_with_mana(t, P1, "Skullcrack");
    t.cast(P1, skullcrack).target(Entity::Player(P0)).go();
    t.resolve_all();
}

#[test]
fn that_many_is_the_damage_but_prevented_this_way_is_what_was_prevented() {
    cr!("615.12", "615.5");
    ruling!(
        "Sekki, Seasons' Guide",
        "Damage that can't be prevented still causes counters to be removed from Sekki and puts creature tokens onto the battlefield."
    );
    ruling!(
        "Stormwild Capridor",
        "If damage that can’t be prevented is dealt to Stormwild Capridor, it doesn’t get +1/+1 counters for that damage."
    );
    supported("Sekki, Seasons' Guide");
    supported("Stormwild Capridor");
    // Sekki: "If damage would be dealt to Sekki, prevent that damage, remove that many
    // +1/+1 counters from Sekki, and create that many 1/1 colorless Spirit creature
    // tokens."
    let mut t = TestGame::new(2);
    let sekki = t.battlefield(P0, "Sekki, Seasons' Guide");
    crate::r_s13_common::add(&mut t, sekki, "+1/+1", 8);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    skullcrack(&mut t);
    deal(&mut t, sorcerer, 2, sekki);
    assert_eq!(t.obj_now(sekki).damage, 2);
    assert_eq!(t.counters(sekki, "+1/+1"), 6);
    assert_eq!(tokens(&t, P0).len(), 2);
    // Stormwild Capridor: "If noncombat damage would be dealt to this creature, prevent
    // that damage. Put a +1/+1 counter on this creature for each 1 damage prevented this
    // way."
    let mut t = TestGame::new(2);
    let capridor = t.battlefield(P0, "Stormwild Capridor");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    skullcrack(&mut t);
    deal(&mut t, sorcerer, 1, capridor);
    assert_eq!(t.obj_now(capridor).damage, 1);
    assert_eq!(t.counters(capridor, "+1/+1"), 0);
}
