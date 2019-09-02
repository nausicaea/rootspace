//! A Scene provides (de)-serializable access to game data.

use serde::{Serialize, Deserialize};

/// A Scene provides (de)-serializable access to game data.
#[derive(Default, Serialize, Deserialize)]
pub struct Scene<W, E, G> {
    world_data: W,
    engine_data: E,
    game_data: G,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Serialize, Deserialize)]
    struct WorldState;

    #[derive(Default, Serialize, Deserialize)]
    struct EngineState;

    #[derive(Default, Serialize, Deserialize)]
    struct GameState;

    #[test]
    fn default() {
        let _: Scene<WorldState, EngineState, GameState> = Default::default();
    }

    #[test]
    fn serialize() {
        let scene: Scene<WorldState, EngineState, GameState> = Scene::default();
        let result = serde_json::to_string(&scene).unwrap();
        assert_eq!(result, "{\"world_data\":null,\"engine_data\":null,\"game_data\":null}");
    }

    #[test]
    fn deserialize() {
        let _: Scene<WorldState, EngineState, GameState> = serde_json::from_str("{\"world_data\":null,\"engine_data\":null,\"game_data\":null}").unwrap();
    }
}
