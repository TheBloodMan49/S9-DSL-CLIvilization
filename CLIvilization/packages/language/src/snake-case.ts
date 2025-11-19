import { DefaultNameProvider } from 'langium';

export class SnakeCaseNameProvider extends DefaultNameProvider {
    /**
     * Override this to keep the original name instead of converting to camelCase.
     */
    override getName(node: any): string | undefined {
        const name = super.getName(node);
        // Return name as-is (prevent camelCase transformation)
        return name;
    }
}