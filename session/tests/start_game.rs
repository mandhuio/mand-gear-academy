#![allow(warnings)]

use gtest::Log;
use session_io::{Action, Event, GameStatus, State};
use utils::*;

mod utils;

const GAME_IS_PLAYING: &str = "A game is in progress for this user";
const DELAY_CHECK_STATUS_DURATION: u32 = 200;

#[test]
fn start_game_should_success_when_first_time() {
    let system = init_system();
    let ProgramPair { proxy_program, .. } = init_programs(&system);

    // Given: User has never started a game
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    assert!(!players.contains_key(&USER.into()));

    // When: User starts a game
    let result = proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    println!("result: {:?}", result);
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload(Event::GameStarted);
    println!("log: {:?}", log);
    // Then:
    // - User is registered in the game
    // - User's info is valid
    let State { players, .. } = proxy_program.read_state(0).expect("Failed to read state");
    assert!(players.contains_key(&USER.into()));

    let info = players.get(&USER.into()).expect("Failed to read state");
    assert_eq!(info.game_status, GameStatus::InProgress);
    assert_eq!(info.attempts_count, 0);
}

#[test]
fn start_game_should_fail_when_player_is_in_game() {
    let system = init_system();
    let ProgramPair { proxy_program, .. } = init_programs(&system);

    // Given: Game is in progress
    proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(info.game_status, GameStatus::InProgress);

    // When: User starts another game
    let result = proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    // Then: The program reverts with appropriate error message
    let log = Log::builder()
        .source(PROXY_PROGRAM)
        .dest(USER)
        .payload_bytes(final_panic_message(GAME_IS_PLAYING));
    println!("log: {:?}", log);
}

#[test]
fn start_game_should_work_when_available_to_play() {
    let system = init_system();
    let ProgramPair { proxy_program, .. } = init_programs(&system);

    // Given: A game is over
    proxy_program.send(USER, Action::StartGame);
    system.run_to_block(DELAY_CHECK_STATUS_DURATION); // fast-forward to timeout
    system.run_next_block();
    let State { players, .. } = proxy_program.read_state(0).unwrap();
    let info = players.get(&USER.into()).unwrap();
    assert_eq!(
        info.game_status,
        GameStatus::InProgress
    );

    // When: Users start a game again
    let result = proxy_program.send(USER, Action::StartGame);
    system.run_next_block();
    // Then: action should not failed
    println!("result: {:?}", result);
}
