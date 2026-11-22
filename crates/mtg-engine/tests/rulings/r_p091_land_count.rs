//! Rulings batch P091 — "land count matters": counting lands by subtype, when the count
//! is taken (on resolution vs continuously), and bonuses that stop applying to damaged
//! creatures.

use crate::r_p091_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::damage;
use crate::r_s25_common::targets_of;
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn land_count_bonuses_that_stop_applying_can_make_marked_damage_lethal() {
    cr!("704.5g", "613.4c", "120.6");
    ruling!(
        "Gigantoad",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Gigantoad may become lethal if the number of lands you control drops below seven during that turn."
    );
    ruling!(
        "Scurrid Colony",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to Scurrid Colony may become lethal if the number of lands you control falls below eight."
    );
    ruling!(
        "Scurrid Colony",
        "If the number of lands you control falls below eight, Scurrid Colony loses the +2/+2 bonus. This may cause it to die to damage that is already marked on it."
    );
    ruling!(
        "Sylvan Advocate",
        "If Sylvan Advocate’s ability stops applying (because Sylvan Advocate leaves the battlefield or you no longer control six or more lands), then any land creatures that needed the toughness bonus to stay alive will be destroyed."
    );
    ruling!(
        "Maja, Bretagard Protector",
        "Because damage remains marked on creatures until the damage is removed as the turn ends, nonlethal damage dealt to creatures you control may become lethal if Maja, Bretagard Protector leaves the battlefield that turn."
    );
    // (card, lands needed, damage that's lethal only without the bonus)
    for (name, lands, dmg) in [("Gigantoad", 7, 5), ("Scurrid Colony", 8, 3)] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let ls = t.lands(P0, "Forest", lands);
        damage(&mut t, ls[0], dmg, c);
        t.settle();
        assert!(t.on_battlefield(c), "{name} survives with the bonus");
        destroy(&mut t, ls[1]);
        t.settle();
        assert!(!t.on_battlefield(c), "{name} dies without the bonus");
    }
    // Sylvan Advocate: a 1/1 Dryad Arbor is 3/3 and has 2 damage. Both ways the ability
    // stops applying destroy it.
    for advocate_leaves in [true, false] {
        let mut t = TestGame::new(2);
        let adv = t.battlefield(P0, "Sylvan Advocate");
        let ls = t.lands(P0, "Forest", 5);
        let arbor = t.battlefield(P0, "Dryad Arbor");
        assert_eq!(t.pt(arbor), (3, 3));
        damage(&mut t, ls[0], 2, arbor);
        t.settle();
        assert!(t.on_battlefield(arbor));
        destroy(&mut t, if advocate_leaves { adv } else { ls[1] });
        t.settle();
        assert!(
            !t.on_battlefield(arbor),
            "advocate leaves: {advocate_leaves}"
        );
    }
    // Maja: other creatures get +1/+1. Bears 3/3 with 2 damage.
    let mut t = TestGame::new(2);
    let maja = t.battlefield(P0, "Maja, Bretagard Protector");
    let bears = t.battlefield(P0, "Grizzly Bears");
    damage(&mut t, maja, 2, bears);
    t.settle();
    assert!(t.on_battlefield(bears));
    destroy(&mut t, maja);
    t.settle();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn land_subtypes_are_counted_not_names() {
    cr!("205.3i", "305.6", "305.7");
    ruling!(
        "Tide Shaper",
        "Tide Shaper will get +1/+1 as long as an opponent controls any land with the Island land type, not just ones named Island."
    );
    ruling!(
        "Harbor Serpent",
        "Harbor Serpent's abilities care about lands with the land type Island, not necessarily lands named Island."
    );
    ruling!(
        "Quag Sickness",
        "The ability cares about lands with the land type Swamp, not necessarily lands named Swamp."
    );
    ruling!(
        "Quag Sickness",
        "This bonus is not fixed; it changes as the number of Swamps you control changes."
    );
    ruling!(
        "Aspect of Wolf",
        "The effect is continuously updated based on the number of Forests you control at any given time."
    );
    supported("Tide Shaper");
    supported("Quag Sickness");
    supported("Aspect of Wolf");
    // Tide Shaper: an opponent's Watery Grave (Island Swamp).
    let mut t = TestGame::new(2);
    let shaper = t.battlefield(P0, "Tide Shaper");
    assert_eq!(t.pt(shaper), (1, 1));
    t.battlefield(P1, "Watery Grave");
    t.g.recompute();
    assert_eq!(t.pt(shaper), (2, 2));
    // Harbor Serpent: islandwalk against Watery Grave; attacks with five Island-typed lands
    // on the battlefield, any controllers.
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Harbor Serpent");
    t.lands(P0, "Watery Grave", 2);
    t.lands(P1, "Breeding Pool", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, serpent));
    t.battlefield(P1, "Tropical Island");
    assert!(crate::r_s02_common::can_attack(&mut t, serpent));
    let wall = t.battlefield(P1, "Wall of Stone");
    t.set_step(P0, Step::PrecombatMain);
    crate::r_s01_common::attack_with(&mut t, &[(serpent, Entity::Player(P1))]);
    assert!(!crate::r_s21_common::legal_blocks(
        &mut t,
        P1,
        &[(wall, serpent)]
    ));
    // Quag Sickness: Watery Grave counts; it changes with the count.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    let q = crate::r_s06_common::attach_new(&mut t, P0, "Quag Sickness", obj(giant));
    let _ = q;
    t.lands(P0, "Watery Grave", 1);
    t.lands(P0, "Swamp", 1);
    t.g.recompute();
    assert_eq!(t.pt(giant), (4, 2));
    let s = t.battlefield(P0, "Swamp");
    t.g.recompute();
    assert_eq!(t.pt(giant), (3, 1));
    destroy(&mut t, s);
    t.g.recompute();
    assert_eq!(t.pt(giant), (4, 2));
    // Aspect of Wolf: +X/+Y, halves of the Forest count (Breeding Pool counts).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Aspect of Wolf", obj(bears));
    t.lands(P0, "Breeding Pool", 3);
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 4));
    t.battlefield(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn serpent_and_kraken_check_the_count_only_when_attacking_or_blocking() {
    cr!("508.1c", "509.1b", "506.4");
    ruling!(
        "Harbor Serpent",
        "The second ability checks how many Islands are on the battlefield (regardless of who controls them) only as attackers are declared. Once Harbor Serpent is declared as an attacker, it will continue to attack even if the number of Islands on the battlefield falls below five."
    );
    ruling!(
        "Kraken of the Straits",
        "The number of Islands you control is counted as blockers are declared. Creatures with power less than that number at that time can’t block Kraken of the Straits. Once blockers have been declared, the power of the blocking creatures and the number of Islands you control don’t matter."
    );
    supported("Harbor Serpent");
    supported("Kraken of the Straits");
    let mut t = TestGame::new(2);
    let serpent = t.battlefield(P0, "Harbor Serpent");
    let mine = t.lands(P0, "Island", 3);
    t.lands(P1, "Island", 2);
    attack_with(&mut t, &[(serpent, Entity::Player(P1))]);
    destroy(&mut t, mine[0]);
    destroy(&mut t, mine[1]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
    // Kraken: four Islands; a 3/3 can't block, a 4/4 can, and keeps blocking when the
    // count rises afterwards.
    let mut t = TestGame::new(2);
    let kraken = t.battlefield(P0, "Kraken of the Straits");
    t.lands(P0, "Island", 4);
    let small = t.battlefield(P1, "Grizzly Bears");
    let big = t.battlefield(P1, "Hill Giant");
    let mut c = t.battlefield(P1, "Craw Wurm");
    let _ = &mut c;
    attack_with(&mut t, &[(kraken, Entity::Player(P1))]);
    assert!(!crate::r_s21_common::legal_blocks(
        &mut t,
        P1,
        &[(small, kraken)]
    ));
    assert!(crate::r_s21_common::legal_blocks(
        &mut t,
        P1,
        &[(c, kraken)]
    ));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(c, kraken)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(crate::r_s10_common::blocking(&t, c));
    t.lands(P0, "Island", 3);
    t.advance_to(P0, Step::EndOfCombat);
    let _ = big;
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn counts_are_taken_as_the_spell_or_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Spawn of Thraxes",
        "Count the number of Mountains you control when the last ability resolves to determine how much damage is dealt. This includes any land with the subtype Mountain, not just those named Mountain."
    );
    ruling!(
        "Glimmerpost",
        "As the triggered ability resolves, count each Locus on the battlefield regardless of who controls it."
    );
    ruling!(
        "Omnath, Locus of Rage",
        "Omanth doesn't have the land type Locus."
    );
    ruling!(
        "Charix, the Raging Isle",
        "Each time you activate Charix's last ability, the value of X for that instance of the ability is determined only as that ability resolves. Once that happens, the size of the power-and-toughness change for that instance of the ability won't change later in the turn, even if the number of Islands you control does change."
    );
    ruling!(
        "Mutilate",
        "The effect is based on the number of Swamps you control when Mutilate resolves. Only creatures on the battlefield at that time will be affected. The effect won't change later in the turn, even if the number of Swamps you control changes."
    );
    ruling!(
        "Priest of the Haunted Edge",
        "Use the number of snow lands you control as the ability resolves to determine the value of X."
    );
    supported("Spawn of Thraxes");
    supported("Glimmerpost");
    supported("Charix, the Raging Isle");
    supported("Mutilate");
    supported("Priest of the Haunted Edge");
    // Spawn of Thraxes: two Mountains, then a Stomping Ground arrives in response: 3.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    put_onto_battlefield(&mut t, P0, "Spawn of Thraxes");
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Stomping Ground");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Glimmerpost: P1's Glimmerpost counts; Omnath, Locus of Rage doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Glimmerpost");
    t.battlefield(P0, "Omnath, Locus of Rage");
    play_from_hand(&mut t, P0, "Glimmerpost");
    t.battlefield(P1, "Glimmerpost");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Charix: two Islands as the ability resolves; a third later changes nothing.
    let mut t = TestGame::new(2);
    let charix = t.battlefield(P0, "Charix, the Raging Isle");
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Island", 1);
    t.activate(P0, charix, 0, &[]).unwrap();
    t.battlefield(P0, "Island");
    t.resolve_all();
    assert_eq!(t.pt(charix), (2, 15));
    t.battlefield(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(charix), (2, 15));
    // Mutilate: two Swamps on resolution; a later Swamp and a later creature are unaffected.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 2);
    let m = t.hand(P0, "Mutilate");
    t.cast(P0, m).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 2));
    t.battlefield(P0, "Swamp");
    let later = t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(giant), (4, 2));
    assert_eq!(t.pt(later), (3, 3));
    // Priest of the Haunted Edge: snow lands counted on resolution.
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Priest of the Haunted Edge");
    t.lands(P0, "Snow-Covered Swamp", 1);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.activate(P0, priest, 0, &[obj(wurm)]).unwrap();
    t.battlefield(P0, "Snow-Covered Island");
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
}

#[test]
fn look_at_x_cards_counts_on_resolution() {
    cr!("608.2h", "107.3");
    ruling!(
        "Nessian Game Warden",
        "The value of X is calculated as the ability resolves."
    );
    ruling!(
        "Pillage the Bog",
        "The value of X is determined only once, as Pillage the Bog resolves."
    );
    supported("Nessian Game Warden");
    supported("Pillage the Bog");
    // Nessian Game Warden: one Forest when it enters, another before the trigger resolves.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears", "Craw Wurm"]);
    put_onto_battlefield(&mut t, P0, "Nessian Game Warden");
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Forest");
    let from = t.asked().len();
    t.resolve_all();
    // The Warden itself isn't a Forest: two Forests, two cards looked at.
    let looked = asked_since(&t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    assert_eq!(looked, 2, "{:?}", asked_since(&t, from));
    // Pillage the Bog: two lands (X = 4), one more in response: X = 6.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let lib: Vec<&str> = vec!["Grizzly Bears"; 8];
    stack_library(&mut t, P0, &lib);
    let pillage = t.hand(P0, "Pillage the Bog");
    t.cast(P0, pillage).go();
    t.battlefield(P0, "Forest");
    let from = t.asked().len();
    t.resolve_all();
    let looked = asked_since(&t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    assert_eq!(looked, 6, "{:?}", asked_since(&t, from));
}

#[test]
fn garruk_counts_on_resolution() {
    cr!("608.2h", "107.1b");
    ruling!(
        "Garruk, Primal Hunter",
        "For Garruk’s second ability, the greatest power among creatures you control is determined when that ability begins resolving."
    );
    ruling!(
        "Garruk, Primal Hunter",
        "For Garruk’s third ability, the number of Wurm tokens you put onto the battlefield is determined when that ability begins resolving."
    );
    ruling!(
        "Garruk, Primal Hunter",
        "If the greatest power among creatures you control is 0 or less, you’ll draw no cards. You won’t discard any cards this way."
    );
    supported("Garruk, Primal Hunter");
    // −3: Bears (2) when activated, Hill Giant (3) arrives in response: draw three.
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Primal Hunter");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, garruk, 1, &[]).unwrap();
    t.battlefield(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // A 0-power creature shrunk to −2: no cards, no discard.
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Primal Hunter");
    t.g.objects[garruk.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 4);
    let wall = t.battlefield(P0, "Wall of Wood");
    crate::r_s26_common::modify_until_eot(
        &mut t,
        wall,
        vec![Modification::ModifyPT(
            mtg_engine::ability::Value::Const(-2),
            mtg_engine::ability::Value::Const(0),
        )],
    );
    t.hand(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let gy = t.graveyard_size(P0);
    t.activate(P0, garruk, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P0), gy);
    // −6: two lands when activated, a third in response: three Wurms.
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk, Primal Hunter");
    t.g.objects[garruk.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 6);
    t.lands(P0, "Forest", 2);
    t.activate(P0, garruk, 2, &[]).unwrap();
    t.battlefield(P0, "Forest");
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Wurm").len(),
        3
    );
}

#[test]
fn liliana_of_the_dark_realms_rulings() {
    cr!("205.3i", "701.23b", "608.2d", "113.2c");
    ruling!(
        "Liliana of the Dark Realms",
        "Each of Liliana's abilities refers to lands (or land cards) with the subtype Swamp, not just ones named Swamp."
    );
    ruling!(
        "Liliana of the Dark Realms",
        "When you activate Liliana's second ability, you choose only the target creature. You choose whether that creature gets +X/+X or -X/-X when the ability resolves. The value of X is determined by the number of Swamps you control when the ability resolves."
    );
    ruling!(
        "Liliana of the Dark Realms",
        "You can activate Liliana's first ability even if your library contains no Swamp cards (or you don't want to find one). You'll still search your library and shuffle it."
    );
    ruling!(
        "Liliana of the Dark Realms",
        "Liliana's emblem doesn't remove any other abilities of Swamps you control."
    );
    supported("Liliana of the Dark Realms");
    // +1 finds a Watery Grave.
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Dark Realms");
    let grave = t.library_top(P0, "Watery Grave");
    t.answer_choose(P0, &[obj(grave)]);
    t.activate(P0, lili, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Watery Grave"));
    // +1 with no Swamp card: still searches and shuffles.
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Dark Realms");
    assert!(t.activate(P0, lili, 0, &[]).is_ok());
    t.resolve_all();
    assert_eq!(t.counters(lili, counters::LOYALTY), 4);
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, mtg_engine::events::Event::Shuffled { player } if *player == P0)));
    // −3: only the target on activation; +X/+X or −X/−X chosen on resolution, X counted
    // then (Watery Grave and a Swamp that arrives in response).
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Dark Realms");
    t.lands(P0, "Watery Grave", 1);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    t.activate(P0, lili, 1, &[obj(wurm)]).unwrap();
    let on_activation = asked_since(&t, from);
    assert!(
        on_activation.iter().all(|(_, d)| !matches!(
            d,
            Decision::ChooseOption { .. } | Decision::ChooseModes { .. }
        )),
        "{on_activation:?}"
    );
    t.battlefield(P0, "Swamp");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    // −6: the emblem adds an ability; Watery Grave keeps its own.
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Dark Realms");
    t.g.objects[lili.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 6);
    let grave = t.battlefield(P0, "Watery Grave");
    t.activate(P0, lili, 2, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    let texts: Vec<String> = t
        .obj_now(grave)
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect();
    assert!(
        texts
            .iter()
            .any(|x| x.to_lowercase().contains("{b}{b}{b}{b}")),
        "{texts:?}"
    );
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, grave, "{U}"));
}

#[test]
fn intervening_if_and_targeting_restrictions() {
    cr!("603.4", "115.1", "608.2b");
    ruling!(
        "Scute Mob",
        "Scute Mob’s ability has an “intervening ‘if’ clause.” That means (1) the ability won’t trigger at all unless you control five or more lands as your upkeep starts, and (2) the ability will do nothing if you control fewer than five lands by the time it resolves."
    );
    ruling!(
        "Crash Landing",
        "Crash Landing can't target a creature without flying just to deal damage to it."
    );
    ruling!(
        "Tendrils of Corruption",
        "If the target creature is an illegal target by the time Tendrils of Corruption tries to resolve, the spell doesn't resolve. You don't gain X life."
    );
    ruling!(
        "Fire Dragon",
        "If there are no other creatures on the battlefield, Fire Dragon will have to target itself. The ability is mandatory."
    );
    supported("Scute Mob");
    supported("Crash Landing");
    supported("Tendrils of Corruption");
    supported("Fire Dragon");
    // Scute Mob: four lands, no trigger; five lands, then one leaves: nothing.
    let mut t = TestGame::new(2);
    let mob = t.battlefield(P0, "Scute Mob");
    let ls = t.lands(P0, "Forest", 4);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(triggers_of(&t, mob), 0);
    t.battlefield(P0, "Forest");
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(triggers_of(&t, mob), 1);
    destroy(&mut t, ls[0]);
    t.resolve_all();
    assert_eq!(t.counters(mob, "+1/+1"), 0);
    // Positive control: with five lands it gets the counters.
    t.battlefield(P0, "Forest");
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(mob, "+1/+1"), 4);
    // Crash Landing: only creatures with flying.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bird = t.battlefield(P1, "Birds of Paradise");
    t.lands(P0, "Forest", 3);
    let targets = crate::r_s04_common::spell_targets(&mut t, P0, "Crash Landing");
    assert!(targets.contains(&obj(bird)));
    assert!(!targets.contains(&obj(bears)));
    // Tendrils: the target leaves; no life gained.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let tendrils = t.hand(P0, "Tendrils of Corruption");
    t.cast(P0, tendrils).target(obj(bears)).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P0, "Tendrils of Corruption"));
    // Fire Dragon alone: its trigger targets itself; six Mountains kill it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 6);
    let dragon = put_onto_battlefield(&mut t, P0, "Fire Dragon");
    assert_eq!(t.stack_len(), 1);
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, trig), vec![obj(dragon)]);
    t.resolve_all();
    assert!(!t.on_battlefield(dragon));
}

#[test]
fn strata_scythe_counts_all_lands_with_the_name_and_incursion_exiles_face_up() {
    cr!("406.3", "613.4c");
    ruling!(
        "Strata Scythe",
        "The bonus counts all lands on the battlefield with the same name as the exiled card, not just lands you control."
    );
    ruling!(
        "Nightmare Incursion",
        "The cards are exiled face up, so everyone can see what they are."
    );
    supported("Strata Scythe");
    supported("Nightmare Incursion");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    let scythe = put_onto_battlefield(&mut t, P0, "Strata Scythe");
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(scythe, obj(bears)));
    t.lands(P0, "Forest", 1);
    t.lands(P1, "Forest", 2);
    t.lands(P1, "Island", 2);
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    // Nightmare Incursion: two Swamps, two cards exiled face up.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 4);
    let cards = stack_library(&mut t, P1, &["Grizzly Bears", "Hill Giant"]);
    t.answer_choose(P0, &[obj(cards[0]), obj(cards[1])]);
    let inc = t.hand(P0, "Nightmare Incursion");
    t.cast(P0, inc).target(Entity::Player(P1)).go();
    t.resolve_all();
    for c in cards {
        assert_eq!(t.zone(c), Zone::Exile);
        assert!(!t.obj_now(c).face_down);
    }
}

#[test]
fn charix_cost_increase_applies_once_and_doesnt_change_mana_value() {
    cr!("601.2f", "202.3", "115.3");
    ruling!(
        "Charix, the Raging Isle",
        "An opponent's spell that targets Charix more than once costs only {2} more to cast."
    );
    ruling!(
        "Charix, the Raging Isle",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as that of Charix), then apply any cost reductions. The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Seeds of Strength");
    // Seeds of Strength ({G}{W}: three targets) all on Charix: {2}{G}{W}.
    let mut t = TestGame::new(2);
    let charix = t.battlefield(P0, "Charix, the Raging Isle");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Plains", 1);
    t.lands(P1, "Wastes", 2);
    let seeds = t.hand(P1, "Seeds of Strength");
    let spell = t
        .cast(P1, seeds)
        .targets(&[obj(charix), obj(charix), obj(charix)])
        .go();
    assert_eq!(tapped_lands(&t, P1), 4);
    assert_eq!(crate::r_s08_common::mana_value(&t, spell), 2);
    t.resolve_all();
    assert_eq!(t.pt(charix), (3, 20));
    // With only three lands it can't be cast at Charix.
    let mut t = TestGame::new(2);
    let charix = t.battlefield(P0, "Charix, the Raging Isle");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 1);
    t.lands(P1, "Plains", 1);
    t.lands(P1, "Wastes", 1);
    let seeds = t.hand(P1, "Seeds of Strength");
    assert!(t
        .cast(P1, seeds)
        .targets(&[obj(charix), obj(charix), obj(charix)])
        .try_go()
        .is_err());
}
