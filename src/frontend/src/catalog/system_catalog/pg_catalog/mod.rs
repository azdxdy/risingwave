// Copyright 2023 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod pg_am;
pub mod pg_attrdef;
pub mod pg_attribute;
pub mod pg_cast;
pub mod pg_class;
pub mod pg_collation;
pub mod pg_constraint;
pub mod pg_conversion;
pub mod pg_database;
pub mod pg_description;
pub mod pg_enum;
pub mod pg_index;
pub mod pg_indexes;
pub mod pg_inherits;
pub mod pg_keywords;
pub mod pg_matviews;
pub mod pg_namespace;
pub mod pg_opclass;
pub mod pg_operator;
pub mod pg_proc;
pub mod pg_roles;
pub mod pg_settings;
pub mod pg_shdescription;
pub mod pg_stat_activity;
pub mod pg_tables;
pub mod pg_tablespace;
pub mod pg_type;
pub mod pg_user;
pub mod pg_views;

use itertools::Itertools;
pub use pg_am::*;
pub use pg_attrdef::*;
pub use pg_attribute::*;
pub use pg_cast::*;
pub use pg_class::*;
pub use pg_collation::*;
pub use pg_constraint::*;
pub use pg_conversion::*;
pub use pg_database::*;
pub use pg_description::*;
pub use pg_enum::*;
pub use pg_index::*;
pub use pg_indexes::*;
pub use pg_inherits::*;
pub use pg_keywords::*;
pub use pg_matviews::*;
pub use pg_namespace::*;
pub use pg_opclass::*;
pub use pg_operator::*;
pub use pg_proc::*;
pub use pg_roles::*;
pub use pg_settings::*;
pub use pg_shdescription::*;
pub use pg_stat_activity::*;
pub use pg_tables::*;
pub use pg_tablespace::*;
pub use pg_type::*;
pub use pg_user::*;
pub use pg_views::*;
use risingwave_common::array::ListValue;
use risingwave_common::catalog::PG_CATALOG_SCHEMA_NAME;
use risingwave_common::error::Result;
use risingwave_common::row::OwnedRow;
use risingwave_common::types::ScalarImpl;
use risingwave_common::util::iter_util::ZipEqDebug;
use risingwave_pb::user::grant_privilege::Object;
use serde_json::json;

use super::SysCatalogReaderImpl;
use crate::catalog::schema_catalog::SchemaCatalog;
use crate::catalog::system_catalog::get_acl_items;

impl SysCatalogReaderImpl {
    pub(super) fn read_types(&self) -> Result<Vec<OwnedRow>> {
        let schema_id = self
            .catalog_reader
            .read_guard()
            .get_schema_by_name(&self.auth_context.database, PG_CATALOG_SCHEMA_NAME)?
            .id();
        Ok(get_pg_type_data(schema_id))
    }

    pub(super) fn read_cast(&self) -> Result<Vec<OwnedRow>> {
        Ok(PG_CAST_DATA_ROWS.clone())
    }

    pub(super) fn read_namespace(&self) -> Result<Vec<OwnedRow>> {
        let schemas = self
            .catalog_reader
            .read_guard()
            .get_all_schema_info(&self.auth_context.database)?;
        let user_reader = self.user_info_reader.read_guard();
        let users = user_reader.get_all_users();
        let username_map = user_reader.get_user_name_map();
        Ok(schemas
            .iter()
            .map(|schema| {
                OwnedRow::new(vec![
                    Some(ScalarImpl::Int32(schema.id as i32)),
                    Some(ScalarImpl::Utf8(schema.name.clone().into())),
                    Some(ScalarImpl::Int32(schema.owner as i32)),
                    Some(ScalarImpl::Utf8(
                        get_acl_items(&Object::SchemaId(schema.id), &users, username_map).into(),
                    )),
                ])
            })
            .collect_vec())
    }

    pub(super) fn read_user_info(&self) -> Result<Vec<OwnedRow>> {
        let reader = self.user_info_reader.read_guard();
        let users = reader.get_all_users();
        Ok(users
            .iter()
            .map(|user| {
                OwnedRow::new(vec![
                    Some(ScalarImpl::Int32(user.id as i32)),
                    Some(ScalarImpl::Utf8(user.name.clone().into())),
                    Some(ScalarImpl::Bool(user.can_create_db)),
                    Some(ScalarImpl::Bool(user.is_super)),
                    // compatible with PG.
                    Some(ScalarImpl::Utf8("********".into())),
                ])
            })
            .collect_vec())
    }

    pub(super) fn read_table_stats(&self) -> Result<Vec<OwnedRow>> {
        let catalog = self.catalog_reader.read_guard();
        let table_stats = catalog.table_stats();
        let mut rows = vec![];
        for (id, stats) in &table_stats.table_stats {
            rows.push(OwnedRow::new(vec![
                Some(ScalarImpl::Int32(*id as i32)),
                Some(ScalarImpl::Int64(stats.total_key_count)),
                Some(ScalarImpl::Int64(stats.total_key_size)),
                Some(ScalarImpl::Int64(stats.total_value_size)),
            ]));
        }
        Ok(rows)
    }

    // FIXME(noel): Tracked by <https://github.com/risingwavelabs/risingwave/issues/3431#issuecomment-1164160988>
    pub(super) fn read_opclass_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    // FIXME(noel): Tracked by <https://github.com/risingwavelabs/risingwave/issues/3431#issuecomment-1164160988>
    pub(super) fn read_operator_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    // FIXME(noel): Tracked by <https://github.com/risingwavelabs/risingwave/issues/3431#issuecomment-1164160988>
    pub(super) fn read_am_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    // FIXME(noel): Tracked by <https://github.com/risingwavelabs/risingwave/issues/3431#issuecomment-1164160988>
    pub(super) fn read_collation_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    pub(super) fn read_attrdef_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    pub(crate) fn read_shdescription_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    pub(crate) fn read_enum_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    pub(super) fn read_roles_info(&self) -> Result<Vec<OwnedRow>> {
        let reader = self.user_info_reader.read_guard();
        let users = reader.get_all_users();
        Ok(users
            .iter()
            .map(|user| {
                OwnedRow::new(vec![
                    Some(ScalarImpl::Int32(user.id as i32)),
                    Some(ScalarImpl::Utf8(user.name.clone().into())),
                    Some(ScalarImpl::Bool(user.is_super)),
                    Some(ScalarImpl::Bool(true)),
                    Some(ScalarImpl::Bool(user.can_create_user)),
                    Some(ScalarImpl::Bool(user.can_create_db)),
                    Some(ScalarImpl::Bool(user.can_login)),
                    Some(ScalarImpl::Utf8("********".into())),
                ])
            })
            .collect_vec())
    }

    pub(super) fn read_class_info(&self) -> Result<Vec<OwnedRow>> {
        let reader = self.catalog_reader.read_guard();
        let schemas = reader.iter_schemas(&self.auth_context.database)?;
        let schema_infos = reader.get_all_schema_info(&self.auth_context.database)?;

        Ok(schemas
            .zip_eq_debug(schema_infos.iter())
            .flat_map(|(schema, schema_info)| {
                // !!! If we need to add more class types, remember to update
                // Catalog::get_id_by_class_name_inner accordingly.

                let rows = schema
                    .iter_table()
                    .map(|table| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(table.id.table_id() as i32)),
                            Some(ScalarImpl::Utf8(table.name.clone().into())),
                            Some(ScalarImpl::Int32(schema_info.id as i32)),
                            Some(ScalarImpl::Int32(table.owner as i32)),
                            Some(ScalarImpl::Utf8("r".into())),
                            Some(ScalarImpl::Int32(0)),
                            Some(ScalarImpl::Int32(0)),
                        ])
                    })
                    .collect_vec();

                let mvs = schema
                    .iter_mv()
                    .map(|mv| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(mv.id.table_id() as i32)),
                            Some(ScalarImpl::Utf8(mv.name.clone().into())),
                            Some(ScalarImpl::Int32(schema_info.id as i32)),
                            Some(ScalarImpl::Int32(mv.owner as i32)),
                            Some(ScalarImpl::Utf8("m".into())),
                            Some(ScalarImpl::Int32(0)),
                            Some(ScalarImpl::Int32(0)),
                        ])
                    })
                    .collect_vec();

                let indexes = schema
                    .iter_index()
                    .map(|index| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(index.index_table.id.table_id as i32)),
                            Some(ScalarImpl::Utf8(index.name.clone().into())),
                            Some(ScalarImpl::Int32(schema_info.id as i32)),
                            Some(ScalarImpl::Int32(index.index_table.owner as i32)),
                            Some(ScalarImpl::Utf8("i".into())),
                            Some(ScalarImpl::Int32(0)),
                            Some(ScalarImpl::Int32(0)),
                        ])
                    })
                    .collect_vec();

                let sources = schema
                    .iter_source()
                    .map(|source| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(source.id as i32)),
                            Some(ScalarImpl::Utf8(source.name.clone().into())),
                            Some(ScalarImpl::Int32(schema_info.id as i32)),
                            Some(ScalarImpl::Int32(source.owner as i32)),
                            Some(ScalarImpl::Utf8("x".into())),
                            Some(ScalarImpl::Int32(0)),
                            Some(ScalarImpl::Int32(0)),
                        ])
                    })
                    .collect_vec();

                let sys_tables = schema
                    .iter_system_tables()
                    .map(|table| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(table.id.table_id() as i32)),
                            Some(ScalarImpl::Utf8(table.name.clone().into())),
                            Some(ScalarImpl::Int32(schema_info.id as i32)),
                            Some(ScalarImpl::Int32(table.owner as i32)),
                            Some(ScalarImpl::Utf8("r".into())),
                            Some(ScalarImpl::Int32(0)),
                            Some(ScalarImpl::Int32(0)),
                        ])
                    })
                    .collect_vec();

                let views = schema
                    .iter_view()
                    .map(|view| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(view.id as i32)),
                            Some(ScalarImpl::Utf8(view.name().into())),
                            Some(ScalarImpl::Int32(schema_info.id as i32)),
                            Some(ScalarImpl::Int32(view.owner as i32)),
                            Some(ScalarImpl::Utf8("v".into())),
                            Some(ScalarImpl::Int32(0)),
                            Some(ScalarImpl::Int32(0)),
                        ])
                    })
                    .collect_vec();

                let internal_tables = schema
                    .iter_internal_table()
                    .map(|table| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(table.id.table_id() as i32)),
                            Some(ScalarImpl::Utf8(table.name.clone().into())),
                            Some(ScalarImpl::Int32(schema_info.id as i32)),
                            Some(ScalarImpl::Int32(table.owner as i32)),
                            Some(ScalarImpl::Utf8("n".into())),
                            Some(ScalarImpl::Int32(0)),
                            Some(ScalarImpl::Int32(0)),
                        ])
                    })
                    .collect_vec();

                rows.into_iter()
                    .chain(mvs.into_iter())
                    .chain(indexes.into_iter())
                    .chain(sources.into_iter())
                    .chain(sys_tables.into_iter())
                    .chain(views.into_iter())
                    .chain(internal_tables.into_iter())
                    .collect_vec()
            })
            .collect_vec())
    }

    pub(super) fn read_index_info(&self) -> Result<Vec<OwnedRow>> {
        let reader = self.catalog_reader.read_guard();
        let schemas = reader.iter_schemas(&self.auth_context.database)?;

        Ok(schemas
            .flat_map(|schema| {
                schema.iter_index().map(|index| {
                    OwnedRow::new(vec![
                        Some(ScalarImpl::Int32(index.id.index_id() as i32)),
                        Some(ScalarImpl::Int32(index.primary_table.id.table_id() as i32)),
                        Some(ScalarImpl::Int16(index.original_columns.len() as i16)),
                        Some(ScalarImpl::List(ListValue::new(
                            index
                                .original_columns
                                .iter()
                                .map(|index| Some(ScalarImpl::Int16(index.get_id() as i16 + 1)))
                                .collect_vec(),
                        ))),
                        None,
                        None,
                    ])
                })
            })
            .collect_vec())
    }

    pub(super) async fn read_mviews_info(&self) -> Result<Vec<OwnedRow>> {
        let mut table_ids = Vec::new();
        {
            let reader = self.catalog_reader.read_guard();
            let schemas = reader.get_all_schema_names(&self.auth_context.database)?;
            for schema in &schemas {
                reader
                    .get_schema_by_name(&self.auth_context.database, schema)?
                    .iter_mv()
                    .for_each(|t| {
                        table_ids.push(t.id.table_id);
                    });
            }
        }

        let table_fragments = self.meta_client.list_table_fragments(&table_ids).await?;
        let mut rows = Vec::new();
        let reader = self.catalog_reader.read_guard();
        let schemas = reader.get_all_schema_names(&self.auth_context.database)?;
        for schema in &schemas {
            reader
                .get_schema_by_name(&self.auth_context.database, schema)?
                .iter_mv()
                .for_each(|t| {
                    if let Some(fragments) = table_fragments.get(&t.id.table_id) {
                        rows.push(OwnedRow::new(vec![
                            Some(ScalarImpl::Utf8(schema.clone().into())),
                            Some(ScalarImpl::Utf8(t.name.clone().into())),
                            Some(ScalarImpl::Int32(t.owner as i32)),
                            Some(ScalarImpl::Utf8(t.definition.clone().into())),
                            Some(ScalarImpl::Int32(t.id.table_id as i32)),
                            Some(ScalarImpl::Utf8(
                                fragments.get_env().unwrap().get_timezone().clone().into(),
                            )),
                            Some(ScalarImpl::Utf8(
                                json!(fragments.get_fragments()).to_string().into(),
                            )),
                        ]));
                    }
                });
        }

        Ok(rows)
    }

    pub(super) fn read_views_info(&self) -> Result<Vec<OwnedRow>> {
        // TODO(zehua): solve the deadlock problem.
        // Get two read locks. The order must be the same as
        // `FrontendObserverNode::handle_initialization_notification`.
        let catalog_reader = self.catalog_reader.read_guard();
        let user_info_reader = self.user_info_reader.read_guard();
        let schemas = catalog_reader.iter_schemas(&self.auth_context.database)?;

        Ok(schemas
            .flat_map(|schema| {
                schema.iter_view().map(|view| {
                    OwnedRow::new(vec![
                        Some(ScalarImpl::Utf8(schema.name().into())),
                        Some(ScalarImpl::Utf8(view.name().into())),
                        Some(ScalarImpl::Utf8(
                            user_info_reader
                                .get_user_name_by_id(view.owner)
                                .unwrap()
                                .into(),
                        )),
                        // TODO(zehua): may be not same as postgresql's "definition" column.
                        Some(ScalarImpl::Utf8(view.sql.clone().into())),
                    ])
                })
            })
            .collect_vec())
    }

    pub(super) fn read_indexes_info(&self) -> Result<Vec<OwnedRow>> {
        let catalog_reader = self.catalog_reader.read_guard();
        let schemas = catalog_reader.iter_schemas(&self.auth_context.database)?;

        Ok(schemas
            .flat_map(|schema: &SchemaCatalog| {
                schema.iter_index().map(|index| {
                    OwnedRow::new(vec![
                        Some(ScalarImpl::Utf8(schema.name().into())),
                        Some(ScalarImpl::Utf8(index.primary_table.name.clone().into())),
                        Some(ScalarImpl::Utf8(index.index_table.name.clone().into())),
                        None,
                        Some(ScalarImpl::Utf8(index.index_table.create_sql().into())),
                    ])
                })
            })
            .collect_vec())
    }

    pub(super) fn read_pg_attribute(&self) -> Result<Vec<OwnedRow>> {
        let reader = self.catalog_reader.read_guard();
        let schemas = reader.iter_schemas(&self.auth_context.database)?;

        Ok(schemas
            .flat_map(|schema| {
                let view_rows = schema.iter_view().flat_map(|view| {
                    view.columns.iter().enumerate().map(|(index, column)| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Int32(view.id as i32)),
                            Some(ScalarImpl::Utf8(column.name.clone().into())),
                            Some(ScalarImpl::Int32(column.data_type().to_oid())),
                            Some(ScalarImpl::Int16(column.data_type().type_len())),
                            Some(ScalarImpl::Int16(index as i16 + 1)),
                            Some(ScalarImpl::Bool(false)),
                            Some(ScalarImpl::Bool(false)),
                            // From https://www.postgresql.org/docs/current/catalog-pg-attribute.html
                            // The value will generally be -1 for types that do not need
                            // `atttypmod`.
                            Some(ScalarImpl::Int32(-1)),
                        ])
                    })
                });

                schema
                    .iter_valid_table()
                    .flat_map(|table| {
                        table
                            .columns()
                            .iter()
                            .enumerate()
                            .filter(|(_, column)| !column.is_hidden())
                            .map(|(index, column)| {
                                OwnedRow::new(vec![
                                    Some(ScalarImpl::Int32(table.id.table_id() as i32)),
                                    Some(ScalarImpl::Utf8(column.name().into())),
                                    Some(ScalarImpl::Int32(column.data_type().to_oid())),
                                    Some(ScalarImpl::Int16(column.data_type().type_len())),
                                    Some(ScalarImpl::Int16(index as i16 + 1)),
                                    Some(ScalarImpl::Bool(false)),
                                    Some(ScalarImpl::Bool(false)),
                                    // From https://www.postgresql.org/docs/current/catalog-pg-attribute.html
                                    // The value will generally be -1 for types that do not need
                                    // `atttypmod`.
                                    Some(ScalarImpl::Int32(-1)),
                                ])
                            })
                    })
                    .chain(view_rows)
            })
            .collect_vec())
    }

    pub(super) fn read_database_info(&self) -> Result<Vec<OwnedRow>> {
        let reader = self.catalog_reader.read_guard();
        let databases = reader.get_all_database_names();

        Ok(databases
            .iter()
            .map(|db| new_pg_database_row(reader.get_database_by_name(db).unwrap().id(), db))
            .collect_vec())
    }

    pub(super) fn read_description_info(&self) -> Result<Vec<OwnedRow>> {
        let reader = self.catalog_reader.read_guard();
        let schemas = reader.iter_schemas(&self.auth_context.database)?;

        Ok(schemas
            .flat_map(|schema| {
                let rows = schema
                    .iter_table()
                    .map(|table| new_pg_description_row(table.id().table_id))
                    .collect_vec();

                let mvs = schema
                    .iter_mv()
                    .map(|mv| new_pg_description_row(mv.id().table_id))
                    .collect_vec();

                let indexes = schema
                    .iter_index()
                    .map(|index| new_pg_description_row(index.id.index_id()))
                    .collect_vec();

                let sources = schema
                    .iter_source()
                    .map(|source| new_pg_description_row(source.id))
                    .collect_vec();

                let sys_tables = schema
                    .iter_system_tables()
                    .map(|table| new_pg_description_row(table.id().table_id))
                    .collect_vec();

                let views = schema
                    .iter_view()
                    .map(|view| new_pg_description_row(view.id))
                    .collect_vec();

                rows.into_iter()
                    .chain(mvs.into_iter())
                    .chain(indexes.into_iter())
                    .chain(sources.into_iter())
                    .chain(sys_tables.into_iter())
                    .chain(views.into_iter())
                    .collect_vec()
            })
            .collect_vec())
    }

    pub(super) fn read_settings_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(PG_SETTINGS_DATA_ROWS.clone())
    }

    pub(super) fn read_keywords_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(PG_KEYWORDS_DATA_ROWS.clone())
    }

    pub(super) fn read_tablespace_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(PG_TABLESPACE_DATA_ROWS.clone())
    }

    pub(crate) fn read_conversion_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    pub(super) fn read_stat_activity(&self) -> Result<Vec<OwnedRow>> {
        Ok(vec![])
    }

    pub(super) fn read_inherits_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(PG_INHERITS_DATA_ROWS.clone())
    }

    pub(super) fn read_constraint_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(PG_CONSTRAINT_DATA_ROWS.clone())
    }

    pub(crate) fn read_pg_proc_info(&self) -> Result<Vec<OwnedRow>> {
        Ok(PG_PROC_DATA_ROWS.clone())
    }

    pub(crate) fn read_pg_tables_info(&self) -> Result<Vec<OwnedRow>> {
        // TODO: avoid acquire two read locks here. The order is the same as in `read_views_info`.
        let reader = self.catalog_reader.read_guard();
        let user_info_reader = self.user_info_reader.read_guard();
        let schemas = reader.iter_schemas(&self.auth_context.database)?;

        Ok(schemas
            .flat_map(|schema| {
                schema
                    .iter_table()
                    .map(|table| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Utf8(schema.name().into())),
                            Some(ScalarImpl::Utf8(table.name().into())),
                            Some(ScalarImpl::Utf8(
                                user_info_reader
                                    .get_user_name_by_id(table.owner)
                                    .unwrap()
                                    .into(),
                            )),
                            None,
                        ])
                    })
                    .chain(schema.iter_system_tables().map(|table| {
                        OwnedRow::new(vec![
                            Some(ScalarImpl::Utf8(schema.name().into())),
                            Some(ScalarImpl::Utf8(table.name().into())),
                            Some(ScalarImpl::Utf8(
                                user_info_reader
                                    .get_user_name_by_id(table.owner)
                                    .unwrap()
                                    .into(),
                            )),
                            None,
                        ])
                    }))
            })
            .collect_vec())
    }
}
