//! ClickHouse-specific table diffing strategy
//!
//! This module implements the TableDiffStrategy for ClickHouse, handling the database's
//! specific limitations around schema changes. ClickHouse has restrictions on certain
//! ALTER TABLE operations, particularly around ORDER BY and primary key changes.

use crate::framework::core::infrastructure::table::Table;
use crate::framework::core::infrastructure_map::{
    ColumnChange, OlapChange, OrderByChange, TableChange, TableDiffStrategy,
};

/// ClickHouse-specific table diff strategy
///
/// ClickHouse has several limitations that require drop+create operations instead of ALTER:
/// - Cannot change ORDER BY clause via ALTER TABLE
/// - Cannot change primary key structure via ALTER TABLE
/// - Some column type changes are not supported
///
/// This strategy identifies these cases and converts table updates into drop+create operations
/// so that users see the actual operations that will be performed.
pub struct ClickHouseTableDiffStrategy;

impl TableDiffStrategy for ClickHouseTableDiffStrategy {
    /// This function is only called when there are actual changes to the table
    /// (column changes, ORDER BY changes, or deduplication changes).
    /// It determines whether those changes can be handled via ALTER TABLE
    /// or require a drop+create operation.
    fn diff_table_update(
        &self,
        before: &Table,
        after: &Table,
        column_changes: Vec<ColumnChange>,
        order_by_change: OrderByChange,
    ) -> Vec<OlapChange> {
        // Check if ORDER BY has changed
        let order_by_changed = order_by_change.before != order_by_change.after;
        if order_by_changed {
            log::debug!(
                "ClickHouse: ORDER BY changed for table '{}', requiring drop+create",
                before.name
            );
            return vec![
                OlapChange::Table(TableChange::Removed(before.clone())),
                OlapChange::Table(TableChange::Added(after.clone())),
            ];
        }

        // Check if primary key structure has changed
        let before_primary_keys = before.primary_key_columns();
        let after_primary_keys = after.primary_key_columns();
        if before_primary_keys != after_primary_keys {
            log::debug!(
                "ClickHouse: Primary key structure changed for table '{}', requiring drop+create",
                before.name
            );
            return vec![
                OlapChange::Table(TableChange::Removed(before.clone())),
                OlapChange::Table(TableChange::Added(after.clone())),
            ];
        }

        // Check if deduplication setting changed (affects engine)
        if before.deduplicate != after.deduplicate {
            log::debug!(
                "ClickHouse: Deduplication setting changed for table '{}', requiring drop+create",
                before.name
            );
            return vec![
                OlapChange::Table(TableChange::Removed(before.clone())),
                OlapChange::Table(TableChange::Added(after.clone())),
            ];
        }

        // For other changes, ClickHouse can handle them via ALTER TABLE.
        // If there are no column changes, return an empty vector since
        // we've already handled all the cases that require drop+create.
        if column_changes.is_empty() {
            vec![]
        } else {
            vec![OlapChange::Table(TableChange::Updated {
                name: before.name.clone(),
                column_changes,
                order_by_change,
                before: before.clone(),
                after: after.clone(),
            })]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::infrastructure::table::{Column, ColumnType};
    use crate::framework::core::infrastructure_map::{PrimitiveSignature, PrimitiveTypes};
    use crate::framework::core::partial_infrastructure_map::LifeCycle;
    use crate::framework::versions::Version;

    fn create_test_table(name: &str, order_by: Vec<String>, deduplicate: bool) -> Table {
        Table {
            name: name.to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: true,
                    default: None,
                    annotations: vec![],
                },
                Column {
                    name: "timestamp".to_string(),
                    data_type: ColumnType::String,
                    required: true,
                    unique: false,
                    primary_key: false,
                    default: None,
                    annotations: vec![],
                },
            ],
            order_by,
            deduplicate,
            engine: None,
            version: Some(Version::from_string("1.0.0".to_string())),
            source_primitive: PrimitiveSignature {
                name: "test".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
            metadata: None,
            life_cycle: LifeCycle::FullyManaged,
        }
    }

    #[test]
    fn test_order_by_change_requires_drop_create() {
        let strategy = ClickHouseTableDiffStrategy;

        let before = create_test_table("test", vec!["id".to_string()], false);
        let after = create_test_table(
            "test",
            vec!["id".to_string(), "timestamp".to_string()],
            false,
        );

        let order_by_change = OrderByChange {
            before: vec!["id".to_string()],
            after: vec!["id".to_string(), "timestamp".to_string()],
        };

        let changes = strategy.diff_table_update(&before, &after, vec![], order_by_change);

        assert_eq!(changes.len(), 2);
        assert!(matches!(
            changes[0],
            OlapChange::Table(TableChange::Removed(_))
        ));
        assert!(matches!(
            changes[1],
            OlapChange::Table(TableChange::Added(_))
        ));
    }

    #[test]
    fn test_deduplication_change_requires_drop_create() {
        let strategy = ClickHouseTableDiffStrategy;

        let before = create_test_table("test", vec!["id".to_string()], false);
        let after = create_test_table("test", vec!["id".to_string()], true);

        let order_by_change = OrderByChange {
            before: vec![],
            after: vec![],
        };

        let changes = strategy.diff_table_update(&before, &after, vec![], order_by_change);

        assert_eq!(changes.len(), 2);
        assert!(matches!(
            changes[0],
            OlapChange::Table(TableChange::Removed(_))
        ));
        assert!(matches!(
            changes[1],
            OlapChange::Table(TableChange::Added(_))
        ));
    }

    #[test]
    fn test_column_only_changes_use_alter() {
        let strategy = ClickHouseTableDiffStrategy;

        let before = create_test_table("test", vec!["id".to_string()], false);
        let after = create_test_table("test", vec!["id".to_string()], false);

        let column_changes = vec![ColumnChange::Added {
            column: Column {
                name: "new_col".to_string(),
                data_type: ColumnType::String,
                required: false,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
            },
            position_after: Some("timestamp".to_string()),
        }];

        let order_by_change = OrderByChange {
            before: vec![],
            after: vec![],
        };

        let changes = strategy.diff_table_update(&before, &after, column_changes, order_by_change);

        assert_eq!(changes.len(), 1);
        assert!(matches!(
            changes[0],
            OlapChange::Table(TableChange::Updated { .. })
        ));
    }

    #[test]
    fn test_identical_order_by_with_column_change_uses_alter() {
        let strategy = ClickHouseTableDiffStrategy;

        let before = create_test_table(
            "test",
            vec!["id".to_string(), "timestamp".to_string()],
            false,
        );
        let after = create_test_table(
            "test",
            vec!["id".to_string(), "timestamp".to_string()],
            false,
        );

        // Add a column change to make this a realistic scenario
        let column_changes = vec![ColumnChange::Added {
            column: Column {
                name: "status".to_string(),
                data_type: ColumnType::String,
                required: false,
                unique: false,
                primary_key: false,
                default: None,
                annotations: vec![],
            },
            position_after: Some("timestamp".to_string()),
        }];

        let order_by_change = OrderByChange {
            before: vec!["id".to_string(), "timestamp".to_string()],
            after: vec!["id".to_string(), "timestamp".to_string()],
        };

        let changes = strategy.diff_table_update(&before, &after, column_changes, order_by_change);

        // With identical ORDER BY but column changes, should use ALTER (not drop+create)
        assert_eq!(changes.len(), 1);
        assert!(matches!(
            changes[0],
            OlapChange::Table(TableChange::Updated { .. })
        ));
    }

    #[test]
    fn test_no_changes_returns_empty_vector() {
        let strategy = ClickHouseTableDiffStrategy;

        let before = create_test_table(
            "test",
            vec!["id".to_string(), "timestamp".to_string()],
            false,
        );
        let after = create_test_table(
            "test",
            vec!["id".to_string(), "timestamp".to_string()],
            false,
        );

        // No column changes
        let column_changes = vec![];

        let order_by_change = OrderByChange {
            before: vec!["id".to_string(), "timestamp".to_string()],
            after: vec!["id".to_string(), "timestamp".to_string()],
        };

        let changes = strategy.diff_table_update(&before, &after, column_changes, order_by_change);

        // With no actual changes, should return empty vector
        assert_eq!(changes.len(), 0);
    }

    #[test]
    fn test_order_by_change_with_no_column_changes_requires_drop_create() {
        let strategy = ClickHouseTableDiffStrategy;

        let before = create_test_table("test", vec!["id".to_string()], false);
        let after = create_test_table("test", vec!["timestamp".to_string()], false);

        // No column changes, but ORDER BY changes
        let column_changes = vec![];
        let order_by_change = OrderByChange {
            before: vec!["id".to_string()],
            after: vec!["timestamp".to_string()],
        };

        let changes = strategy.diff_table_update(&before, &after, column_changes, order_by_change);

        // Should still require drop+create even with no column changes
        assert_eq!(changes.len(), 2);
        assert!(matches!(
            changes[0],
            OlapChange::Table(TableChange::Removed(_))
        ));
        assert!(matches!(
            changes[1],
            OlapChange::Table(TableChange::Added(_))
        ));
    }
}
