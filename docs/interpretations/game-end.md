# Ending the game, loops and limits

Judgment calls about how games end: conceding, loops (CR 104, 732) and the simulation's
own limits. Format and kinds: `docs/agents/items/interpretation-register.md`; index:
`docs/INTERPRETATIONS.md`.

## mandatory-loop-detection: A loop of mandatory actions is recognized when the game state repeats
- Kind: interpretation
- Status: decided
- Rules: 104.4b "If a game that’s not using the limited range of influence option (including a two-player game) somehow enters a “loop” of mandatory actions, repeating a sequence of events with no way to stop, the game is a draw. Loops that contain an optional action don’t result in a draw." / 732.4 "If a loop contains only mandatory actions, the game is a draw. (See rules 104.4b and 104.4f.)" / 732.5 "No player can be forced to perform an action that would end a loop other than actions called for by objects involved in the loop." / 732.6 "If a loop contains an effect that says “[A] unless [B],” where [A] and [B] are each actions, no player can be forced to perform [B] to break the loop. If no player chooses to perform [B], the loop will continue as though [A] were mandatory." / 732.3 "Sometimes a loop can be fragmented, meaning that each player involved in the loop performs an independent action that results in the same game state being reached multiple times. If that happens, the active player (or, if the active player is not involved in the loop, the first player in turn order who is involved) must then make a different game choice so the loop does not continue."
- Question: The engine can't see "a loop" directly. When has the game entered one, and which choices along the way count as optional actions that keep it from being a draw?
- Decision: While something is on the stack and players only pass priority, the engine fingerprints the game state each time a player receives priority; after a stretch of at least 8 such passes, the third occurrence of the same state is a loop of mandatory actions and the game is a draw (with the limited range of influence option, only for the players involved, 104.4f). Passing priority while other actions are available, declining the [B] of an "[A] unless [B]" effect, and choices the looping objects call for (the targets of a mandatory trigger, the order of simultaneous triggers) don't interrupt the stretch; any other action does. A fragmented loop of optional actions (732.3) is likewise recognized when the same state at priority comes up a third time in a step; the active player (or the first involved player in turn order) must then choose differently.
- Alternatives: Count any available but untaken action as an optional action that prevents the draw (104.4b's last sentence read literally): a player holding an instant could keep a mandatory loop going forever. Or recognize loops by repeated event sequences instead of states: finds more loops, including ones that grow the state (see `growing-loops-run-to-the-limit`), but needs a model of "the same sequence".
- Why: 732.5 and 732.6 say a player can't be forced to break a loop, so passing or declining keeps it mandatory; a repeated state at priority with the same objects on the stack is the observable form of "repeating a sequence of events with no way to stop". The thresholds (8 passes, 3 repeats) only keep ordinary stretches of passing from being fingerprinted.
- Affects: every game with a loop of triggered or replacement effects (Faceless Devourer chains, "return it unless you pay" loops, ...).
- Code: `crates/mtg-engine/src/game_end.rs` (`check_mandatory_loop`, `LOOP_REPEATS`, `LOOP_MIN_LEN`, `note_decision`); `crates/mtg-engine/src/shortcuts.rs` (`priority_decision`, `FRAGMENTED_REPEATS`).
- Tests: `tests/cr/r732_shortcuts.rs` (`a_loop_of_only_mandatory_actions_is_a_draw`, `no_player_is_forced_to_end_a_loop`, `a_loop_with_a_declined_unless_payment_continues_as_though_mandatory`, `a_fragmented_loop_is_broken_by_the_active_player`, `a_fragmented_loop_is_broken_by_the_first_player_in_turn_order_involved_in_it`); `tests/cr/r104_loop_with_targets.rs` (`a_loop_of_targeted_triggered_abilities_is_a_draw`).

## growing-loops-run-to-the-limit: A loop that grows the game state isn't recognized and ends only at the action limit
- Kind: deviation
- Status: decided
- Rules: 104.4b (see `mandatory-loop-detection`) / 732.6 "If a loop contains an effect that says “[A] unless [B],” ... If no player chooses to perform [B], the loop will continue as though [A] were mandatory."
- Question: With two Charismatic Conquerors on each side ("Whenever an artifact or creature an opponent controls enters untapped, they may tap that permanent. If they don't, you create a 1/1 white Vampire creature token with flying."), each entering token triggers both of the opponent's Conquerors and only one of them can tap it, so every token makes another one for the other side. The optional action (tapping) can't stop the chain. Is that a draw, and when?
- Decision: The engine recognizes loops only when the game state repeats, and here it never does (there are more tokens each time around). The game goes on until it reaches the configured action limit (`simulation-limits`) and ends in a draw there.
- Alternatives: Treat it as a loop of mandatory actions under 104.4b ("with no way to stop") and 732.6 by analogy, and end the game in a draw as soon as the pattern is recognized: needs loop recognition by repeated event sequences rather than states. Or treat it as not a loop at all (the tap is an optional action, 104.4b's last sentence): the game could never end, which the limits turn into a draw anyway.
- Why: Each trigger resolves correctly; only the game's ending is open. The result (a draw) matches the most likely ruling, reached late and slowly (tokens pile up and every action gets slower, so these games hit mtg-sim's 60 s watchdog; see `docs/EVERY_CARD.md`). Decided by the user, Oct 21: an unendable loop ends the game in a draw (104.4b), and reaching that draw at the action limit is accepted for now. Loops and loop detection will be revisited once the rules work is settled: recognizing such loops early, and automating loops and shortcuts (CR 732), which a future UI will need. The action item is in `docs/agents/verification-plan.md` under "Later".
- Affects: Charismatic Conqueror with two or more on each side; other symmetric token loops (count not measured).
- Code: `crates/mtg-engine/src/game_end.rs` (`check_mandatory_loop` fingerprints states); `crates/mtg-engine/src/turn.rs` (`Game::run`: the action limit).
- Tests: `tests/cr/interp_engine_policies.rs` (`a_loop_that_grows_the_game_state_runs_until_the_action_limit`).

## simulation-limits: A game past the turn or decision limit is a draw
- Kind: policy
- Status: decided
- Rules: 104.4a "If all the players remaining in a game lose simultaneously, the game is a draw." (the CR has no turn or time limit; tournament time limits are outside the game rules)
- Question: The CR lets a game go on indefinitely. A simulator needs every game to end.
- Decision: `GameConfig::max_turns` (default 200) and `max_actions` (decisions asked, default 200,000) bound every game: when either is passed, `Game::run` declares the game a draw. mtg-sim uses 100 turns by default and the every-card fuzzer 30 turns and 30,000 decisions; both take `--max-turns` and `--max-actions`.
- Alternatives: Decide the game by life totals or another tiebreak (like a tournament's end-of-round procedure); or stop without a result. A draw records the least.
- Why: A safety valve for games that would not end (undetected loops, agents that never attack); reported as a draw so statistics don't credit either player.
- Affects: games that reach the limits.
- Code: `crates/mtg-engine/src/turn.rs` (`Game::run`); `crates/mtg-engine/src/game.rs` (`GameConfig::max_turns`, `max_actions`, `draw_game`); `crates/mtg-sim/src/main.rs`, `crates/mtg-sim/src/every_card.rs`.
- Tests: `tests/cr/interp_engine_policies.rs` (`a_game_past_the_turn_or_action_limit_is_a_draw`).

## concede-with-priority: A player concedes only as a priority action
- Kind: deviation
- Status: decided
- Rules: 104.3a "A player can concede the game at any time. A player who concedes leaves the game immediately. That player loses the game." / 723.6 "The controller of another player can’t make that player concede. A player may concede the game at any time, even if they are controlled by another player. See rule 104.3a."
- Question: The CR lets a player concede at any time, including in the middle of a spell's resolution or while making another choice. Agents answer one decision at a time.
- Decision: Conceding is the priority action `Action::Concede`; it is accepted whenever the player has priority (mtg-api lists it with the legal actions; the engine's own `legal_actions` leaves it out), and isn't available as the answer to any other decision. A controlling player's concession for the player they control is ignored (723.6); the controlled player's own agent isn't asked anything while they are controlled, so they can't concede until control ends.
- Alternatives: Let any decision be answered with a concession: exact, but every decision would have to accept it and the engine would have to unwind mid-resolution.
- Why: Accepted: conceding a moment later, at the next priority, never changes the outcome for the conceding player, and the agent interface stays one answer per decision. The controlled-player gap (723.6's second sentence) would need a way to reach an agent that isn't being asked; no agent needs it today.
- Affects: agents that concede.
- Code: `crates/mtg-engine/src/turn.rs` (`take_action`, `Action::Concede`); `crates/mtg-api/src/legal.rs`; `crates/mtg-engine/src/player_control.rs`.
- Tests: `tests/cr/interp_engine_policies.rs` (`a_player_concedes_by_answering_a_priority_decision`).
