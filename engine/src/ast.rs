// Include generated sources
include!(concat!(env!("OUT_DIR"), "/ast.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    const sample: &str = r##"{
  "sections": [
    {
      "x": 10,
      "y": 10
    },
    {
      "cities": [
        {
          "name": "Rome",
          "x": 1,
          "y": 1,
          "color": "#000000",
          "startingResources": 10,
          "playerType": "PLAYER",
          "civilization": "Test",
          "nbSlotsBuildings": 2,
          "buildings": {
            "elements": [
              {
                "id_building": 1,
                "level": 0
              }
            ]
          },
          "blacklist_buildings": {
            "values": []
          },
          "nbSlotsUnits": 2,
          "units": {
            "units": [
              {
                "id_units": 2
              }
            ]
          },
          "blacklist_units": {
            "values": []
          }
        },
        {
          "name": "Athene",
          "x": 1,
          "y": 1,
          "color": "#000000",
          "startingResources": 10
        },
        {
          "name": "Test"
        }
      ]
    },
    {
      "currentTurn": 0,
      "uiColor": "#FFFFFF"
    },
    {
      "nbTurns": 100,
      "resourcesSpent": 1000
    },
    {
      "buildings": [
        {
          "name": "caserne",
          "cost": 2,
          "buildTime": 3,
          "slots": 1,
          "production": {
            "prodType": "unit",
            "prodUnitId": 2,
            "amount": 1,
            "time": 2,
            "cost": 3
          },
          "prerequisites": {
            "prereqs": []
          }
        },
        {
          "name": "caserne2",
          "cost": 2,
          "buildTime": 3,
          "slots": 1,
          "production": {
            "prodType": "unit",
            "prodUnitId": 2,
            "amount": 1,
            "time": 2,
            "cost": 3
          },
          "prerequisites": {
            "prereqs": []
          }
        }
      ]
    },
    {
      "units": [
        {
          "name": "légionnaire",
          "attack": 1
        },
        {
          "name": "légionnaire",
          "attack": 1
        }
      ]
    }
  ]
}"##;

    #[test]
    fn test_sample() {
        println!(
            "{}",
            match serde_json::to_string(&Model {
                sections: vec![Section::Game(Game {
                    currentTurn: 0,
                    uiColor: "#FFFFFF".to_string()
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
