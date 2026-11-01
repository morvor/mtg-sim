//! Rulings batch P038 — mana abilities (CR 605), costs locked in before they're paid
//! (CR 601.2f–h), additional costs of spells cast without paying their mana cost
//! (CR 118.9d), copies and mana spent (CR 707.10), X fixed as a spell is cast, and
//! permissions that keep the normal timing (CR 601.3).

use crate::r_p038_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, can_cast, create_token};
use crate::r_s35_common::creature_type_options;
use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn creature_count_mana_abilities_dont_use_the_stack() {
    cr!("605.1a", "605.3a", "605.3b");
    ruling!(
        "Shaman of Forgotten Ways",
        "The first ability of Shaman of Forgotten Ways is a mana ability. It doesn’t use the stack and can’t be responded to."
    );
    ruling!(
        "Brigid, Clachan's Heart // Brigid, Doun's Mind",
        "The first ability of Brigid, Doun's Mind is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Shaman of Forgotten Ways");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shaman of Forgotten Ways");
    let r = t.activate(P0, s, 0, &[]).unwrap();
    assert!(r.is_none());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_total(&t, P0), 2);

    // Brigid, Doun's Mind with two other creatures: two mana right away.
    supported("Brigid, Clachan's Heart // Brigid, Doun's Mind");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Brigid, Clachan's Heart // Brigid, Doun's Mind");
    assert!(mtg_engine::dfc::transform(&mut t.g, b));
    t.g.recompute();
    assert_eq!(t.obj_now(b).chars.name.as_str(), "Brigid, Doun's Mind");
    bears(&mut t, P0, 2);
    let r = t.activate(P0, b, 0, &[]).unwrap();
    assert!(r.is_none());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_total(&t, P0), 2);
}

#[test]
fn shaman_mana_pays_for_face_down_creature_spells() {
    cr!("106.6", "708.4");
    ruling!(
        "Shaman of Forgotten Ways",
        "The mana produced by Shaman of Forgotten Ways can be used to cast face-down creature spells with morph or megamorph."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shaman of Forgotten Ways");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, s, 0, &[]).unwrap();
    assert_eq!(pool_total(&t, P0), 2);
    let demon = t.hand(P0, "Grinning Demon");
    t.cast(P0, demon)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert!(t.obj_now(demon).face_down);
}

#[test]
fn magnus_cost_is_locked_in_before_mana_abilities_sacrifice_a_token() {
    cr!("601.2f", "601.2g", "601.2h");
    ruling!(
        "Magnus the Red",
        "The total cost to cast a spell is locked in before you pay that cost."
    );
    supported("Magnus the Red");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magnus the Red");
    create_token(&mut t, P0, "Spawn");
    create_token(&mut t, P0, "Spawn");
    // A creature token with "Sacrifice this creature: Add {B}." (Blood Pet's ability).
    let spec = TokenSpec {
        name: "Blood Pet".into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec!["Thrull".into()],
        power: Some(1),
        toughness: Some(1),
        abilities: mtg_engine::card::card("Blood Pet").front().chars.abilities.clone(),
        scryfall_name: None,
        pt_values: None,
    };
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.lands(P0, "Mountain", 1);
    // Lava Axe ({4}{R}) costs {1}{R} with three creature tokens; the Blood Pet token is
    // sacrificed for mana while paying.
    let axe = t.hand(P0, "Lava Axe");
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, axe).go();
    assert!(t.named_on_battlefield("Blood Pet").is_empty());
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn volcanic_wind_fixes_x_and_targets_as_it_is_cast() {
    cr!("601.2d", "107.3", "115.1");
    ruling!(
        "Volcanic Wind",
        "The value of X is determined at the time you announce this spell, and you choose the targets and the division of damage among those targets as part of the announcement as well."
    );
    ruling!(
        "Volcanic Wind",
        "If there are no creatures on the battlefield when you cast this, you do not pick any targets and it deals no damage."
    );
    supported("Volcanic Wind");
    // Two creatures as it's cast: 2 damage to the Wurm, even with a third creature
    // arriving in response.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.battlefield(P0, "Grizzly Bears");
    cast_only(&mut t, P0, "Volcanic Wind", &[obj(wurm)]);
    add(&mut t, P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);

    // No creatures: cast with no targets, nothing happens.
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Volcanic Wind", &[]);
    assert!(t.in_graveyard(P0, "Volcanic Wind"));
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
}

#[test]
fn officious_interrogation_free_still_pays_for_extra_targets() {
    cr!("118.9d", "601.2f");
    ruling!(
        "Officious Interrogation",
        "If a spell or ability allows you to cast Officious Interrogation without paying its mana cost, you must still pay the additional cost for any targets beyond the first."
    );
    supported("Officious Interrogation");
    supported("Omniscience");
    // (targets, W and U available, castable)
    for (two, paid, ok) in [(false, false, true), (true, false, false), (true, true, true)] {
        let mut t = TestGame::new(2);
        // "You may cast spells from your hand without paying their mana costs."
        t.battlefield(P0, "Omniscience");
        bears(&mut t, P1, 2);
        if paid {
            t.g.players[0].mana_pool.add_type(ManaType::W, 1);
            t.g.players[0].mana_pool.add_type(ManaType::U, 1);
        }
        let s = t.hand(P0, "Officious Interrogation");
        let targets = if two { vec![pl(P0), pl(P1)] } else { vec![pl(P1)] };
        t.answer_targets(P0, &targets);
        let free = crate::r_s08_common::legal_cast_methods(&mut t, P0, s)
            .into_iter()
            .find(|m| *m != CastMethod::Normal)
            .expect("a free cast");
        let r = t.cast(P0, s).method(free).try_go();
        assert_eq!(r.is_ok(), ok, "two={two} paid={paid} {r:?}");
        if two && paid {
            assert_eq!(pool_total(&t, P0), 0);
            t.resolve_all();
            assert_eq!(tokens_of(&t, P0, "Clue").len(), 2);
        }
    }
}

#[test]
fn copies_of_dryads_caress_never_had_white_spent() {
    cr!("707.10", "601.2h");
    ruling!(
        "Dryad's Caress",
        "If the spell is copied (such as with Twincast), the copy will never have had mana of the stated color paid for it, no matter what colors were spent on the original spell."
    );
    supported("Dryad's Caress");
    let mut t = TestGame::new(2);
    let b = bears(&mut t, P0, 2);
    for c in &b {
        tap(&mut t, *c);
    }
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    let caress = t.hand(P0, "Dryad's Caress");
    let spell = t.cast(P0, caress).go();
    t.lands(P0, "Island", 2);
    let twincast = t.hand(P0, "Twincast");
    t.answer_targets(P0, &[obj(spell)]);
    t.cast(P0, twincast).go();
    t.resolve(); // Twincast: the copy goes on the stack
    t.resolve(); // the copy: 2 life, no untap
    assert_eq!(t.life(P0), 22);
    assert!(b.iter().all(|c| tapped(&t, *c)));
    t.resolve_all(); // the original, with {W} spent
    assert_eq!(t.life(P0), 24);
    assert!(b.iter().all(|c| !tapped(&t, *c)));
}

#[test]
fn gentlemans_rise_from_the_graveyard_follows_normal_casting_rules() {
    cr!("601.3", "601.2f", "715.3");
    ruling!(
        "Hildibrand Manderville // Gentleman's Rise",
        "You must follow the normal timing permissions and restrictions for Gentleman's Rise when casting it with the permission of Hildibrand Manderville's triggered ability. You must pay its mana cost (or, if another effect allows, an alternative cost)."
    );
    supported("Hildibrand Manderville // Gentleman's Rise");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hildibrand Manderville // Gentleman's Rise");
    kill(&mut t, h);
    t.resolve_all();
    let card = t.g.current(h);
    assert_eq!(t.zone(card), mtg_engine::object::Zone::Graveyard(P0));
    // No mana: it can't be cast.
    assert!(!can_cast(&mut t, P0, card, CastMethod::Half(1)));
    // With its mana cost available, it can be cast — at instant speed, as an instant,
    // even in the opponent's turn.
    t.set_step(P1, Step::Upkeep);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    assert!(can_cast(&mut t, P0, card, CastMethod::Half(1)));
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Zombie").len(), 1);
}

#[test]
fn single_use_choices_and_activations() {
    cr!("205.3m", "602.2");
    ruling!(
        "Three Tree City",
        "To choose a creature type, you must choose an existing creature type, such as Lizard or Wizard. You can’t choose multiple creature types, such as “Lizard Wizard.” Card types such as artifact can’t be chosen, nor can subtypes that aren’t creature types, such as Jace, Vehicle, or Treasure."
    );
    ruling!(
        "Centaur's Herald",
        "You can only sacrifice Centaur’s Herald once, so it’s not possible to activate its ability more than once."
    );
    supported("Three Tree City");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    t.enter(P0, "Three Tree City");
    let opts = creature_type_options(&t, P0, from);
    assert_eq!(opts.len(), 1);
    let o = &opts[0];
    for yes in ["Lizard", "Wizard"] {
        assert!(o.iter().any(|x| x == yes), "{yes}");
    }
    for no in ["Lizard Wizard", "Artifact", "artifact", "Jace", "Vehicle", "Treasure"] {
        assert!(!o.iter().any(|x| x == no), "{no}");
    }

    supported("Centaur's Herald");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Centaur's Herald");
    mana_all(&mut t, P0, 3);
    t.activate(P0, herald, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, "Centaur's Herald"));
    assert!(!can_activate(&mut t, P0, herald));
    let card = t.g.current(herald);
    assert!(!can_activate(&mut t, P0, card));
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Centaur").len(), 1);
}
