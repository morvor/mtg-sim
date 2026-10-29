//! Rulings batch P151 — lands with "{T}, Sacrifice this land: ..." abilities (sacrificed as
//! a cost, CR 602.2, 118.3): targets checked on resolution (CR 608.2b), values counted on
//! resolution, activation restrictions (CR 602.5b), and what the copies and returns they
//! make are (CR 707.2, 400.7, 111.7).

use crate::r_p151_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Adds `n` mana of each of `types` to `p`'s pool.
fn pool(t: &mut TestGame, p: PlayerId, types: &[ManaType], n: u32) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, n);
    }
}

const ALL: [ManaType; 6] = [
    ManaType::W,
    ManaType::U,
    ManaType::B,
    ManaType::R,
    ManaType::G,
    ManaType::C,
];

#[test]
fn a_desert_can_be_sacrificed_for_its_own_ability() {
    cr!("602.2");
    ruling!(
        "Ramunap Ruins",
        "If a Desert has an ability with a cost of \"Sacrifice a Desert,\" you can sacrifice that Desert to pay the cost for its own ability."
    );
    ruling!(
        "Ifnir Deadlands",
        "If a Desert has an ability with a cost of “Sacrifice a Desert,” you can sacrifice that Desert to pay the cost for its own ability."
    );
    for (name, target) in [
        ("Ramunap Ruins", None),
        ("Ifnir Deadlands", Some(P1)),
        ("Hashep Oasis", Some(P0)),
        ("Ipnu Rivulet", None),
        ("Scavenger Grounds", None),
        ("Shefet Dunes", None),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let desert = t.battlefield(P0, name);
        let targets: Vec<Entity> = match target {
            Some(p) => vec![obj(t.battlefield(p, "Grizzly Bears"))],
            None if name == "Ipnu Rivulet" => vec![Entity::Player(P1)],
            None => vec![],
        };
        pool(&mut t, P0, &ALL, 2);
        act(&mut t, P0, desert, "Sacrifice a Desert", &targets)
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert!(t.in_graveyard(P0, name), "{name}");
        t.resolve_all();
        if name == "Ramunap Ruins" {
            assert_eq!(t.life(P1), 18);
        }
    }
}

#[test]
fn blast_zone_x_x_costs_twice_x() {
    cr!("107.3", "601.2f");
    ruling!(
        "Blast Zone",
        "An activation cost of {X}{X} means that you pay twice X. If you want X to be 3, you pay {6} to activate Blast Zone’s ability."
    );
    supported("Blast Zone");
    let mut t = TestGame::new(2);
    let zone = t.battlefield(P0, "Blast Zone");
    t.lands(P0, "Wastes", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    act(&mut t, P0, zone, "charge counters on", &[]).expect("Blast Zone");
    t.resolve_all();
    assert_eq!(t.counters(zone, "charge"), 3);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 7);
}

#[test]
fn tectonic_edge_can_target_any_nonbasic_land() {
    cr!("602.5b");
    ruling!(
        "Tectonic Edge",
        "Assuming you can activate Tectonic Edge’s second ability, you can target any nonbasic land with it (not just one controlled by an opponent that controls four or more lands)."
    );
    supported("Tectonic Edge");
    let mut t = TestGame::new(3);
    let edge = t.battlefield(P0, "Tectonic Edge");
    t.lands(P0, "Wastes", 1);
    let mine = t.battlefield(P0, "Desert");
    t.lands(P1, "Forest", 4);
    let p2s = t.battlefield(P2, "Desert");
    let n = t.asked().len();
    act(&mut t, P0, edge, "Destroy target nonbasic land", &[obj(mine)]).expect("Tectonic Edge");
    let cands = crate::r_s02_common::target_candidates(&t, P0, n);
    assert!(cands[0].contains(&obj(p2s)), "a land of an opponent with fewer lands");
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    // Without an opponent controlling four lands, it can't be activated.
    let mut t = TestGame::new(2);
    let edge = t.battlefield(P0, "Tectonic Edge");
    t.lands(P0, "Wastes", 1);
    let theirs = t.battlefield(P1, "Desert");
    assert!(act(&mut t, P0, edge, "Destroy target nonbasic land", &[obj(theirs)]).is_err());
}

#[test]
fn land_abilities_that_dont_resolve_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Ghost Quarter",
        "If the targeted land is an illegal target by the time Ghost Quarter's ability resolves, it won't resolve and none of its effects will happen. The land's controller won't get to search for a basic land card."
    );
    supported("Ghost Quarter");
    let mut t = TestGame::new(2);
    let quarter = t.battlefield(P0, "Ghost Quarter");
    let forest = t.battlefield(P1, "Forest");
    t.library_top(P1, "Forest");
    let lib = t.library_size(P1);
    t.answer_yes(P1, true);
    act(&mut t, P0, quarter, "Destroy target land", &[obj(forest)]).expect("Ghost Quarter");
    exile(&mut t, forest);
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib);
    assert!(t.named_on_battlefield("Forest").is_empty());

    ruling!(
        "Gnottvold Slumbermound",
        "If the target land is an illegal target by the time the last activated ability tries to resolve, the ability won’t resolve and none of its effects will happen. You won’t create a Troll Warrior creature token."
    );
    supported("Gnottvold Slumbermound");
    let mut t = TestGame::new(2);
    let mound = t.battlefield(P0, "Gnottvold Slumbermound");
    let forest = t.battlefield(P1, "Forest");
    pool(&mut t, P0, &ALL, 3);
    act(&mut t, P0, mound, "Troll Warrior", &[obj(forest)]).expect("Gnottvold Slumbermound");
    exile(&mut t, forest);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());

    ruling!(
        "Skemfar Elderhall",
        "If you activate the last ability without choosing a target, you just create Elf Warrior tokens as the ability resolves. However, if you do choose a target, and that creature is an illegal target as the ability tries to resolve, the ability won’t resolve and none of its effects will happen. In that case, you won’t create Elf Warrior tokens."
    );
    supported("Skemfar Elderhall");
    for target in [false, true] {
        let mut t = TestGame::new(2);
        let hall = t.battlefield(P0, "Skemfar Elderhall");
        let bears = t.battlefield(P1, "Grizzly Bears");
        pool(&mut t, P0, &ALL, 3);
        let targets = if target { vec![obj(bears)] } else { vec![] };
        if !target {
            t.answer_targets(P0, &[]);
        }
        act(&mut t, P0, hall, "Elf Warrior", &targets).expect("Skemfar Elderhall");
        if target {
            exile(&mut t, bears);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P0).len(), if target { 0 } else { 2 });
    }
}

#[test]
fn cryptic_caves_counts_itself_and_only_one_of_two_can_be_activated() {
    cr!("602.5b");
    ruling!(
        "Cryptic Caves",
        "Cryptic Caves can be one of the five lands you control. You don't need to control five other lands."
    );
    supported("Cryptic Caves");
    let mut t = TestGame::new(2);
    let caves = t.battlefield(P0, "Cryptic Caves");
    t.lands(P0, "Wastes", 3);
    pool(&mut t, P0, &[ManaType::C], 1);
    assert!(act(&mut t, P0, caves, "Draw a card", &[]).is_err(), "four lands");
    t.lands(P0, "Wastes", 1);
    let hand = t.hand_size(P0);
    act(&mut t, P0, caves, "Draw a card", &[]).expect("five lands, Caves included");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);

    ruling!(
        "Cryptic Caves",
        "If you control two Cryptic Caves and exactly three other lands, you can activate the last ability of only one of the Cryptic Caves."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cryptic Caves");
    let b = t.battlefield(P0, "Cryptic Caves");
    t.lands(P0, "Wastes", 3);
    pool(&mut t, P0, &[ManaType::C], 2);
    act(&mut t, P0, a, "Draw a card", &[]).expect("the first");
    assert!(act(&mut t, P0, b, "Draw a card", &[]).is_err(), "the second");
}

#[test]
fn haunted_fengraf_can_return_a_card_put_there_in_response() {
    cr!("608.2b");
    ruling!(
        "Haunted Fengraf",
        "Because Haunted Fengraf's ability doesn't target any creature card, any creature card put into the graveyard in response to that ability may be returned to your hand."
    );
    supported("Haunted Fengraf");
    let mut t = TestGame::new(2);
    let fengraf = t.battlefield(P0, "Haunted Fengraf");
    t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, fengraf, "at random", &[]).expect("with no creature card in the graveyard");
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn values_and_conditions_checked_as_land_abilities_resolve() {
    cr!("608.2h", "603.4");
    ruling!(
        "Blighted Steppe",
        "Count the number of creatures you control as the last ability resolves to determine how much life to gain."
    );
    supported("Blighted Steppe");
    let mut t = TestGame::new(2);
    let steppe = t.battlefield(P0, "Blighted Steppe");
    t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, &ALL, 1);
    pool(&mut t, P0, &[ManaType::C], 2);
    act(&mut t, P0, steppe, "gain 2 life", &[]).expect("Blighted Steppe");
    t.battlefield(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 24);

    ruling!(
        "Inventors' Fair",
        "If you control three artifacts as the ability resolves, you gain 1 life. The artifacts you control as the ability resolves don't have to be the same ones you controlled as it triggered. If you don't control three artifacts at that time, you won't gain life."
    );
    supported("Inventors' Fair");
    for replace in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Inventors' Fair");
        let thopters = t.lands(P0, "Ornithopter", 3);
        next_upkeep(&mut t);
        assert_eq!(abilities_on_stack(&t), 1);
        destroy(&mut t, thopters[0]);
        if replace {
            t.battlefield(P0, "Ornithopter");
        }
        t.resolve_all();
        assert_eq!(t.life(P0), if replace { 21 } else { 20 });
    }

    ruling!(
        "Roadside Reliquary",
        "If you control both an artifact and an enchantment, you will draw two cards. If a single permanent is both an artifact and an enchantment, that counts as controlling both."
    );
    supported("Roadside Reliquary");
    let mut t = TestGame::new(2);
    let reliquary = t.battlefield(P0, "Roadside Reliquary");
    t.battlefield(P0, "Spear of Heliod");
    pool(&mut t, P0, &[ManaType::C], 2);
    let hand = t.hand_size(P0);
    act(&mut t, P0, reliquary, "Draw a card", &[]).expect("Roadside Reliquary");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn horizon_of_progress_produces_what_your_lands_could() {
    cr!("106.7");
    ruling!(
        "Horizon of Progress",
        "Horizon of Progress's first ability checks the effects of all mana-producing abilities of lands you control, but it doesn't check their costs or legality."
    );
    supported("Horizon of Progress");
    let mut t = TestGame::new(2);
    let horizon = t.battlefield(P0, "Horizon of Progress");
    let spire = t.battlefield(P0, "Spire of Industry");
    t.g.tap(spire);
    t.g.recompute();
    let types = mtg_engine::mana_abilities::could_produce(&t.g, horizon);
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        assert!(types.contains(&ty), "{ty:?}");
    }

    ruling!(
        "Horizon of Progress",
        "Any change to a land's type or abilities gained by a land can affect the types of mana that land can produce."
    );
    let mut t = TestGame::new(2);
    let horizon = t.battlefield(P0, "Horizon of Progress");
    let forest = t.battlefield(P0, "Forest");
    t.g.recompute();
    assert!(mtg_engine::mana_abilities::could_produce(&t.g, horizon).contains(&ManaType::G));
    let seas = t.battlefield(P1, "Spreading Seas");
    t.attach(seas, obj(forest));
    t.settle();
    let types = mtg_engine::mana_abilities::could_produce(&t.g, horizon);
    assert!(types.contains(&ManaType::U));
    assert!(!types.contains(&ManaType::G));
}

#[test]
fn ally_encampment_mana_only_for_ally_spells() {
    cr!("106.6");
    ruling!(
        "Ally Encampment",
        "Mana produced by the second ability can’t be spent to pay the costs of abilities of Allies you control."
    );
    supported("Ally Encampment");
    supported("Kor Sky Climber");
    let mut t = TestGame::new(2);
    let camp = t.battlefield(P0, "Ally Encampment");
    let climber = t.battlefield(P0, "Kor Sky Climber");
    t.lands(P0, "Plains", 1);
    // "{1}{W}: This creature gains flying": Plains for {W}, Encampment's restricted mana
    // can't pay the {1}.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    act(&mut t, P0, camp, "Spend this mana only", &[]).expect("mana");
    assert!(act(&mut t, P0, climber, "flying", &[]).is_err());
    // It can cast an Ally spell.
    let camp2 = t.battlefield(P0, "Ally Encampment");
    let ally = t.hand(P0, "Kor Sky Climber");
    t.g.players[P0.idx()].mana_pool.mana.clear();
    t.lands(P0, "Plains", 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    act(&mut t, P0, camp2, "Spend this mana only", &[]).expect("mana");
    let camp3 = t.battlefield(P0, "Ally Encampment");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    act(&mut t, P0, camp3, "Spend this mana only", &[]).expect("mana");
    t.cast(P0, ally).go();
}

#[test]
fn lazotep_quarry_tokens() {
    cr!("707.2", "707.9b", "107.3b");
    ruling!(
        "Lazotep Quarry",
        "If a card copied by the token had any \"when [this creature] enters the battlefield\" abilities, the token also has those abilities and will trigger them when it's created."
    );
    ruling!(
        "Lazotep Quarry",
        "Except for the listed exceptions, the token created by Lazotep Quarry's last ability copies exactly what was printed on the original card and nothing else."
    );
    supported("Lazotep Quarry");
    let mut t = TestGame::new(2);
    let quarry = t.battlefield(P0, "Lazotep Quarry");
    let visionary = t.graveyard(P0, "Elvish Visionary");
    pool(&mut t, P0, &[ManaType::C], 4);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let hand = t.hand_size(P0);
    act(&mut t, P0, quarry, "Zombie", &[obj(visionary)]).expect("Lazotep Quarry");
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    let o = t.g.obj(token);
    assert_eq!(o.chars.name.as_str(), "Elvish Visionary");
    assert_eq!((o.power(), o.toughness()), (4, 4));
    assert!(o.chars.subtypes.iter().any(|s| s == "Zombie"));
    assert_eq!(t.hand_size(P0), hand + 1, "its enters ability triggered");

    ruling!("Lazotep Quarry", "If the copied card has {X} in its mana cost, {X} is 0.");
    let mut t = TestGame::new(2);
    let quarry = t.battlefield(P0, "Lazotep Quarry");
    let one = t.graveyard(P0, "Endless One");
    pool(&mut t, P0, &[ManaType::C], 2);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    act(&mut t, P0, quarry, "Zombie", &[obj(one)]).expect("Lazotep Quarry");
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    assert_eq!(t.counters(token, "+1/+1"), 0);
    assert_eq!(t.pt(token), (4, 4));
}

#[test]
fn mirrorpool_creature_copies() {
    cr!("707.2");
    ruling!(
        "Mirrorpool",
        "For Mirrorpool’s last ability, the token copies exactly what’s printed on the original creature and nothing else"
    );
    supported("Mirrorpool");
    let mut t = TestGame::new(2);
    let pool_land = t.battlefield(P0, "Mirrorpool");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    let strength = t.battlefield(P0, "Holy Strength");
    t.attach(strength, obj(bears));
    t.g.tap(bears);
    t.settle();
    pool(&mut t, P0, &[ManaType::C], 5);
    act(&mut t, P0, pool_land, "copy of target creature", &[obj(bears)]).expect("Mirrorpool");
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    let o = t.g.obj(token);
    assert!(!o.tapped);
    assert_eq!(t.counters(token, "+1/+1"), 0);
    assert_eq!(t.pt(token), (2, 2));

    ruling!(
        "Mirrorpool",
        "if a token copies a Plains that’s been affected by a spell with awaken, the token will be just a land"
    );
    let mut t = TestGame::new(2);
    let pool_land = t.battlefield(P0, "Mirrorpool");
    let colonnade = t.battlefield(P0, "Celestial Colonnade");
    pool(&mut t, P0, &[ManaType::C], 3);
    pool(&mut t, P0, &[ManaType::W, ManaType::U], 1);
    act(&mut t, P0, colonnade, "Elemental", &[]).expect("animate");
    t.resolve_all();
    assert!(t.obj_now(colonnade).is(CardType::Creature));
    pool(&mut t, P0, &[ManaType::C], 5);
    act(&mut t, P0, pool_land, "copy of target creature", &[obj(colonnade)]).expect("Mirrorpool");
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    assert!(t.g.obj(token).is(CardType::Land));
    assert!(!t.g.obj(token).is(CardType::Creature));
}

#[test]
fn mirrorpool_spell_copies_keep_x_and_modes() {
    cr!("707.10");
    ruling!(
        "Mirrorpool",
        "If the copied spell has an X whose value was determined as it was cast (like Fall of the Titans does), the copy has the same value of X."
    );
    supported("Mirrorpool");
    let mut t = TestGame::new(2);
    let pool_land = t.battlefield(P0, "Mirrorpool");
    pool(&mut t, P0, &[ManaType::R], 4);
    let blaze = t.hand(P0, "Blaze");
    let blaze = t.cast(P0, blaze).x(3).target(Entity::Player(P1)).go();
    pool(&mut t, P0, &[ManaType::C], 3);
    act(&mut t, P0, pool_land, "Copy target instant or sorcery", &[obj(blaze)]).expect("copy");
    t.resolve_all();
    assert_eq!(t.life(P1), 14);

    ruling!(
        "Mirrorpool",
        "If the copied spell is modal (that is, it says “Choose one —” or the like), the copy will have the same modes. You can’t choose different ones."
    );
    let mut t = TestGame::new(2);
    let pool_land = t.battlefield(P0, "Mirrorpool");
    pool(&mut t, P0, &[ManaType::W], 1);
    let salve = t.hand(P0, "Healing Salve");
    let salve = t.cast(P0, salve).modes(&[0]).target(Entity::Player(P0)).go();
    pool(&mut t, P0, &[ManaType::C], 3);
    let n = t.asked().len();
    act(&mut t, P0, pool_land, "Copy target instant or sorcery", &[obj(salve)]).expect("copy");
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert!(!t.asked()[n..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseModes { .. })));
}

#[test]
fn littjara_mirrorlake_copies_copiable_values() {
    cr!("707.2", "707.3", "111.4");
    ruling!(
        "Littjara Mirrorlake",
        "If the target creature is copying something else, the token you create will use the copiable values of the target creature. In most cases, it will be a copy of whatever the target creature is copying."
    );
    supported("Littjara Mirrorlake");
    let mut t = TestGame::new(2);
    let lake = t.battlefield(P0, "Littjara Mirrorlake");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Grizzly Bears");
    pool(&mut t, P0, &[ManaType::G, ManaType::G, ManaType::U, ManaType::C, ManaType::C], 1);
    act(&mut t, P0, lake, "copy of target creature", &[obj(clone)]).expect("Mirrorlake");
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    assert_eq!(t.g.obj(token).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.pt(token), (3, 3));

    ruling!(
        "Littjara Mirrorlake",
        "If the target creature is itself a token, the token created by Littjara Mirrorlake's ability copies the original characteristics of that token as stated by the effect that created it."
    );
    let mut t = TestGame::new(2);
    let lake = t.battlefield(P0, "Littjara Mirrorlake");
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    crate::r_p119_common::giant_growth(&mut t, soldier);
    assert_eq!(t.pt(soldier), (4, 4));
    pool(&mut t, P0, &[ManaType::G, ManaType::G, ManaType::U, ManaType::C, ManaType::C], 1);
    act(&mut t, P0, lake, "copy of target creature", &[obj(soldier)]).expect("Mirrorlake");
    t.resolve_all();
    let copy = tokens(&t, P0).into_iter().find(|x| *x != t.g.current(soldier)).unwrap();
    assert_eq!(t.pt(copy), (2, 2), "1/1 plus a +1/+1 counter");
}

#[test]
fn exiled_tokens_and_noncreature_cards_dont_come_back() {
    cr!("111.7", "400.7");
    ruling!(
        "Northampton Farm",
        "If a token is exiled this way, it will cease to exist. It won't return to the battlefield or its owner's hand."
    );
    supported("Northampton Farm");
    let mut t = TestGame::new(2);
    let farm = t.battlefield(P0, "Northampton Farm");
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    pool(&mut t, P0, &[ManaType::C], 3);
    act(&mut t, P0, farm, "Exile target creature you own", &[obj(soldier)]).expect("exile");
    t.resolve_all();
    t.g.untap(farm);
    act(&mut t, P0, farm, "Return a creature card", &[]).expect("return");
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(t.hand_size(P0), 0);

    ruling!(
        "Endless Sands",
        "If you exile a creature that isn’t a creature card with the second ability of Endless Sands (such as a token creature or a land that has become a creature), the third ability won’t return that token or card from exile."
    );
    supported("Endless Sands");
    let mut t = TestGame::new(2);
    let sands = t.battlefield(P0, "Endless Sands");
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    let vault = t.battlefield(P0, "Mutavault");
    let bears = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0, &[ManaType::C], 1);
    act(&mut t, P0, vault, "creature types", &[]).expect("animate");
    t.resolve_all();
    for c in [soldier, vault, bears] {
        pool(&mut t, P0, &[ManaType::C], 2);
        t.g.untap(sands);
        act(&mut t, P0, sands, "Exile target creature", &[obj(c)]).expect("exile");
        t.resolve_all();
    }
    t.g.untap(sands);
    pool(&mut t, P0, &[ManaType::C], 4);
    act(&mut t, P0, sands, "Return each creature card", &[]).expect("return");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.named_on_battlefield("Mutavault").is_empty());
    assert!(t.in_exile("Mutavault"));
    assert!(tokens(&t, P0).is_empty());
}

/// P0's Safe Haven exiles `c` (P0 controls it) with its first ability.
fn haven(t: &mut TestGame, safe: ObjectId, c: ObjectId) {
    pool(t, P0, &[ManaType::C], 2);
    t.g.untap(safe);
    act(t, P0, safe, "Exile target creature", &[obj(c)]).expect("Safe Haven");
    t.resolve_all();
}

#[test]
fn safe_haven_cases() {
    supported("Safe Haven");
    ruling!(
        "Safe Haven",
        "All cards in the Haven stay there even if they cease to be creatures. When the Haven is sacrificed, the cards come back onto the battlefield whether or not they are creatures."
    );
    ruling!(
        "Safe Haven",
        "Auras on creatures are put into the graveyard and counters on creatures are removed when the creatures are sent to the Haven."
    );
    ruling!("Safe Haven", "Creatures return to the battlefield simultaneously.");
    let mut t = TestGame::new(2);
    let safe = t.battlefield(P0, "Safe Haven");
    let vault = t.battlefield(P0, "Mutavault");
    pool(&mut t, P0, &[ManaType::C], 1);
    act(&mut t, P0, vault, "creature types", &[]).expect("animate");
    t.resolve_all();
    haven(&mut t, safe, vault);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    let strength = t.battlefield(P0, "Holy Strength");
    t.attach(strength, obj(bears));
    t.settle();
    haven(&mut t, safe, bears);
    assert!(t.in_graveyard(P0, "Holy Strength"));
    let warden = t.battlefield(P0, "Soul Warden");
    haven(&mut t, safe, warden);
    t.answer_yes(P0, true);
    next_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mutavault").len(), 1, "a land, back");
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.counters(b[0], "+1/+1"), 0);
    // Soul Warden saw the other creatures enter with it (two creatures: the Bears and the
    // Mutavault isn't a creature now).
    assert_eq!(t.life(P0), 21);

    ruling!(
        "Safe Haven",
        "Creatures return to the control of their owners, regardless of who controls the Haven when it is sacrificed."
    );
    let mut t = TestGame::new(2);
    let safe = t.battlefield(P0, "Safe Haven");
    let bears = t.battlefield(P0, "Grizzly Bears");
    haven(&mut t, safe, bears);
    // P1 gains control of Safe Haven for good and sacrifices it in their upkeep.
    steal_for_good(&mut t, P1, safe);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(t.g.obj(b[0]).controller, P0);

    ruling!(
        "Safe Haven",
        "If changed to another land type, the creature cards are not lost but can’t be released until the land reverts to normal."
    );
    let mut t = TestGame::new(2);
    let safe = t.battlefield(P0, "Safe Haven");
    let bears = t.battlefield(P0, "Grizzly Bears");
    haven(&mut t, safe, bears);
    let presence = t.battlefield(P1, "Evil Presence");
    t.attach(presence, obj(safe));
    t.settle();
    t.answer_yes(P0, true);
    next_upkeep(&mut t);
    assert_eq!(abilities_on_stack(&t), 0, "a Swamp has no upkeep ability");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    exile(&mut t, presence);
    t.answer_yes(P0, true);
    next_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn command_beacon_and_the_commander_tax() {
    cr!("903.8");
    ruling!(
        "Command Beacon",
        "If you cast a commander from your hand, the additional cost based on the number of times you've cast it from your command zone (sometimes referred to as the \"commander tax\") doesn't apply. Additionally, that casting won't add to the tax if you later cast the commander from the command zone."
    );
    ruling!(
        "Command Beacon",
        "If you cast a commander from your hand, the commander tax doesn't apply. Additionally, that casting won't add to commander tax if you later cast the commander from the command zone."
    );
    supported("Command Beacon");
    let mut t = crate::r_s13_common::commander_game();
    let isamaru = crate::r_s13_common::commander(&mut t, P0, "Isamaru, Hound of Konda");
    // Cast once from the command zone: tax {2} from now on.
    t.lands(P0, "Plains", 1);
    t.cast(P0, isamaru).go();
    t.resolve_all();
    let on = t.named_on_battlefield("Isamaru, Hound of Konda")[0];
    t.answer_yes(P0, true);
    destroy(&mut t, on);
    t.resolve_all();
    assert_eq!(t.zone(isamaru), Zone::Command);
    let beacon = t.battlefield(P0, "Command Beacon");
    // It isn't put back into the command zone instead (CR 903.9b).
    t.answer_yes(P0, false);
    act(&mut t, P0, beacon, "commander into your hand", &[]).expect("Command Beacon");
    t.resolve_all();
    let card = t.g.find_in_zone(Zone::Hand(P0), "Isamaru, Hound of Konda")[0];
    // From hand: just {W}.
    t.g.players[P0.idx()].mana_pool.mana.clear();
    for o in t.g.permanents().map(|o| o.id).collect::<Vec<_>>() {
        t.g.untap(o);
    }
    pool(&mut t, P0, &[ManaType::W], 1);
    t.cast(P0, card).go();
    t.resolve_all();
    // Back to the command zone: the tax is still {2} (one cast from the command zone).
    let on = t.named_on_battlefield("Isamaru, Hound of Konda")[0];
    t.answer_yes(P0, true);
    destroy(&mut t, on);
    t.resolve_all();
    let isamaru = t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda")[0];
    pool(&mut t, P0, &[ManaType::W], 1);
    pool(&mut t, P0, &[ManaType::C], 2);
    t.cast(P0, isamaru).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{{W}} plus a tax of {{2}}");
}

#[test]
fn command_beacon_puts_one_of_two_partners_into_your_hand() {
    cr!("702.124e");
    ruling!(
        "Command Beacon",
        "If you have two commanders with the partner ability in the command zone, Command Beacon's effect puts one of your choice into your hand, not both."
    );
    let mut t = crate::r_s13_common::commander_game();
    let kraum = crate::r_s13_common::commander(&mut t, P0, "Kraum, Ludevic's Opus");
    let tymna = crate::r_s13_common::commander(&mut t, P0, "Tymna the Weaver");
    let beacon = t.battlefield(P0, "Command Beacon");
    t.answer_choose(P0, &[obj(kraum)]);
    t.answer_yes(P0, false);
    act(&mut t, P0, beacon, "commander into your hand", &[]).expect("Command Beacon");
    t.resolve_all();
    assert_eq!(t.zone(kraum), Zone::Hand(P0));
    assert_eq!(t.zone(tymna), Zone::Command);
}

#[test]
fn mount_doom_chooses_without_targeting() {
    cr!("115.10a");
    ruling!(
        "Mount Doom",
        "For the last ability, none of the chosen creatures are targets of the ability. You may choose creatures with shroud, for example."
    );
    supported("Mount Doom");
    let mut t = TestGame::new(2);
    let doom = t.battlefield(P0, "Mount Doom");
    let mox = t.battlefield(P0, "Mox Opal");
    let shroud = t.battlefield(P1, "Argothian Enchantress");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    pool(&mut t, P0, &[ManaType::B, ManaType::R], 1);
    pool(&mut t, P0, &[ManaType::C], 5);
    t.answer_choose(P0, &[obj(mox)]);
    t.answer_choose(P0, &[obj(shroud), obj(bears)]);
    act(&mut t, P0, doom, "Choose up to two creatures", &[]).expect("Mount Doom");
    t.resolve_all();
    assert!(t.on_battlefield(shroud) && t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
}

#[test]
fn tyrite_sanctum_indestructible_counter() {
    cr!("122.1b", "702.12b");
    ruling!(
        "Tyrite Sanctum",
        "An indestructible counter on a permanent gives that permanent indestructible."
    );
    supported("Tyrite Sanctum");
    let mut t = TestGame::new(2);
    let sanctum = t.battlefield(P0, "Tyrite Sanctum");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    pool(&mut t, P0, &[ManaType::C], 2);
    act(&mut t, P0, sanctum, "becomes a God", &[obj(isamaru)]).expect("God");
    t.resolve_all();
    t.g.untap(sanctum);
    pool(&mut t, P0, &[ManaType::C], 4);
    act(&mut t, P0, sanctum, "indestructible counter", &[obj(isamaru)]).expect("counter");
    t.resolve_all();
    assert_eq!(t.counters(isamaru, "indestructible"), 1);
    destroy(&mut t, isamaru);
    assert!(t.on_battlefield(isamaru));
}

#[test]
fn a_token_without_a_mana_cost_has_mana_value_0() {
    cr!("202.3", "111.4");
    ruling!(
        "HELIOS One",
        "A token that’s not a copy of something else normally won’t have a mana cost. Anything without a mana cost normally has a mana value of 0."
    );
    supported("HELIOS One");
    let mut t = TestGame::new(2);
    let helios = t.battlefield(P0, "HELIOS One");
    let soldier = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    pool(&mut t, P0, &[ManaType::C], 3);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    act(&mut t, P0, helios, "Destroy target nonland permanent", &[obj(soldier)]).expect("HELIOS");
    t.resolve_all();
    assert!(!t.on_battlefield(soldier));

    ruling!(
        "Pernicious Deed",
        "A token has a mana value of 0 unless it is copying something else."
    );
    supported("Pernicious Deed");
    let mut t = TestGame::new(2);
    let deed = t.battlefield(P0, "Pernicious Deed");
    let soldier = crate::r_s02_common::create_token(&mut t, P1, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    act(&mut t, P0, deed, "Destroy each", &[]).expect("Pernicious Deed");
    t.resolve_all();
    assert!(!t.on_battlefield(soldier));
    assert!(t.on_battlefield(bears));
}

#[test]
fn searching_for_a_card_with_a_quality_you_dont_have_to_find_it() {
    cr!("701.23b");
    for (name, wanted) in [
        ("Bad River", "Island"),
        ("Flood Plain", "Plains"),
        ("Rocky Tar Pit", "Mountain"),
        ("Terminal Moraine", "Forest"),
        ("Sterling Grove", "Glorious Anthem"),
        ("Goblin Matron", "Goblin Piker"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.library_top(P0, wanted);
        let lib = t.library_size(P0);
        t.answer_choose(P0, &[]);
        if name == "Goblin Matron" {
            t.answer_yes(P0, true);
            t.enter(P0, name);
        } else {
            let src = t.battlefield(P0, name);
            pool(&mut t, P0, &[ManaType::C], 2);
            act(&mut t, P0, src, "Search", &[]).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        }
        t.resolve_all();
        assert_eq!(t.library_size(P0), lib, "{name}: found nothing");
        assert!(!t.in_hand(P0, wanted) && t.named_on_battlefield(wanted).is_empty());
    }
    ruling!("Bad River", "you don't have to find the card if you don't want to");
    ruling!("Flood Plain", "you don't have to find the card if you don't want to");
    ruling!("Rocky Tar Pit", "you don't have to find the card if you don't want to");
    ruling!("Terminal Moraine", "you don't have to find the card if you don't want to");
    ruling!("Sterling Grove", "you don't have to find the card if you don't want to");
    ruling!("Goblin Matron", "you don't have to find the card if you don't want to");
}

#[test]
fn city_of_traitors_triggers_only_on_playing_another_land() {
    cr!("305.1", "603.2");
    ruling!(
        "City of Traitors",
        "City of Traitors does not trigger on itself being played."
    );
    ruling!(
        "City of Traitors",
        "City of Traitors does not trigger on lands that are put onto the battlefield without playing them."
    );
    supported("City of Traitors");
    let mut t = TestGame::new(2);
    let city = t.hand(P0, "City of Traitors");
    t.play_land(P0, city).expect("play City of Traitors");
    t.settle();
    assert_eq!(abilities_on_stack(&t), 0);
    t.enter(P0, "Forest");
    t.settle();
    assert_eq!(abilities_on_stack(&t), 0);
    assert_eq!(t.named_on_battlefield("City of Traitors").len(), 1);
    // Playing another land: sacrificed.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).expect("play a Forest");
    t.resolve_all();
    assert!(t.named_on_battlefield("City of Traitors").is_empty());
}
