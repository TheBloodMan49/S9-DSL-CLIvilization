// Include generated sources
include!(concat!(env!("OUT_DIR"), "/ast.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    const sample: &str = include_str!("../../examples/variant_1/game.json");

    #[test]
    fn test_sample() {
        println!(
            "{}",
            match serde_json::to_string_pretty(&Model {
                sections: vec![Section::Game(Game {
                    currentTurn: 0,
                    uiColor: "#FFFFFF".to_string()
                }),
                Section::Cities(Cities {
                    cities: vec![City {
                        blacklist_buildings: None,
                        blacklist_units: None,
                        buildings: BuildingInstanceArray {
                            elements: vec![BuildingInstance {
                                id_building: "caserne".to_string(),
                                level: 0,
                            }],
                        },
                        civilization: "Test".to_string(),
                        color: "#000000".to_string(),
                        name: "Rome".to_string(),
                        nbSlotsBuildings: 2,
                        nbSlotsUnits: 2,
                        playerType: PlayerType::AI,
                        startingResources: 0,
                        units: UnitInstanceArray {
                            units: vec![UnitInstance {
                                id_units: "légionnaire".to_string(),
                            }],
                        },
                        whitelist_buildings: None,
                        whitelist_units: None,
                        x: 0,
                        y: 0,
                    }],
                })]
            }) {
                Ok(json) => json,
                Err(e) => panic!("{}", e),
            }
        );

        println!(
            "{:?}",
            match serde_json::from_str::<Model>(sample) {
                Ok(ast) => ast,
                Err(e) => panic!("{}", e),
            }
        );
    }
}
