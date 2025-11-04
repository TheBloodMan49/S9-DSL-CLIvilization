import type { ValidationChecks, ValidationAcceptor } from 'langium';
import type { ClIvilizationAstType, City, Model } from './generated/ast.js';
import type { ClIvilizationServices } from './clivilization-module.js';

/**
 * Register custom validation checks.
 */
export function registerValidationChecks(services: ClIvilizationServices) {
    const registry = services.validation.ValidationRegistry;
    const validator = services.validation.ClIvilizationValidator;
    const checks: ValidationChecks<ClIvilizationAstType> = {
        // Validate each city
        City: validator.checkCity,
        // Validate model-wide constraints (unique names)
        Model: validator.checkModel
    };
    registry.register(checks, validator);
}

/**
 * Implementation of custom validations.
 */
export class ClIvilizationValidator {

    // Check that every starting building referenced in a city exists in the [buildings] section
    checkCity(city: City, accept: ValidationAcceptor): void {
        // Find model root by walking up $container
        let root: any = city;
        while (root && root.$container) {
            root = root.$container;
        }
        const model = root as Model | undefined;
        if (!model || !Array.isArray(model.sections)) {
            return;
        }

        // Collect all building names defined in [buildings] sections
        const definedBuildingNames = new Set<string>();
        for (const sec of model.sections) {
            if (sec && Array.isArray((sec as any).buildings)) {
                for (const b of (sec as any).buildings) {
                    if (b && typeof (b as any).name === 'string') {
                        definedBuildingNames.add((b as any).name as string);
                    }
                }
            }
        }

        // Collect all unit names defined in [units] sections
        const definedUnitNames = new Set<string>();
        for (const sec of model.sections) {
            if (sec && Array.isArray((sec as any).units)) {
                for (const u of (sec as any).units) {
                    if (u && typeof (u as any).name === 'string') {
                        definedUnitNames.add((u as any).name as string);
                    }
                }
            }
        }

        // Get starting buildings instances from the city (property name 'buildings' as per grammar)
        const buildingArray = (city as any).buildings;
        const buildingElements = Array.isArray(buildingArray?.elements) ? buildingArray.elements : [];

        for (const inst of buildingElements) {
            const id = inst?.id_building;
            if (typeof id === 'string' && !definedBuildingNames.has(id)) {
                // Report a validation error on the id_building property of the instance
                accept('error', `Building '${id}' used in city '${(city as any).name}' is not defined in [buildings] section.`, {
                    node: inst,
                });
            }
        }

        // Get starting unit instances from the city (UnitInstanceArray uses property 'units' for elements)
        const unitArray = (city as any).units;
        const unitElements = Array.isArray(unitArray?.units) ? unitArray.units : [];

        for (const inst of unitElements) {
            const id = inst?.id_units;
            // id_units may be parsed as number (INT) or string depending on grammar/usage; normalize to string for lookup
            if (id !== undefined && id !== null) {
                const idStr = String(id);
                if (!definedUnitNames.has(idStr)) {
                    accept('error', `Unit '${idStr}' used in city '${(city as any).name}' is not defined in [units] section.`, {
                        node: inst,
                    });
                }
            }
        }

        // Validation: a city must not declare both whitelist and blacklist at the same time
        // for buildings and for units.
        const hasWhitelistBuildings = (city as any).whitelist_buildings !== undefined;
        const hasBlacklistBuildings = (city as any).blacklist_buildings !== undefined;
        if (hasWhitelistBuildings && hasBlacklistBuildings) {
            accept('error', `City '${(city as any).name}' cannot have both 'whitelist_buildings' and 'blacklist_buildings'. Choose at most one.`, {
                node: city
            });
        }

        const hasWhitelistUnits = (city as any).whitelist_units !== undefined;
        const hasBlacklistUnits = (city as any).blacklist_units !== undefined;
        if (hasWhitelistUnits && hasBlacklistUnits) {
            accept('error', `City '${(city as any).name}' cannot have both 'whitelist_units' and 'blacklist_units'. Choose at most one.`, {
                node: city
            });
        }
    }

    // Model-level checks: unique city names, unique building names, unique unit names
    checkModel(model: Model, accept: ValidationAcceptor): void {
        if (!model || !Array.isArray(model.sections)) {
            return;
        }

        const cityNameMap = new Map<string, any[]>();
        const buildingNameMap = new Map<string, any[]>();
        const unitNameMap = new Map<string, any[]>();

        for (const sec of model.sections) {
            // collect cities
            if (sec && Array.isArray((sec as any).cities)) {
                for (const c of (sec as any).cities) {
                    const name = typeof c?.name === 'string' ? c.name : undefined;
                    if (name) {
                        const arr = cityNameMap.get(name) ?? [];
                        arr.push(c);
                        cityNameMap.set(name, arr);
                    }
                }
            }

            // collect building definitions
            if (sec && Array.isArray((sec as any).buildings)) {
                for (const b of (sec as any).buildings) {
                    const name = typeof b?.name === 'string' ? b.name : undefined;
                    if (name) {
                        const arr = buildingNameMap.get(name) ?? [];
                        arr.push(b);
                        buildingNameMap.set(name, arr);
                    }
                }
            }

            // collect unit definitions
            if (sec && Array.isArray((sec as any).units)) {
                for (const u of (sec as any).units) {
                    const name = typeof u?.name === 'string' ? u.name : undefined;
                    if (name) {
                        const arr = unitNameMap.get(name) ?? [];
                        arr.push(u);
                        unitNameMap.set(name, arr);
                    }
                }
            }
        }

        // report duplicate city names
        for (const [name, arr] of cityNameMap) {
            if (arr.length > 1) {
                for (const dup of arr.slice(1)) {
                    accept('error', `Duplicate city name '${name}'. City names must be unique.`, { node: dup });
                }
            }
        }

        // report duplicate building definition names
        for (const [name, arr] of buildingNameMap) {
            if (arr.length > 1) {
                for (const dup of arr.slice(1)) {
                    accept('error', `Duplicate building name '${name}'. Building names must be unique.`, { node: dup });
                }
            }
        }

        // report duplicate unit definition names
        for (const [name, arr] of unitNameMap) {
            if (arr.length > 1) {
                for (const dup of arr.slice(1)) {
                    accept('error', `Duplicate unit name '${name}'. Unit names must be unique.`, { node: dup });
                }
            }
        }
    }
}