/**
 * Defines how Moose manages the lifecycle of database resources when your code changes.
 *
 * This enum controls the behavior when there are differences between your code definitions
 * and the actual database schema or structure.
 */
export enum LifeCycle {
  /**
   * Full automatic management (default behavior).
   * Moose will automatically modify database resources to match your code definitions,
   * including potentially destructive operations like dropping columns or tables.
   */
  FULLY_MANAGED = "FULLY_MANAGED",

  /**
   * Deletion-protected automatic management.
   * Moose will modify resources to match your code but will avoid destructive actions
   * such as dropping columns, or tables. Only additive changes are applied.
   */
  DELETION_PROTECTED = "DELETION_PROTECTED",

  /**
   * External management - no automatic changes.
   * Moose will not modify the database resources. You are responsible for managing
   * the schema and ensuring it matches your code definitions manually.
   */
  EXTERNALLY_MANAGED = "EXTERNALLY_MANAGED",
}
