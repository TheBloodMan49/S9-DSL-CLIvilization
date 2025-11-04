import { beforeAll, describe, it, expect } from "vitest";
import { createClIvilizationServices } from "../src/clivilization-module.js";
import { parseHelper } from "langium/test";
import { Model } from "../src/index.js";
import { EmptyFileSystem } from "langium";

let services: ReturnType<typeof createClIvilizationServices>;
let parse: ReturnType<typeof parseHelper<Model>>;

beforeAll(async () => {
    services = createClIvilizationServices(EmptyFileSystem);
    parse = parseHelper<Model>(services.ClIvilization);
});

describe('Parsing tests', () => {
    it('parses a complete model without errors and produces expected AST values', async () => {
        const input = `
[size]
x = 10
y = 8

[cities]
CityA {
    x = 1
    y = 2
    color = #ff0000
    starting_resources = 100
    player_type = PLAYER
    civilization = "Rome"
    nb_slots_buildings = 2
    starting_buildings = [ { id_building = barracks level = 1 } ]
    nb_slots_units = 1
    starting_units = [ { id_units = infantry } ]
}

[game]
current_turn = 3
ui_color = #00ff00

[victory_conditions]
nb_turns = 50
resources_spent = 1000

[buildings]
barracks {
    cost = 100
    build_time = 2
    slots = 1
    production = { type = "produce" id_units = 10 amount = 1 time = 2 cost = 50 }
    prerequisites = [ { id_building = "town" } ]
}

[units]
infantry { attack = 5 }
`;

        const result = await parse(input);

        // Basic sanity checks
        expect(result).toBeDefined();
        expect(result.parseResult?.value).toBeDefined();
        expect(Array.isArray((result.parseResult?.value as any).sections)).toBe(true);
        expect((result.parseResult?.value as any).sections.length).toBeGreaterThanOrEqual(6);

        // Check size section values
        const size = (result.parseResult?.value as any).sections.find((s: any) => s && typeof s.x === 'number' && typeof s.y === 'number');
        expect(size).toBeDefined();
        expect(size.x).toBe(10);
        expect(size.y).toBe(8);

        // Check cities section and first city fields
        const citiesSection = (result.parseResult?.value as any).sections.find((s: any) => Array.isArray(s.cities));
        expect(citiesSection).toBeDefined();
        expect(Array.isArray(citiesSection.cities)).toBe(true);
        expect(citiesSection.cities.length).toBeGreaterThanOrEqual(1);
        const city = citiesSection.cities[0];

        expect(city).toBeDefined();
        // name comes from Value (STRING), parser typically stores the raw string value without quotes
        expect(city.name).toBe("CityA");
        expect(city.color).toBe("#ff0000");
        expect(city.startingResources).toBe(100);
        expect(city.playerType).toBe("PLAYER");

        // Check buildings section
        const buildingsSection = (result.parseResult?.value as any).sections.find((s: any) => Array.isArray(s.buildings));
        expect(buildingsSection).toBeDefined();
        expect(buildingsSection.buildings.length).toBeGreaterThanOrEqual(1);
        const building = buildingsSection.buildings[0];
        expect(building.name).toBe("barracks");
        expect(building.cost).toBe(100);
        expect(building.production).toBeDefined();
        expect(building.production.prodUnitId).toBe(10);

        // Ensure there are no parse diagnostics (try common diagnostic fields)
        expect(result.parseResult.parserErrors).toHaveLength(0);
        expect(result.parseResult.lexerErrors).toHaveLength(0);
    });
});
