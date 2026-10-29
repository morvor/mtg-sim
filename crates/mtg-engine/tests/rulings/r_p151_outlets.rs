//! Rulings batch P151 — sacrifice outlets: abilities and spells that sacrifice any number
//! of permanents or lands (CR 701.21), as costs or as part of their effects (CR 608.2,
//! 614.12 for "as ~ enters"), and replacement effects that sacrifice instead.

use crate::r_p151_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The candidates of the entity choices asked of `p` since decision `from`.
fn choices_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn infernal_denizen_must_sacrifice_the_swamps() {
    cr!("701.21a");
    ruling!(
        "Infernal Denizen",
        "You must sacrifice the Swamps if you can. You can’t choose not to pay if you have them."
    );
    supported("Infernal Denizen");
    let mut t = TestGame::new(2);
    let denizen = t.battlefield(P0, "Infernal Denizen");
    let swamps = t.lands(P0, "Swamp", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Even answering "no" to any question, the Swamps are sacrificed.
    t.answer_yes(P0, false);
    next_upkeep(&mut t);
    t.resolve_all();
    assert!(swamps.iter().all(|s| !t.on_battlefield(*s)));
    assert!(!t.obj_now(denizen).tapped);
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn evendo_brushrazer_cards_are_played_following_the_timing_rules() {
    cr!("305.1");
    ruling!(
        "Evendo Brushrazer",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Evendo Brushrazer’s second ability. For example, if an exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Evendo Brushrazer");
    let mut t = TestGame::new(2);
    let razer = t.battlefield(P0, "Evendo Brushrazer");
    t.battlefield(P0, "Mountain");
    let forest = t.library_top(P0, "Forest");
    // "{T}, Sacrifice a land: Add {R}{R}." sacrifices a nontoken permanent.
    act(&mut t, P0, razer, "Add {R}{R}", &[]).expect("mana ability");
    t.resolve_all();
    let forest = t.g.current(forest);
    assert_eq!(t.zone(forest), Zone::Exile);
    // With a spell on the stack, the land can't be played.
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Shock", &[obj(bears)]);
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    t.resolve_all();
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, forest));
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
}

#[test]
fn ruinous_minotaur_its_controller_sacrifices_a_land() {
    ruling!(
        "Ruinous Minotaur",
        "You sacrifice a land, not the opponent who was dealt damage."
    );
    supported("Ruinous Minotaur");
    let mut t = TestGame::new(2);
    let minotaur = t.battlefield(P0, "Ruinous Minotaur");
    let mine = t.battlefield(P0, "Mountain");
    let theirs = t.battlefield(P1, "Forest");
    damage(&mut t, minotaur, Entity::Player(P1), 5);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn hew_the_entwood_sacrifices_on_resolution() {
    cr!("608.2c");
    ruling!(
        "Hew the Entwood",
        "You sacrifice the lands as part of the resolution of Hew the Entwood. It isn't an additional cost. If Hew the Entwood is countered, you won't sacrifice any lands."
    );
    supported("Hew the Entwood");
    let mut t = TestGame::new(2);
    let hew = cast_new(&mut t, P0, "Hew the Entwood", &[]);
    let lands = crate::r_s01_common::tapped_lands(&t, P0);
    assert_eq!(lands, 5, "only the mana cost was paid");
    counter_spell(&mut t, hew);
    t.resolve_all();
    assert_eq!(
        t.g.permanents().filter(|o| o.controller == P0).count(),
        5,
        "no land was sacrificed"
    );
}

#[test]
fn overlaid_terrain_sacrifices_the_lands_before_it_enters() {
    cr!("614.12");
    ruling!(
        "Overlaid Terrain",
        "You sacrifice your lands right before this enchantment enters. There is no way to tap your lands using the 2-mana ability before they are sacrificed."
    );
    supported("Overlaid Terrain");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Forest", 3);
    t.enter(P0, "Overlaid Terrain");
    t.settle();
    assert!(lands.iter().all(|l| !t.on_battlefield(*l)));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // A land that enters later has the ability.
    let forest = t.battlefield(P0, "Forest");
    t.g.recompute();
    assert!(t
        .obj_now(forest)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("two mana of any one color")));
}

#[test]
fn yuma_must_be_on_the_battlefield() {
    cr!("603.10a");
    ruling!(
        "Yuma, Proud Protector",
        "Yuma must be on the battlefield for its last ability to trigger. If it is put into your graveyard at the same time as a Desert card you own, its ability won't trigger."
    );
    supported("Yuma, Proud Protector");
    // A Desert put into the graveyard while Yuma is on the battlefield: a Plant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yuma, Proud Protector");
    let desert = t.battlefield(P0, "Desert");
    destroy(&mut t, desert);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Both at once: no trigger.
    let mut t = TestGame::new(2);
    let yuma = t.battlefield(P0, "Yuma, Proud Protector");
    let desert = t.battlefield(P0, "Desert");
    let to_gy = |obj| MoveEv {
        obj,
        to: Zone::Graveyard(P0),
        pos: mtg_engine::ability::LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(P0),
        etb: EtbInfo::default(),
        source: None,
    };
    t.g.move_objects(vec![to_gy(yuma), to_gy(desert)]);
    t.g.flush_events();
    t.settle();
    assert_eq!(abilities_on_stack(&t), 0);
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn despotic_scepter_targets_permanents_you_own() {
    cr!("108.3", "111.2");
    ruling!(
        "Despotic Scepter",
        "A token's owner is the player under who created it."
    );
    ruling!(
        "Despotic Scepter",
        "You own a nontoken permanent if it started the game in your deck."
    );
    supported("Despotic Scepter");
    let mut t = TestGame::new(2);
    let scepter = t.battlefield(P0, "Despotic Scepter");
    // P0's card and P0's token, both controlled by P1; P1's card controlled by P0.
    let mine = t.battlefield(P0, "Grizzly Bears");
    gain_control(&mut t, P1, mine);
    let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    gain_control(&mut t, P1, token);
    let theirs = t.battlefield(P1, "Hill Giant");
    gain_control(&mut t, P0, theirs);
    let n = t.asked().len();
    t.answer_targets(P0, &[obj(token)]);
    t.activate(P0, scepter, 0, &[]).expect("Despotic Scepter");
    let cands = crate::r_s02_common::target_candidates(&t, P0, n);
    assert!(cands[0].contains(&obj(mine)));
    assert!(cands[0].contains(&obj(token)));
    assert!(!cands[0].contains(&obj(theirs)));
    t.resolve_all();
    assert!(!t.on_battlefield(token));
}

#[test]
fn dralnu_sacrifices_instead_of_being_dealt_damage() {
    cr!("614.1a");
    ruling!(
        "Dralnu, Lich Lord",
        "Dralnu may be one of the permanents you sacrifice while applying its replacement effect."
    );
    supported("Dralnu, Lich Lord");
    let mut t = TestGame::new(2);
    let dralnu = t.battlefield(P0, "Dralnu, Lich Lord");
    let lands = t.lands(P0, "Swamp", 2);
    let shocker = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(dralnu)]);
    damage(&mut t, shocker, obj(dralnu), 1);
    assert!(t.in_graveyard(P0, "Dralnu, Lich Lord"));
    assert!(lands.iter().all(|l| t.on_battlefield(*l)));

    ruling!(
        "Dralnu, Lich Lord",
        "If Dralnu would be dealt more damage than the number of permanents you control, instead you sacrifice all permanents you control."
    );
    let mut t = TestGame::new(2);
    let dralnu = t.battlefield(P0, "Dralnu, Lich Lord");
    t.lands(P0, "Swamp", 2);
    let shocker = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, shocker, obj(dralnu), 5);
    assert_eq!(permanents_of(&t, P0), 0);
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn dralnus_sacrificed_permanents_deal_their_combat_damage_but_dont_trigger() {
    cr!("510.2", "603.10");
    ruling!(
        "Dralnu, Lich Lord",
        "If any of the sacrificed permanents would deal or be dealt damage at the same time as Dralnu, they deal or are dealt damage at the same time that they're sacrificed. Any abilities of the sacrificed permanents that would trigger on that damage don't trigger, but abilities of other permanents that trigger on that damage do."
    );
    supported("Dralnu, Lich Lord");
    supported("Zebra Unicorn");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let their_unicorn = t.battlefield(P1, "Zebra Unicorn");
    let dralnu = t.battlefield(P0, "Dralnu, Lich Lord");
    let my_unicorn = t.battlefield(P0, "Zebra Unicorn");
    let swamp = t.battlefield(P0, "Swamp");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P0)), (their_unicorn, Entity::Player(P0))],
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(dralnu, bears), (my_unicorn, their_unicorn)]),
    );
    // Dralnu would be dealt 2: sacrifice the Unicorn and the Swamp.
    t.answer_choose(P0, &[obj(my_unicorn), obj(swamp)]);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(dralnu));
    assert!(!t.on_battlefield(my_unicorn) && !t.on_battlefield(swamp));
    // The sacrificed Unicorn dealt its damage, but its "you gain that much life" didn't
    // trigger; P1's Unicorn, which dealt damage to the sacrificed one, did.
    // Both Unicorns were dealt their damage and died.
    assert!(t.in_graveyard(P1, "Zebra Unicorn"));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 22);
}

#[test]
fn pious_evangel_and_wayward_disciple_trigger_for_themselves_and_others() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Pious Evangel // Wayward Disciple",
        "If Pious Evangel enters the battlefield at the same time as another creature you control, Pious Evangel's ability triggers once for the other creature as well as once for itself."
    );
    supported("Pious Evangel // Wayward Disciple");
    let mut t = TestGame::new(2);
    crate::r_p116_common::enter_together(
        &mut t,
        P0,
        &["Pious Evangel // Wayward Disciple", "Grizzly Bears"],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 22);

    ruling!(
        "Pious Evangel // Wayward Disciple",
        "If Wayward Disciple dies at the same time as another creature you control, Wayward Disciple's ability triggers once for the other creature as well as once for itself."
    );
    let mut t = TestGame::new(2);
    crate::r_p119_common::enter_with(&mut t, P0, "Pious Evangel // Wayward Disciple", 0, true);
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t);
    assert_eq!(abilities_on_stack(&t), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn renounce_chooses_on_resolution() {
    cr!("608.2c");
    ruling!("Renounce", "You choose what to sacrifice on resolution.");
    supported("Renounce");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Renounce", &[]);
    // A permanent that arrives after Renounce was cast can be sacrificed.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn drinker_of_sorrow_sacrifices_one_permanent() {
    cr!("510.2");
    ruling!(
        "Drinker of Sorrow",
        "You only sacrifice one permanent, no matter how many things it deals damage to."
    );
    supported("Drinker of Sorrow");
    let mut t = TestGame::new(2);
    let drinker = t.battlefield(P0, "Drinker of Sorrow");
    t.lands(P0, "Swamp", 3);
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(drinker, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(e1, drinker), (e2, drinker)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(e1) && !t.on_battlefield(e2));
    assert_eq!(permanents_of(&t, P0), 3, "one of four permanents sacrificed");
}

#[test]
fn shimatsu_cant_sacrifice_creatures_entering_with_it() {
    cr!("614.12");
    ruling!(
        "Shimatsu the Bloodcloaked",
        "You sacrifice the permanents before Shimatsu enters, so you can't sacrifice any creatures that enter at the same time it does."
    );
    supported("Shimatsu the Bloodcloaked");
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "Hill Giant");
    let n = t.asked().len();
    let ids = crate::r_p116_common::enter_together(
        &mut t,
        P0,
        &["Shimatsu the Bloodcloaked", "Grizzly Bears"],
    );
    t.settle();
    let cands = choices_since(&t, P0, n);
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|c| c.contains(&obj(old))));
    for id in &ids {
        assert!(cands.iter().all(|c| !c.contains(&obj(*id))));
    }
}
