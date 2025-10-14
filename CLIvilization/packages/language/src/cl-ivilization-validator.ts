import type { ValidationChecks } from 'langium';
import type { ClIvilizationAstType } from './generated/ast.js';
import type { ClIvilizationServices } from './cl-ivilization-module.js';

/**
 * Register custom validation checks.
 */
export function registerValidationChecks(services: ClIvilizationServices) {
    const registry = services.validation.ValidationRegistry;
    const validator = services.validation.ClIvilizationValidator;
    const checks: ValidationChecks<ClIvilizationAstType> = {
        // TODO: Declare validators for your properties
        // See doc : https://langium.org/docs/learn/workflow/create_validations/
        /*
        Element: validator.checkElement
        */
    };
    registry.register(checks, validator);
}

/**
 * Implementation of custom validations.
 */
export class ClIvilizationValidator {

    // TODO: Add logic here for validation checks of properties
    // See doc : https://langium.org/docs/learn/workflow/create_validations/
    /*
    checkElement(element: Element, accept: ValidationAcceptor): void {
        // Always accepts
    }
    */
}
